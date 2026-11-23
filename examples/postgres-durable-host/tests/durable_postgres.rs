//! Integration tests against a real ephemeral PostgreSQL server exercising:
//! - `SELECT ... FOR UPDATE SKIP LOCKED` + monotonic epoch fencing,
//! - killing a worker process (`SIGKILL`) mid-step and resuming on another
//!   worker without double-applying effects,
//! - stale-epoch rejection via `choregraph_core::ownership::step_epoch_fenced`,
//! - byte-for-byte state reconstruction by replaying `workflow_log` alone, and
//! - durable suspension + resume through the Postgres log.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use choregraph_core::model::automaton::{AdmittedMachine, MachineSpec};
use choregraph_core::model::capacity::AdmissionLimits;
use choregraph_core::model::dispatch::{EffectDeclarationSpec, EffectRequest, Event};
use choregraph_core::model::graph::{EdgeSpec, NodeSpec};
use choregraph_core::model::semantics::{
    Action, DURABLE_SUSPENSION_V1, LocalState, TransitionSelector,
};
use choregraph_core::ownership::{EntityEpoch, EpochFenceRejection};
use choregraph_postgres_host::{
    HostError, StepClaim, all_terminal, append_fenced_event, canonical_state_bytes,
    claim_next_step, expire_lease_for_test, init_schema, register_workflow, release_step,
    replay_workflow,
};
use postgres::{Client, NoTls};

static CLUSTER_COUNTER: AtomicU64 = AtomicU64::new(0);

struct EphemeralPostgres {
    data_dir: PathBuf,
    conn_str: String,
}

impl EphemeralPostgres {
    fn start() -> Result<Self, String> {
        let base = std::env::temp_dir();
        let pid = std::process::id();
        let seq = CLUSTER_COUNTER.fetch_add(1, Ordering::Relaxed);
        let data_dir = base.join(format!("choregraph-pg-{pid}-{seq}"));
        let _ = std::fs::remove_dir_all(&data_dir);
        std::fs::create_dir_all(&data_dir).map_err(|err| format!("mkdir pgdata: {err}"))?;

        let init_status = Command::new("initdb")
            .arg("-D")
            .arg(&data_dir)
            .arg("-U")
            .arg("postgres")
            .arg("-A")
            .arg("trust")
            .arg("--no-sync")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
            .map_err(|err| format!("spawn initdb (ensure postgresql is on PATH): {err}"))?;
        if !init_status.success() {
            return Err(format!("initdb failed with status {init_status}"));
        }

        let log_file = data_dir.join("postgres.log");
        let opts = format!("-k {} -h ''", data_dir.display());
        let start_status = Command::new("pg_ctl")
            .arg("-D")
            .arg(&data_dir)
            .arg("-l")
            .arg(&log_file)
            .arg("-o")
            .arg(opts)
            .arg("-w")
            .arg("start")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
            .map_err(|err| format!("spawn pg_ctl start: {err}"))?;
        if !start_status.success() {
            let log = std::fs::read_to_string(&log_file).unwrap_or_default();
            return Err(format!("pg_ctl start failed ({start_status}): {log}"));
        }

        let conn_str = format!("host={} user=postgres dbname=postgres", data_dir.display());
        Ok(Self { data_dir, conn_str })
    }

    fn connect(&self) -> Result<Client, String> {
        Client::connect(&self.conn_str, NoTls).map_err(|err| format!("pg connect: {err}"))
    }
}

impl Drop for EphemeralPostgres {
    fn drop(&mut self) {
        let _ = Command::new("pg_ctl")
            .arg("-D")
            .arg(&self.data_dir)
            .arg("-m")
            .arg("immediate")
            .arg("-w")
            .arg("stop")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = std::fs::remove_dir_all(&self.data_dir);
    }
}

fn diamond_machine() -> Result<AdmittedMachine, String> {
    let spec = MachineSpec::new(
        vec![
            NodeSpec::new("root", Vec::new()),
            NodeSpec::new("left", Vec::new()),
            NodeSpec::new("right", Vec::new()),
            NodeSpec::new("sink", Vec::new()),
        ],
        vec![
            EdgeSpec::new("root", "left"),
            EdgeSpec::new("root", "right"),
            EdgeSpec::new("left", "sink"),
            EdgeSpec::new("right", "sink"),
        ],
    );
    AdmittedMachine::admit(spec, AdmissionLimits::default())
        .map_err(|diag| format!("admit diamond machine: {diag:?}"))
}

