use preserves::boundary::{Item as BoundaryItem, Type as BoundaryType, start as start_boundary};
use preserves::reader::{NextToken, Reader};
use preserves::{
    Atom, BytesBinarySource, CompoundClass, IOValue, PackedReader, PackedWriter, TextReader,
    ValueClass, ValueImpl,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::io::{Error as IoError, ErrorKind, Write};

pub const PRESERVES_CANONICAL_CODEC: &str =
    "kamacite-preserves-packed-canonical-v2-preserves-rs-5.0.0-rc.7";
pub const PRESERVES_LEGACY_PREFIXED_JSON_CODEC: &str =
    "kamacite-legacy-labeled-deterministic-json-v1";
pub const PRESERVES_PACKAGE_VERSION: &str = "5.0.0-rc.7";
pub const PRESERVES_SPEC_VERSION: &str = "0.996.3";

const DEFAULT_MAX_INPUT_BYTES: usize = 1_048_576;
const DEFAULT_MAX_OUTPUT_BYTES: usize = 2_097_152;
const DEFAULT_MAX_DEPTH: usize = 64;
const DEFAULT_MAX_COLLECTION_ITEMS: usize = 4_096;
const DEFAULT_MAX_ATOM_BYTES: usize = 262_144;
const DEFAULT_MAX_VALUES: usize = 65_536;
const ROOT_DEPTH: usize = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreservesLimits {
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
    pub max_depth: usize,
    pub max_collection_items: usize,
    pub max_atom_bytes: usize,
    pub max_values: usize,
}

impl Default for PreservesLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: DEFAULT_MAX_INPUT_BYTES,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
            max_depth: DEFAULT_MAX_DEPTH,
            max_collection_items: DEFAULT_MAX_COLLECTION_ITEMS,
            max_atom_bytes: DEFAULT_MAX_ATOM_BYTES,
            max_values: DEFAULT_MAX_VALUES,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreservesSyntax {
    Text,
    Packed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreservesSchema<'a> {
    pub schema_version: &'a str,
    pub root_label: &'a str,
}

#[derive(Clone, Debug)]
pub struct CanonicalPreservesArtifact {
    pub schema_version: String,
    pub canonical_codec: String,
    pub value: IOValue,
    pub packed_bytes: Vec<u8>,
    pub text_projection: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreservesCodecError {
    InvalidPolicy(&'static str),
    InputLimit { actual: usize, limit: usize },
    OutputLimit { actual: usize, limit: usize },
    DepthLimit { actual: usize, limit: usize },
    CollectionLimit { actual: usize, limit: usize },
    AtomLimit { actual: usize, limit: usize },
    ValueLimit { actual: usize, limit: usize },
    Malformed(String),
    UnsupportedAnnotation,
    UnsupportedEmbeddedValue,
    UnsupportedSchema { expected: String, actual: String },
    NonCanonicalPacked,
    TypedDecode(String),
    TypedContentMismatch,
}

impl Display for PreservesCodecError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPolicy(field) => {
                write!(formatter, "Preserves limit {field} must be nonzero")
            }
            Self::InputLimit { actual, limit } => {
                write!(
                    formatter,
                    "Preserves input size {actual} exceeds limit {limit}"
                )
            }
            Self::OutputLimit { actual, limit } => {
                write!(
                    formatter,
                    "Preserves output size {actual} exceeds limit {limit}"
                )
            }
            Self::DepthLimit { actual, limit } => {
                write!(
                    formatter,
                    "Preserves value depth {actual} exceeds limit {limit}"
                )
            }
            Self::CollectionLimit { actual, limit } => {
                write!(
                    formatter,
                    "Preserves collection size {actual} exceeds limit {limit}"
                )
            }
            Self::AtomLimit { actual, limit } => {
                write!(
                    formatter,
                    "Preserves atom size {actual} exceeds limit {limit}"
                )
            }
            Self::ValueLimit { actual, limit } => {
                write!(
                    formatter,
                    "Preserves value count {actual} exceeds limit {limit}"
                )
            }
            Self::Malformed(message) => write!(formatter, "malformed Preserves input: {message}"),
            Self::UnsupportedAnnotation => {
                write!(
                    formatter,
                    "Preserves annotations are not admitted by this profile"
                )
            }
            Self::UnsupportedEmbeddedValue => {
                write!(
                    formatter,
                    "embedded Preserves values are not admitted by this profile"
                )
            }
            Self::UnsupportedSchema { expected, actual } => write!(
                formatter,
                "Preserves root schema label {actual:?} does not match {expected:?}"
            ),
            Self::NonCanonicalPacked => {
                write!(formatter, "packed Preserves input is not canonical")
            }
            Self::TypedDecode(message) => {
                write!(formatter, "Preserves typed decode failed: {message}")
            }
            Self::TypedContentMismatch => write!(
                formatter,
                "Preserves canonical content does not match the supplied typed artifact"
            ),
        }
    }
}

