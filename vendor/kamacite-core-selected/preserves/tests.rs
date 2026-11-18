use super::*;
use serde::{Deserialize, Serialize};

const TEST_SCHEMA_VERSION: &str = "test.record.v1";
const TEST_ROOT_LABEL: &str = "BoundRecord";
const TEST_TEXT: &str = "<BoundRecord \"alpha\" [1 2]>";
const ALTERNATE_TEST_TEXT: &str = "<BoundRecord\n  \"alpha\"\n  [1, 2]\n>";
const EXPECTED_ITEM_COUNT: usize = 2;
const SHALLOW_DEPTH_LIMIT: usize = 2;
const SMALL_COLLECTION_LIMIT: usize = 1;
const SMALL_ATOM_LIMIT: usize = 2;
const SMALL_INPUT_LIMIT: usize = 8;
const SMALL_OUTPUT_LIMIT: usize = 8;
const LEGACY_PREFIXED_JSON_INPUT: &[u8] =
    b"preserves-canonical-binary-v1:{\"name\":\"alpha\",\"values\":[1,2]}";
const NON_CANONICAL_RECORD: &[u8] = &[
    0xb4, 0xb3, 0x0b, b'B', b'o', b'u', b'n', b'd', b'R', b'e', b'c', b'o', b'r', b'd', 0xb1, 0x05,
    b'a', b'l', b'p', b'h', b'a', 0xb5, 0xb0, 0x02, 0x00, 0x01, 0xb0, 0x01, 0x02, 0x84, 0x84,
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct BoundRecord {
    name: String,
    values: Vec<u64>,
}

fn schema() -> PreservesSchema<'static> {
    PreservesSchema {
        schema_version: TEST_SCHEMA_VERSION,
        root_label: TEST_ROOT_LABEL,
    }
}

fn expected() -> BoundRecord {
    BoundRecord {
        name: "alpha".to_string(),
        values: vec![1, 2],
    }
}

#[test]
fn text_and_packed_transfer_syntaxes_converge() {
    let first = canonicalize_preserves(
        TEST_TEXT.as_bytes(),
        PreservesSyntax::Text,
        schema(),
        PreservesLimits::default(),
    );
    let alternate = canonicalize_preserves(
        ALTERNATE_TEST_TEXT.as_bytes(),
        PreservesSyntax::Text,
        schema(),
        PreservesLimits::default(),
    );
    assert!(first.is_ok(), "first equivalent value must canonicalize");
    assert!(
        alternate.is_ok(),
        "alternate equivalent value must canonicalize"
    );
    let (Ok(first), Ok(alternate)) = (first, alternate) else {
        return;
    };
    assert_eq!(first.packed_bytes, alternate.packed_bytes);
    let packed = canonicalize_preserves(
        &first.packed_bytes,
        PreservesSyntax::Packed,
        schema(),
        PreservesLimits::default(),
    );
    assert!(matches!(&packed, Ok(artifact) if artifact.packed_bytes == first.packed_bytes));
}

#[test]
fn typed_roundtrip_binds_normative_fields() {
    let encoded = encode_typed_preserves(&expected(), schema(), PreservesLimits::default());
    assert!(encoded.is_ok(), "typed test value must encode");
    let Ok(encoded) = encoded else {
        return;
    };
    let decoded: Result<BoundRecord, _> = decode_typed_preserves(&encoded);
    assert_eq!(decoded, Ok(expected()));
    let mismatched = BoundRecord {
        name: "other".to_string(),
        values: vec![1, 2],
    };
    let verification = verify_typed_preserves(
        &encoded.packed_bytes,
        PreservesSyntax::Packed,
        &mismatched,
        schema(),
        PreservesLimits::default(),
    );
    assert_eq!(
        verification.err(),
        Some(PreservesCodecError::TypedContentMismatch)
    );
}

#[test]
fn rejects_malformed_trailing_and_unsupported_schema_inputs() {
    let malformed = canonicalize_preserves(
        b"<BoundRecord",
        PreservesSyntax::Text,
        schema(),
        PreservesLimits::default(),
    );
    assert!(matches!(malformed, Err(PreservesCodecError::Malformed(_))));

    let invalid_utf8 = canonicalize_preserves(
        &[u8::MAX],
        PreservesSyntax::Text,
        schema(),
        PreservesLimits::default(),
    );
    assert!(matches!(
        invalid_utf8,
        Err(PreservesCodecError::Malformed(_))
    ));

    let trailing = canonicalize_preserves(
        b"<BoundRecord \"alpha\" [1 2]> extra",
        PreservesSyntax::Text,
        schema(),
        PreservesLimits::default(),
    );
    assert!(matches!(trailing, Err(PreservesCodecError::Malformed(_))));

    let wrong_schema = canonicalize_preserves(
        b"<Other \"alpha\" [1 2]>",
        PreservesSyntax::Text,
        schema(),
        PreservesLimits::default(),
    );
    assert!(matches!(
        wrong_schema,
        Err(PreservesCodecError::UnsupportedSchema { .. })
    ));
}

