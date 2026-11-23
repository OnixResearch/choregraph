//! Minimal worker subprocess used by the integration test so the harness can
//! send a real `SIGKILL` mid-step after `MarkReady` + `Start` have been
//! appended to Postgres `workflow_log`.

use std::io::Write;
use std::thread;
use std::time::Duration;

use choregraph_core::model::automaton::{AdmittedMachine, MachineSpec};
use choregraph_core::model::capacity::AdmissionLimits;
use choregraph_core::model::dispatch::Event;
use choregraph_core::model::graph::{EdgeSpec, NodeSpec};
use choregraph_core::model::semantics::{Action, LocalState};
use choregraph_postgres_host::{append_fenced_event, claim_next_step, replay_workflow};
use postgres::{Client, NoTls};

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

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        return Err(String::from(
            "usage: worker_sim <conn_str> <workflow_id> <worker_id>",
        ));
    }
    let conn_str = &args[1];
    let workflow_id = &args[2];
    let worker_id = &args[3];

    let machine = diamond_machine()?;
    let mut client = Client::connect(conn_str, NoTls).map_err(|err| format!("connect: {err}"))?;

    let Some(claim) = claim_next_step(&mut client, workflow_id, &machine, worker_id, 60.0)
        .map_err(|err| format!("claim: {err}"))?
    else {
        return Err(String::from("no runnable step to claim"));
    };

    let replayed = replay_workflow(&mut client, workflow_id, &machine)
        .map_err(|err| format!("replay: {err}"))?;
    let current = replayed
        .state
        .entries()
        .iter()
        .find(|(id, _)| id.as_str() == claim.node_id)
        .map(|(_, state)| *state);

    if current == Some(LocalState::Pending) {
        append_fenced_event(
            &mut client,
            &machine,
            &claim,
            &Event::new(&claim.node_id, Action::MarkReady),
        )
        .map_err(|err| format!("mark-ready: {err}"))?;
    }
    append_fenced_event(
        &mut client,
        &machine,
        &claim,
        &Event::new(&claim.node_id, Action::Start),
    )
    .map_err(|err| format!("start: {err}"))?;

    println!("CLAIMED:{}:{}", claim.node_id, claim.epoch.get());
    let _ = std::io::stdout().flush();

    // Pause mid-step so the test harness can kill this process with SIGKILL.
    thread::sleep(Duration::from_secs(60));
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("worker_sim error: {err}");
        std::process::exit(1);
    }
}