impl std::error::Error for PreservesCodecError {}

pub fn canonicalize_preserves(
    input: &[u8],
    syntax: PreservesSyntax,
    schema: PreservesSchema<'_>,
    limits: PreservesLimits,
) -> Result<CanonicalPreservesArtifact, PreservesCodecError> {
    validate_limits(limits)?;
    if input.len() > limits.max_input_bytes {
        return Err(PreservesCodecError::InputLimit {
            actual: input.len(),
            limit: limits.max_input_bytes,
        });
    }

    let value = match syntax {
        PreservesSyntax::Text => {
            let text = std::str::from_utf8(input)
                .map_err(|error| PreservesCodecError::Malformed(error.to_string()))?;
            let mut reader = TextReader::new(BytesBinarySource::new(text.as_bytes()));
            decode_one(&mut reader, limits)?
        }
        PreservesSyntax::Packed => {
            let mut reader = PackedReader::new(BytesBinarySource::new(input));
            decode_one(&mut reader, limits)?
        }
    };
    validate_root_schema(&value, schema)?;
    let packed_bytes = write_packed_bounded(&value, limits.max_output_bytes)?;
    if syntax == PreservesSyntax::Packed && packed_bytes != input {
        return Err(PreservesCodecError::NonCanonicalPacked);
    }
    let text_projection = write_text_bounded(&value, limits.max_output_bytes)?;
    Ok(CanonicalPreservesArtifact {
        schema_version: schema.schema_version.to_string(),
        canonical_codec: PRESERVES_CANONICAL_CODEC.to_string(),
        value,
        packed_bytes,
        text_projection,
    })
}

pub fn encode_typed_preserves<T: Serialize>(
    value: &T,
    schema: PreservesSchema<'_>,
    limits: PreservesLimits,
) -> Result<CanonicalPreservesArtifact, PreservesCodecError> {
    validate_limits(limits)?;
    let mut sink = LimitedVec::new(limits.max_output_bytes);
    {
        let mut writer = PackedWriter::new(&mut sink);
        value
            .serialize(&mut preserves::serde::ser::Serializer::new(&mut writer))
            .map_err(|error| typed_encode_error(error, limits.max_output_bytes))?;
    }
    canonicalize_preserves(sink.as_slice(), PreservesSyntax::Packed, schema, limits)
}

pub fn decode_typed_preserves<T: DeserializeOwned>(
    artifact: &CanonicalPreservesArtifact,
) -> Result<T, PreservesCodecError> {
    preserves::serde::from_value(&artifact.value)
        .map_err(|error| PreservesCodecError::TypedDecode(error.to_string()))
}

pub fn verify_typed_preserves<T>(
    input: &[u8],
    syntax: PreservesSyntax,
    expected: &T,
    schema: PreservesSchema<'_>,
    limits: PreservesLimits,
) -> Result<CanonicalPreservesArtifact, PreservesCodecError>
where
    T: DeserializeOwned + PartialEq,
{
    let artifact = canonicalize_preserves(input, syntax, schema, limits)?;
    let decoded: T = decode_typed_preserves(&artifact)?;
    if &decoded != expected {
        return Err(PreservesCodecError::TypedContentMismatch);
    }
    Ok(artifact)
}

fn validate_limits(limits: PreservesLimits) -> Result<(), PreservesCodecError> {
    let fields = [
        ("max_input_bytes", limits.max_input_bytes),
        ("max_output_bytes", limits.max_output_bytes),
        ("max_depth", limits.max_depth),
        ("max_collection_items", limits.max_collection_items),
        ("max_atom_bytes", limits.max_atom_bytes),
        ("max_values", limits.max_values),
    ];
    for (field, value) in fields {
        if value == 0 {
            return Err(PreservesCodecError::InvalidPolicy(field));
        }
    }
    Ok(())
}