#[test]
fn rejects_annotations_embedded_values_and_noncanonical_packed() {
    let annotation = canonicalize_preserves(
        b"@note <BoundRecord \"alpha\" [1 2]>",
        PreservesSyntax::Text,
        schema(),
        PreservesLimits::default(),
    );
    assert_eq!(
        annotation.err(),
        Some(PreservesCodecError::UnsupportedAnnotation)
    );

    let embedded = canonicalize_preserves(
        b"<BoundRecord #:\"opaque\" [1 2]>",
        PreservesSyntax::Text,
        schema(),
        PreservesLimits::default(),
    );
    assert_eq!(
        embedded.err(),
        Some(PreservesCodecError::UnsupportedEmbeddedValue)
    );

    let noncanonical = canonicalize_preserves(
        NON_CANONICAL_RECORD,
        PreservesSyntax::Packed,
        PreservesSchema {
            schema_version: TEST_SCHEMA_VERSION,
            root_label: TEST_ROOT_LABEL,
        },
        PreservesLimits::default(),
    );
    assert_eq!(
        noncanonical.err(),
        Some(PreservesCodecError::NonCanonicalPacked)
    );
}

#[test]
fn rejects_legacy_prefixed_json_as_canonical_preserves() {
    let result = canonicalize_preserves(
        LEGACY_PREFIXED_JSON_INPUT,
        PreservesSyntax::Text,
        schema(),
        PreservesLimits::default(),
    );

    assert!(matches!(result, Err(PreservesCodecError::Malformed(_))));
}

#[test]
fn enforces_every_named_limit_without_emitting_an_artifact() {
    let base = PreservesLimits::default();
    let input = canonicalize_preserves(
        TEST_TEXT.as_bytes(),
        PreservesSyntax::Text,
        schema(),
        PreservesLimits {
            max_input_bytes: SMALL_INPUT_LIMIT,
            ..base
        },
    );
    assert!(matches!(input, Err(PreservesCodecError::InputLimit { .. })));

    let depth = canonicalize_preserves(
        TEST_TEXT.as_bytes(),
        PreservesSyntax::Text,
        schema(),
        PreservesLimits {
            max_depth_count: SHALLOW_DEPTH_LIMIT,
            ..base
        },
    );
    assert!(matches!(depth, Err(PreservesCodecError::DepthLimit { .. })));

    let collection = canonicalize_preserves(
        TEST_TEXT.as_bytes(),
        PreservesSyntax::Text,
        schema(),
        PreservesLimits {
            max_collection_item_count: SMALL_COLLECTION_LIMIT,
            ..base
        },
    );
    assert!(matches!(
        collection,
        Err(PreservesCodecError::CollectionLimit { .. })
    ));

    let atom = canonicalize_preserves(
        TEST_TEXT.as_bytes(),
        PreservesSyntax::Text,
        schema(),
        PreservesLimits {
            max_atom_bytes: SMALL_ATOM_LIMIT,
            ..base
        },
    );
    assert!(matches!(atom, Err(PreservesCodecError::AtomLimit { .. })));

    let values = canonicalize_preserves(
        TEST_TEXT.as_bytes(),
        PreservesSyntax::Text,
        schema(),
        PreservesLimits {
            max_value_count: EXPECTED_ITEM_COUNT,
            ..base
        },
    );
    assert!(matches!(
        values,
        Err(PreservesCodecError::ValueLimit { .. })
    ));

    let output = canonicalize_preserves(
        TEST_TEXT.as_bytes(),
        PreservesSyntax::Text,
        schema(),
        PreservesLimits {
            max_output_bytes: SMALL_OUTPUT_LIMIT,
            ..base
        },
    );
    assert!(matches!(
        output,
        Err(PreservesCodecError::OutputLimit { .. })
    ));
}

#[test]
fn input_bound_failure_precedes_depth_failure() {
    let result = canonicalize_preserves(
        TEST_TEXT.as_bytes(),
        PreservesSyntax::Text,
        schema(),
        PreservesLimits {
            max_input_bytes: 1,
            max_depth_count: 1,
            ..PreservesLimits::default()
        },
    );
    assert_eq!(
        result.err(),
        Some(PreservesCodecError::InputLimit {
            actual_bytes: TEST_TEXT.len(),
            limit_bytes: 1
        })
    );
}

#[test]
fn rejects_zero_limit_policy() {
    let result = canonicalize_preserves(
        TEST_TEXT.as_bytes(),
        PreservesSyntax::Text,
        schema(),
        PreservesLimits {
            max_depth_count: 0,
            ..PreservesLimits::default()
        },
    );
    assert!(matches!(result, Err(PreservesCodecError::InvalidPolicy(_))));
}
