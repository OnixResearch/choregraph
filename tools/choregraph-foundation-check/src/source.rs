const CORE_MARKER_ISSUE_LIMIT: usize = 2;

const CORE_FORBIDDEN_SOURCE_TOKENS: &[&str] = &[
    "std::env",
    "std::fs",
    "std::net",
    "std::process",
    "std::time",
    "async ",
    "eprintln!",
    "println!",
    "tokio::",
    "reqwest::",
    "cap_std::",
    "nickel_lang",
];

#[derive(Clone, Copy)]
struct Markers {
    is_freestanding: bool,
    uses_alloc: bool,
}

fn markers(model: &crate::types::WorkspaceModel) -> Markers {
    let mut result = Markers {
        is_freestanding: false,
        uses_alloc: false,
    };
    for source in &model.core_sources {
        result.is_freestanding |= source.text.contains("#![no_std]");
        result.uses_alloc |= source.text.contains("extern crate alloc");
    }
    result
}

fn validate_markers(markers: Markers) -> Vec<crate::types::ValidationIssue> {
    let mut issues = Vec::with_capacity(CORE_MARKER_ISSUE_LIMIT);
    if !markers.is_freestanding {
        issues.push(crate::validator::issue(crate::validator::IssueFacts {
            code: "foundation.core-no-std",
            subject: crate::validator::CORE_PACKAGE,
            detail: String::from("core source does not declare no_std"),
        }));
    }
    if !markers.uses_alloc {
        issues.push(crate::validator::issue(crate::validator::IssueFacts {
            code: "foundation.core-alloc",
            subject: crate::validator::CORE_PACKAGE,
            detail: String::from("core source does not declare alloc"),
        }));
    }
    issues
}

fn validate_tokens(model: &crate::types::WorkspaceModel) -> Vec<crate::types::ValidationIssue> {
    let maximum_issue_count = model
        .core_sources
        .len()
        .saturating_mul(CORE_FORBIDDEN_SOURCE_TOKENS.len());
    let mut issues = Vec::with_capacity(maximum_issue_count);
    for source in &model.core_sources {
        for token in CORE_FORBIDDEN_SOURCE_TOKENS {
            if source.text.contains(token) {
                issues.push(crate::validator::issue(crate::validator::IssueFacts {
                    code: "foundation.forbidden-core-source",
                    subject: &source.path,
                    detail: format!("forbidden source token: {token}"),
                }));
            }
        }
    }
    issues
}

pub(crate) fn validate_sources(
    model: &crate::types::WorkspaceModel,
) -> Vec<crate::types::ValidationIssue> {
    let mut issues = validate_markers(markers(model));
    issues.extend(validate_tokens(model));
    issues
}
