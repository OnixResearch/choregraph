use super::{SOURCE_ENVELOPE, durable_machine, must};
use crate::aggregate::assembly::{
    AggregateError, durable_machine_descriptor, encode_durable_machine_descriptor,
    validate_durable_machine_artifact,
};
use crate::aggregate::policy::build_profile_artifact;
use crate::codec::encode;
use crate::codec::state::{FSM_SPEC_ROOT_LABEL, FSM_SPEC_SCHEMA_VERSION};
use crate::identity::{KamaciteIdentityDomain, compute_kamacite_identity};
use crate::lifecycle::conversion::{
    LifecycleError, build_durable_lifecycle_artifact, build_durable_lifecycle_artifact_for_profile,
    validate_durable_lifecycle_artifact,
};
use choregraph_core::model::semantics::{DURABLE_SUSPENSION_V2, LifecycleProfile};

#[test]
fn round_trips_with_an_unguarded_effect_free_suspended_cancel() {
    let machine = durable_machine(DURABLE_SUSPENSION_V2);
    let lifecycle = must(build_durable_lifecycle_artifact_for_profile(
        machine.lifecycle_profile(),
        None,
    ));
    let legacy = must(build_durable_lifecycle_artifact(None));
    assert_ne!(lifecycle.project_identity, legacy.project_identity);
    assert_ne!(lifecycle.identity, legacy.identity);
    let decoded = must(validate_durable_lifecycle_artifact(
        &lifecycle.canonical.packed_bytes,
    ));
    assert_eq!(decoded, lifecycle);
    assert_eq!(decoded.descriptor.fsm_id, DURABLE_SUSPENSION_V2);
    assert!(decoded.descriptor.transitions.iter().any(|row| {
        row.from_state == "suspended"
            && row.event == "cancel"
            && row.to_state == "cancelled"
            && row.guards.is_empty()
            && row.side_effect == "none"
    }));
    assert!(decoded.descriptor.transitions.iter().any(|row| {
        row.from_state == "running"
            && row.event == "cancel"
            && row.side_effect == "machine-declared-effect-intents"
    }));

    let (_, _, profile_identity) = must(build_profile_artifact());
    let descriptor = must(durable_machine_descriptor(
        &machine,
        decoded.project_identity.clone(),
        profile_identity.clone(),
    ));
    let artifact = must(encode_durable_machine_descriptor(&descriptor));
    let validated = must(validate_durable_machine_artifact(
        &artifact.packed_bytes,
        &decoded,
        &profile_identity,
    ));
    assert_eq!(validated.canonical, artifact);
    assert_eq!(
        validated.descriptor.lifecycle_profile_id,
        DURABLE_SUSPENSION_V2
    );
    assert_eq!(validated.machine, machine);

    let other_lifecycle = must(build_durable_lifecycle_artifact_for_profile(
        machine.lifecycle_profile(),
        Some(compute_kamacite_identity(
            KamaciteIdentityDomain::Source,
            SOURCE_ENVELOPE,
        )),
    ));
    assert!(matches!(
        validate_durable_machine_artifact(
            &artifact.packed_bytes,
            &other_lifecycle,
            &profile_identity
        ),
        Err(AggregateError::UnsupportedProfile)
    ));
}

#[test]
fn rejects_a_relabeled_or_mismatched_lifecycle() {
    let legacy = must(build_durable_lifecycle_artifact(None));
    let mut mislabeled = legacy.descriptor.clone();
    mislabeled.fsm_id = String::from(DURABLE_SUSPENSION_V2);
    let mislabeled = must(encode(
        &mislabeled,
        FSM_SPEC_SCHEMA_VERSION,
        FSM_SPEC_ROOT_LABEL,
    ));
    assert!(matches!(
        validate_durable_lifecycle_artifact(&mislabeled.packed_bytes),
        Err(LifecycleError::ProfileMismatch)
    ));
    assert!(matches!(
        build_durable_lifecycle_artifact_for_profile(LifecycleProfile::LinearV1, None),
        Err(LifecycleError::UnsupportedProfile)
    ));

    let machine = durable_machine(DURABLE_SUSPENSION_V2);
    let (_, _, profile_identity) = must(build_profile_artifact());
    let descriptor = must(durable_machine_descriptor(
        &machine,
        legacy.project_identity.clone(),
        profile_identity.clone(),
    ));
    let artifact = must(encode_durable_machine_descriptor(&descriptor));
    assert!(matches!(
        validate_durable_machine_artifact(&artifact.packed_bytes, &legacy, &profile_identity),
        Err(AggregateError::UnsupportedProfile)
    ));
}
