//! Reference durable host adapter keeping a Choregraph workflow log in one
//! Postgres table (`workflow_log`).
//!
//! This crate lives strictly outside the verified `no_std + alloc` kernel
//! (`choregraph-core`). Postgres owns row locking (`SELECT ... FOR UPDATE SKIP
//! LOCKED`), monotonic epoch allocation, and durable persistence; Choregraph
//! owns pure transition decisions (`step_epoch_fenced`), readiness queries
//! (`ready_nodes`), and deterministic restart recovery (`replay`).

use std::collections::BTreeSet;
use std::fmt;

use choregraph_core::analytics::ready_nodes;
use choregraph_core::id::bounded::NodeId;
use choregraph_core::model::automaton::{AdmittedMachine, MachineState};
use choregraph_core::model::dispatch::{EffectIntent, EffectRequest, Event};
use choregraph_core::model::history::{
    BoundedTrace, ReplayBindings, ReplayDecision, ReplayFailure, ReplayLimits, TRACE_BINDING_BYTES,
    TraceBinding,
};
use choregraph_core::model::outcome::{Decision, DenialReason};
use choregraph_core::model::semantics::{Action, LocalState};
use choregraph_core::ownership::{
    EntityEpoch, EpochFence, EpochFenceDisposition, EpochFenceRejection, step_epoch_fenced,
};
use choregraph_core::transition::replay;
use postgres::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_REPLAY_MAX_EVENTS: u64 = 65_536;
const STATE_HASH_CONTEXT: &str = "choregraph postgres host state bytes v1";
const MACHINE_BINDING_CONTEXT: &str = "choregraph postgres host machine binding v1";
const INITIAL_BINDING_CONTEXT: &str = "choregraph postgres host initial binding v1";

/// DDL for the single Postgres table storing both per-node claim slots
/// (`entry_kind = 'step'`, negative `seq`) and the append-only kernel event log
/// (`entry_kind = 'event'`, positive `seq`).
pub const WORKFLOW_LOG_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS workflow_log (
    workflow_id TEXT NOT NULL,
    seq         BIGINT NOT NULL,
    entry_kind  TEXT NOT NULL CHECK (entry_kind IN ('step', 'event')),
    node_id     TEXT NOT NULL,
    epoch       BIGINT NOT NULL DEFAULT 0,
    claimed_by  TEXT,
    lease_until TIMESTAMPTZ,
    event_json  JSONB,
    state_hash  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workflow_id, seq)
);
CREATE INDEX IF NOT EXISTS workflow_log_step_lookup
    ON workflow_log (workflow_id, entry_kind, node_id);
"#;

/// Serializable event record persisted inside `workflow_log.event_json`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoredEvent {
    pub node_id: String,
    pub action: String,
    pub suspension_class: Option<String>,
    pub suspension_reference: Option<String>,
    pub effects: Vec<StoredEffectRequest>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoredEffectRequest {
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: String,
    pub payload_reference: String,
}

/// Active lease and epoch fence held by one worker for one runnable node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepClaim {
    pub workflow_id: String,
    pub node_id: String,
    pub epoch: EntityEpoch,
    pub worker_id: String,
}

/// Result of one epoch-fenced transition appended to `workflow_log`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppendedStep {
    pub seq: i64,
    pub next_state: MachineState,
    pub intents: Vec<EffectIntent>,
    pub state_hash: String,
}

/// Replayed workflow state reconstructed solely from `workflow_log`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayedWorkflow {
    pub state: MachineState,
    pub consumed_events: u64,
    pub canonical_bytes: Vec<u8>,
    pub state_hash: String,
}

