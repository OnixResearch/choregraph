// A fail-closed lexical subset of Nickel 1.17, before the sandboxed evaluator.

// r[impl choregraph.nickel.profile]
type Issue = crate::diagnostic::FrontendIssueCode;
type FrontendFailure = crate::diagnostic::FrontendFailure;

fn fail(code: Issue) -> FrontendFailure {
    FrontendFailure::one(code)
}

const fn is_identifier_tail(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'\'')
}

fn skip_string(mut remaining: &[u8]) -> Result<&[u8], FrontendFailure> {
    while let Some((&byte, tail)) = remaining.split_first() {
        remaining = match byte {
            b'"' => return Ok(tail),
            b'%' if tail.starts_with(b"{") => return Err(fail(Issue::ForbiddenNickelAuthority)),
            b'\n' | b'\r' => return Err(fail(Issue::ForbiddenNickelAuthority)),
            b'\\' => match tail.split_first() {
                Some((b'\'' | b'"' | b'\\' | b'%' | b'n' | b'r' | b't', escaped_tail)) => {
                    escaped_tail
                }
                Some((b'x' | b'u', _)) => return Err(fail(Issue::ForbiddenNickelAuthority)),
                Some(_) | None => return Err(fail(Issue::MalformedNickelSource)),
            },
            _ => tail,
        };
    }
    Err(fail(Issue::MalformedNickelSource))
}

fn forbidden_prefix(byte: u8, tail: &[u8]) -> bool {
    match byte {
        // Multiline, symbolic and raw-primop forms all contain an unquoted %.
        b'%' => true,
        b'/' => tail.starts_with(b"/") || tail.starts_with(b"*"),
        // Don't infer a left boundary; a preceding byte can end another token.
        b'i' => tail
            .strip_prefix(b"mport")
            .is_some_and(|after| !after.first().is_some_and(|byte| is_identifier_tail(*byte))),
        _ => false,
    }
}

fn after_comment(tail: &[u8]) -> &[u8] {
    tail.splitn(2, |byte| *byte == b'\n')
        .nth(1)
        .unwrap_or_default()
}

fn is_plain_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b" \t\r\n_-'?,;:$=&.+*/@|!{}[]()<>".contains(&byte)
}

fn advance(byte: u8, tail: &[u8]) -> Result<&[u8], FrontendFailure> {
    match byte {
        b'#' => Ok(after_comment(tail)),
        b'"' => skip_string(tail),
        _ if forbidden_prefix(byte, tail) => Err(fail(Issue::ForbiddenNickelAuthority)),
        _ if is_plain_byte(byte) => Ok(tail),
        _ => Err(fail(Issue::MalformedNickelSource)),
    }
}

fn scan(mut remaining: &[u8]) -> Result<(), FrontendFailure> {
    while let Some((&byte, tail)) = remaining.split_first() {
        let rest = advance(byte, tail)?;
        debug_assert!(rest.len() < remaining.len());
        remaining = rest;
    }
    Ok(())
}

pub fn validate_restricted(source: &[u8], max_source_bytes: u64) -> Result<(), FrontendFailure> {
    let source_bytes =
        u64::try_from(source.len()).map_err(|_| fail(Issue::NickelSourceTooLarge))?;
    if source.is_empty() || source_bytes > max_source_bytes {
        return Err(fail(Issue::NickelSourceTooLarge));
    }
    let source = core::str::from_utf8(source).map_err(|_| fail(Issue::MalformedNickelSource))?;
    let bytes = source.as_bytes();
    if bytes.contains(&0)
        || bytes
            .windows(2)
            .any(|pair| pair[0] == b'\r' && pair[1] != b'\n')
        || bytes.ends_with(b"\r")
    {
        return Err(fail(Issue::MalformedNickelSource));
    }
    scan(bytes)
}

#[cfg(test)]
mod tests {
    const SOURCE_LIMIT_BYTES: u64 = 1_024;

    fn rejection(source: &[u8]) -> Option<crate::diagnostic::FrontendIssueCode> {
        super::validate_restricted(source, SOURCE_LIMIT_BYTES)
            .err()
            .map(|failure| failure.issues[0].code())
    }

    #[test]
    fn accepts_plain_data_strings_and_comments() {
        let source = concat!(
            "# A comment with import, quotes, and percent delimiters.\r\n",
            r#"{ imports = [], note = "é: import, /* */, //, \" and \%{text}", value = 'tag }"#,
        );
        assert_eq!(
            super::validate_restricted(source.as_bytes(), SOURCE_LIMIT_BYTES),
            Ok(())
        );
    }

    #[test]
    fn rejects_non_profile_forms() {
        for source in [
            &br#"let value = import "example.ncl" in value"#[..],
            br#"{ value = "prefix %{1}" }"#,
            br#"{ value = m%"plain text"% }"#,
            br#"{ value = m%%"plain text"%% }"#,
            br#"{ value = example-s%"plain text"% }"#,
            b"{ value = \"first\nsecond\" }",
            b"{ value = 5 % 2 }",
            br#"{ value = "\x41" }"#,
            br#"{ value = "\u{41}" }"#,
            b"// ordinary text\n{}",
            b"/* ordinary text */ {}",
        ] {
            assert_eq!(
                rejection(source),
                Some(super::Issue::ForbiddenNickelAuthority)
            );
        }
    }

    #[test]
    fn rejects_malformed_and_oversized_input() {
        for source in [
            &b"{ value = \"unterminated }"[..],
            b"{ value = \"bad\\q\" }",
            b"{ value = \"\xff\" }",
            b"{ value = \"\0\" }",
            b"# comment\r{}",
        ] {
            assert_eq!(rejection(source), Some(super::Issue::MalformedNickelSource));
        }
        assert_eq!(
            super::validate_restricted(b"{}", 1)
                .err()
                .map(|failure| failure.issues[0].code()),
            Some(super::Issue::NickelSourceTooLarge)
        );
    }
}
