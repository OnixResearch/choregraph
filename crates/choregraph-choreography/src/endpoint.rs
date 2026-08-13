#![allow(
    unknown_lints,
    missing_const_fn,
    non_trait_imports,
    no_recursion,
    reason = "the bounded conversion preserves the selected recursive Trellis protocol shape and keeps exact local and Trellis owner types visible at the endpoint boundary"
)]

use crate::admission::AdmittedChoreography;
use crate::issue::{ChoreographyDiagnostic, DiagnosticCode};
use crate::model::{
    LabelId, PayloadTag, RoleDeclaration, RoleId, RoleKind, SourceSpan, ValueDeclaration, ValueId,
};
use serde::{Deserialize, Serialize};
use verified_logic::choreography_global::{GlobalBranch, GlobalChoreo};
use verified_logic::choreography_local::{LocalBranch, LocalChoreo};
use verified_logic::choreography_projection::project_endpoint;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GlobalProtocolBranch {
    pub label: LabelId,
    pub body: GlobalProtocol,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum GlobalProtocol {
    End,
    Communication {
        from: RoleId,
        to: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        next: Box<GlobalProtocol>,
    },
    Transfer {
        from: RoleId,
        to: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        value: ValueId,
        next: Box<GlobalProtocol>,
    },
    Choice {
        decider: RoleId,
        branches: Vec<GlobalProtocolBranch>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalProtocolBranch {
    pub label: LabelId,
    pub body: LocalProtocol,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum LocalProtocol {
    End,
    Send {
        peer: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        next: Box<LocalProtocol>,
    },
    Receive {
        peer: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        next: Box<LocalProtocol>,
    },
    SendOwned {
        peer: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        value: ValueId,
        next: Box<LocalProtocol>,
    },
    ReceiveOwned {
        peer: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        value: ValueId,
        next: Box<LocalProtocol>,
    },
    InternalChoice {
        branches: Vec<LocalProtocolBranch>,
    },
    Offer {
        from: RoleId,
        branches: Vec<LocalProtocolBranch>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GlobalProtocolArtifactBody {
    pub roles: Vec<RoleDeclaration>,
    pub values: Vec<ValueDeclaration>,
    pub protocol: GlobalProtocol,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalProtocolArtifactBody {
    pub role: RoleId,
    pub role_kind: RoleKind,
    pub protocol: LocalProtocol,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalProtocolSetBody {
    pub locals: Vec<LocalProtocolArtifactBody>,
}

#[derive(Debug)]
pub struct ProjectedChoreography {
    pub global: GlobalProtocolArtifactBody,
    pub locals: LocalProtocolSetBody,
}

fn issue(code: DiagnosticCode, note: &'static str) -> Vec<ChoreographyDiagnostic> {
    vec![ChoreographyDiagnostic::error(
        code,
        SourceSpan { start: 0, end: 0 },
        note,
    )]
}

fn global_branch(branch: &GlobalBranch) -> GlobalProtocolBranch {
    GlobalProtocolBranch {
        label: LabelId(branch.label),
        body: global_protocol(&branch.body),
    }
}

fn global_protocol(global: &GlobalChoreo) -> GlobalProtocol {
    match global {
        GlobalChoreo::End => GlobalProtocol::End,
        GlobalChoreo::Comm {
            from,
            to,
            label,
            payload_tag,
            next,
        } => GlobalProtocol::Communication {
            from: RoleId(*from),
            to: RoleId(*to),
            label: LabelId(*label),
            payload_tag: PayloadTag(payload_tag.raw),
            next: Box::new(global_protocol(next)),
        },
        GlobalChoreo::OwnedTransfer {
            from,
            to,
            label,
            payload_tag,
            value,
            next,
        } => GlobalProtocol::Transfer {
            from: RoleId(*from),
            to: RoleId(*to),
            label: LabelId(*label),
            payload_tag: PayloadTag(payload_tag.raw),
            value: ValueId(value.raw),
            next: Box::new(global_protocol(next)),
        },
        GlobalChoreo::Choice { decider, branches } => GlobalProtocol::Choice {
            decider: RoleId(*decider),
            branches: branches.iter().map(global_branch).collect(),
        },
    }
}

fn local_branch(branch: &LocalBranch) -> LocalProtocolBranch {
    LocalProtocolBranch {
        label: LabelId(branch.label),
        body: local_protocol(&branch.body),
    }
}

fn local_protocol(local: &LocalChoreo) -> LocalProtocol {
    match local {
        LocalChoreo::End => LocalProtocol::End,
        LocalChoreo::Send {
            peer,
            label,
            payload_tag,
            next,
        } => LocalProtocol::Send {
            peer: RoleId(*peer),
            label: LabelId(*label),
            payload_tag: PayloadTag(payload_tag.raw),
            next: Box::new(local_protocol(next)),
        },
        LocalChoreo::Recv {
            peer,
            label,
            payload_tag,
            next,
        } => LocalProtocol::Receive {
            peer: RoleId(*peer),
            label: LabelId(*label),
            payload_tag: PayloadTag(payload_tag.raw),
            next: Box::new(local_protocol(next)),
        },
        LocalChoreo::SendOwned {
            peer,
            label,
            payload_tag,
            value,
            next,
        } => LocalProtocol::SendOwned {
            peer: RoleId(*peer),
            label: LabelId(*label),
            payload_tag: PayloadTag(payload_tag.raw),
            value: ValueId(value.raw),
            next: Box::new(local_protocol(next)),
        },
        LocalChoreo::RecvOwned {
            peer,
            label,
            payload_tag,
            value,
            next,
        } => LocalProtocol::ReceiveOwned {
            peer: RoleId(*peer),
            label: LabelId(*label),
            payload_tag: PayloadTag(payload_tag.raw),
            value: ValueId(value.raw),
            next: Box::new(local_protocol(next)),
        },
        LocalChoreo::InternalChoice { branches } => LocalProtocol::InternalChoice {
            branches: branches.iter().map(local_branch).collect(),
        },
        LocalChoreo::Offer { from, branches } => LocalProtocol::Offer {
            from: RoleId(*from),
            branches: branches.iter().map(local_branch).collect(),
        },
    }
}

fn exact_role_kind(source: &AdmittedChoreography, role: RoleId) -> Option<RoleKind> {
    source
        .source
        .roles
        .iter()
        .find(|declaration| declaration.id == role)
        .map(|declaration| declaration.kind.clone())
}

pub fn project_all(
    admitted: &AdmittedChoreography,
) -> Result<ProjectedChoreography, Vec<ChoreographyDiagnostic>> {
    let mut locals = Vec::with_capacity(admitted.roles.len());
    for role in &admitted.roles {
        let Some(role_kind) = exact_role_kind(admitted, *role) else {
            return Err(issue(
                DiagnosticCode::IncompleteLocalSet,
                "normalized role has no declaration",
            ));
        };
        let trellis_local = project_endpoint(role.0, &admitted.global);
        locals.push(LocalProtocolArtifactBody {
            role: *role,
            role_kind,
            protocol: local_protocol(&trellis_local),
        });
    }
    let local_set = LocalProtocolSetBody { locals };
    validate_complete_set(&admitted.roles, &local_set)?;
    debug_assert_eq!(local_set.locals.len(), admitted.roles.len());
    debug_assert!(
        local_set
            .locals
            .iter()
            .zip(&admitted.roles)
            .all(|(local, role)| local.role == *role)
    );
    Ok(ProjectedChoreography {
        global: GlobalProtocolArtifactBody {
            roles: admitted.source.roles.clone(),
            values: admitted.source.values.clone(),
            protocol: global_protocol(&admitted.global),
        },
        locals: local_set,
    })
}

pub fn validate_complete_set(
    expected_roles: &[RoleId],
    local_set: &LocalProtocolSetBody,
) -> Result<(), Vec<ChoreographyDiagnostic>> {
    if local_set.locals.len() != expected_roles.len() {
        return Err(issue(
            DiagnosticCode::IncompleteLocalSet,
            "local artifact count differs from declared role count",
        ));
    }
    for (index, local) in local_set.locals.iter().enumerate() {
        if index > 0 && local_set.locals[index.saturating_sub(1)].role == local.role {
            return Err(issue(
                DiagnosticCode::DuplicateLocalRole,
                "local artifact role is duplicated",
            ));
        }
        if expected_roles.get(index) != Some(&local.role) {
            return Err(issue(
                DiagnosticCode::LocalOrderMismatch,
                "local artifacts are not in normalized role order",
            ));
        }
    }
    debug_assert_eq!(local_set.locals.len(), expected_roles.len());
    debug_assert!(
        local_set
            .locals
            .iter()
            .zip(expected_roles)
            .all(|(local, role)| local.role == *role)
    );
    Ok(())
}