/// Host adapter error categories.
#[derive(Debug)]
pub enum HostError {
    Database(postgres::Error),
    Serialization(serde_json::Error),
    InvalidNodeId(String),
    UnknownAction(String),
    ReplayFailed(ReplayFailure),
    EpochFenceRejected {
        rejection: EpochFenceRejection,
        current_epoch: EntityEpoch,
        offered_epoch: EntityEpoch,
    },
    TransitionDenied(DenialReason),
    InvalidMachineQuery(String),
    InvariantViolation(String),
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(err) => write!(f, "postgres error: {err}"),
            Self::Serialization(err) => write!(f, "event json error: {err}"),
            Self::InvalidNodeId(id) => write!(f, "invalid or unknown node id: {id}"),
            Self::UnknownAction(action) => write!(f, "unknown stored action: {action}"),
            Self::ReplayFailed(failure) => write!(f, "kernel replay failed: {failure:?}"),
            Self::EpochFenceRejected {
                rejection,
                current_epoch,
                offered_epoch,
            } => write!(
                f,
                "epoch fence rejected ({rejection:?}): current={}, offered={}",
                current_epoch.get(),
                offered_epoch.get()
            ),
            Self::TransitionDenied(reason) => write!(f, "kernel transition denied: {reason:?}"),
            Self::InvalidMachineQuery(detail) => write!(f, "machine query failed: {detail}"),
            Self::InvariantViolation(detail) => write!(f, "host invariant violation: {detail}"),
        }
    }
}

impl std::error::Error for HostError {}

impl From<postgres::Error> for HostError {
    fn from(value: postgres::Error) -> Self {
        Self::Database(value)
    }
}

impl From<serde_json::Error> for HostError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

const fn action_to_str(action: Action) -> &'static str {
    match action {
        Action::MarkReady => "mark-ready",
        Action::Start => "start",
        Action::Complete => "complete",
        Action::Fail => "fail",
        Action::Cancel => "cancel",
        Action::Suspend => "suspend",
        Action::ResumeSignal => "resume-signal",
    }
}

fn action_from_str(text: &str) -> Result<Action, HostError> {
    match text {
        "mark-ready" => Ok(Action::MarkReady),
        "start" => Ok(Action::Start),
        "complete" => Ok(Action::Complete),
        "fail" => Ok(Action::Fail),
        "cancel" => Ok(Action::Cancel),
        "suspend" => Ok(Action::Suspend),
        "resume-signal" => Ok(Action::ResumeSignal),
        other => Err(HostError::UnknownAction(String::from(other))),
    }
}

const fn local_state_tag(state: LocalState) -> u8 {
    match state {
        LocalState::Pending => 0,
        LocalState::Ready => 1,
        LocalState::Running => 2,
        LocalState::Completed => 3,
        LocalState::Failed => 4,
        LocalState::Cancelled => 5,
        LocalState::Suspended => 6,
    }
}

const fn is_terminal(state: LocalState) -> bool {
    matches!(
        state,
        LocalState::Completed | LocalState::Failed | LocalState::Cancelled
    )
}

impl StoredEvent {
    #[must_use]
    pub fn from_kernel_event(event: &Event) -> Self {
        let (suspension_class, suspension_reference) = match &event.suspension {
            Some(fact) => (Some(fact.class.clone()), Some(fact.reference.clone())),
            None => (None, None),
        };
        let effects = event
            .effects
            .iter()
            .map(|effect| StoredEffectRequest {
                operation: effect.operation.clone(),
                resource: effect.resource.clone(),
                ability: effect.ability.clone(),
                request: effect.request.clone(),
                payload_reference: effect.payload.clone(),
            })
            .collect();
        Self {
            node_id: event.node_id.clone(),
            action: String::from(action_to_str(event.action)),
            suspension_class,
            suspension_reference,
            effects,
        }
    }

