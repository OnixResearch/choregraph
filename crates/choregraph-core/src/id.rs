use alloc::string::String;
use core::fmt;
use core::marker::PhantomData;

pub const BLAKE3_REFERENCE_PREFIX: &str = "blake3:";
pub const BLAKE3_DIGEST_HEX_BYTES: usize = 64;
pub const BLAKE3_REFERENCE_BYTES: usize = BLAKE3_REFERENCE_PREFIX.len() + BLAKE3_DIGEST_HEX_BYTES;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NodeDomain;
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OperationDomain;
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ResourceDomain;
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AbilityDomain;
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RequestDomain;

/// A validated bounded identifier in one semantic domain.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BoundedId<Domain> {
    value: String,
    marker: PhantomData<Domain>,
}

pub type NodeId = BoundedId<NodeDomain>;
pub type OperationId = BoundedId<OperationDomain>;
pub type ResourceId = BoundedId<ResourceDomain>;
pub type AbilityId = BoundedId<AbilityDomain>;
pub type RequestId = BoundedId<RequestDomain>;

/// A deterministic identifier validation error.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IdentifierError {
    Empty,
    TooLong { observed: usize, maximum: usize },
    InvalidByte { index: usize },
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/' | b'@')
}

impl<Domain> BoundedId<Domain> {
    /// Parses a bounded ASCII identifier.
    pub fn parse(value: impl Into<String>, maximum_bytes: usize) -> Result<Self, IdentifierError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IdentifierError::Empty);
        }
        if value.len() > maximum_bytes {
            return Err(IdentifierError::TooLong {
                observed: value.len(),
                maximum: maximum_bytes,
            });
        }
        for (index, byte) in value.bytes().enumerate() {
            if !is_identifier_byte(byte) {
                return Err(IdentifierError::InvalidByte { index });
            }
        }
        Ok(Self {
            value,
            marker: PhantomData,
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl<Domain> fmt::Display for BoundedId<Domain> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.value)
    }
}

/// A canonical BLAKE3 payload reference.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PayloadRef(String);

/// A deterministic canonical-reference validation error.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PayloadRefError {
    TooLong { observed: usize, maximum: usize },
    Malformed,
}

impl PayloadRef {
    /// Parses `blake3:` followed by one lowercase hexadecimal digest.
    pub fn parse(value: impl Into<String>, maximum_bytes: usize) -> Result<Self, PayloadRefError> {
        let value = value.into();
        if value.len() > maximum_bytes {
            return Err(PayloadRefError::TooLong {
                observed: value.len(),
                maximum: maximum_bytes,
            });
        }
        let Some(digest) = value.strip_prefix(BLAKE3_REFERENCE_PREFIX) else {
            return Err(PayloadRefError::Malformed);
        };
        if digest.len() != BLAKE3_DIGEST_HEX_BYTES {
            return Err(PayloadRefError::Malformed);
        }
        if !digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(PayloadRefError::Malformed);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PayloadRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use super::{BLAKE3_DIGEST_HEX_BYTES, NodeId, PayloadRef};

    const IDENTIFIER_LIMIT: usize = 16;

    #[test]
    fn accepts_bounded_identifiers_and_blake3_references() {
        assert!(NodeId::parse("node/a", IDENTIFIER_LIMIT).is_ok());
        let reference = String::from("blake3:") + &"a".repeat(BLAKE3_DIGEST_HEX_BYTES);
        assert!(PayloadRef::parse(reference, super::BLAKE3_REFERENCE_BYTES).is_ok());
    }

    #[test]
    fn rejects_whitespace_and_non_blake3_references() {
        assert!(NodeId::parse("node a", IDENTIFIER_LIMIT).is_err());
        assert!(PayloadRef::parse("sha256:abc", super::BLAKE3_REFERENCE_BYTES).is_err());
    }
}
