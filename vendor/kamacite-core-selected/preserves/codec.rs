use preserves::boundary::{Item as BoundaryItem, Type as BoundaryType, start as start_boundary};
use preserves::reader::{NextToken, Reader};
use preserves::{
    Atom, BytesBinarySource, CompoundClass, IOValue, PackedReader, PackedWriter, TextReader,
    ValueClass, ValueImpl,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::io::{Error as IoError, ErrorKind, Write};

pub mod profile {
    pub const PRESERVES_CANONICAL_CODEC: &str =
        "kamacite-preserves-packed-canonical-v2-preserves-rs-5.0.0-rc.7";
    pub const PRESERVES_LEGACY_PREFIXED_JSON_CODEC: &str =
        "kamacite-legacy-labeled-deterministic-json-v1";
    pub const PRESERVES_PACKAGE_VERSION: &str = "5.0.0-rc.7";
    pub const PRESERVES_SPEC_VERSION: &str = "0.996.3";
}

use profile::PRESERVES_CANONICAL_CODEC;

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
    pub max_depth_count: usize,
    pub max_collection_item_count: usize,
    pub max_atom_bytes: usize,
    pub max_value_count: usize,
}

impl Default for PreservesLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: DEFAULT_MAX_INPUT_BYTES,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
            max_depth_count: DEFAULT_MAX_DEPTH,
            max_collection_item_count: DEFAULT_MAX_COLLECTION_ITEMS,
            max_atom_bytes: DEFAULT_MAX_ATOM_BYTES,
            max_value_count: DEFAULT_MAX_VALUES,
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
    InputLimit {
        actual_bytes: usize,
        limit_bytes: usize,
    },
    OutputLimit {
        actual_bytes: usize,
        limit_bytes: usize,
    },
    DepthLimit {
        actual_count: usize,
        max_depth_count: usize,
    },
    CollectionLimit {
        actual_count: usize,
        max_item_count: usize,
    },
    AtomLimit {
        actual_bytes: usize,
        limit_bytes: usize,
    },
    ValueLimit {
        actual_count: usize,
        max_value_count: usize,
    },
    Malformed(Cow<'static, str>),
    UnsupportedAnnotation,
    UnsupportedEmbeddedValue,
    UnsupportedSchema {
        expected: String,
        actual: String,
    },
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
            Self::InputLimit {
                actual_bytes,
                limit_bytes,
            } => {
                write!(
                    formatter,
                    "Preserves input size {actual_bytes} exceeds limit {limit_bytes}"
                )
            }
            Self::OutputLimit {
                actual_bytes,
                limit_bytes,
            } => {
                write!(
                    formatter,
                    "Preserves output size {actual_bytes} exceeds limit {limit_bytes}"
                )
            }
            Self::DepthLimit {
                actual_count,
                max_depth_count,
            } => {
                write!(
                    formatter,
                    "Preserves value depth {actual_count} exceeds limit {max_depth_count}"
                )
            }
            Self::CollectionLimit {
                actual_count,
                max_item_count,
            } => {
                write!(
                    formatter,
                    "Preserves collection size {actual_count} exceeds limit {max_item_count}"
                )
            }
            Self::AtomLimit {
                actual_bytes,
                limit_bytes,
            } => {
                write!(
                    formatter,
                    "Preserves atom size {actual_bytes} exceeds limit {limit_bytes}"
                )
            }
            Self::ValueLimit {
                actual_count,
                max_value_count,
            } => {
                write!(
                    formatter,
                    "Preserves value count {actual_count} exceeds limit {max_value_count}"
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
            actual_bytes: input.len(),
            limit_bytes: limits.max_input_bytes,
        });
    }

    let value = match syntax {
        PreservesSyntax::Text => {
            let text = std::str::from_utf8(input)
                .map_err(|error| PreservesCodecError::Malformed(error.to_string().into()))?;
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
        ("max_depth", limits.max_depth_count),
        ("max_collection_items", limits.max_collection_item_count),
        ("max_atom_bytes", limits.max_atom_bytes),
        ("max_values", limits.max_value_count),
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
                frame.accept(value, reader, limits.max_collection_item_count)?;
                continue;
            }
            if reader.peek_class().map_err(reader_error)?.is_some() {
                return Err(PreservesCodecError::Malformed(Cow::Borrowed(
                    "trailing value after root",
                )));
            }
            return Ok(value);
        }

        if let Some(frame) = frames.last_mut()
            && frame.prepare_next(reader)? == FrameAction::Close
        {
            let frame = frames
                .pop()
                .ok_or(PreservesCodecError::Malformed(Cow::Borrowed(
                    "decoder frame underflow",
                )))?;
            let value = frame.finish()?;
            register_value(&mut value_count, limits.max_value_count)?;
            completed = Some(value);
            continue;
        }

        let depth =
            frames
                .len()
                .checked_add(ROOT_DEPTH)
                .ok_or(PreservesCodecError::DepthLimit {
                    actual_count: usize::MAX,
                    max_depth_count: limits.max_depth_count,
                })?;
        if depth > limits.max_depth_count {
            return Err(PreservesCodecError::DepthLimit {
                actual_count: depth,
                max_depth_count: limits.max_depth_count,
            });
        }
        let token =
            reader
                .peek_class()
                .map_err(reader_error)?
                .ok_or(PreservesCodecError::Malformed(Cow::Borrowed(
                    "unexpected end of input",
                )))?;
        match token {
            NextToken::Annotation => return Err(PreservesCodecError::UnsupportedAnnotation),
            NextToken::End => {
                return Err(PreservesCodecError::Malformed(Cow::Borrowed(
                    "unexpected close delimiter",
                )));
            }
            NextToken::Value(ValueClass::Embedded) => {
                return Err(PreservesCodecError::UnsupportedEmbeddedValue);
            }
            NextToken::Value(ValueClass::Atomic(_)) => {
                let atom = reader.next_atom().map_err(reader_error)?;
                let value = atom_to_value(atom, limits.max_atom_bytes)?;
                register_value(&mut value_count, limits.max_value_count)?;
                completed = Some(value);
            }
            NextToken::Value(ValueClass::Compound(class)) => {
                frames.push(Frame::open(reader, class)?);
            }
        }
    }
}

