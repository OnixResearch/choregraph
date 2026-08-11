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

const WASM_REQUIRED_DELEGATION_TOKENS: &[&str] = &[
    "validate_machine_artifact(",
    "validate_durable_machine_artifact(",
    "validate_concrete_machine_artifact(",
    "step(&context.machine",
    "step_concrete(&context.machine",
    "replay(input)",
    "replay_concrete(input)",
];

const WASM_FORBIDDEN_SOURCE_TOKENS: &[&str] = &[
    "std::fs",
    "std::net",
    "std::process",
    "std::time",
    "Command::",
    "OnceLock",
    "Mutex",
    "RwLock",
    "thread_local!",
    "static mut",
    "unsafe ",
    "unsafe{",
    "MachineSpec::new",
    "MachineSpecV2::new",
    "NodeSpec::new",
    "crate::verified::indexed_step",
    "(LocalState::Pending, Action::MarkReady)",
];

const EXPECTED_WIT_EXPORT: &str = "export call: func(request: list<u8>) -> list<u8>;";

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

fn combined_text(sources: &[crate::types::SourceUnit]) -> String {
    let total_bytes = sources
        .iter()
        .map(|source| source.text.len())
        .fold(0_usize, usize::saturating_add);
    let mut text = String::with_capacity(total_bytes);
    for source in sources {
        text.push_str(&source.text);
        text.push('\n');
    }
    text
}

fn validate_wasm_delegation(
    model: &crate::types::WorkspaceModel,
) -> Vec<crate::types::ValidationIssue> {
    let text = combined_text(&model.wasm_adapter_sources);
    let mut issues = Vec::with_capacity(WASM_REQUIRED_DELEGATION_TOKENS.len());
    for token in WASM_REQUIRED_DELEGATION_TOKENS {
        if !text.contains(token) {
            issues.push(crate::validator::issue(crate::validator::IssueFacts {
                code: "foundation.wasm-missing-delegation",
                subject: "choregraph-wasm",
                detail: format!("required production delegation is absent: {token}"),
            }));
        }
    }
    issues
}

fn validate_wasm_forbidden_tokens(
    model: &crate::types::WorkspaceModel,
) -> Vec<crate::types::ValidationIssue> {
    let maximum_issue_count = model
        .wasm_adapter_sources
        .len()
        .saturating_mul(WASM_FORBIDDEN_SOURCE_TOKENS.len());
    let mut issues = Vec::with_capacity(maximum_issue_count);
    for source in &model.wasm_adapter_sources {
        for token in WASM_FORBIDDEN_SOURCE_TOKENS {
            if source.text.contains(token) {
                issues.push(crate::validator::issue(crate::validator::IssueFacts {
                    code: "foundation.forbidden-wasm-source",
                    subject: &source.path,
                    detail: format!("forbidden WASM adapter token: {token}"),
                }));
            }
        }
    }
    issues
}

fn validate_wit_export(text: &str) -> Option<crate::types::ValidationIssue> {
    if text
        .lines()
        .filter(|line| line.trim_start().starts_with("export "))
        .count()
        == 1
        && text.lines().any(|line| line.trim() == EXPECTED_WIT_EXPORT)
    {
        None
    } else {
        Some(crate::validator::issue(crate::validator::IssueFacts {
            code: "foundation.wasm-export-surface",
            subject: "kernel.wit",
            detail: String::from("WIT must expose only the selected byte call"),
        }))
    }
}

fn validate_wit_imports(text: &str) -> Option<crate::types::ValidationIssue> {
    if text
        .lines()
        .any(|line| line.trim_start().starts_with("import "))
    {
        Some(crate::validator::issue(crate::validator::IssueFacts {
            code: "foundation.wasm-host-import",
            subject: "kernel.wit",
            detail: String::from("WIT must not declare a host import"),
        }))
    } else {
        None
    }
}

fn validate_wit(model: &crate::types::WorkspaceModel) -> Vec<crate::types::ValidationIssue> {
    let text = combined_text(&model.wasm_wit_sources);
    [validate_wit_export(&text), validate_wit_imports(&text)]
        .into_iter()
        .flatten()
        .collect()
}

pub(crate) fn validate_sources(
    model: &crate::types::WorkspaceModel,
) -> Vec<crate::types::ValidationIssue> {
    let mut issues = validate_markers(markers(model));
    issues.extend(validate_tokens(model));
    if model
        .packages
        .iter()
        .any(|package| package.name == "choregraph-wasm")
    {
        issues.extend(validate_wasm_delegation(model));
        issues.extend(validate_wasm_forbidden_tokens(model));
        issues.extend(validate_wit(model));
    }
    issues
}
