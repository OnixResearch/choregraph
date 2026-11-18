/// A core lifecycle value not representable by the legacy vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversionError {
    SuspendedState,
    SuspensionAction,
}

/// Legacy lifecycle states mapped to Choregraph production states.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StepState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Legacy lifecycle actions mapped to Choregraph production actions.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StepAction {
    MarkReady,
    Start,
    Complete,
    Fail,
    Cancel,
}

/// Legacy local-transition error categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepTransitionError {
    InvalidTransition { from: StepState, action: StepAction },
    TerminalState { from: StepState, action: StepAction },
}
#[allow(
    unknown_lints,
    fragile_exhaustive_enum_match,
    reason = "the closed compatibility lifecycle requires total mapping to the verified core vocabulary"
)]
pub(super) const fn core_state(state: StepState) -> choregraph_core::model::semantics::LocalState {
    match state {
        StepState::Pending => choregraph_core::model::semantics::LocalState::Pending,
        StepState::Ready => choregraph_core::model::semantics::LocalState::Ready,
        StepState::Running => choregraph_core::model::semantics::LocalState::Running,
        StepState::Completed => choregraph_core::model::semantics::LocalState::Completed,
        StepState::Failed => choregraph_core::model::semantics::LocalState::Failed,
        StepState::Cancelled => choregraph_core::model::semantics::LocalState::Cancelled,
    }
}

pub(super) const fn compatibility_state(
    state: choregraph_core::model::semantics::LocalState,
) -> Result<StepState, ConversionError> {
    match state {
        choregraph_core::model::semantics::LocalState::Pending => Ok(StepState::Pending),
        choregraph_core::model::semantics::LocalState::Ready => Ok(StepState::Ready),
        choregraph_core::model::semantics::LocalState::Running => Ok(StepState::Running),
        choregraph_core::model::semantics::LocalState::Completed => Ok(StepState::Completed),
        choregraph_core::model::semantics::LocalState::Failed => Ok(StepState::Failed),
        choregraph_core::model::semantics::LocalState::Cancelled => Ok(StepState::Cancelled),
        choregraph_core::model::semantics::LocalState::Suspended => {
            Err(ConversionError::SuspendedState)
        }
    }
}

pub(super) const fn compatibility_action(
    action: choregraph_core::model::semantics::Action,
) -> Result<StepAction, ConversionError> {
    match action {
        choregraph_core::model::semantics::Action::MarkReady => Ok(StepAction::MarkReady),
        choregraph_core::model::semantics::Action::Start => Ok(StepAction::Start),
        choregraph_core::model::semantics::Action::Complete => Ok(StepAction::Complete),
        choregraph_core::model::semantics::Action::Fail => Ok(StepAction::Fail),
        choregraph_core::model::semantics::Action::Cancel => Ok(StepAction::Cancel),
        choregraph_core::model::semantics::Action::Suspend
        | choregraph_core::model::semantics::Action::ResumeSignal => {
            Err(ConversionError::SuspensionAction)
        }
    }
}

#[allow(
    unknown_lints,
    fragile_exhaustive_enum_match,
    reason = "the closed compatibility lifecycle requires total mapping to the verified core vocabulary"
)]
pub(super) const fn core_action(action: StepAction) -> choregraph_core::model::semantics::Action {
    match action {
        StepAction::MarkReady => choregraph_core::model::semantics::Action::MarkReady,
        StepAction::Start => choregraph_core::model::semantics::Action::Start,
        StepAction::Complete => choregraph_core::model::semantics::Action::Complete,
        StepAction::Fail => choregraph_core::model::semantics::Action::Fail,
        StepAction::Cancel => choregraph_core::model::semantics::Action::Cancel,
    }
}

/// Applies one local compatibility transition through the verified production lifecycle kernel.
pub const fn transition_step_state(
    from: StepState,
    action: StepAction,
) -> Result<StepState, StepTransitionError> {
    match choregraph_core::transition::lifecycle_target(core_state(from), core_action(action)) {
        Ok(target) => match compatibility_state(target) {
            Ok(state) => Ok(state),
            Err(_) => Err(StepTransitionError::InvalidTransition { from, action }),
        },
        Err(choregraph_core::model::semantics::LocalTransitionError::Terminal { .. }) => {
            Err(StepTransitionError::TerminalState { from, action })
        }
        Err(choregraph_core::model::semantics::LocalTransitionError::Invalid { .. }) => {
            Err(StepTransitionError::InvalidTransition { from, action })
        }
    }
}

/// Returns whether one compatibility state is terminal.
#[must_use]
pub const fn is_terminal_state(state: StepState) -> bool {
    matches!(
        state,
        StepState::Completed | StepState::Failed | StepState::Cancelled
    )
}

#[cfg(test)]
mod tests {
    use super::{
        ConversionError, StepAction, StepState, compatibility_action, compatibility_state,
        core_action, core_state,
    };

    #[test]
    fn legacy_states_and_actions_round_trip_without_fabrication() {
        for state in [
            StepState::Pending,
            StepState::Ready,
            StepState::Running,
            StepState::Completed,
            StepState::Failed,
            StepState::Cancelled,
        ] {
            assert_eq!(compatibility_state(core_state(state)), Ok(state));
        }
        for action in [
            StepAction::MarkReady,
            StepAction::Start,
            StepAction::Complete,
            StepAction::Fail,
            StepAction::Cancel,
        ] {
            assert_eq!(compatibility_action(core_action(action)), Ok(action));
        }
    }

    #[test]
    fn durable_values_have_explicit_conversion_failures() {
        assert_eq!(
            compatibility_state(choregraph_core::model::semantics::LocalState::Suspended),
            Err(ConversionError::SuspendedState)
        );
        assert_eq!(
            compatibility_action(choregraph_core::model::semantics::Action::Suspend),
            Err(ConversionError::SuspensionAction)
        );
        assert_eq!(
            compatibility_action(choregraph_core::model::semantics::Action::ResumeSignal),
            Err(ConversionError::SuspensionAction)
        );
    }
}