    pub fn to_kernel_event(&self) -> Result<Event, HostError> {
        let action = action_from_str(&self.action)?;
        let mut event = match (&self.suspension_class, &self.suspension_reference) {
            (Some(class), Some(reference)) => {
                Event::suspension(&self.node_id, action, class, reference)
            }
            (None, None) => Event::new(&self.node_id, action),
            _ => {
                return Err(HostError::InvariantViolation(String::from(
                    "stored event has partial suspension metadata",
                )));
            }
        };
        for effect in &self.effects {
            event.effects.push(EffectRequest {
                operation: effect.operation.clone(),
                resource: effect.resource.clone(),
                ability: effect.ability.clone(),
                request: effect.request.clone(),
                payload: effect.payload_reference.clone(),
            });
        }
        Ok(event)
    }
}

/// Deterministic canonical byte encoding of a `MachineState` for byte-for-byte
/// equality and BLAKE3 state-hash verification.
#[must_use]
pub fn canonical_state_bytes(state: &MachineState) -> Vec<u8> {
    let mut bytes = Vec::new();
    let count = u64::try_from(state.entries().len()).unwrap_or(u64::MAX);
    bytes.extend_from_slice(&count.to_le_bytes());
    for (node_id, local_state) in state.entries() {
        let id_bytes = node_id.as_str().as_bytes();
        let id_len = u64::try_from(id_bytes.len()).unwrap_or(u64::MAX);
        bytes.extend_from_slice(&id_len.to_le_bytes());
        bytes.extend_from_slice(id_bytes);
        bytes.push(local_state_tag(*local_state));
        match state.suspension(node_id) {
            Some(fact) => {
                bytes.push(1);
                let class_bytes = fact.class.as_str().as_bytes();
                let class_len = u64::try_from(class_bytes.len()).unwrap_or(u64::MAX);
                bytes.extend_from_slice(&class_len.to_le_bytes());
                bytes.extend_from_slice(class_bytes);
                let ref_bytes = fact.reference.as_str().as_bytes();
                let ref_len = u64::try_from(ref_bytes.len()).unwrap_or(u64::MAX);
                bytes.extend_from_slice(&ref_len.to_le_bytes());
                bytes.extend_from_slice(ref_bytes);
            }
            None => bytes.push(0),
        }
    }
    bytes
}

#[must_use]
pub fn canonical_state_hash(state: &MachineState) -> String {
    let bytes = canonical_state_bytes(state);
    let mut hasher = blake3::Hasher::new_derive_key(STATE_HASH_CONTEXT);
    hasher.update(&bytes);
    hasher.finalize().to_hex().to_string()
}

fn derive_replay_bindings(machine: &AdmittedMachine, initial: &MachineState) -> ReplayBindings {
    let mut machine_hasher = blake3::Hasher::new_derive_key(MACHINE_BINDING_CONTEXT);
    for node_id in machine.node_ids() {
        machine_hasher.update(node_id.as_str().as_bytes());
        machine_hasher.update(&[0]);
    }
    for edge in machine.edges() {
        machine_hasher.update(edge.source.as_str().as_bytes());
        machine_hasher.update(&[1]);
        machine_hasher.update(edge.target.as_str().as_bytes());
        machine_hasher.update(&[0]);
    }
    let mut machine_bytes = [0u8; TRACE_BINDING_BYTES];
    machine_bytes.copy_from_slice(machine_hasher.finalize().as_bytes());

    let mut initial_hasher = blake3::Hasher::new_derive_key(INITIAL_BINDING_CONTEXT);
    initial_hasher.update(&canonical_state_bytes(initial));
    let mut initial_bytes = [0u8; TRACE_BINDING_BYTES];
    initial_bytes.copy_from_slice(initial_hasher.finalize().as_bytes());

    ReplayBindings::new(
        TraceBinding::from_bytes(machine_bytes),
        TraceBinding::from_bytes(initial_bytes),
    )
}

/// Ensures the single `workflow_log` table exists.
pub fn init_schema(client: &mut Client) -> Result<(), HostError> {
    client.batch_execute(WORKFLOW_LOG_SCHEMA_SQL)?;
    Ok(())
}

