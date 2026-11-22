// Bounded identifiers keep stable usize-based compatibility and explicit BLAKE3 arithmetic.
#![allow(
    unknown_lints,
    non_trait_imports,
    path_segment_repetition,
    usize_in_public_api,
    reason = "bounded identifiers keep stable usize-based compatibility and explicit BLAKE3 arithmetic"
)]

pub mod bounded;
pub mod domains;
pub mod reference;

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use super::bounded::{IdentifierError, NodeId};
    use super::reference::{
        BLAKE3_DIGEST_HEX_BYTES, BLAKE3_REFERENCE_BYTES, PayloadRef, PayloadRefError,
    };

    const IDENTIFIER_LIMIT: usize = 16;
    const EMPTY_LIMIT: usize = 0;
    const FIRST_NON_ASCII_BYTE: usize = 1;

    #[test]
    fn accepts_bounded_identifiers_and_blake3_references() {
        assert!(NodeId::parse("node/a", IDENTIFIER_LIMIT).is_ok());
        let reference = String::from("blake3:") + &"a".repeat(BLAKE3_DIGEST_HEX_BYTES);
        assert!(PayloadRef::parse(reference, BLAKE3_REFERENCE_BYTES).is_ok());
    }

    #[test]
    fn rejects_whitespace_and_non_blake3_references() {
        assert!(NodeId::parse("node a", IDENTIFIER_LIMIT).is_err());
        assert!(PayloadRef::parse("sha256:abc", BLAKE3_REFERENCE_BYTES).is_err());
    }

    #[test]
    fn identifier_errors_keep_empty_length_and_byte_precedence() {
        assert_eq!(NodeId::parse("", EMPTY_LIMIT), Err(IdentifierError::Empty));
        assert_eq!(
            NodeId::parse(" ", EMPTY_LIMIT),
            Err(IdentifierError::TooLong {
                observed: " ".len(),
                maximum: EMPTY_LIMIT,
            })
        );
        assert_eq!(
            NodeId::parse("aé", IDENTIFIER_LIMIT),
            Err(IdentifierError::InvalidByte {
                index: FIRST_NON_ASCII_BYTE,
            })
        );
        let identifier = "aZ09-_.:/@";
        let parsed = NodeId::parse(identifier, identifier.len());
        assert_eq!(parsed.as_ref().map(NodeId::as_str), Ok(identifier));
    }

    #[test]
    fn reference_errors_keep_length_before_canonical_validation() {
        let malformed = "sha256:abc";
        assert_eq!(
            PayloadRef::parse(malformed, EMPTY_LIMIT),
            Err(PayloadRefError::TooLong {
                observed: malformed.len(),
                maximum: EMPTY_LIMIT,
            })
        );
        let uppercase = String::from("blake3:") + &"A".repeat(BLAKE3_DIGEST_HEX_BYTES);
        assert_eq!(
            PayloadRef::parse(uppercase, BLAKE3_REFERENCE_BYTES),
            Err(PayloadRefError::Malformed)
        );
        let short = String::from("blake3:") + &"a".repeat(IDENTIFIER_LIMIT);
        assert_eq!(
            PayloadRef::parse(short, BLAKE3_REFERENCE_BYTES),
            Err(PayloadRefError::Malformed)
        );
        let reference = String::from("blake3:") + &"0".repeat(BLAKE3_DIGEST_HEX_BYTES);
        let parsed = PayloadRef::parse(reference.clone(), BLAKE3_REFERENCE_BYTES);
        assert_eq!(
            parsed.as_ref().map(PayloadRef::as_str),
            Ok(reference.as_str())
        );
    }
}
