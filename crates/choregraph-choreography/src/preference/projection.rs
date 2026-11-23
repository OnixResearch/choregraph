//! Attach admitted annotations to the unchanged Trellis local set.
//!
//! This module does not project. It reads the local protocols produced by
//! `crate::endpoint::project_all` and checks that each annotated global choice
//! corresponds to exactly one decider `InternalChoice` with the same labels.
//! Any address it cannot establish is refused, not approximated.

use serde::{Deserialize, Serialize};

/// Declared metadata attached to one decider's `InternalChoice`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "DeciderPreference", deny_unknown_fields)]
pub struct DeciderAnnotation {
    pub global_path: Vec<super::model::PathStep>,
    pub local_path: Vec<super::model::PathStep>,
    pub labels: Vec<crate::model::participants::LabelId>,
    pub factors: Vec<super::model::FactorDeclaration>,
    pub world: Option<super::model::WorldDeclaration>,
    pub span: crate::model::participants::SourceSpan,
}

/// One role's entry in the annotated complete local set.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "AnnotatedLocal", deny_unknown_fields)]
pub struct AnnotatedLocal {
    pub role: crate::model::participants::RoleId,
    pub local_artifact: crate::linkage::digest::Identity,
    pub decisions: Vec<DeciderAnnotation>,
}

/// One entry for every declared role, in normalized role order.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "AnnotatedLocalSetBody", deny_unknown_fields)]
pub struct AnnotatedLocalSet {
    pub complete_local_set: crate::linkage::digest::Identity,
    pub locals: Vec<AnnotatedLocal>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Direction {
    Send,
    Receive,
    SendOwned,
    ReceiveOwned,
}

struct LocalMessage<'a> {
    direction: Direction,
    peer: crate::model::participants::RoleId,
    label: crate::model::participants::LabelId,
    next: &'a crate::endpoint::LocalProtocol,
}

struct GlobalMessage<'a> {
    from: crate::model::participants::RoleId,
    to: crate::model::participants::RoleId,
    label: crate::model::participants::LabelId,
    is_owned: bool,
    next: &'a crate::model::tree::Node,
}

struct Cursor<'a> {
    node: &'a crate::model::tree::Node,
    local: &'a crate::endpoint::LocalProtocol,
}

type Advance<'a> =
    Result<(Cursor<'a>, Option<super::model::PathStep>), crate::issue::DiagnosticCode>;

const fn origin() -> crate::model::participants::SourceSpan {
    crate::model::participants::SourceSpan { start: 0, end: 0 }
}

fn issue(
    code: crate::issue::DiagnosticCode,
    span: crate::model::participants::SourceSpan,
    note: &'static str,
) -> Vec<crate::issue::Diagnostic> {
    vec![crate::issue::Diagnostic::error(code, span, note)]
}

fn local_message(local: &crate::endpoint::LocalProtocol) -> Option<LocalMessage<'_>> {
    let (direction, peer, label, next) = match local {
        crate::endpoint::LocalProtocol::Send {
            peer, label, next, ..
        } => (Direction::Send, peer, label, next),
        crate::endpoint::LocalProtocol::Receive {
            peer, label, next, ..
        } => (Direction::Receive, peer, label, next),
        crate::endpoint::LocalProtocol::SendOwned {
            peer, label, next, ..
        } => (Direction::SendOwned, peer, label, next),
        crate::endpoint::LocalProtocol::ReceiveOwned {
            peer, label, next, ..
        } => (Direction::ReceiveOwned, peer, label, next),
        _other => return None,
    };
    debug_assert!(!core::ptr::eq(local, &**next));
    debug_assert!(local_branches(local, *peer, *peer).is_none());
    Some(LocalMessage {
        direction,
        peer: *peer,
        label: *label,
        next,
    })
}

fn global_message(node: &crate::model::tree::Node) -> Option<GlobalMessage<'_>> {
    let (from, to, label, is_owned, next) = match node {
        crate::model::tree::Node::Communication {
            from,
            to,
            label,
            next,
            ..
        } => (from, to, label, false, next),
        crate::model::tree::Node::Transfer {
            from,
            to,
            label,
            next,
            ..
        } => (from, to, label, true, next),
        _other => return None,
    };
    debug_assert!(!core::ptr::eq(node, &**next));
    debug_assert!(node.span().is_some());
    Some(GlobalMessage {
        from: *from,
        to: *to,
        label: *label,
        is_owned,
        next,
    })
}