fn effectful_root_machine() -> Result<AdmittedMachine, String> {
    let mut spec = MachineSpec::new(
        vec![
            NodeSpec::new("root", vec![String::from("host/apply")]),
            NodeSpec::new("sink", Vec::new()),
        ],
        vec![EdgeSpec::new("root", "sink")],
    );
    spec.effects.push(EffectDeclarationSpec {
        node_id: String::from("root"),
        transition: TransitionSelector {
            from: LocalState::Ready,
            action: Action::Start,
        },
        operation: String::from("host/apply"),
        resource: String::from("ledger/root"),
        ability: String::from("write"),
        request: String::from("req-1"),
        payload: String::from(
            "blake3:1111111111111111111111111111111111111111111111111111111111111111",
        ),
    });
    AdmittedMachine::admit(spec, AdmissionLimits::default())
        .map_err(|diag| format!("admit effectful machine: {diag:?}"))
}

fn drive_claimed_step_to_completion(
    client: &mut Client,
    machine: &AdmittedMachine,
    claim: &StepClaim,
    applied_effects: &mut BTreeMap<String, u64>,
) -> Result<(), String> {
    let replayed = replay_workflow(client, &claim.workflow_id, machine)
        .map_err(|err| format!("replay before step: {err}"))?;
    let current = replayed
        .state
        .entries()
        .iter()
        .find(|(id, _)| id.as_str() == claim.node_id)
        .map(|(_, state)| *state)
        .ok_or_else(|| format!("missing node {}", claim.node_id))?;

    if current == LocalState::Pending {
        append_fenced_event(
            client,
            machine,
            claim,
            &Event::new(&claim.node_id, Action::MarkReady),
        )
        .map_err(|err| format!("mark-ready: {err}"))?;
        append_fenced_event(
            client,
            machine,
            claim,
            &Event::new(&claim.node_id, Action::Start),
        )
        .map_err(|err| format!("start: {err}"))?;
    } else if current == LocalState::Ready {
        append_fenced_event(
            client,
            machine,
            claim,
            &Event::new(&claim.node_id, Action::Start),
        )
        .map_err(|err| format!("start: {err}"))?;
    }

    // Complete the step through the epoch-fenced kernel gate; only record
    // the external side effect when the fenced append succeeds.
    append_fenced_event(
        client,
        machine,
        claim,
        &Event::new(&claim.node_id, Action::Complete),
    )
    .map_err(|err| format!("complete: {err}"))?;
    let counter = applied_effects.entry(claim.node_id.clone()).or_insert(0);
    *counter = counter.saturating_add(1);
    release_step(client, claim).map_err(|err| format!("release: {err}"))?;
    Ok(())
}

