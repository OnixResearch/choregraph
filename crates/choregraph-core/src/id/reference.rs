use alloc::string::String;
use core::fmt;

pub const BLAKE3_REFERENCE_PREFIX: &str = "blake3:";
pub const BLAKE3_DIGEST_HEX_BYTES: usize = 64;
pub const BLAKE3_REFERENCE_BYTES: usize = BLAKE3_REFERENCE_PREFIX.len() + BLAKE3_DIGEST_HEX_BYTES;

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
        debug_assert_eq!(value.len(), BLAKE3_REFERENCE_BYTES);
        debug_assert!(value.len() <= maximum_bytes);
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