/// Local direction and peer expected for `role`, or `None` when uninvolved.
fn expected_local(
    message: &GlobalMessage<'_>,
    role: crate::model::participants::RoleId,
) -> Option<(Direction, crate::model::participants::RoleId)> {
    let (send, receive) = if message.is_owned {
        (Direction::SendOwned, Direction::ReceiveOwned)
    } else {
        (Direction::Send, Direction::Receive)
    };
    if role == message.from {
        Some((send, message.to))
    } else if role == message.to {
        Some((receive, message.from))
    } else {
        None
    }
}

fn advance_message<'a>(
    message: &GlobalMessage<'a>,
    local: &'a crate::endpoint::LocalProtocol,
    role: crate::model::participants::RoleId,
) -> Advance<'a> {
    let Some((direction, peer)) = expected_local(message, role) else {
        let cursor = Cursor {
            node: message.next,
            local,
        };
        return Ok((cursor, None));
    };
    let observed =
        local_message(local).ok_or(crate::issue::DiagnosticCode::PreferenceLocalMismatch)?;
    let is_match = observed.direction == direction && observed.peer == peer;
    if !(is_match && observed.label == message.label) {
        return Err(crate::issue::DiagnosticCode::PreferenceLocalMismatch);
    }
    debug_assert_eq!(observed.peer, peer);
    debug_assert_eq!(observed.label, message.label);
    let cursor = Cursor {
        node: message.next,
        local: observed.next,
    };
    Ok((cursor, Some(super::model::PathStep::Next)))
}

/// Local branches of a choice: the decider's `InternalChoice`, or a peer's
/// `Offer` from that decider.
fn local_branches(
    local: &crate::endpoint::LocalProtocol,
    decider: crate::model::participants::RoleId,
    role: crate::model::participants::RoleId,
) -> Option<&[crate::endpoint::LocalProtocolBranch]> {
    match local {
        crate::endpoint::LocalProtocol::InternalChoice { branches } if decider == role => {
            Some(branches)
        }
        crate::endpoint::LocalProtocol::Offer { from, branches }
            if *from == decider && decider != role =>
        {
            Some(branches)
        }
        _other => None,
    }
}

fn same_labels(
    global: &[crate::model::tree::ChoiceBranch],
    local: &[crate::endpoint::LocalProtocolBranch],
) -> bool {
    global
        .iter()
        .map(|branch| branch.label)
        .eq(local.iter().map(|branch| branch.label))
}

fn advance_choice<'a>(
    cursor: &Cursor<'a>,
    step: super::model::PathStep,
    role: crate::model::participants::RoleId,
) -> Advance<'a> {
    let crate::model::tree::Node::Choice {
        decider, branches, ..
    } = cursor.node
    else {
        return Err(crate::issue::DiagnosticCode::InvalidPreferencePath);
    };
    let local = local_branches(cursor.local, *decider, role).ok_or(if *decider == role {
        crate::issue::DiagnosticCode::PreferenceLocalMismatch
    } else {
        crate::issue::DiagnosticCode::PreferencePathNotLocallyAddressable
    })?;
    if !same_labels(branches, local) {
        return Err(crate::issue::DiagnosticCode::PreferenceLocalMismatch);
    }
    let node = super::admission::next_node(cursor.node, step)
        .ok_or(crate::issue::DiagnosticCode::InvalidPreferencePath)?;
    let super::model::PathStep::Branch { label } = step else {
        return Err(crate::issue::DiagnosticCode::InvalidPreferencePath);
    };
    let body = local
        .iter()
        .find(|branch| branch.label == label)
        .map(|branch| &branch.body)
        .ok_or(crate::issue::DiagnosticCode::PreferenceLocalMismatch)?;
    debug_assert_eq!(local.len(), branches.len());
    debug_assert!(branches.iter().any(|branch| branch.label == label));
    Ok((Cursor { node, local: body }, Some(step)))
}

fn advance<'a>(
    cursor: &Cursor<'a>,
    step: super::model::PathStep,
    role: crate::model::participants::RoleId,
) -> Advance<'a> {
    match (global_message(cursor.node), step) {
        (Some(message), super::model::PathStep::Next) => {
            advance_message(&message, cursor.local, role)
        }
        (None, super::model::PathStep::Branch { .. }) => advance_choice(cursor, step, role),
        _other => Err(crate::issue::DiagnosticCode::InvalidPreferencePath),
    }
}

