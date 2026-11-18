// Verus proof syntax and production-linked specifications require these local source shapes.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    clippy::assign_op_pattern,
    dead_code,
    nested_conditionals,
    non_trait_imports,
    raw_arithmetic_overflow,
    too_many_parameters,
    unbounded_collection_growth,
    usize_in_public_api,
    reason = "Verus proof syntax and production-linked proof witnesses require these local source shapes"
)]

// r[impl choregraph.proof.single_step.specifications]
// r[impl choregraph.proof.single_step.production_linkage]
// r[impl choregraph.proof.single_step.readiness]
// r[impl choregraph.proof.single_step.readiness_completeness]
// r[impl choregraph.proof.single_step.readiness_frontier]
// r[impl choregraph.proof.single_step.determinism]
// r[impl choregraph.proof.single_step.preservation]
// r[impl choregraph.proof.single_step.denial]
// r[impl choregraph.proof.single_step.trellis]
// r[impl choregraph.effect_v2.proof]
// r[verify choregraph.effect_v2.proof]
#[path = "verified/codes.rs"]
pub(crate) mod codes;

#[path = "verified/primitive.rs"]
pub(crate) mod primitive;

#[path = "verified/kernel.rs"]
pub(crate) mod kernel;

#[path = "verified/readiness.rs"]
pub(crate) mod readiness;

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::codes::action::ACTION_CANCEL;
    use super::codes::action::ACTION_MARK_READY;
    use super::codes::action::ACTION_RESUME_SIGNAL;
    use super::codes::action::ACTION_SUSPEND;
    use super::codes::denial::suspension::DENIAL_PROFILE_ACTION;
    use super::codes::denial::suspension::DENIAL_SUSPENSION_EFFECTS;
    use super::codes::denial::suspension::DENIAL_SUSPENSION_MISMATCH;
    use super::codes::denial::transition::DENIAL_DEPENDENCIES_INCOMPLETE;
    use super::codes::denial::transition::DENIAL_DUPLICATE_EFFECT;
    use super::codes::denial::transition::DENIAL_NONE;
    use super::codes::profile::PROFILE_DURABLE_SUSPENSION_V1;
    use super::codes::profile::PROFILE_LINEAR_V1;
    use super::codes::status::STATE_COMPLETED;
    use super::codes::status::STATE_PENDING;
    use super::codes::status::STATE_READY;
    use super::codes::status::STATE_RUNNING;
    use super::codes::status::STATE_SUSPENDED;
    use super::kernel::decision::indexed_step;
    use super::primitive::support::IndexedSuspensionFact;

    const DECLARED_INTENT_INDEX: usize = 3;

    #[test]
    fn indexed_step_accepts_ready_transition_with_declared_intent() {
        let dependencies = vec![vec![], vec![0]];
        let states = vec![STATE_COMPLETED, STATE_PENDING];
        let suspensions = vec![None, None];
        let allowed = vec![DECLARED_INTENT_INDEX];
        let requested = vec![DECLARED_INTENT_INDEX];

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_LINEAR_V1,
            1,
            ACTION_MARK_READY,
            &None,
            &allowed,
            &requested,
        );

        assert!(decision.accepted);
        assert_eq!(decision.reason, DENIAL_NONE);
        assert_eq!(decision.next_states, vec![STATE_COMPLETED, STATE_READY]);
        assert_eq!(decision.intent_indices, requested);
    }

    #[test]
    fn indexed_step_denies_incomplete_dependency_without_outputs() {
        let dependencies = vec![vec![], vec![0]];
        let states = vec![STATE_RUNNING, STATE_PENDING];
        let suspensions = vec![None, None];

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_LINEAR_V1,
            1,
            ACTION_MARK_READY,
            &None,
            &[],
            &[],
        );

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_DEPENDENCIES_INCOMPLETE);
        assert!(decision.next_states.is_empty());
        assert!(decision.intent_indices.is_empty());
    }

    #[test]
    fn indexed_step_denies_duplicate_intents_without_outputs() {
        let dependencies = vec![vec![]];
        let states = vec![STATE_PENDING];
        let suspensions = vec![None];
        let allowed = vec![DECLARED_INTENT_INDEX];
        let requested = vec![DECLARED_INTENT_INDEX, DECLARED_INTENT_INDEX];

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_LINEAR_V1,
            0,
            ACTION_CANCEL,
            &None,
            &allowed,
            &requested,
        );

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_DUPLICATE_EFFECT);
        assert!(decision.next_states.is_empty());
        assert!(decision.intent_indices.is_empty());
    }

    fn suspension_fact(class: &[u8], reference: &[u8]) -> Option<IndexedSuspensionFact> {
        Some(IndexedSuspensionFact {
            class: class.to_vec(),
            reference: reference.to_vec(),
        })
    }

    #[test]
    fn durable_profile_suspend_records_fact_without_effects() {
        let dependencies = vec![vec![]];
        let states = vec![STATE_READY];
        let suspensions = vec![None];
        let fact = suspension_fact(b"approval", b"blake3:00");

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_DURABLE_SUSPENSION_V1,
            0,
            ACTION_SUSPEND,
            &fact,
            &[],
            &[],
        );

        assert!(decision.accepted);
        assert_eq!(decision.next_states, vec![STATE_SUSPENDED]);
        assert!(decision.intent_indices.is_empty());
    }

    #[test]
    fn durable_profile_matching_resume_clears_fact() {
        let dependencies = vec![vec![]];
        let fact = suspension_fact(b"approval", b"blake3:00");
        let states = vec![STATE_SUSPENDED];
        let suspensions = vec![fact.clone()];

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_DURABLE_SUSPENSION_V1,
            0,
            ACTION_RESUME_SIGNAL,
            &fact,
            &[],
            &[],
        );

        assert!(decision.accepted);
        assert_eq!(decision.next_states, vec![STATE_READY]);
        assert!(decision.intent_indices.is_empty());
    }

    #[test]
    fn durable_profile_mismatched_resume_denies() {
        let dependencies = vec![vec![]];
        let states = vec![STATE_SUSPENDED];
        let suspensions = vec![suspension_fact(b"approval", b"blake3:00")];
        let wrong = suspension_fact(b"approval", b"blake3:ff");

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_DURABLE_SUSPENSION_V1,
            0,
            ACTION_RESUME_SIGNAL,
            &wrong,
            &[],
            &[],
        );

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_SUSPENSION_MISMATCH);
        assert!(decision.next_states.is_empty());
    }

    #[test]
    fn linear_profile_rejects_suspension_actions() {
        let dependencies = vec![vec![]];
        let states = vec![STATE_READY];
        let suspensions = vec![None];
        let fact = suspension_fact(b"approval", b"blake3:00");

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_LINEAR_V1,
            0,
            ACTION_SUSPEND,
            &fact,
            &[],
            &[],
        );

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_PROFILE_ACTION);
    }

    #[test]
    fn suspension_action_rejects_effect_requests() {
        let dependencies = vec![vec![]];
        let states = vec![STATE_READY];
        let suspensions = vec![None];
        let fact = suspension_fact(b"approval", b"blake3:00");
        let allowed = vec![DECLARED_INTENT_INDEX];
        let requested = vec![DECLARED_INTENT_INDEX];

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_DURABLE_SUSPENSION_V1,
            0,
            ACTION_SUSPEND,
            &fact,
            &allowed,
            &requested,
        );

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_SUSPENSION_EFFECTS);
    }
}
