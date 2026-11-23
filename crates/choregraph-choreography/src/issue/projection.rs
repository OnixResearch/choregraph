//! Diagnostic-only witness analysis for rejected Trellis endpoint projections.
//!
//! Trellis's production `projectable` and `project_endpoint` remain the sole
//! acceptance authority. This module runs only after `projectable(&global)` has
//! already returned `false`. It never accepts, widens, or rewrites a protocol;
//! if a precise witness cannot be established within reviewed bounds, it falls
//! back to an explicit incomplete-witness rejection.

const MAX_EXPLAINED_ROLES: usize = 4;
const HINT_REVIEW_CHOICE: &str =
    "review choice communication for affected roles; adding a notification alone may not suffice";
const NOTE_PROJECTION_REJECTED: &str = "Trellis rejected endpoint projectability";
const NOTE_WITNESS_INCOMPLETE: &str = "projection rejection witness is incomplete";

struct ResponsibleChoice<'a> {
    span: crate::model::participants::SourceSpan,
    decider: crate::model::participants::RoleId,
    branches: &'a [crate::model::tree::ChoiceBranch],
    gbranches: &'a [verified_logic::choreography_global::GlobalBranch],
}

struct RoleConflict<'a> {
    role: crate::model::participants::RoleId,
    first_branch: &'a crate::model::tree::ChoiceBranch,
    first_head: &'static str,
    second_branch: &'a crate::model::tree::ChoiceBranch,
    second_head: &'static str,
}

const fn is_decider_guarded(
    decider: u32,
    local: &verified_logic::choreography_local::LocalChoreo,
) -> bool {
    match local {
        verified_logic::choreography_local::LocalChoreo::Recv { peer, .. }
        | verified_logic::choreography_local::LocalChoreo::RecvOwned { peer, .. } => {
            *peer == decider
        }
        verified_logic::choreography_local::LocalChoreo::Offer { from, .. } => *from == decider,
        verified_logic::choreography_local::LocalChoreo::End
        | verified_logic::choreography_local::LocalChoreo::Send { .. }
        | verified_logic::choreography_local::LocalChoreo::SendOwned { .. }
        | verified_logic::choreography_local::LocalChoreo::InternalChoice { .. } => false,
    }
}

const fn local_head_kind(
    local: &verified_logic::choreography_local::LocalChoreo,
    decider: u32,
) -> &'static str {
    match local {
        verified_logic::choreography_local::LocalChoreo::End => "end",
        verified_logic::choreography_local::LocalChoreo::Send { .. } => "send",
        verified_logic::choreography_local::LocalChoreo::Recv { peer, .. } if *peer == decider => {
            "recv from decider"
        }
        verified_logic::choreography_local::LocalChoreo::Recv { .. } => "recv from other",
        verified_logic::choreography_local::LocalChoreo::SendOwned { .. } => "owned send",
        verified_logic::choreography_local::LocalChoreo::RecvOwned { peer, .. }
            if *peer == decider =>
        {
            "owned recv from decider"
        }
        verified_logic::choreography_local::LocalChoreo::RecvOwned { .. } => {
            "owned recv from other"
        }
        verified_logic::choreography_local::LocalChoreo::InternalChoice { .. } => "internal choice",
        verified_logic::choreography_local::LocalChoreo::Offer { from, .. } if *from == decider => {
            "offer from decider"
        }
        verified_logic::choreography_local::LocalChoreo::Offer { .. } => "offer from other",
    }
}

fn all_tails_equal(tails: &[&verified_logic::choreography_local::LocalChoreo]) -> bool {
    let Some(first) = tails.first() else {
        return false;
    };
    tails.iter().skip(1).all(|candidate| candidate == first)
}

fn all_decider_guarded(
    decider: u32,
    tails: &[&verified_logic::choreography_local::LocalChoreo],
) -> bool {
    !tails.is_empty()
        && tails
            .iter()
            .all(|candidate| is_decider_guarded(decider, candidate))
}