/// Follows the global annotation path in the decider's local protocol.
///
/// Returns the local path and the exact labels of the decider's
/// `InternalChoice` at the annotated choice.
fn local_address(
    protocol: &crate::model::tree::Node,
    local: &crate::endpoint::LocalProtocol,
    choice: &super::model::ChoiceAnnotation,
) -> Result<
    (
        Vec<super::model::PathStep>,
        Vec<crate::model::participants::LabelId>,
    ),
    Vec<crate::issue::Diagnostic>,
> {
    let refuse = |code: crate::issue::DiagnosticCode| {
        issue(
            code,
            choice.span,
            "annotated choice has no exact decider local choice address",
        )
    };
    let mut cursor = Cursor {
        node: protocol,
        local,
    };
    let mut local_path = Vec::with_capacity(choice.path.len());
    for step in &choice.path {
        let (next, local_step) = advance(&cursor, *step, choice.decider).map_err(refuse)?;
        local_path.extend(local_step);
        cursor = next;
    }
    let crate::model::tree::Node::Choice { branches, .. } = cursor.node else {
        return Err(refuse(
            crate::issue::DiagnosticCode::PreferenceTargetNotChoice,
        ));
    };
    let local_choice = local_branches(cursor.local, choice.decider, choice.decider)
        .ok_or_else(|| refuse(crate::issue::DiagnosticCode::PreferenceLocalMismatch))?;
    if !same_labels(branches, local_choice) {
        return Err(refuse(
            crate::issue::DiagnosticCode::PreferenceLocalMismatch,
        ));
    }
    debug_assert!(local_path.len() <= choice.path.len());
    debug_assert_eq!(local_choice.len(), branches.len());
    Ok((
        local_path,
        branches.iter().map(|branch| branch.label).collect(),
    ))
}

fn declared_factors(
    admitted: &super::admission::Admitted,
    choice: &super::model::ChoiceAnnotation,
) -> Result<Vec<super::model::FactorDeclaration>, Vec<crate::issue::Diagnostic>> {
    choice
        .factors
        .iter()
        .map(|id| {
            admitted
                .annotations
                .factors
                .iter()
                .find(|factor| factor.id == *id)
                .cloned()
                .ok_or_else(|| {
                    issue(
                        crate::issue::DiagnosticCode::UnknownPreferenceFactor,
                        choice.span,
                        "choice annotation names an undeclared factor",
                    )
                })
        })
        .collect()
}

fn declared_world(
    admitted: &super::admission::Admitted,
    choice: &super::model::ChoiceAnnotation,
) -> Result<Option<super::model::WorldDeclaration>, Vec<crate::issue::Diagnostic>> {
    choice
        .world
        .map(|id| {
            admitted
                .annotations
                .worlds
                .iter()
                .find(|world| world.id == id)
                .cloned()
                .ok_or_else(|| {
                    issue(
                        crate::issue::DiagnosticCode::UnknownWorldVariable,
                        choice.span,
                        "choice annotation names an undeclared world variable",
                    )
                })
        })
        .transpose()
}

fn decision(
    admitted: &super::admission::Admitted,
    choice: &super::model::ChoiceAnnotation,
    local: &crate::endpoint::LocalProtocol,
) -> Result<DeciderAnnotation, Vec<crate::issue::Diagnostic>> {
    let (local_path, labels) = local_address(&admitted.base.source.protocol, local, choice)?;
    Ok(DeciderAnnotation {
        global_path: choice.path.clone(),
        local_path,
        labels,
        factors: declared_factors(admitted, choice)?,
        world: declared_world(admitted, choice)?,
        span: choice.span,
    })
}

fn annotated_local(
    admitted: &super::admission::Admitted,
    local: &crate::endpoint::LocalProtocolArtifactBody,
    artifact: &crate::linkage::output::CanonicalLocalArtifact,
) -> Result<AnnotatedLocal, Vec<crate::issue::Diagnostic>> {
    if local.role != artifact.role {
        return Err(issue(
            crate::issue::DiagnosticCode::LocalOrderMismatch,
            origin(),
            "local artifacts are not in normalized role order",
        ));
    }
    let decisions = admitted
        .annotations
        .choices
        .iter()
        .filter(|choice| choice.decider == local.role)
        .map(|choice| decision(admitted, choice, &local.protocol))
        .collect::<Result<Vec<_>, _>>()?;
    debug_assert!(decisions.len() <= admitted.annotations.choices.len());
    debug_assert_eq!(local.role, artifact.role);
    Ok(AnnotatedLocal {
        role: local.role,
        local_artifact: artifact.artifact.identity.clone(),
        decisions,
    })
}

