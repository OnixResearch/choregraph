#![allow(
    unknown_lints,
    ambiguous_params,
    non_trait_imports,
    reason = "the canonical codec boundary keeps explicit schema and root inputs beside source-identical Kamacite types"
)]

use serde::Serialize;
use serde::de::DeserializeOwned;

use choregraph_kamacite::{CanonicalArtifact, CodecBoundaryError};

#[allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    dead_code,
    explicit_defaults,
    fragile_exhaustive_enum_match,
    function_length,
    missing_const_fn,
    non_trait_imports,
    path_segment_repetition,
    renamed_imports,
    unbounded_collection_growth,
    reason = "the reviewed Kamacite codec stays source-identical across canonical adapters"
)]
#[path = "../../../vendor/kamacite-core-selected/preserves/codec.rs"]
mod selected_codec;

fn schema<'a>(schema_version: &'a str, root_label: &'a str) -> selected_codec::PreservesSchema<'a> {
    selected_codec::PreservesSchema {
        schema_version,
        root_label,
    }
}

fn public_artifact(artifact: selected_codec::CanonicalPreservesArtifact) -> CanonicalArtifact {
    CanonicalArtifact {
        schema_version: artifact.schema_version,
        canonical_codec: artifact.canonical_codec,
        packed_bytes: artifact.packed_bytes,
        text_projection: artifact.text_projection,
    }
}

fn map_error(error: selected_codec::PreservesCodecError) -> CodecBoundaryError {
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

pub(crate) fn encode_preserves<T: Serialize>(
    value: &T,
    schema_version: &str,
    root_label: &str,
) -> Result<CanonicalArtifact, CodecBoundaryError> {
    selected_codec::encode_typed_preserves(
        value,
        schema(schema_version, root_label),
        selected_codec::PreservesLimits::default(),
    )
    .map(public_artifact)
    .map_err(map_error)
}

pub(crate) fn decode_preserves<T: DeserializeOwned>(
    input: &[u8],
    schema_version: &str,
    root_label: &str,
) -> Result<(T, CanonicalArtifact), CodecBoundaryError> {
    if looks_like_json(input) {
        return Err(CodecBoundaryError::JsonProjectionIsNotAuthority);
    }
    let internal = selected_codec::canonicalize_preserves(
        input,
        selected_codec::PreservesSyntax::Packed,
        schema(schema_version, root_label),
        selected_codec::PreservesLimits::default(),
    )
    .map_err(map_error)?;
    let value = selected_codec::decode_typed_preserves(&internal).map_err(map_error)?;
    Ok((value, public_artifact(internal)))
}
