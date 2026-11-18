use crate::concrete::event::ConcreteEvent;
use crate::model::automaton::{AdmittedMachine, MachineState};
use crate::model::history::{ReplayBindings, ReplayLimits};

/// An ordered v2 event slice with exact canonical identity bindings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConcreteBoundedTrace<'a> {
    bindings: ReplayBindings,
    events: &'a [ConcreteEvent],
}

impl<'a> ConcreteBoundedTrace<'a> {
    #[must_use]
    pub const fn new(bindings: ReplayBindings, events: &'a [ConcreteEvent]) -> Self {
        Self { bindings, events }
    }

    #[must_use]
    pub const fn bindings(&self) -> ReplayBindings {
        self.bindings
    }

    #[must_use]
    pub const fn events(&self) -> &'a [ConcreteEvent] {
        self.events
    }
}

/// All explicit inputs to one deterministic v2 replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConcreteReplayInput<'a> {
    pub(crate) machine: &'a AdmittedMachine,
    pub(crate) initial_state: &'a MachineState,
    pub(crate) expected_bindings: ReplayBindings,
    pub(crate) trace: ConcreteBoundedTrace<'a>,
    pub(crate) limits: ReplayLimits,
}

impl<'a> ConcreteReplayInput<'a> {
    #[must_use]
    pub const fn new(
        machine: &'a AdmittedMachine,
        initial_state: &'a MachineState,
        expected_bindings: ReplayBindings,
        trace: ConcreteBoundedTrace<'a>,
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