fn validate_root_schema(
    value: &IOValue,
    schema: PreservesSchema<'_>,
) -> Result<(), PreservesCodecError> {
    let actual = if value.is_record() {
        value
            .label()
            .as_symbol()
            .map(|label| label.into_owned())
            .unwrap_or_else(|| "<non-symbol-record-label>".to_string())
    } else {
        "<non-record-root>".to_string()
    };
    if actual != schema.root_label {
        return Err(PreservesCodecError::UnsupportedSchema {
            expected: schema.root_label.to_string(),
            actual,
        });
    }
    Ok(())
}

fn decode_one<'de, R: Reader<'de>>(
    reader: &mut R,
    limits: PreservesLimits,
) -> Result<IOValue, PreservesCodecError> {
    let mut frames: Vec<Frame> = Vec::new();
    let mut completed = None;
    let mut value_count = 0usize;

    loop {
        if let Some(value) = completed.take() {
            if let Some(frame) = frames.last_mut() {
                frame.accept(value, reader, limits.max_collection_items)?;
                continue;
            }
            if reader.peek_class().map_err(reader_error)?.is_some() {
                return Err(PreservesCodecError::Malformed(
                    "trailing value after root".to_string(),
                ));
            }
            return Ok(value);
        }

        if let Some(frame) = frames.last_mut()
            && frame.prepare_next(reader)? == FrameAction::Close
        {
            let frame = frames.pop().ok_or_else(|| {
                PreservesCodecError::Malformed("decoder frame underflow".to_string())
            })?;
            let value = frame.finish()?;
            register_value(&mut value_count, limits.max_values)?;
            completed = Some(value);
            continue;
        }

        let depth =
            frames
                .len()
                .checked_add(ROOT_DEPTH)
                .ok_or(PreservesCodecError::DepthLimit {
                    actual: usize::MAX,
                    limit: limits.max_depth,
                })?;
        if depth > limits.max_depth {
            return Err(PreservesCodecError::DepthLimit {
                actual: depth,
                limit: limits.max_depth,
            });
        }
        let token = reader
            .peek_class()
            .map_err(reader_error)?
            .ok_or_else(|| PreservesCodecError::Malformed("unexpected end of input".to_string()))?;
        match token {
            NextToken::Annotation => return Err(PreservesCodecError::UnsupportedAnnotation),
            NextToken::End => {
                return Err(PreservesCodecError::Malformed(
                    "unexpected close delimiter".to_string(),
                ));
            }
            NextToken::Value(ValueClass::Embedded) => {
                return Err(PreservesCodecError::UnsupportedEmbeddedValue);
            }
            NextToken::Value(ValueClass::Atomic(_)) => {
                let atom = reader.next_atom().map_err(reader_error)?;
                let value = atom_to_value(atom, limits.max_atom_bytes)?;
                register_value(&mut value_count, limits.max_values)?;
                completed = Some(value);
            }
            NextToken::Value(ValueClass::Compound(class)) => {
                frames.push(Frame::open(reader, class)?);
            }
        }
    }
}

fn register_value(count: &mut usize, limit: usize) -> Result<(), PreservesCodecError> {
    *count = count
        .checked_add(1)
        .ok_or(PreservesCodecError::ValueLimit {
            actual: usize::MAX,
            limit,
        })?;
    if *count > limit {
        return Err(PreservesCodecError::ValueLimit {
            actual: *count,
            limit,
        });
    }
    Ok(())
}

fn atom_to_value(atom: Atom<'_>, max_atom_bytes: usize) -> Result<IOValue, PreservesCodecError> {
    match atom {
        Atom::Boolean(value) => Ok(IOValue::new(value)),
        Atom::Double(value) => Ok(IOValue::new(value)),
        Atom::SignedInteger(value) => Ok(IOValue::new(value.into_owned())),
        Atom::String(value) => {
            check_atom_limit(value.len(), max_atom_bytes)?;
            Ok(IOValue::new(value.into_owned()))
        }
        Atom::ByteString(value) => {
            check_atom_limit(value.len(), max_atom_bytes)?;
            Ok(IOValue::bytes(value.into_owned()))
        }
        Atom::Symbol(value) => {
            check_atom_limit(value.len(), max_atom_bytes)?;
            Ok(IOValue::symbol(value.into_owned()))
        }
    }
}