fn peel_common_send(tails: &mut [&verified_logic::choreography_local::LocalChoreo]) -> bool {
    let Some(verified_logic::choreography_local::LocalChoreo::Send {
        peer,
        label,
        payload_tag,
        ..
    }) = tails.first().copied()
    else {
        return false;
    };
    let is_common_send = tails.iter().all(|candidate| {
        matches!(
            candidate,
            verified_logic::choreography_local::LocalChoreo::Send {
                peer: candidate_peer,
                label: candidate_label,
                payload_tag: candidate_payload,
                ..
            } if *candidate_peer == *peer
                && *candidate_label == *label
                && *candidate_payload == *payload_tag
        )
    });
    if !is_common_send {
        return false;
    }
    for slot in tails.iter_mut() {
        if let verified_logic::choreography_local::LocalChoreo::Send { next, .. } = *slot {
            *slot = next;
        }
    }
    debug_assert!(!tails.is_empty());
    debug_assert!(tails.len() <= crate::model::bounds::MAX_BRANCHES);
    true
}

fn peel_common_recv(tails: &mut [&verified_logic::choreography_local::LocalChoreo]) -> bool {
    let Some(verified_logic::choreography_local::LocalChoreo::Recv {
        peer,
        label,
        payload_tag,
        ..
    }) = tails.first().copied()
    else {
        return false;
    };
    let is_common_recv = tails.iter().all(|candidate| {
        matches!(
            candidate,
            verified_logic::choreography_local::LocalChoreo::Recv {
                peer: candidate_peer,
                label: candidate_label,
                payload_tag: candidate_payload,
                ..
            } if *candidate_peer == *peer
                && *candidate_label == *label
                && *candidate_payload == *payload_tag
        )
    });
    if !is_common_recv {
        return false;
    }
    for slot in tails.iter_mut() {
        if let verified_logic::choreography_local::LocalChoreo::Recv { next, .. } = *slot {
            *slot = next;
        }
    }
    debug_assert!(!tails.is_empty());
    debug_assert!(tails.len() <= crate::model::bounds::MAX_BRANCHES);
    true
}

fn find_conflicting_pair(
    decider: u32,
    tails: &[&verified_logic::choreography_local::LocalChoreo],
) -> Option<(usize, usize)> {
    debug_assert!(!tails.is_empty());
    debug_assert!(tails.len() <= crate::model::bounds::MAX_BRANCHES);
    for first in 0..tails.len() {
        for second in first.saturating_add(1)..tails.len() {
            if tails[first] != tails[second]
                && (!is_decider_guarded(decider, tails[first])
                    || !is_decider_guarded(decider, tails[second]))
            {
                return Some((first, second));
            }
        }
    }
    None
}

fn find_responsible_choice<'a>(
    protocol: &'a crate::model::tree::Node,
    global: &'a verified_logic::choreography_global::GlobalChoreo,
) -> Option<ResponsibleChoice<'a>> {
    debug_assert!(!verified_logic::choreography_projection::projectable(
        global
    ));
    debug_assert!(protocol.span().is_some());
    let mut current_node = protocol;
    let mut current_global = global;
    for _step in 0..crate::model::bounds::MAX_STEPS {
        match (current_node, current_global) {
            (
                crate::model::tree::Node::Communication { next, .. },
                verified_logic::choreography_global::GlobalChoreo::Comm { next: gnext, .. },
            )
            | (
                crate::model::tree::Node::Transfer { next, .. },
                verified_logic::choreography_global::GlobalChoreo::OwnedTransfer {
                    next: gnext,
                    ..
                },
            ) => {
                current_node = next;
                current_global = gnext;
            }
            (
                crate::model::tree::Node::Choice {
                    decider,
                    branches,
                    span,
                },
                verified_logic::choreography_global::GlobalChoreo::Choice {
                    decider: gdecider,
                    branches: gbranches,
                },
            ) => {
                if decider.0 != *gdecider || branches.len() != gbranches.len() {
                    return None;
                }
                let mut failing_branch = None;
                for (branch, gbranch) in branches.iter().zip(gbranches.iter()) {
                    if !verified_logic::choreography_projection::projectable(&gbranch.body) {
                        failing_branch = Some((&branch.body, &gbranch.body));
                        break;
                    }
                }
                if let Some((next_node, next_global)) = failing_branch {
                    current_node = next_node;
                    current_global = next_global;
                } else {
                    return Some(ResponsibleChoice {
                        span: *span,
                        decider: *decider,
                        branches,
                        gbranches,
                    });
                }
            }
            _other => return None,
        }
    }
    None
}