fn register_value(count: &mut usize, max_value_count: usize) -> Result<(), PreservesCodecError> {
    *count = count
        .checked_add(1)
        .ok_or(PreservesCodecError::ValueLimit {
            actual_count: usize::MAX,
            max_value_count,
        })?;
    if *count > max_value_count {
        return Err(PreservesCodecError::ValueLimit {
            actual_count: *count,
            max_value_count,
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

fn check_atom_limit(actual_bytes: usize, limit_bytes: usize) -> Result<(), PreservesCodecError> {
    if actual_bytes > limit_bytes {
        return Err(PreservesCodecError::AtomLimit {
            actual_bytes,
            limit_bytes,
        });
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
        max_collection_item_count: usize,
    ) -> Result<(), PreservesCodecError> {
        match self {
            Self::Record { label, fields, .. } => {
                if label.is_none() {
                    *label = Some(value);
                    return Ok(());
                }
                check_collection_growth(fields.len(), max_collection_item_count)?;
                fields.push(value);
            }
            Self::Sequence { values, .. } => {
                check_collection_growth(values.len(), max_collection_item_count)?;
                values.push(value);
            }
            Self::Set { values, .. } => {
                check_collection_growth(values.len(), max_collection_item_count)?;
                if !values.insert(value) {
                    return Err(PreservesCodecError::Malformed(Cow::Borrowed(
                        "duplicate set element",
                    )));
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
                check_collection_growth(values.len(), max_collection_item_count)?;
                let key =
                    pending_key
                        .take()
                        .ok_or(PreservesCodecError::Malformed(Cow::Borrowed(
                            "dictionary key state lost",
                        )))?;
                if values.insert(key, value).is_some() {
                    return Err(PreservesCodecError::Malformed(Cow::Borrowed(
                        "duplicate dictionary key",
                    )));
                }
            }
        }
        Ok(())
    }

    fn finish(self) -> Result<IOValue, PreservesCodecError> {
        match self {
            Self::Record { label, fields, .. } => {
                let label = label.ok_or(PreservesCodecError::Malformed(Cow::Borrowed(
                    "record is missing a label",
                )))?;
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
                    return Err(PreservesCodecError::Malformed(Cow::Borrowed(
                        "dictionary is missing a value",
                    )));
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

fn check_collection_growth(
    current: usize,
    max_item_count: usize,
) -> Result<(), PreservesCodecError> {
    let next = current
        .checked_add(1)
        .ok_or(PreservesCodecError::CollectionLimit {
            actual_count: usize::MAX,
            max_item_count,
        })?;
    if next > max_item_count {
        return Err(PreservesCodecError::CollectionLimit {
            actual_count: next,
            max_item_count,
        });
    }
    Ok(())
}

fn reader_error(error: preserves::Error) -> PreservesCodecError {
    PreservesCodecError::Malformed(error.to_string().into())
}

fn write_packed_bounded(
    value: &IOValue,
    limit_bytes: usize,
) -> Result<Vec<u8>, PreservesCodecError> {
    let mut sink = LimitedVec::new(limit_bytes);
    preserves::write_iovalue_packed_into(value, false, &mut sink)
        .map_err(|error| output_error(error, limit_bytes))?;
    Ok(sink.into_inner())
}

fn write_text_bounded(value: &IOValue, limit_bytes: usize) -> Result<String, PreservesCodecError> {
    let mut sink = LimitedVec::new(limit_bytes);
    preserves::write_iovalue_text_into(value, false, &mut sink)
        .map_err(|error| output_error(error, limit_bytes))?;
    String::from_utf8(sink.into_inner())
        .map_err(|error| PreservesCodecError::Malformed(error.to_string().into()))
}

fn typed_encode_error(error: preserves::serde::Error, limit_bytes: usize) -> PreservesCodecError {
    match error {
        preserves::serde::Error::Preserves(preserves::Error::Io(error)) => {
            output_error(error, limit_bytes)
        }
        other => PreservesCodecError::Malformed(other.to_string().into()),
    }
}

fn output_error(error: IoError, limit_bytes: usize) -> PreservesCodecError {
    if error.kind() == ErrorKind::FileTooLarge {
        return PreservesCodecError::OutputLimit {
            actual_bytes: limit_bytes.saturating_add(1),
            limit_bytes,
        };
    }
    PreservesCodecError::Malformed(error.to_string().into())
}

struct LimitedVec {
    bytes: Vec<u8>,
    limit_bytes: usize,
}

impl LimitedVec {
    fn new(limit_bytes: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit_bytes,
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
        if next > self.limit_bytes {
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
mod tests;