/// Initializes one claimable step slot (`entry_kind = 'step'`, negative `seq`)
/// per node in the admitted machine. Idempotent if the workflow already exists.
pub fn register_workflow(
    client: &mut Client,
    workflow_id: &str,
    machine: &AdmittedMachine,
) -> Result<(), HostError> {
    let mut tx = client.transaction()?;
    for (index, node_id) in machine.node_ids().iter().enumerate() {
        let slot_seq = -i64::try_from(index.saturating_add(1))
            .map_err(|_| HostError::InvariantViolation(String::from("node index overflow")))?;
        tx.execute(
            "INSERT INTO workflow_log (workflow_id, seq, entry_kind, node_id, epoch)
             VALUES ($1, $2, 'step', $3, 0)
             ON CONFLICT (workflow_id, seq) DO NOTHING",
            &[&workflow_id, &slot_seq, &node_id.as_str()],
        )?;
    }
    tx.commit()?;
    Ok(())
}

fn replay_events_in_tx(
    tx: &mut postgres::Transaction<'_>,
    workflow_id: &str,
    machine: &AdmittedMachine,
) -> Result<ReplayedWorkflow, HostError> {
    let rows = tx.query(
        "SELECT event_json
         FROM workflow_log
         WHERE workflow_id = $1 AND entry_kind = 'event'
         ORDER BY seq ASC",
        &[&workflow_id],
    )?;
    let mut events = Vec::with_capacity(rows.len());
    for row in rows {
        let raw: serde_json::Value = row.get(0);
        let stored: StoredEvent = serde_json::from_value(raw)?;
        events.push(stored.to_kernel_event()?);
    }
    replay_slice(machine, &events)
}

fn replay_slice(
    machine: &AdmittedMachine,
    events: &[Event],
) -> Result<ReplayedWorkflow, HostError> {
    let initial = machine.initial_state();
    let bindings = derive_replay_bindings(machine, &initial);
    let trace = BoundedTrace::new(bindings, events);
    let input = choregraph_core::model::history::ReplayInput::new(
        machine,
        &initial,
        bindings,
        trace,
        ReplayLimits {
            max_events: DEFAULT_REPLAY_MAX_EVENTS,
        },
    );
    match replay(input) {
        ReplayDecision::Complete {
            final_state,
            consumed,
        } => {
            let canonical_bytes = canonical_state_bytes(&final_state);
            let state_hash = canonical_state_hash(&final_state);
            Ok(ReplayedWorkflow {
                state: final_state,
                consumed_events: consumed.get(),
                canonical_bytes,
                state_hash,
            })
        }
        ReplayDecision::Failed(failure) => Err(HostError::ReplayFailed(failure)),
    }
}

/// Reconstructs the workflow's kernel state strictly by replaying `workflow_log`
/// event rows through `choregraph_core::transition::replay`.
pub fn replay_workflow(
    client: &mut Client,
    workflow_id: &str,
    machine: &AdmittedMachine,
) -> Result<ReplayedWorkflow, HostError> {
    let mut tx = client.transaction()?;
    let replayed = replay_events_in_tx(&mut tx, workflow_id, machine)?;
    tx.commit()?;
    Ok(replayed)
}

fn runnable_node_ids(
    machine: &AdmittedMachine,
    state: &MachineState,
) -> Result<Vec<String>, HostError> {
    let mut runnable = BTreeSet::new();
    let ready_frontier = ready_nodes(machine, state)
        .map_err(|err| HostError::InvalidMachineQuery(format!("{err:?}")))?;
    for node_id in ready_frontier {
        runnable.insert(String::from(node_id.as_str()));
    }
    for (node_id, local_state) in state.entries() {
        if matches!(local_state, LocalState::Ready | LocalState::Running) {
            runnable.insert(String::from(node_id.as_str()));
        }
    }
    Ok(runnable.into_iter().collect())
}