fn find_role_conflict<'a>(
    role: crate::model::participants::RoleId,
    choice: &ResponsibleChoice<'a>,
) -> Option<RoleConflict<'a>> {
    debug_assert_ne!(role, choice.decider);
    debug_assert_eq!(choice.branches.len(), choice.gbranches.len());
    let mut projected = Vec::with_capacity(choice.gbranches.len());
    for gbranch in choice.gbranches {
        projected.push(verified_logic::choreography_projection::project_endpoint(
            role.0,
            &gbranch.body,
        ));
    }
    let mut tails = Vec::with_capacity(projected.len());
    for local in &projected {
        tails.push(local);
    }
    for _peel in 0..crate::model::bounds::MAX_STEPS {
        if all_tails_equal(&tails) || all_decider_guarded(choice.decider.0, &tails) {
            return None;
        }
        if peel_common_send(&mut tails) || peel_common_recv(&mut tails) {
            continue;
        }
        let (first, second) = find_conflicting_pair(choice.decider.0, &tails)?;
        return Some(RoleConflict {
            role,
            first_branch: &choice.branches[first],
            first_head: local_head_kind(tails[first], choice.decider.0),
            second_branch: &choice.branches[second],
            second_head: local_head_kind(tails[second], choice.decider.0),
        });
    }
    None
}

fn fallback_incomplete_witness(
    span: crate::model::participants::SourceSpan,
) -> Vec<super::Diagnostic> {
    let mut diagnostic = super::Diagnostic::error(
        super::DiagnosticCode::TrellisProjectionRejected,
        span,
        NOTE_PROJECTION_REJECTED,
    )
    .with_hint(HINT_REVIEW_CHOICE);
    diagnostic.notes.push(String::from(NOTE_WITNESS_INCOMPLETE));
    let mut diagnostics = vec![diagnostic];
    super::normalize_diagnostics(&mut diagnostics);
    debug_assert!(super::diagnostics_are_safe(&diagnostics));
    debug_assert_eq!(diagnostics.len(), 1);
    diagnostics
}

fn build_witness_diagnostics(
    choice: &ResponsibleChoice<'_>,
    roles: &[crate::model::participants::RoleId],
) -> Option<Vec<super::Diagnostic>> {
    let mut diagnostic = super::Diagnostic {
        code: super::DiagnosticCode::TrellisProjectionRejected,
        severity: super::DiagnosticSeverity::Error,
        primary: choice.span,
        secondary: Vec::new(),
        notes: Vec::new(),
        hints: vec![String::from(HINT_REVIEW_CHOICE)],
        safe_replacement: None,
    };
    let mut explained_roles: usize = 0;
    for &role in roles {
        if role == choice.decider {
            continue;
        }
        if let Some(conflict) = find_role_conflict(role, choice) {
            diagnostic = diagnostic
                .with_secondary(conflict.first_branch.span)
                .with_secondary(conflict.second_branch.span);
            if explained_roles < MAX_EXPLAINED_ROLES {
                let note = format!(
                    "role {} cannot distinguish choice by decider {}: incompatible offer heads on branch {} ({}) and branch {} ({})",
                    conflict.role.0,
                    choice.decider.0,
                    conflict.first_branch.label.0,
                    conflict.first_head,
                    conflict.second_branch.label.0,
                    conflict.second_head,
                );
                if super::safe_diagnostic_text(&note) {
                    diagnostic.notes.push(note);
                }
                explained_roles = explained_roles.saturating_add(1);
            }
        }
    }
    if explained_roles == 0 {
        return None;
    }
    let mut diagnostics = vec![diagnostic];
    super::normalize_diagnostics(&mut diagnostics);
    debug_assert!(super::diagnostics_are_safe(&diagnostics));
    debug_assert_eq!(diagnostics.len(), 1);
    Some(diagnostics)
}

pub(crate) fn explain_rejection(
    protocol: &crate::model::tree::Node,
    global: &verified_logic::choreography_global::GlobalChoreo,
    roles: &[crate::model::participants::RoleId],
) -> Result<Vec<super::Diagnostic>, crate::admission::Error> {
    let root_span = protocol
        .span()
        .ok_or(crate::admission::Error::UnsupportedNode)?;
    let diagnostics = find_responsible_choice(protocol, global)
        .and_then(|choice| build_witness_diagnostics(&choice, roles))
        .unwrap_or_else(|| fallback_incomplete_witness(root_span));
    if !super::diagnostics_are_safe(&diagnostics) {
        return Ok(vec![super::Diagnostic::error(
            super::DiagnosticCode::UnsafeDiagnostic,
            root_span,
            "diagnostic safety invariant failed",
        )]);
    }
    debug_assert!(super::diagnostics_are_safe(&diagnostics));
    debug_assert!(!diagnostics.is_empty());
    Ok(diagnostics)
}