#[test]
fn worker_killed_mid_step_resumes_without_double_applying_and_replays_byte_for_byte()
-> Result<(), String> {
    let pg = EphemeralPostgres::start()?;
    let mut client = pg.connect()?;
    init_schema(&mut client).map_err(|err| format!("init schema: {err}"))?;

    let machine = diamond_machine()?;
    let workflow_id = "wf-crash-recovery";
    register_workflow(&mut client, workflow_id, &machine)
        .map_err(|err| format!("register workflow: {err}"))?;

    // 1. Spawn a real OS subprocess worker that claims `root` at epoch 1,
    //    appends `MarkReady` and `Start`, prints `CLAIMED:root:1`, and pauses.
    let worker_bin = env!("CARGO_BIN_EXE_worker_sim");
    let mut child = Command::new(worker_bin)
        .arg(&pg.conn_str)
        .arg(workflow_id)
        .arg("worker-1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("spawn worker_sim: {err}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| String::from("missing worker stdout"))?;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|err| format!("read worker stdout: {err}"))?;
    assert_eq!(line.trim(), "CLAIMED:root:1");

    // 2. Kill worker-1 mid-step with SIGKILL before it can append `Complete`.
    child
        .kill()
        .map_err(|err| format!("kill worker_sim: {err}"))?;
    let _ = child.wait();

    // Capture the stale claim that worker-1 held before dying.
    let stale_worker_1_claim = StepClaim {
        workflow_id: String::from(workflow_id),
        node_id: String::from("root"),
        epoch: EntityEpoch::new(1),
        worker_id: String::from("worker-1"),
    };

    // Confirm log replay sees `root` in `Running` state after worker-1 died.
    let mid_replay = replay_workflow(&mut client, workflow_id, &machine)
        .map_err(|err| format!("mid replay: {err}"))?;
    let root_state = mid_replay
        .state
        .entries()
        .iter()
        .find(|(id, _)| id.as_str() == "root")
        .map(|(_, state)| *state);
    assert_eq!(root_state, Some(LocalState::Running));

    // 3. Lease expires -> worker-2 claims `root` at epoch 2.
    expire_lease_for_test(&mut client, workflow_id, "root")
        .map_err(|err| format!("expire lease: {err}"))?;
    let worker_2_claim = claim_next_step(&mut client, workflow_id, &machine, "worker-2", 30.0)
        .map_err(|err| format!("worker-2 claim: {err}"))?
        .ok_or_else(|| String::from("worker-2 expected to claim root"))?;
    assert_eq!(worker_2_claim.node_id, "root");
    assert_eq!(worker_2_claim.epoch, EntityEpoch::new(2));

    // 4. Stale worker-1 wakes up / zombie retries its fenced append with epoch 1.
    //    `step_epoch_fenced` MUST reject with EpochMismatch without modifying the log.
    let stale_result = append_fenced_event(
        &mut client,
        &machine,
        &stale_worker_1_claim,
        &Event::new("root", Action::Complete),
    );
    assert!(matches!(
        stale_result,
        Err(HostError::EpochFenceRejected {
            rejection: EpochFenceRejection::EpochMismatch,
            current_epoch,
            offered_epoch,
        }) if current_epoch == EntityEpoch::new(2) && offered_epoch == EntityEpoch::new(1)
    ));
    // Stale release must NOT clear worker-2's active lease.
    let stale_released = release_step(&mut client, &stale_worker_1_claim)
        .map_err(|err| format!("release: {err}"))?;
    assert!(!stale_released);

    // 5. Worker-2 resumes `root` and completes it once.
    let mut applied_effects = BTreeMap::new();
    drive_claimed_step_to_completion(&mut client, &machine, &worker_2_claim, &mut applied_effects)?;
    assert_eq!(applied_effects.get("root"), Some(&1));

    // Even after worker-2 releases `root`, a duplicate append at epoch 2 is rejected
    // as TerminalMutation and never double-applies the effect.
    let duplicate_after_complete = append_fenced_event(
        &mut client,
        &machine,
        &worker_2_claim,
        &Event::new("root", Action::Complete),
    );
    assert!(matches!(
        duplicate_after_complete,
        Err(HostError::EpochFenceRejected {
            rejection: EpochFenceRejection::TerminalMutation,
            ..
        })
    ));

    // 6. Drive the rest of the diamond (`left`, `right`, `sink`) via claim/append/release.
    let mut last_live_state = replay_workflow(&mut client, workflow_id, &machine)
        .map_err(|err| format!("replay after root: {err}"))?
        .state;
    while let Some(claim) = claim_next_step(&mut client, workflow_id, &machine, "worker-2", 30.0)
        .map_err(|err| format!("claim next: {err}"))?
    {
        drive_claimed_step_to_completion(&mut client, &machine, &claim, &mut applied_effects)?;
        last_live_state = replay_workflow(&mut client, workflow_id, &machine)
            .map_err(|err| format!("replay step: {err}"))?
            .state;
    }

    assert!(all_terminal(&last_live_state));
    assert_eq!(applied_effects.len(), 4);
    for node in ["root", "left", "right", "sink"] {
        assert_eq!(applied_effects.get(node), Some(&1));
    }

    // 7. Simulate full host restart on a fresh Postgres connection: replay from
    //    `workflow_log` alone reproduces kernel state byte-for-byte.
    let mut fresh_client = pg.connect()?;
    let restarted = replay_workflow(&mut fresh_client, workflow_id, &machine)
        .map_err(|err| format!("restart replay: {err}"))?;
    assert_eq!(restarted.state, last_live_state);
    assert_eq!(
        restarted.canonical_bytes,
        canonical_state_bytes(&last_live_state)
    );
    assert_eq!(restarted.consumed_events, 12);
    Ok(())
}

#[test]
fn skip_locked_prevents_concurrent_workers_from_claiming_same_step() -> Result<(), String> {
    let pg = EphemeralPostgres::start()?;
    let mut client_a = pg.connect()?;
    let mut client_b = pg.connect()?;
    init_schema(&mut client_a).map_err(|err| format!("init schema: {err}"))?;

    let machine = diamond_machine()?;
    let workflow_id = "wf-skip-locked";
    register_workflow(&mut client_a, workflow_id, &machine)
        .map_err(|err| format!("register: {err}"))?;

    // Complete `root` so both `left` and `right` become runnable in parallel.
    let root_claim = claim_next_step(&mut client_a, workflow_id, &machine, "worker-a", 30.0)
        .map_err(|err| format!("claim root: {err}"))?
        .ok_or_else(|| String::from("expected root claim"))?;
    let mut effects = BTreeMap::new();
    drive_claimed_step_to_completion(&mut client_a, &machine, &root_claim, &mut effects)?;

    // Worker A claims one branch; Worker B claims the other via SKIP LOCKED.
    let claim_a = claim_next_step(&mut client_a, workflow_id, &machine, "worker-a", 30.0)
        .map_err(|err| format!("claim A: {err}"))?
        .ok_or_else(|| String::from("expected branch claim for worker-a"))?;
    let claim_b = claim_next_step(&mut client_b, workflow_id, &machine, "worker-b", 30.0)
        .map_err(|err| format!("claim B: {err}"))?
        .ok_or_else(|| String::from("expected branch claim for worker-b"))?;
    assert_ne!(claim_a.node_id, claim_b.node_id);

    // No third runnable step exists while both branches are leased (`sink` waits).
    let claim_c = claim_next_step(&mut client_a, workflow_id, &machine, "worker-c", 30.0)
        .map_err(|err| format!("claim C: {err}"))?;
    assert!(claim_c.is_none());
    Ok(())
}

