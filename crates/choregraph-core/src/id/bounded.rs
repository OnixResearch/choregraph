use crate::id::domains::{
    AbilityDomain, DeclarationDomain, NodeDomain, OperationDomain, RequestDomain, ResourceDomain,
    SlotDomain, SourceDomain, SuspensionClassDomain,
};
use alloc::string::String;
use core::fmt;
use core::marker::PhantomData;

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
pub type DeclarationKey = BoundedId<DeclarationDomain>;
pub type SlotId = BoundedId<SlotDomain>;
pub type SourceRef = BoundedId<SourceDomain>;
pub type SuspensionClass = BoundedId<SuspensionClassDomain>;

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
        debug_assert!(!value.is_empty());
        debug_assert!(value.len() <= maximum_bytes);
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
