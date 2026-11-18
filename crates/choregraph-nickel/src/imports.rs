// Import closure checks retain explicit bounded collection and digest compatibility shapes.
#![allow(
    unknown_lints,
    assertion_density,
    borrowed_argument_types,
    compound_condition,
    function_length,
    missing_const_fn,
    raw_arithmetic_overflow,
    reason = "import closure checks retain explicit bounded collection and digest compatibility shapes"
)]

// r[impl choregraph.nickel.imports]

const IMPORT_CLOSURE_CONTEXT: &str = "choregraph import closure v1";
const LENGTH_FRAME_BYTES: usize = size_of::<u64>();
const BLAKE3_HEX_BYTES: usize = blake3::OUT_LEN * 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObservedImport<'a> {
    pub logical_id: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedImportClosure {
    pub canonical_material: Vec<u8>,
    pub blake3: String,
    pub import_count: usize,
    pub total_bytes: usize,
}

fn fail(code: crate::diagnostic::FrontendIssueCode) -> crate::diagnostic::FrontendFailure {
    crate::diagnostic::FrontendFailure::one(code)
}

fn safe_identifier(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/'))
}

fn safe_relative_path(value: &str, maximum: usize) -> bool {
    if value.is_empty()
        || value.len() > maximum
        || value.starts_with('/')
        || value.contains('\\')
        || value.contains(':')
        || !value.is_ascii()
    {
        return false;
    }
    value.split('/').all(|segment| {
        !segment.is_empty()
            && !matches!(segment, "." | "..")
            && segment
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    })
}

fn valid_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn media_label(
    media: &crate::model::dependency::ImportMediaClass,
) -> Result<&'static str, crate::diagnostic::FrontendFailure> {
    if matches!(
        media,
        crate::model::dependency::ImportMediaClass::NickelData
    ) {
        return Ok("nickel-data");
    }
    if matches!(media, crate::model::dependency::ImportMediaClass::Utf8Text) {
        return Ok("utf8-text");
    }
    if matches!(
        media,
        crate::model::dependency::ImportMediaClass::PreservesPacked
    ) {
        return Ok("preserves-packed");
    }
    Err(fail(
        crate::diagnostic::FrontendIssueCode::UnsupportedImportMediaClass,
    ))
}

fn frame(material: &mut Vec<u8>, value: &[u8]) -> Result<(), crate::diagnostic::FrontendFailure> {
    let length_bytes = u64::try_from(value.len())
        .map_err(|_| fail(crate::diagnostic::FrontendIssueCode::ImportTotalBytesExceeded))?;
    material.extend_from_slice(&length_bytes.to_le_bytes()[..LENGTH_FRAME_BYTES]);
    material.extend_from_slice(value);
    Ok(())
}

pub fn validate_import_closure(
    declarations: &[crate::model::dependency::ImportDeclaration],
    observations: &[ObservedImport<'_>],
    limits: crate::profile::FrontendLimits,
) -> Result<ValidatedImportClosure, crate::diagnostic::FrontendFailure> {
    if declarations.len() > limits.max_imports {
        return Err(fail(crate::diagnostic::FrontendIssueCode::TooManyImports));
    }
    let mut declared = std::collections::BTreeMap::new();
    for declaration in declarations {
        if !safe_identifier(&declaration.logical_id, limits.max_import_identifier_bytes)
            || !safe_identifier(&declaration.root_id, limits.max_import_identifier_bytes)
        {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::InvalidImportIdentifier,
            ));
        }
        if !safe_relative_path(&declaration.relative_path, limits.max_import_path_bytes) {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::InvalidImportPath,
            ));
        }
        let declared_limit_bytes = usize::try_from(declaration.max_bytes)
            .map_err(|_| fail(crate::diagnostic::FrontendIssueCode::ImportByteLimitExceeded))?;
        if declared_limit_bytes == 0 || declared_limit_bytes > limits.max_import_bytes {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::ImportByteLimitExceeded,
            ));
        }
        if !valid_blake3(&declaration.expected_blake3) {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::InvalidImportHash,
            ));
        }
        if declared
            .insert(declaration.logical_id.as_str(), declaration)
            .is_some()
        {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::DuplicateImportDeclaration,
            ));
        }
    }

    let mut observed = std::collections::BTreeMap::new();
    for observation in observations {
        if observed
            .insert(observation.logical_id, observation.bytes)
            .is_some()
        {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::DuplicateImportObservation,
            ));
        }
        if !declared.contains_key(observation.logical_id) {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::UndeclaredImportObservation,
            ));
        }
    }

    let mut total_bytes = 0usize;
    let mut canonical_material = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (logical_id, declaration) in declared {
        let bytes = observed
            .get(logical_id)
            .copied()
            .ok_or_else(|| fail(crate::diagnostic::FrontendIssueCode::MissingImportObservation))?;
        let declared_limit_bytes = usize::try_from(declaration.max_bytes)
            .map_err(|_| fail(crate::diagnostic::FrontendIssueCode::ImportByteLimitExceeded))?;
        if bytes.len() > declared_limit_bytes {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::ImportByteLimitExceeded,
            ));
        }
        total_bytes = total_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| fail(crate::diagnostic::FrontendIssueCode::ImportTotalBytesExceeded))?;
        if total_bytes > limits.max_total_import_bytes {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::ImportTotalBytesExceeded,
            ));
        }
        let observed_hash = blake3::hash(bytes).to_hex();
        if observed_hash.as_str() != declaration.expected_blake3 {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::ImportHashMismatch,
            ));
        }
        frame(&mut canonical_material, logical_id.as_bytes())?;
        frame(&mut canonical_material, declaration.root_id.as_bytes())?;
        frame(
            &mut canonical_material,
            declaration.relative_path.as_bytes(),
        )?;
        frame(
            &mut canonical_material,
            media_label(&declaration.media_class)?.as_bytes(),
        )?;
        frame(
            &mut canonical_material,
            declaration.expected_blake3.as_bytes(),
        )?;
        if !seen.insert(logical_id) {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::DuplicateImportDeclaration,
            ));
        }
    }
    let blake3 = blake3::derive_key(IMPORT_CLOSURE_CONTEXT, &canonical_material);
    Ok(ValidatedImportClosure {
        canonical_material,
        blake3: blake3::Hash::from_bytes(blake3).to_hex().to_string(),
        import_count: observations.len(),
        total_bytes,
    })
}