/// Builds the annotated complete local set over one v1 projection cohort.
pub fn annotate(
    admitted: &super::admission::Admitted,
    projected: &crate::endpoint::Projected,
    base: &crate::linkage::ProjectionBundle,
) -> Result<AnnotatedLocalSet, Vec<crate::issue::Diagnostic>> {
    crate::endpoint::validate_complete_set(&admitted.base.roles, &projected.locals)?;
    if projected.locals.locals.len() != base.locals().len() {
        return Err(issue(
            crate::issue::DiagnosticCode::IncompleteLocalSet,
            origin(),
            "local artifact count differs from declared role count",
        ));
    }
    let locals = projected
        .locals
        .locals
        .iter()
        .zip(base.locals())
        .map(|(local, artifact)| annotated_local(admitted, local, artifact))
        .collect::<Result<Vec<_>, _>>()?;
    let set = AnnotatedLocalSet {
        complete_local_set: base.local_set().identity.clone(),
        locals,
    };
    validate_set(base, &set)?;
    debug_assert_eq!(set.locals.len(), admitted.base.roles.len());
    debug_assert_eq!(set.complete_local_set, base.local_set().identity);
    Ok(set)
}

fn validate_entry_ownership(entry: &AnnotatedLocal) -> Result<(), Vec<crate::issue::Diagnostic>> {
    let is_owned = entry
        .decisions
        .iter()
        .flat_map(|item| &item.factors)
        .all(|factor| factor.role == entry.role);
    if !is_owned {
        return Err(issue(
            crate::issue::DiagnosticCode::PreferenceFactorRoleMismatch,
            origin(),
            "annotated local carries another role's factor",
        ));
    }
    debug_assert!(
        entry
            .decisions
            .iter()
            .flat_map(|item| &item.factors)
            .all(|factor| factor.role == entry.role)
    );
    let is_ordered = entry
        .decisions
        .is_sorted_by(|left, right| left.global_path < right.global_path);
    if !is_ordered {
        return Err(issue(
            crate::issue::DiagnosticCode::UnsortedChoiceAnnotation,
            origin(),
            "annotated local decisions are duplicated or not in path order",
        ));
    }
    debug_assert!(
        entry
            .decisions
            .is_sorted_by(|left, right| left.global_path < right.global_path)
    );
    Ok(())
}

fn validate_entry(
    entry: &AnnotatedLocal,
    expected: &crate::linkage::output::CanonicalLocalArtifact,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    if entry.role != expected.role {
        return Err(issue(
            crate::issue::DiagnosticCode::LocalOrderMismatch,
            origin(),
            "annotated locals are not in normalized role order",
        ));
    }
    if entry.local_artifact != expected.artifact.identity {
        return Err(issue(
            crate::issue::DiagnosticCode::IdentityMismatch,
            origin(),
            "annotated local names a different local artifact",
        ));
    }
    validate_entry_ownership(entry)
}

/// Checks that an annotated local set is complete, ordered, and bound to the
/// exact v1 local artifacts and complete local set of `base`.
pub fn validate_set(
    base: &crate::linkage::ProjectionBundle,
    set: &AnnotatedLocalSet,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    if set.complete_local_set != base.local_set().identity {
        return Err(issue(
            crate::issue::DiagnosticCode::IdentityMismatch,
            origin(),
            "annotated set names a different complete local set",
        ));
    }
    if set.locals.len() != base.locals().len() {
        return Err(issue(
            crate::issue::DiagnosticCode::IncompleteLocalSet,
            origin(),
            "annotated local count differs from declared role count",
        ));
    }
    let is_duplicated = set
        .locals
        .iter()
        .zip(set.locals.iter().skip(1))
        .any(|(previous, current)| previous.role == current.role);
    if is_duplicated {
        return Err(issue(
            crate::issue::DiagnosticCode::DuplicateLocalRole,
            origin(),
            "annotated local role is duplicated",
        ));
    }
    for (entry, expected) in set.locals.iter().zip(base.locals()) {
        validate_entry(entry, expected)?;
    }
    debug_assert_eq!(set.locals.len(), base.locals().len());
    debug_assert!(set.complete_local_set == base.local_set().identity);
    Ok(())
}