fn check_atom_limit(actual: usize, limit: usize) -> Result<(), PreservesCodecError> {
    if actual > limit {
        return Err(PreservesCodecError::AtomLimit { actual, limit });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FrameAction {
    Value,
    Close,
}

enum Frame {
    Record {
        boundary: BoundaryType,
        label: Option<IOValue>,
        fields: Vec<IOValue>,
    },
    Sequence {
        boundary: BoundaryType,
        values: Vec<IOValue>,
    },
    Set {
        boundary: BoundaryType,
        values: BTreeSet<IOValue>,
    },
    Dictionary {
        boundary: BoundaryType,
        pending_key: Option<IOValue>,
        values: BTreeMap<IOValue, IOValue>,
    },
}

impl Frame {
    fn open<'de, R: Reader<'de>>(
        reader: &mut R,
        class: CompoundClass,
    ) -> Result<Self, PreservesCodecError> {
        match class {
            CompoundClass::Record => {
                reader.open_record().map_err(reader_error)?;
                let boundary = start_boundary(BoundaryItem::RecordLabel);
                reader.boundary(&boundary).map_err(reader_error)?;
                Ok(Self::Record {
                    boundary,
                    label: None,
                    fields: Vec::new(),
                })
            }
            CompoundClass::Sequence => {
                reader.open_sequence().map_err(reader_error)?;
                Ok(Self::Sequence {
                    boundary: BoundaryType::default(),
                    values: Vec::new(),
                })
            }
            CompoundClass::Set => {
                reader.open_set().map_err(reader_error)?;
                Ok(Self::Set {
                    boundary: BoundaryType::default(),
                    values: BTreeSet::new(),
                })
            }
            CompoundClass::Dictionary => {
                reader.open_dictionary().map_err(reader_error)?;
                Ok(Self::Dictionary {
                    boundary: BoundaryType::default(),
                    pending_key: None,
                    values: BTreeMap::new(),
                })
            }
        }
    }

    fn prepare_next<'de, R: Reader<'de>>(
        &mut self,
        reader: &mut R,
    ) -> Result<FrameAction, PreservesCodecError> {
        match self {
            Self::Record {
                boundary, label, ..
            } => {
                if label.is_none() {
                    return Ok(FrameAction::Value);
                }
                close_action(reader, boundary, BoundaryItem::RecordField)
            }
            Self::Sequence { boundary, .. } => {
                close_action(reader, boundary, BoundaryItem::SequenceValue)
            }
            Self::Set { boundary, .. } => close_action(reader, boundary, BoundaryItem::SetValue),
            Self::Dictionary {
                boundary,
                pending_key,
                ..
            } => {
                if pending_key.is_some() {
                    return Ok(FrameAction::Value);
                }
                close_action(reader, boundary, BoundaryItem::DictionaryKey)
            }
        }
    }

    fn accept<'de, R: Reader<'de>>(
        &mut self,
        value: IOValue,
        reader: &mut R,
        collection_limit: usize,
    ) -> Result<(), PreservesCodecError> {
        match self {
            Self::Record { label, fields, .. } => {
                if label.is_none() {
                    *label = Some(value);
                    return Ok(());
                }
                check_collection_growth(fields.len(), collection_limit)?;
                fields.push(value);
            }
            Self::Sequence { values, .. } => {
                check_collection_growth(values.len(), collection_limit)?;
                values.push(value);
            }
            Self::Set { values, .. } => {
                check_collection_growth(values.len(), collection_limit)?;
                if !values.insert(value) {
                    return Err(PreservesCodecError::Malformed(
                        "duplicate set element".to_string(),
                    ));
                }
            }
            Self::Dictionary {
                boundary,
                pending_key,
                values,
            } => {
                if pending_key.is_none() {
                    *pending_key = Some(value);
                    boundary.shift(Some(BoundaryItem::DictionaryValue));
                    reader.boundary(boundary).map_err(reader_error)?;
                    return Ok(());
                }
                check_collection_growth(values.len(), collection_limit)?;
                let key = pending_key.take().ok_or_else(|| {
                    PreservesCodecError::Malformed("dictionary key state lost".to_string())
                })?;
                if values.insert(key, value).is_some() {
                    return Err(PreservesCodecError::Malformed(
                        "duplicate dictionary key".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn finish(self) -> Result<IOValue, PreservesCodecError> {
        match self {
            Self::Record { label, fields, .. } => {
                let label = label.ok_or_else(|| {
                    PreservesCodecError::Malformed("record is missing a label".to_string())
                })?;
                Ok(IOValue::record(label, fields))
            }
            Self::Sequence { values, .. } => Ok(IOValue::new(values)),
            Self::Set { values, .. } => Ok(IOValue::new(values)),
            Self::Dictionary {
                pending_key,
                values,
                ..
            } => {
                if pending_key.is_some() {
                    return Err(PreservesCodecError::Malformed(
                        "dictionary is missing a value".to_string(),
                    ));
                }
                Ok(IOValue::new(values))
            }
        }
    }
}

fn close_action<'de, R: Reader<'de>>(
    reader: &mut R,
    boundary: &mut BoundaryType,
    item: BoundaryItem,
) -> Result<FrameAction, PreservesCodecError> {
    if reader
        .close_compound(boundary, &item)
        .map_err(reader_error)?
    {
        Ok(FrameAction::Close)
    } else {
        Ok(FrameAction::Value)
    }
}

fn check_collection_growth(current: usize, limit: usize) -> Result<(), PreservesCodecError> {
    let next = current
        .checked_add(1)
        .ok_or(PreservesCodecError::CollectionLimit {
            actual: usize::MAX,
            limit,
        })?;
    if next > limit {
        return Err(PreservesCodecError::CollectionLimit {
            actual: next,
            limit,
        });
    }
    Ok(())
}

fn reader_error(error: preserves::Error) -> PreservesCodecError {
    PreservesCodecError::Malformed(error.to_string())
}

fn write_packed_bounded(value: &IOValue, limit: usize) -> Result<Vec<u8>, PreservesCodecError> {
    let mut sink = LimitedVec::new(limit);
    preserves::write_iovalue_packed_into(value, false, &mut sink)
        .map_err(|error| output_error(error, limit))?;
    Ok(sink.into_inner())
}

fn write_text_bounded(value: &IOValue, limit: usize) -> Result<String, PreservesCodecError> {
    let mut sink = LimitedVec::new(limit);
    preserves::write_iovalue_text_into(value, false, &mut sink)
        .map_err(|error| output_error(error, limit))?;
    String::from_utf8(sink.into_inner())
        .map_err(|error| PreservesCodecError::Malformed(error.to_string()))
}

fn typed_encode_error(error: preserves::serde::Error, limit: usize) -> PreservesCodecError {
    match error {
        preserves::serde::Error::Preserves(preserves::Error::Io(error)) => {
            output_error(error, limit)
        }
        other => PreservesCodecError::Malformed(other.to_string()),
    }
}

fn output_error(error: IoError, limit: usize) -> PreservesCodecError {
    if error.kind() == ErrorKind::FileTooLarge {
        return PreservesCodecError::OutputLimit {
            actual: limit.saturating_add(1),
            limit,
        };
    }
    PreservesCodecError::Malformed(error.to_string())
}

struct LimitedVec {
    bytes: Vec<u8>,
    limit: usize,
}

impl LimitedVec {
    fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    fn into_inner(self) -> Vec<u8> {
        self.bytes
    }
}

impl Write for LimitedVec {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let next = self
            .bytes
            .len()
            .checked_add(buffer.len())
            .ok_or_else(|| IoError::from(ErrorKind::FileTooLarge))?;
        if next > self.limit {
            return Err(IoError::from(ErrorKind::FileTooLarge));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
        0xb4, 0xb3, 0x0b, b'B', b'o', b'u', b'n', b'd', b'R', b'e', b'c', b'o', b'r', b'd', 0xb1,
        0x05, b'a', b'l', b'p', b'h', b'a', 0xb5, 0xb0, 0x02, 0x00, 0x01, 0xb0, 0x01, 0x02, 0x84,
        0x84,
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
        assert!(matches!(packed, Ok(ref artifact) if artifact.packed_bytes == first.packed_bytes));
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
                max_depth: SHALLOW_DEPTH_LIMIT,
                ..base
            },
        );
        assert!(matches!(depth, Err(PreservesCodecError::DepthLimit { .. })));

        let collection = canonicalize_preserves(
            TEST_TEXT.as_bytes(),
            PreservesSyntax::Text,
            schema(),
            PreservesLimits {
                max_collection_items: SMALL_COLLECTION_LIMIT,
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
                max_values: EXPECTED_ITEM_COUNT,
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
    fn rejects_zero_limit_policy() {
        let result = canonicalize_preserves(
            TEST_TEXT.as_bytes(),
            PreservesSyntax::Text,
            schema(),
            PreservesLimits {
                max_depth: 0,
                ..PreservesLimits::default()
            },
        );
        assert_eq!(
            result.err(),
            Some(PreservesCodecError::InvalidPolicy("max_depth"))
        );
    }
}
