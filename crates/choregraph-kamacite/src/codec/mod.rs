// The pure codec adapter mirrors the selected Kamacite API and canonical schema paths.
#![allow(
    unknown_lints,
    ambiguous_params,
    missing_const_fn,
    non_trait_imports,
    path_segment_repetition,
    reason = "the pure codec adapter mirrors the selected Kamacite API and canonical schema paths"
)]

// r[impl choregraph.interchange.kamacite.adapter]
// r[impl choregraph.interchange.kamacite.preserves]
use crate::selected_kamacite_codec::{
    PreservesCodecError, PreservesLimits, PreservesSchema, PreservesSyntax, canonicalize_preserves,
    decode_typed_preserves, encode_typed_preserves,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub mod effect;
pub mod graph;
pub mod state;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalArtifact {
    pub schema_version: String,
    pub canonical_codec: String,
    pub packed_bytes: Vec<u8>,
    pub text_projection: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodecBoundaryError {
    JsonProjectionIsNotAuthority,
    Preserves(String),
}

impl std::fmt::Display for CodecBoundaryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::Preserves(message) = self {
            return formatter.write_str(message);
        }
        const DIAGNOSTICS: [(CodecBoundaryError, &str); 1] = [(
            CodecBoundaryError::JsonProjectionIsNotAuthority,
            "JSON projection cannot replace canonical packed Preserves",
        )];
        if let Some((_, message)) = DIAGNOSTICS.iter().find(|(category, _)| category == self) {
            formatter.write_str(message)
        } else {
            // New categories retain their exact variant and payload in diagnostics.
            std::fmt::Debug::fmt(self, formatter)
        }
    }
}

impl std::error::Error for CodecBoundaryError {}

fn schema<'a>(schema_version: &'a str, root_label: &'a str) -> PreservesSchema<'a> {
    PreservesSchema {
        schema_version,
        root_label,
    }
}

fn public_artifact(
    artifact: crate::selected_kamacite_codec::CanonicalPreservesArtifact,
) -> CanonicalArtifact {
    CanonicalArtifact {
        schema_version: artifact.schema_version,
        canonical_codec: artifact.canonical_codec,
        packed_bytes: artifact.packed_bytes,
        text_projection: artifact.text_projection,
    }
}

fn map_codec_error(error: PreservesCodecError) -> CodecBoundaryError {
    CodecBoundaryError::Preserves(error.to_string())
}

fn looks_like_json(input: &[u8]) -> bool {
    let Some(first) = input
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
    else {
        return false;
    };
    matches!(first, b'{' | b'[')
}

pub fn encode<T: Serialize>(
    value: &T,
    schema_version: &str,
    root_label: &str,
) -> Result<CanonicalArtifact, CodecBoundaryError> {
    encode_typed_preserves(
        value,
        schema(schema_version, root_label),
        PreservesLimits::default(),
    )
    .map(public_artifact)
    .map_err(map_codec_error)
}

pub fn decode<T: DeserializeOwned>(
    input: &[u8],
    schema_version: &str,
    root_label: &str,
) -> Result<(T, CanonicalArtifact), CodecBoundaryError> {
    if looks_like_json(input) {
        return Err(CodecBoundaryError::JsonProjectionIsNotAuthority);
    }
    let internal = canonicalize_preserves(
        input,
        PreservesSyntax::Packed,
        schema(schema_version, root_label),
        PreservesLimits::default(),
    )
    .map_err(map_codec_error)?;
    let value = decode_typed_preserves(&internal).map_err(map_codec_error)?;
    Ok((value, public_artifact(internal)))
}