#[test]
fn effect_intents_and_durable_suspension_replay_byte_for_byte() -> Result<(), String> {
    let pg = EphemeralPostgres::start()?;
    let mut client = pg.connect()?;
    init_schema(&mut client).map_err(|err| format!("init schema: {err}"))?;

    // Check effect-intent emission on live fenced step vs zero-intent replay.
    let effect_machine = effectful_root_machine()?;
    register_workflow(&mut client, "wf-effect", &effect_machine)
        .map_err(|err| format!("register effect wf: {err}"))?;
    let claim = claim_next_step(&mut client, "wf-effect", &effect_machine, "w1", 30.0)
        .map_err(|err| format!("claim effect root: {err}"))?
        .ok_or_else(|| String::from("expected root claim"))?;
    append_fenced_event(
        &mut client,
        &effect_machine,
        &claim,
        &Event::new("root", Action::MarkReady),
    )
    .map_err(|err| format!("mark-ready: {err}"))?;
    let mut start_event = Event::new("root", Action::Start);
    start_event.effects.push(EffectRequest {
        operation: String::from("host/apply"),
        resource: String::from("ledger/root"),
        ability: String::from("write"),
        request: String::from("req-1"),
        payload: String::from(
            "blake3:1111111111111111111111111111111111111111111111111111111111111111",
        ),
    });
    let started = append_fenced_event(&mut client, &effect_machine, &claim, &start_event)
        .map_err(|err| format!("start with effect: {err}"))?;
    assert_eq!(started.intents.len(), 1);
    let completed = append_fenced_event(
        &mut client,
        &effect_machine,
        &claim,
        &Event::new("root", Action::Complete),
    )
    .map_err(|err| format!("complete: {err}"))?;
    let replayed_effect = replay_workflow(&mut client, "wf-effect", &effect_machine)
        .map_err(|err| format!("replay effect wf: {err}"))?;
    assert_eq!(
        replayed_effect.canonical_bytes,
        canonical_state_bytes(&completed.next_state)
    );

    // Check durable suspension + resume + byte-for-byte replay.
    let mut spec = MachineSpec::new(vec![NodeSpec::new("gate", Vec::new())], Vec::new());
    spec.lifecycle_profile = String::from(DURABLE_SUSPENSION_V1);
    let wait_machine = AdmittedMachine::admit(spec, AdmissionLimits::default())
        .map_err(|diag| format!("admit wait machine: {diag:?}"))?;
    register_workflow(&mut client, "wf-wait", &wait_machine)
        .map_err(|err| format!("register wait wf: {err}"))?;
    let wait_claim = claim_next_step(&mut client, "wf-wait", &wait_machine, "w1", 30.0)
        .map_err(|err| format!("claim gate: {err}"))?
        .ok_or_else(|| String::from("expected gate claim"))?;

    let wait_ref = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    append_fenced_event(
        &mut client,
        &wait_machine,
        &wait_claim,
        &Event::new("gate", Action::MarkReady),
    )
    .map_err(|err| format!("gate mark-ready: {err}"))?;
    let suspended = append_fenced_event(
        &mut client,
        &wait_machine,
        &wait_claim,
        &Event::suspension("gate", Action::Suspend, "approval", wait_ref),
    )
    .map_err(|err| format!("gate suspend: {err}"))?;
    let replayed_suspended = replay_workflow(&mut client, "wf-wait", &wait_machine)
        .map_err(|err| format!("replay suspended: {err}"))?;
    assert_eq!(
        replayed_suspended.canonical_bytes,
        canonical_state_bytes(&suspended.next_state)
    );
    let resumed = append_fenced_event(
        &mut client,
        &wait_machine,
        &wait_claim,
        &Event::suspension("gate", Action::ResumeSignal, "approval", wait_ref),
    )
    .map_err(|err| format!("gate resume: {err}"))?;
    assert_eq!(
        resumed.next_state.entries().values().next(),
        Some(&LocalState::Ready)
    );
    Ok(())
}
