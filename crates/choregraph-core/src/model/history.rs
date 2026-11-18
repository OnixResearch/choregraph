use crate::model::automaton::{AdmittedMachine, MachineState};
use crate::model::capacity::DEFAULT_MAX_TRACE_EVENTS;
use crate::model::dispatch::Event;
use crate::model::outcome::DenialReason;

pub const TRACE_BINDING_BYTES: usize = 32;

/// One caller-supplied canonical BLAKE3 binding.
///
/// The core compares bindings but does not compute or authorize them. A consumer adapter must
/// derive these bytes from its admitted canonical machine and initial-state artifacts.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TraceBinding([u8; TRACE_BINDING_BYTES]);

impl TraceBinding {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; TRACE_BINDING_BYTES]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn bytes(self) -> [u8; TRACE_BINDING_BYTES] {
        self.0
    }
}

/// Exact machine and initial-state identities carried by a trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayBindings {
    pub machine: TraceBinding,
    pub initial_state: TraceBinding,
}

impl ReplayBindings {
    #[must_use]
    pub const fn new(machine: TraceBinding, initial_state: TraceBinding) -> Self {
        Self {
            machine,
            initial_state,
        }
    }
}

/// Caller-selected finite replay limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayLimits {
    pub max_events: u64,
}

impl Default for ReplayLimits {
    fn default() -> Self {
        Self {
            max_events: DEFAULT_MAX_TRACE_EVENTS,
        }
    }
}

/// An ordered event slice with exact canonical identity bindings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedTrace<'a> {
    bindings: ReplayBindings,
    events: &'a [Event],
}

impl<'a> BoundedTrace<'a> {
    #[must_use]
    pub const fn new(bindings: ReplayBindings, events: &'a [Event]) -> Self {
        Self { bindings, events }
    }

    #[must_use]
    pub const fn bindings(&self) -> ReplayBindings {
        self.bindings
    }

    #[must_use]
    pub const fn events(&self) -> &'a [Event] {
        self.events
    }
}

/// All explicit inputs to one deterministic replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayInput<'a> {
    pub(crate) machine: &'a AdmittedMachine,
    pub(crate) initial_state: &'a MachineState,
    pub(crate) expected_bindings: ReplayBindings,
    pub(crate) trace: BoundedTrace<'a>,
    pub(crate) limits: ReplayLimits,
}

impl<'a> ReplayInput<'a> {
    #[must_use]
    pub const fn new(
        machine: &'a AdmittedMachine,
        initial_state: &'a MachineState,
        expected_bindings: ReplayBindings,
        trace: BoundedTrace<'a>,
        limits: ReplayLimits,
    ) -> Self {
        Self {
            machine,
            initial_state,
            expected_bindings,
            trace,
            limits,
        }
    }
}

/// The number of accepted events in a replay prefix.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConsumedCount(u64);

impl ConsumedCount {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    pub(crate) fn from_usize(value: usize) -> Self {
        match u64::try_from(value) {
            Ok(value) => Self(value),
            Err(_) => Self(u64::MAX),
        }
    }
}

/// The zero-based position of the first denied event.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EventIndex(u64);

impl EventIndex {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    pub(crate) fn from_usize(value: usize) -> Self {
        match u64::try_from(value) {
            Ok(value) => Self(value),
            Err(_) => Self(u64::MAX),
        }
    }
}

/// The state after the accepted prefix and before a denied event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LastValidState(MachineState);

impl LastValidState {
    #[must_use]
    pub fn new(state: MachineState) -> Self {
        Self(state)
    }

    #[must_use]
    pub const fn state(&self) -> &MachineState {
        &self.0
    }

    #[must_use]
    pub fn into_state(self) -> MachineState {
        self.0
    }
}

/// Stable reasons that stop replay before the full trace is consumed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayFailureReason {
    TraceLimitExceeded { observed: u64, maximum: u64 },
    MachineBindingMismatch,
    InitialStateBindingMismatch,
    InitialStateInvalid,
    TransitionDenied(DenialReason),
}

/// Exact first-failure information with the accepted-prefix state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayFailure {
    pub event_index: Option<EventIndex>,
    pub consumed: ConsumedCount,
    pub reason: ReplayFailureReason,
    pub last_valid_state: LastValidState,
}

/// Pure replay output. This type contains no effect intent or dispatch authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayDecision {
    Complete {
        final_state: MachineState,
        consumed: ConsumedCount,
    },
    Failed(ReplayFailure),
}
