// Restricted source validation is a conservative lexical gate before Nickel execution.
#![allow(
    unknown_lints,
    assertion_density,
    function_length,
    reason = "restricted source validation uses one explicit lexer with positive and negative assertions"
)]

// r[impl choregraph.nickel.profile]
const IMPORT_KEYWORD: &[u8] = b"import";
const INTERPOLATION_MARKER: &[u8] = b"%{";
const BLOCK_COMMENT_OPEN: &[u8] = b"/*";
const BLOCK_COMMENT_CLOSE: &[u8] = b"*/";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScanMode {
    Normal,
    String,
    LineComment,
    BlockComment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ScanTransition {
    mode: ScanMode,
    block_depth: usize,
    skip_next: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ScanCursor {
    index_bytes: usize,
    block_depth: usize,
}

fn fail(code: crate::diagnostic::FrontendIssueCode) -> crate::diagnostic::FrontendFailure {
    crate::diagnostic::FrontendFailure::one(code)
}

const fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

const fn starts_with_at(source: &[u8], index_bytes: usize, needle: &[u8]) -> bool {
    let Some(end_bytes) = index_bytes.checked_add(needle.len()) else {
        return false;
    };
    if end_bytes > source.len() {
        return false;
    }
    let mut offset_bytes = 0usize;
    while offset_bytes < needle.len() {
        let source_index_bytes = index_bytes.saturating_add(offset_bytes);
        if source[source_index_bytes] != needle[offset_bytes] {
            return false;
        }
        offset_bytes += 1;
    }
    true
}

const fn keyword_at(source: &[u8], index_bytes: usize, keyword: &[u8]) -> bool {
    if !starts_with_at(source, index_bytes, keyword) {
        return false;
    }
    let is_before_identifier = if index_bytes == 0 {
        false
    } else {
        is_identifier_byte(source[index_bytes.saturating_sub(1)])
    };
    let after_index_bytes = index_bytes.saturating_add(keyword.len());
    let is_after_identifier =
        after_index_bytes < source.len() && is_identifier_byte(source[after_index_bytes]);
    !is_before_identifier && !is_after_identifier
}

const fn scan_normal(source: &[u8], index_bytes: usize, byte: u8) -> Option<ScanTransition> {
    if byte == b'#' || starts_with_at(source, index_bytes, b"//") {
        return Some(ScanTransition {
            mode: ScanMode::LineComment,
            block_depth: 0,
            skip_next: false,
        });
    }
    if starts_with_at(source, index_bytes, BLOCK_COMMENT_OPEN) {
        return Some(ScanTransition {
            mode: ScanMode::BlockComment,
            block_depth: 1,
            skip_next: true,
        });
    }
    if byte == b'"' {
        return Some(ScanTransition {
            mode: ScanMode::String,
            block_depth: 0,
            skip_next: false,
        });
    }
    if keyword_at(source, index_bytes, IMPORT_KEYWORD) {
        return None;
    }
    Some(ScanTransition {
        mode: ScanMode::Normal,
        block_depth: 0,
        skip_next: false,
    })
}

const fn scan_string(source: &[u8], index_bytes: usize, byte: u8) -> Option<ScanTransition> {
    if starts_with_at(source, index_bytes, INTERPOLATION_MARKER) {
        return None;
    }
    let mode = if byte == b'"' {
        ScanMode::Normal
    } else {
        ScanMode::String
    };
    Some(ScanTransition {
        mode,
        block_depth: 0,
        skip_next: byte == b'\\',
    })
}

const fn scan_block(source: &[u8], cursor: ScanCursor) -> Option<ScanTransition> {
    if starts_with_at(source, cursor.index_bytes, BLOCK_COMMENT_OPEN) {
        let Some(block_depth) = cursor.block_depth.checked_add(1) else {
            return None;
        };
        return Some(ScanTransition {
            mode: ScanMode::BlockComment,
            block_depth,
            skip_next: true,
        });
    }
    if starts_with_at(source, cursor.index_bytes, BLOCK_COMMENT_CLOSE) {
        let block_depth = cursor.block_depth.saturating_sub(1);
        let mode = if block_depth == 0 {
            ScanMode::Normal
        } else {
            ScanMode::BlockComment
        };
        return Some(ScanTransition {
            mode,
            block_depth,
            skip_next: true,
        });
    }
    Some(ScanTransition {
        mode: ScanMode::BlockComment,
        block_depth: cursor.block_depth,
        skip_next: false,
    })
}

// The private scanner modes form one closed lexical automaton. An exhaustive match keeps mode changes visible.
#[allow(
    fragile_exhaustive_enum_match,
    reason = "the private scanner modes are a closed lexical automaton and must change together"
)]
pub fn validate_restricted(
    source: &[u8],
    max_source_bytes: u64,
) -> Result<(), crate::diagnostic::FrontendFailure> {
    let source_bytes = u64::try_from(source.len())
        .map_err(|_| fail(crate::diagnostic::FrontendIssueCode::NickelSourceTooLarge))?;
    if source.is_empty() || source_bytes > max_source_bytes {
        return Err(fail(
            crate::diagnostic::FrontendIssueCode::NickelSourceTooLarge,
        ));
    }
    if core::str::from_utf8(source).is_err() || source.contains(&0) {
        return Err(fail(
            crate::diagnostic::FrontendIssueCode::MalformedNickelSource,
        ));
    }

    let mut mode = ScanMode::Normal;
    let mut index_bytes = 0usize;
    let mut block_depth = 0usize;
    while index_bytes < source.len() {
        let byte = source[index_bytes];
        let transition = match mode {
            ScanMode::Normal => scan_normal(source, index_bytes, byte).ok_or_else(|| {
                fail(crate::diagnostic::FrontendIssueCode::ForbiddenNickelAuthority)
            })?,
            ScanMode::String => scan_string(source, index_bytes, byte).ok_or_else(|| {
                fail(crate::diagnostic::FrontendIssueCode::ForbiddenNickelAuthority)
            })?,
            ScanMode::LineComment => ScanTransition {
                mode: if byte == b'\n' {
                    ScanMode::Normal
                } else {
                    ScanMode::LineComment
                },
                block_depth: 0,
                skip_next: false,
            },
            ScanMode::BlockComment => scan_block(
                source,
                ScanCursor {
                    index_bytes,
                    block_depth,
                },
            )
            .ok_or_else(|| fail(crate::diagnostic::FrontendIssueCode::MalformedNickelSource))?,
        };
        mode = transition.mode;
        block_depth = transition.block_depth;
        if transition.skip_next {
            index_bytes = index_bytes.saturating_add(1);
        }
        index_bytes = index_bytes.saturating_add(1);
    }

    if matches!(mode, ScanMode::String | ScanMode::BlockComment) {
        return Err(fail(
            crate::diagnostic::FrontendIssueCode::MalformedNickelSource,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    const SOURCE_LIMIT_BYTES: u64 = 1_024;

    #[test]
    fn accepts_data_records_and_import_named_fields() {
        let source_bytes = br#"{ imports = [], note = "import is inert text", value = 1 }"#;
        assert_eq!(
            super::validate_restricted(source_bytes, SOURCE_LIMIT_BYTES),
            Ok(())
        );
    }

    #[test]
    fn rejects_import_authority_and_interpolation() {
        let import_source_bytes = br#"let value = import "ambient.ncl" in value"#;
        let interpolation_source_bytes = br#"{ value = "prefix %{std.foo}" }"#;
        assert_eq!(
            super::validate_restricted(import_source_bytes, SOURCE_LIMIT_BYTES)
                .err()
                .map(|failure| failure.issues[0].code()),
            Some(crate::FrontendIssueCode::ForbiddenNickelAuthority)
        );
        assert_eq!(
            super::validate_restricted(interpolation_source_bytes, SOURCE_LIMIT_BYTES)
                .err()
                .map(|failure| failure.issues[0].code()),
            Some(crate::FrontendIssueCode::ForbiddenNickelAuthority)
        );
    }

    #[test]
    fn rejects_malformed_and_oversized_input() {
        assert_eq!(
            super::validate_restricted(b"{ value = \"unterminated }", SOURCE_LIMIT_BYTES)
                .err()
                .map(|failure| failure.issues[0].code()),
            Some(crate::FrontendIssueCode::MalformedNickelSource)
        );
        assert_eq!(
            super::validate_restricted(b"{}", 1)
                .err()
                .map(|failure| failure.issues[0].code()),
            Some(crate::FrontendIssueCode::NickelSourceTooLarge)
        );
    }
}