/// Claims the next runnable step for `worker_id` using
/// `SELECT ... FOR UPDATE SKIP LOCKED`, incrementing the step's monotonic epoch
/// and setting its lease expiry inside Postgres.
pub fn claim_next_step(
    client: &mut Client,
    workflow_id: &str,
    machine: &AdmittedMachine,
    worker_id: &str,
    lease_secs: f64,
) -> Result<Option<StepClaim>, HostError> {
    let mut tx = client.transaction()?;
    let replayed = replay_events_in_tx(&mut tx, workflow_id, machine)?;
    let candidates = runnable_node_ids(machine, &replayed.state)?;
    if candidates.is_empty() {
        tx.commit()?;
        return Ok(None);
    }

    let claimed_row = tx.query_opt(
        "SELECT seq, node_id, epoch
         FROM workflow_log
         WHERE workflow_id = $1
           AND entry_kind = 'step'
           AND node_id = ANY($2)
           AND (claimed_by IS NULL OR lease_until <= NOW())
         ORDER BY seq DESC
         FOR UPDATE SKIP LOCKED
         LIMIT 1",
        &[&workflow_id, &candidates],
    )?;

    let Some(row) = claimed_row else {
        tx.commit()?;
        return Ok(None);
    };

    let slot_seq: i64 = row.get(0);
    let node_id: String = row.get(1);
    let prev_epoch: i64 = row.get(2);
    let next_epoch = prev_epoch
        .checked_add(1)
        .ok_or_else(|| HostError::InvariantViolation(String::from("epoch counter overflow")))?;

    tx.execute(
        "UPDATE workflow_log
         SET epoch = $1,
             claimed_by = $2,
             lease_until = NOW() + make_interval(secs => $3)
         WHERE workflow_id = $4 AND seq = $5",
        &[
            &next_epoch,
            &worker_id,
            &lease_secs,
            &workflow_id,
            &slot_seq,
        ],
    )?;
    tx.commit()?;

    let epoch_u64 = u64::try_from(next_epoch)
        .map_err(|_| HostError::InvariantViolation(String::from("negative epoch")))?;
    Ok(Some(StepClaim {
        workflow_id: String::from(workflow_id),
        node_id,
        epoch: EntityEpoch::new(epoch_u64),
        worker_id: String::from(worker_id),
    }))
}

fn find_admitted_node_id(machine: &AdmittedMachine, target: &str) -> Result<NodeId, HostError> {
    for node_id in machine.node_ids() {
        if node_id.as_str() == target {
            return Ok(node_id.clone());
        }
    }
    Err(HostError::InvalidNodeId(String::from(target)))
}

