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

const HISTORY_FORBIDDEN_SOURCE_TOKENS: &[&str] = &[
    "std::env",
    "std::fs",
    "std::net",
    "std::process",
    "std::time",
    "bounded_exec",
    "redb::",
    "reqwest::",
    "tokio::",
    "lattice::",
    "animus::",
    "weft::",
    "rat_widgets::",
    "tile::",
    "println!",
    "eprintln!",
];

const HISTORY_MACHINE_REQUIRED_DELEGATION_TOKENS: &[&str] = &[
    "choregraph_core::replay(replay_input)",
    "choregraph_core::replay_concrete(replay_input)",
];

const HISTORY_MACHINE_FORBIDDEN_TRANSITION_TOKENS: &[&str] = &[
    "choregraph_core::step(",
    "choregraph_core::step_concrete(",
    "crate::verified::indexed_step",
    "(LocalState::Pending, Action::MarkReady)",
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

const READINESS_SOURCE_SUFFIX: &str = "analytics/mod.rs";
const READINESS_FUNCTION_START: &str = "pub fn ready_nodes(";
const READINESS_FUNCTION_END: &str = "/// Returns one deterministic resume decision";
const READINESS_REQUIRED_DELEGATION: &str = "crate::verified::ready_frontier(";
const READINESS_FORBIDDEN_COPY_TOKENS: &[&str] = &[
    "for node_id in &machine.topological_order",
    "LocalState::Pending",
    "dependencies_complete(machine, state, node_id)",
];
const PROOF_SOURCE_SUFFIXES: &[&str] = &[
    "verified.rs",
    "verified/kernel.rs",
    "verified/readiness.rs",
    "verified/topology.rs",
];
const PROOF_TRUST_FORBIDDEN_TOKENS: &[&str] = &[
    "#[verifier::external_body]",
    "assume_specification",
    "assume(",
    "axiom ",
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

fn validate_no_std_boundary(
    package: &'static str,
    sources: &[crate::types::SourceUnit],
) -> Vec<crate::types::ValidationIssue> {
    let text = combined_text(sources);
    let mut issues = Vec::with_capacity(CORE_MARKER_ISSUE_LIMIT);
    if !text.contains("#![no_std]") {
        issues.push(crate::validator::issue(crate::validator::IssueFacts {
            code: "foundation.history-no-std",
            subject: package,
            detail: String::from("history boundary does not declare no_std"),
        }));
    }
    if !text.contains("extern crate alloc") {
        issues.push(crate::validator::issue(crate::validator::IssueFacts {
            code: "foundation.history-alloc",
            subject: package,
            detail: String::from("history boundary does not declare alloc"),
        }));
    }
    issues
}

fn validate_forbidden_history_tokens(
    package: &'static str,
    sources: &[crate::types::SourceUnit],
) -> Vec<crate::types::ValidationIssue> {
    let maximum_issue_count = sources
        .len()
        .saturating_mul(HISTORY_FORBIDDEN_SOURCE_TOKENS.len());
    let mut issues = Vec::with_capacity(maximum_issue_count);
    for source in sources {
        for token in HISTORY_FORBIDDEN_SOURCE_TOKENS {
            if source.text.contains(token) {
                issues.push(crate::validator::issue(crate::validator::IssueFacts {
                    code: "foundation.forbidden-history-source",
                    subject: &source.path,
                    detail: format!("{package} contains forbidden authority token: {token}"),
                }));
            }
        }
    }
    issues
}

fn validate_history_machine_delegation(
    sources: &[crate::types::SourceUnit],
) -> Vec<crate::types::ValidationIssue> {
    let text = combined_text(sources);
    debug_assert!(!HISTORY_MACHINE_REQUIRED_DELEGATION_TOKENS.is_empty());
    debug_assert!(!HISTORY_MACHINE_FORBIDDEN_TRANSITION_TOKENS.is_empty());
    let maximum_issue_count = HISTORY_MACHINE_REQUIRED_DELEGATION_TOKENS
        .len()
        .saturating_add(HISTORY_MACHINE_FORBIDDEN_TRANSITION_TOKENS.len());
    let mut issues = Vec::with_capacity(maximum_issue_count);
    for token in HISTORY_MACHINE_REQUIRED_DELEGATION_TOKENS {
        if !text.contains(token) {
            issues.push(crate::validator::issue(crate::validator::IssueFacts {
                code: "foundation.history-missing-replay-delegation",
                subject: "choregraph-history-machine",
                detail: format!("required production replay delegation is absent: {token}"),
            }));
        }
    }
    for token in HISTORY_MACHINE_FORBIDDEN_TRANSITION_TOKENS {
        if text.contains(token) {
            issues.push(crate::validator::issue(crate::validator::IssueFacts {
                code: "foundation.history-duplicate-transition",
                subject: "choregraph-history-machine",
                detail: format!("forbidden transition implementation token: {token}"),
            }));
        }
    }
    issues
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

// r[impl choregraph.proof.single_step.readiness_completeness_negative]
// r[impl choregraph.proof.single_step.readiness_completeness_compatibility]
// r[impl choregraph.proof.single_step.readiness_completeness_evidence]
// r[impl choregraph.proof.single_step.readiness_completeness_validation]
// r[impl choregraph.proof.single_step.readiness_completeness_boundary]
// r[verify choregraph.proof.single_step.readiness_completeness_negative]
// r[verify choregraph.proof.single_step.readiness_completeness_compatibility]
// r[verify choregraph.proof.single_step.readiness_completeness_evidence]
// r[verify choregraph.proof.single_step.readiness_completeness_validation]
// r[verify choregraph.proof.single_step.readiness_completeness_boundary]
fn validate_readiness_delegation(
    model: &crate::types::WorkspaceModel,
) -> Vec<crate::types::ValidationIssue> {
    debug_assert!(!READINESS_REQUIRED_DELEGATION.is_empty());
    debug_assert!(!READINESS_FORBIDDEN_COPY_TOKENS.is_empty());
    let Some(source) = model
        .core_sources
        .iter()
        .find(|source| source.path.ends_with(READINESS_SOURCE_SUFFIX))
    else {
        return vec![crate::validator::issue(crate::validator::IssueFacts {
            code: "foundation.readiness-source-missing",
            subject: READINESS_SOURCE_SUFFIX,
            detail: String::from("production readiness source is absent"),
        })];
    };
    let Some(start) = source.text.find(READINESS_FUNCTION_START) else {
        return vec![crate::validator::issue(crate::validator::IssueFacts {
            code: "foundation.readiness-function-missing",
            subject: &source.path,
            detail: String::from("production ready_nodes function is absent"),
        })];
    };
    let function_source = &source.text[start..];
    let Some(end) = function_source.find(READINESS_FUNCTION_END) else {
        return vec![crate::validator::issue(crate::validator::IssueFacts {
            code: "foundation.readiness-function-boundary",
            subject: &source.path,
            detail: String::from("production ready_nodes boundary marker is absent"),
        })];
    };
    let function_source = &function_source[..end];
    let maximum_issue_count = READINESS_FORBIDDEN_COPY_TOKENS.len().saturating_add(1);
    let mut issues = Vec::with_capacity(maximum_issue_count);
    if !function_source.contains(READINESS_REQUIRED_DELEGATION) {
        issues.push(crate::validator::issue(crate::validator::IssueFacts {
            code: "foundation.readiness-delegation-missing",
            subject: &source.path,
            detail: format!("required production readiness delegation is absent: {READINESS_REQUIRED_DELEGATION}"),
        }));
    }
    for token in READINESS_FORBIDDEN_COPY_TOKENS {
        if function_source.contains(token) {
            issues.push(crate::validator::issue(crate::validator::IssueFacts {
                code: "foundation.readiness-copy",
                subject: &source.path,
                detail: format!("production ready_nodes contains copied decision token: {token}"),
            }));
        }
    }
    issues
}

// r[verify choregraph.proof.single_step.readiness_completeness_negative]
fn validate_proof_trust_surface(
    model: &crate::types::WorkspaceModel,
) -> Vec<crate::types::ValidationIssue> {
    debug_assert!(!PROOF_SOURCE_SUFFIXES.is_empty());
    debug_assert!(!PROOF_TRUST_FORBIDDEN_TOKENS.is_empty());
    let maximum_issue_count = PROOF_SOURCE_SUFFIXES
        .len()
        .saturating_mul(PROOF_TRUST_FORBIDDEN_TOKENS.len());
    let mut issues = Vec::with_capacity(maximum_issue_count);
    for source in &model.core_sources {
        if !PROOF_SOURCE_SUFFIXES
            .iter()
            .any(|suffix| source.path.ends_with(suffix))
        {
            continue;
        }
        for token in PROOF_TRUST_FORBIDDEN_TOKENS {
            if source.text.contains(token) {
                issues.push(crate::validator::issue(crate::validator::IssueFacts {
                    code: "foundation.proof-trust-escape",
                    subject: &source.path,
                    detail: format!("proof source contains forbidden trust token: {token}"),
                }));
            }
        }
    }
    issues
}

pub(crate) fn validate_sources(
    model: &crate::types::WorkspaceModel,
) -> Vec<crate::types::ValidationIssue> {
    debug_assert!(!CORE_FORBIDDEN_SOURCE_TOKENS.is_empty());
    debug_assert!(!WASM_FORBIDDEN_SOURCE_TOKENS.is_empty());
    let mut issues = validate_markers(markers(model));
    issues.extend(validate_tokens(model));
    issues.extend(validate_readiness_delegation(model));
    issues.extend(validate_proof_trust_surface(model));
    if model
        .packages
        .iter()
        .any(|package| package.name == "choregraph-history")
    {
        issues.extend(validate_no_std_boundary(
            "choregraph-history",
            &model.history_sources,
        ));
        issues.extend(validate_forbidden_history_tokens(
            "choregraph-history",
            &model.history_sources,
        ));
        issues.extend(validate_no_std_boundary(
            "choregraph-history-machine",
            &model.history_machine_sources,
        ));
        issues.extend(validate_forbidden_history_tokens(
            "choregraph-history-machine",
            &model.history_machine_sources,
        ));
        issues.extend(validate_no_std_boundary(
            "choregraph-weave",
            &model.weave_sources,
        ));
        issues.extend(validate_forbidden_history_tokens(
            "choregraph-weave",
            &model.weave_sources,
        ));
        issues.extend(validate_history_machine_delegation(
            &model.history_machine_sources,
        ));
    }
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
