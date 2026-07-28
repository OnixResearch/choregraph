// Disposable downstream fixture. Lattice retains every field that creates runtime authority.
#![allow(
    clippy::manual_unwrap_or,
    clippy::manual_unwrap_or_default,
    reason = "explicit downstream fixture branches keep fallback values visible"
)]

use choregraph_seaglass::ConcreteIntentHandoff;

pub const HANDOFF_SCHEMA: &str = "choregraph.lattice-concrete-intent-handoff.v1";
pub const NON_CLAIM: &str =
    "fixture binding does not prove policy, authority, persistence, dispatch, or external success";
const OCCURRENCE_IDENTITY_CONTEXT: &str = "lattice bound concrete intent occurrence v1";
const LENGTH_FRAME_BYTES: usize = core::mem::size_of::<u64>();
const BLAKE3_PREFIX: &str = "blake3:";
const BLAKE3_DIGEST_HEX_BYTES: usize = 64;

#[derive(Clone, Copy, Debug)]
pub struct BindingInput<'a> {
    pub handoff_schema: &'a str,
    pub intent: Option<&'a ConcreteIntentHandoff>,
    pub intent_content_identity: &'a str,
    pub machine_identity: &'a str,
    pub workflow_revision: &'a str,
    pub run_identity: &'a str,
    pub policy_identity: &'a str,
    pub authority_identity: &'a str,
    pub persistence_identity: &'a str,
    pub attempt_identity: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundIntent {
    pub declaration_key: String,
    pub request_source_ref: Option<String>,
    pub payload_source_ref: Option<String>,
    pub intent_content_identity: String,
    pub machine_identity: String,
    pub workflow_revision: String,
    pub run_identity: String,
    pub policy_identity: String,
    pub authority_identity: String,
    pub persistence_identity: String,
    pub attempt_identity: String,
    pub occurrence_identity: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingError {
    UnsupportedSchema,
    MissingChoregraphIntent,
    InvalidIdentity,
}

fn valid_identity(value: &str) -> bool {
    let Some(digest) = value.strip_prefix(BLAKE3_PREFIX) else {
        return false;
    };
    digest.len() == BLAKE3_DIGEST_HEX_BYTES
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn frame(hasher: &mut blake3::Hasher, value: &str) {
    let length = match u64::try_from(value.len()) {
        Ok(length) => length,
        Err(_) => u64::MAX,
    };
    hasher.update(&length.to_le_bytes()[..LENGTH_FRAME_BYTES]);
    hasher.update(value.as_bytes());
}

fn optional_text(value: Option<&str>) -> &str {
    match value {
        Some(value) => value,
        None => "",
    }
}

fn occurrence_identity(input: BindingInput<'_>, intent: &ConcreteIntentHandoff) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(OCCURRENCE_IDENTITY_CONTEXT);
    for value in [
        input.handoff_schema,
        input.machine_identity,
        input.workflow_revision,
        input.run_identity,
        input.intent_content_identity,
        input.policy_identity,
        input.authority_identity,
        input.persistence_identity,
        input.attempt_identity,
        intent.declaration_key.as_str(),
        optional_text(intent.request.source_ref.as_deref()),
        optional_text(intent.payload.source_ref.as_deref()),
    ] {
        frame(&mut hasher, value);
    }
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

pub fn bind(input: BindingInput<'_>) -> Result<BoundIntent, BindingError> {
    if input.handoff_schema != HANDOFF_SCHEMA {
        return Err(BindingError::UnsupportedSchema);
    }
    let intent = input.intent.ok_or(BindingError::MissingChoregraphIntent)?;
    for identity in [
        input.intent_content_identity,
        input.machine_identity,
        input.workflow_revision,
        input.run_identity,
        input.policy_identity,
        input.authority_identity,
        input.persistence_identity,
        input.attempt_identity,
    ] {
        if !valid_identity(identity) {
            return Err(BindingError::InvalidIdentity);
        }
    }
    Ok(BoundIntent {
        declaration_key: intent.declaration_key.clone(),
        request_source_ref: intent.request.source_ref.clone(),
        payload_source_ref: intent.payload.source_ref.clone(),
        intent_content_identity: String::from(input.intent_content_identity),
        machine_identity: String::from(input.machine_identity),
        workflow_revision: String::from(input.workflow_revision),
        run_identity: String::from(input.run_identity),
        policy_identity: String::from(input.policy_identity),
        authority_identity: String::from(input.authority_identity),
        persistence_identity: String::from(input.persistence_identity),
        attempt_identity: String::from(input.attempt_identity),
        occurrence_identity: occurrence_identity(input, intent),
    })
}
