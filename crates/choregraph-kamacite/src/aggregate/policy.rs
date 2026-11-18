use crate::aggregate::assembly::AggregateError;
use crate::codec::graph::{MACHINE_SCHEMA_VERSION, PROFILE_ROOT_LABEL, PROFILE_SCHEMA_VERSION};
use crate::codec::{CanonicalArtifact, encode};
use crate::identity::{ArtifactDomain, ArtifactIdentity, compute_identity};
use crate::lifecycle::conversion::LIFECYCLE_FSM_ID;
use serde::{Deserialize, Serialize};

pub const KAMACITE_REVISION: &str = "6245a52498f7de511df660eabec3f9b350338cb6";
pub const EXECUTION_GRAPH_REVISION: &str = "c415ddf2c8252dd9ba6cb267a4e6346158069823";
pub const LATTICE_REVISION: &str = "c8d56ad83a57287064edb85f58808f53e4bf4536";
pub const GLOBAL_STATE_MODEL: &str = "parameterized-map-from-node-id-to-local-state";
pub const GLOBAL_STATE_CARDINALITY: &str = "not-enumerated";
const PROFILE_SCHEMA_NUMBER: u32 = 1;
const KAMACITE_LICENSE: &str = "MIT OR Apache-2.0";
const MACHINE_SCHEMA_SOURCE_BLAKE3: &str =
    "4cb7079deb17e77fe2806bd24a80f47e5df1ca7c405725429a3ea47236ecb0f6";
const LINK_RECEIPT_SCHEMA_SOURCE_BLAKE3: &str =
    "35fe04687697fe9d8027f924aade0a2ddccb65f5b9843a17c8692b8fa93212c1";
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-kamacite-profile-v1", deny_unknown_fields)]
pub struct SupportedKamaciteProfile {
    pub schema_version: u32,
    pub kamacite_revision: String,
    pub kamacite_license: String,
    pub canonical_codec: String,
    pub preserves_package_version: String,
    pub preserves_spec_version: String,
    pub machine_schema: String,
    pub machine_schema_source_blake3: String,
    pub link_receipt_schema_source_blake3: String,
    pub fsm_spec_schema: String,
    pub fsm_receipt_schema: String,
    pub fsm_determinism_policy: String,
    pub fsm_receipt_kinds: Vec<String>,
    pub fsm_denial_no_effect_class: String,
    pub lifecycle_fsm_id: String,
    pub authority: String,
    pub non_claims: Vec<String>,
}
fn profile() -> SupportedKamaciteProfile {
    SupportedKamaciteProfile {
        schema_version: PROFILE_SCHEMA_NUMBER,
        kamacite_revision: String::from(KAMACITE_REVISION),
        kamacite_license: String::from(KAMACITE_LICENSE),
        canonical_codec: String::from(
            crate::selected_kamacite_codec::profile::PRESERVES_CANONICAL_CODEC,
        ),
        preserves_package_version: String::from(
            crate::selected_kamacite_codec::profile::PRESERVES_PACKAGE_VERSION,
        ),
        preserves_spec_version: String::from(
            crate::selected_kamacite_codec::profile::PRESERVES_SPEC_VERSION,
        ),
        machine_schema: String::from(MACHINE_SCHEMA_VERSION),
        machine_schema_source_blake3: String::from(MACHINE_SCHEMA_SOURCE_BLAKE3),
        link_receipt_schema_source_blake3: String::from(LINK_RECEIPT_SCHEMA_SOURCE_BLAKE3),
        fsm_spec_schema: String::from(crate::codec::state::FSM_SPEC_SCHEMA_VERSION),
        fsm_receipt_schema: String::from(crate::codec::state::FSM_RECEIPT_SCHEMA_VERSION),
        fsm_determinism_policy: String::from("deterministic"),
        fsm_receipt_kinds: vec![String::from("exhaustiveness"), String::from("trace")],
        fsm_denial_no_effect_class: String::from("no-side-effect"),
        lifecycle_fsm_id: String::from(LIFECYCLE_FSM_ID),
        authority: String::from("canonical-packed-preserves"),
        non_claims: vec![
            String::from("profile identity does not prove transition correctness"),
            String::from("profile identity does not prove source equivalence"),
            String::from("profile identity does not authorize runtime effects"),
        ],
    }
}

pub fn build_profile_artifact() -> Result<
    (
        SupportedKamaciteProfile,
        CanonicalArtifact,
        ArtifactIdentity,
    ),
    AggregateError,
> {
    let profile = profile();
    let artifact = encode(&profile, PROFILE_SCHEMA_VERSION, PROFILE_ROOT_LABEL)?;
    let identity = compute_identity(ArtifactDomain::Profile, &artifact.packed_bytes)?;
    Ok((profile, artifact, identity))
}