/// Evaluates `event` through `choregraph_core::ownership::step_epoch_fenced`
/// against the authoritative Postgres epoch and replayed state, appending the
/// event row atomically only if both the epoch fence and kernel transition admit.
pub fn append_fenced_event(
    client: &mut Client,
    machine: &AdmittedMachine,
    claim: &StepClaim,
    event: &Event,
) -> Result<AppendedStep, HostError> {
    let mut tx = client.transaction()?;
    let slot_row = tx
        .query_opt(
            "SELECT epoch
             FROM workflow_log
             WHERE workflow_id = $1 AND entry_kind = 'step' AND node_id = $2
             FOR UPDATE",
            &[&claim.workflow_id, &claim.node_id],
        )?
        .ok_or_else(|| HostError::InvalidNodeId(claim.node_id.clone()))?;

    let current_epoch_i64: i64 = slot_row.get(0);
    let current_epoch_u64 = u64::try_from(current_epoch_i64)
        .map_err(|_| HostError::InvariantViolation(String::from("negative current epoch")))?;
    let current_epoch = EntityEpoch::new(current_epoch_u64);

    let replayed = replay_events_in_tx(&mut tx, &claim.workflow_id, machine)?;
    let entity = find_admitted_node_id(machine, &claim.node_id)?;
    let fence = EpochFence::new(entity, current_epoch, claim.epoch);
    let fenced = step_epoch_fenced(machine, &replayed.state, event, &fence);

    match fenced.fence.disposition {
        EpochFenceDisposition::Reject(rejection) => {
            tx.rollback()?;
            return Err(HostError::EpochFenceRejected {
                rejection,
                current_epoch,
                offered_epoch: claim.epoch,
            });
        }
        EpochFenceDisposition::Admit => {}
    }

    let Some(decision) = fenced.transition else {
        tx.rollback()?;
        return Err(HostError::InvariantViolation(String::from(
            "admitted epoch fence produced no transition decision",
        )));
    };

    let (next_state, intents) = match decision {
        Decision::Accept { next, intents } => (next, intents),
        Decision::Deny { reason } => {
            tx.rollback()?;
            return Err(HostError::TransitionDenied(reason));
        }
    };

    let max_seq_row = tx.query_one(
        "SELECT COALESCE(MAX(seq), 0)
         FROM workflow_log
         WHERE workflow_id = $1 AND entry_kind = 'event'",
        &[&claim.workflow_id],
    )?;
    let prev_seq: i64 = max_seq_row.get(0);
    let next_seq = prev_seq
        .checked_add(1)
        .ok_or_else(|| HostError::InvariantViolation(String::from("event sequence overflow")))?;

    let stored = StoredEvent::from_kernel_event(event);
    let event_json = serde_json::to_value(&stored)?;
    let state_hash = canonical_state_hash(&next_state);
    let epoch_i64 = i64::try_from(claim.epoch.get())
        .map_err(|_| HostError::InvariantViolation(String::from("epoch exceeds i64")))?;

    tx.execute(
        "INSERT INTO workflow_log
            (workflow_id, seq, entry_kind, node_id, epoch, claimed_by, event_json, state_hash)
         VALUES ($1, $2, 'event', $3, $4, $5, $6, $7)",
        &[
            &claim.workflow_id,
            &next_seq,
            &claim.node_id,
            &epoch_i64,
            &claim.worker_id,
            &event_json,
            &state_hash,
        ],
    )?;
    tx.commit()?;

    Ok(AppendedStep {
        seq: next_seq,
        next_state,
        intents,
        state_hash,
    })
}

/// Releases a worker's lease on a step only if the stored epoch still matches
/// the worker's claim epoch (never clobbering a newer worker's lease).
pub fn release_step(client: &mut Client, claim: &StepClaim) -> Result<bool, HostError> {
    let epoch_i64 = i64::try_from(claim.epoch.get())
        .map_err(|_| HostError::InvariantViolation(String::from("epoch exceeds i64")))?;
    let updated = client.execute(
        "UPDATE workflow_log
         SET claimed_by = NULL, lease_until = NULL
         WHERE workflow_id = $1
           AND entry_kind = 'step'
           AND node_id = $2
           AND epoch = $3",
        &[&claim.workflow_id, &claim.node_id, &epoch_i64],
    )?;
    Ok(updated == 1)
}

/// Forces a claim's lease to expire immediately (useful for deterministic crash
/// takeover testing without waiting for wall-clock lease timeouts).
pub fn expire_lease_for_test(
    client: &mut Client,
    workflow_id: &str,
    node_id: &str,
) -> Result<(), HostError> {
    client.execute(
        "UPDATE workflow_log
         SET lease_until = NOW() - INTERVAL '1 second'
         WHERE workflow_id = $1 AND entry_kind = 'step' AND node_id = $2",
        &[&workflow_id, &node_id],
    )?;
    Ok(())
}

/// Returns true iff every node in the replayed workflow is in a terminal state.
#[must_use]
pub fn all_terminal(state: &MachineState) -> bool {
    state
        .entries()
        .values()
        .all(|local_state| is_terminal(*local_state))
}
