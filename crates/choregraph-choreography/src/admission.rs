#![allow(
    unknown_lints,
    missing_const_fn,
    non_trait_imports,
    no_recursion,
    reason = "bounded protocol-tree admission and exact Trellis lowering keep selected domain types and explicit recursive structure together; stable const evaluation cannot express the iterator and string checks"
)]

use crate::issue::{ChoreographyDiagnostic, DiagnosticCode, diagnostics_are_safe, span_is_valid};
use crate::model::{
    ChoreographyLimits, ChoreographyNode, ImportIdentity, MAX_BRANCHES, MAX_IDENTIFIER_BYTES,
    MAX_IMPORTS, MAX_NESTING_DEPTH, MAX_ROLES, MAX_STEPS, MAX_VALUES, REQUIRED_NON_CLAIMS,
    ROLE_CHOREOGRAPHY_SCHEMA, RoleChoreographySource, RoleDeclaration, RoleId, RoleKind,
    SourceSpan, ValueDeclaration, ValueId,
};
use verified_logic::choreography_global::{GlobalBranch, GlobalChoreo, PayloadTag};
use verified_logic::choreography_ownership::{
    OwnershipEntry, admit_ownership, owner_of, transfer_owner,
};
use verified_logic::choreography_projection::projectable;

const BLAKE3_HEX_BYTES: usize = 64;
const BLAKE3_PREFIX: &str = "blake3:";

#[derive(Debug)]
pub struct AdmittedChoreography {
    pub(crate) source: RoleChoreographySource,
    pub(crate) global: GlobalChoreo,
    pub(crate) roles: Vec<RoleId>,
}

#[derive(Clone)]
struct OwnershipSnapshot {
    entries: Vec<OwnershipEntry>,
}

#[derive(Default)]
struct TraversalCount {
    steps: usize,
    branches: usize,
}

struct NodeValidationContext<'a, 'b> {
    roles: &'a [RoleDeclaration],
    values: &'a [ValueDeclaration],
    count: &'b mut TraversalCount,
}

struct ChoiceValidation<'a> {
    decider: RoleId,
    branches: &'a [crate::model::ChoiceBranch],
    span: SourceSpan,
}

fn one(diagnostic: ChoreographyDiagnostic) -> Vec<ChoreographyDiagnostic> {
    vec![diagnostic]
}

fn issue(
    code: DiagnosticCode,
    span: SourceSpan,
    note: &'static str,
) -> Vec<ChoreographyDiagnostic> {
    one(ChoreographyDiagnostic::error(code, span, note))
}

fn projection_issue(node: &ChoreographyNode) -> Vec<ChoreographyDiagnostic> {
    let mut diagnostic = ChoreographyDiagnostic::error(
        DiagnosticCode::TrellisProjectionRejected,
        node.span(),
        "Trellis rejected endpoint projectability",
    );
    if let ChoreographyNode::Choice { branches, .. } = node {
        diagnostic.secondary = branches.iter().map(|branch| branch.span).collect();
        diagnostic.secondary.sort_unstable();
        diagnostic.secondary.dedup();
    }
    one(diagnostic)
}

fn duplicate_issue(
    code: DiagnosticCode,
    primary: SourceSpan,
    secondary: SourceSpan,
    note: &'static str,
) -> Vec<ChoreographyDiagnostic> {
    one(ChoreographyDiagnostic::error(code, primary, note).with_secondary(secondary))
}

fn lowercase_hex(text: &str) -> bool {
    text.bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn exact_blake3(text: &str) -> bool {
    text.strip_prefix(BLAKE3_PREFIX)
        .is_some_and(|hex| hex.len() == BLAKE3_HEX_BYTES && lowercase_hex(hex))
}

fn raw_blake3(text: &str) -> bool {
    text.len() == BLAKE3_HEX_BYTES && lowercase_hex(text)
}

fn bounded_identifier(text: &str) -> bool {
    !text.is_empty() && text.len() <= MAX_IDENTIFIER_BYTES
}

fn role_declared(roles: &[RoleDeclaration], role: RoleId) -> bool {
    roles.iter().any(|declaration| declaration.id == role)
}

fn validate_distinct_participants(
    roles: &[RoleDeclaration],
    from: RoleId,
    to: RoleId,
    span: SourceSpan,
) -> Result<(), Vec<ChoreographyDiagnostic>> {
    if !role_declared(roles, from) || !role_declared(roles, to) {
        return Err(issue(
            DiagnosticCode::UndeclaredRole,
            span,
            "protocol action names an undeclared role",
        ));
    }
    if from == to {
        return Err(issue(
            DiagnosticCode::SelfTransfer,
            span,
            "protocol action source and destination are equal",
        ));
    }
    debug_assert!(role_declared(roles, from));
    debug_assert!(role_declared(roles, to));
    Ok(())
}

fn value_declared(values: &[ValueDeclaration], value: ValueId) -> Option<ValueDeclaration> {
    values
        .iter()
        .copied()
        .find(|declaration| declaration.id == value)
}

fn validate_span(span: SourceSpan) -> Result<(), Vec<ChoreographyDiagnostic>> {
    if span_is_valid(span) {
        Ok(())
    } else {
        Err(issue(
            DiagnosticCode::InvalidSpan,
            span,
            "source span end precedes its start",
        ))
    }
}

fn validate_role_kind(role: &RoleDeclaration) -> Result<(), Vec<ChoreographyDiagnostic>> {
    match &role.kind {
        RoleKind::Native => Ok(()),
        RoleKind::External {
            contract_identity,
            input_schema_identity,
            output_schema_identity,
        } => {
            let is_valid = bounded_identifier(contract_identity)
                && bounded_identifier(input_schema_identity)
                && bounded_identifier(output_schema_identity)
                && exact_blake3(contract_identity)
                && exact_blake3(input_schema_identity)
                && exact_blake3(output_schema_identity);
            if is_valid {
                Ok(())
            } else {
                Err(issue(
                    DiagnosticCode::InvalidRoleKind,
                    role.span,
                    "external role identities must be exact domain values",
                ))
            }
        }
    }
}

fn validate_roles(
    roles: &[RoleDeclaration],
    root: SourceSpan,
) -> Result<(), Vec<ChoreographyDiagnostic>> {
    if roles.is_empty() {
        return Err(issue(
            DiagnosticCode::EmptyRoles,
            root,
            "role choreography declares no roles",
        ));
    }
    if roles.len() > MAX_ROLES {
        return Err(issue(
            DiagnosticCode::RoleLimitExceeded,
            root,
            "role count exceeds the reviewed limit",
        ));
    }
    for (index, role) in roles.iter().enumerate() {
        validate_span(role.span)?;
        validate_role_kind(role)?;
        if let Some(previous) = roles[..index]
            .iter()
            .find(|candidate| candidate.id == role.id)
        {
            return Err(duplicate_issue(
                DiagnosticCode::DuplicateRole,
                role.span,
                previous.span,
                "role identity is duplicated",
            ));
        }
    }
    debug_assert!(!roles.is_empty());
    debug_assert!(roles.len() <= MAX_ROLES);
    Ok(())
}

fn validate_values(
    values: &[ValueDeclaration],
    roles: &[RoleDeclaration],
    root: SourceSpan,
) -> Result<(), Vec<ChoreographyDiagnostic>> {
    if values.len() > MAX_VALUES {
        return Err(issue(
            DiagnosticCode::ValueLimitExceeded,
            root,
            "value count exceeds the reviewed limit",
        ));
    }
    for (index, value) in values.iter().enumerate() {
        validate_span(value.span)?;
        if !role_declared(roles, value.initial_owner) {
            return Err(issue(
                DiagnosticCode::UndeclaredRole,
                value.span,
                "initial value owner is undeclared",
            ));
        }
        if let Some(previous) = values[..index]
            .iter()
            .find(|candidate| candidate.id == value.id)
        {
            return Err(duplicate_issue(
                DiagnosticCode::DuplicateValue,
                value.span,
                previous.span,
                "value identity is duplicated",
            ));
        }
    }
    debug_assert!(values.len() <= MAX_VALUES);
    debug_assert!(
        values
            .iter()
            .all(|value| role_declared(roles, value.initial_owner))
    );
    Ok(())
}

fn validate_imports(
    imports: &[ImportIdentity],
    root: SourceSpan,
) -> Result<(), Vec<ChoreographyDiagnostic>> {
    if imports.len() > MAX_IMPORTS {
        return Err(issue(
            DiagnosticCode::ImportLimitExceeded,
            root,
            "import count exceeds the reviewed limit",
        ));
    }
    for (index, import) in imports.iter().enumerate() {
        if !bounded_identifier(&import.logical_id) || !raw_blake3(&import.blake3) {
            return Err(issue(
                DiagnosticCode::InvalidImportIdentity,
                root,
                "import identity is empty, oversized, or not BLAKE3",
            ));
        }
        if index > 0 {
            let previous = &imports[index.saturating_sub(1)];
            if previous.logical_id == import.logical_id {
                return Err(issue(
                    DiagnosticCode::DuplicateImport,
                    root,
                    "import logical identity is duplicated",
                ));
            }
            if previous.logical_id > import.logical_id {
                return Err(issue(
                    DiagnosticCode::UnsortedImport,
                    root,
                    "imports are not in logical identity order",
                ));
            }
        }
    }
    debug_assert!(imports.len() <= MAX_IMPORTS);
    debug_assert!(imports.iter().all(|import| raw_blake3(&import.blake3)));
    Ok(())
}

fn validate_non_claims(source: &RoleChoreographySource) -> Result<(), Vec<ChoreographyDiagnostic>> {
    let span = source.protocol.span();
    for required in REQUIRED_NON_CLAIMS {
        if !source.non_claims.iter().any(|item| item == required) {
            return Err(issue(
                DiagnosticCode::MissingNonClaim,
                span,
                "one required compiler non-claim is missing",
            ));
        }
    }
    if source.non_claims.len() != REQUIRED_NON_CLAIMS.len() {
        return Err(issue(
            DiagnosticCode::MissingNonClaim,
            span,
            "compiler non-claims must match the reviewed closed set",
        ));
    }
    debug_assert_eq!(source.non_claims.len(), REQUIRED_NON_CLAIMS.len());
    debug_assert!(
        REQUIRED_NON_CLAIMS
            .iter()
            .all(|required| source.non_claims.iter().any(|item| item == required))
    );
    Ok(())
}

fn initial_ownership(values: &[ValueDeclaration]) -> OwnershipSnapshot {
    let mut ordered = values.to_vec();
    ordered.sort_unstable_by_key(|value| value.id);
    OwnershipSnapshot {
        entries: ordered
            .iter()
            .map(|value| OwnershipEntry {
                value: verified_logic::choreography_ownership::ValueId { raw: value.id.0 },
                owner: value.initial_owner.0,
            })
            .collect(),
    }
}

fn trellis_state(
    snapshot: &OwnershipSnapshot,
    span: SourceSpan,
) -> Result<verified_logic::choreography_ownership::OwnershipState, Vec<ChoreographyDiagnostic>> {
    admit_ownership(&snapshot.entries).ok_or_else(|| {
        issue(
            DiagnosticCode::TrellisOwnershipRejected,
            span,
            "Trellis rejected normalized ownership facts",
        )
    })
}

struct TransferValidation<'a> {
    from: RoleId,
    to: RoleId,
    payload_tag: crate::model::PayloadTag,
    value: ValueId,
    span: SourceSpan,
    roles: &'a [RoleDeclaration],
    values: &'a [ValueDeclaration],
    snapshot: &'a OwnershipSnapshot,
}

fn validate_transfer(
    transfer: TransferValidation<'_>,
) -> Result<OwnershipSnapshot, Vec<ChoreographyDiagnostic>> {
    let TransferValidation {
        from,
        to,
        payload_tag,
        value,
        span,
        roles,
        values,
        snapshot,
    } = transfer;
    validate_distinct_participants(roles, from, to, span)?;
    let Some(declaration) = value_declared(values, value) else {
        return Err(issue(
            DiagnosticCode::UnknownValue,
            span,
            "transfer names an undeclared value",
        ));
    };
    if declaration.payload_tag != payload_tag {
        return Err(issue(
            DiagnosticCode::PayloadMismatch,
            span,
            "transfer payload tag differs from the declared value tag",
        ));
    }
    let state = trellis_state(snapshot, span)?;
    let trellis_value = verified_logic::choreography_ownership::ValueId { raw: value.0 };
    if owner_of(&state, trellis_value) != Some(from.0) {
        return Err(issue(
            DiagnosticCode::StaleOwner,
            span,
            "transfer source does not own the selected value",
        ));
    }
    if transfer_owner(&state, from.0, to.0, trellis_value).is_none() {
        return Err(issue(
            DiagnosticCode::TrellisOwnershipRejected,
            span,
            "Trellis rejected the ownership transfer",
        ));
    }
    let mut next = snapshot.clone();
    if let Some(entry) = next
        .entries
        .iter_mut()
        .find(|entry| entry.value == trellis_value)
    {
        entry.owner = to.0;
    }
    debug_assert_eq!(next.entries.len(), snapshot.entries.len());
    debug_assert!(
        next.entries
            .iter()
            .any(|entry| entry.value == trellis_value && entry.owner == to.0)
    );
    Ok(next)
}

fn validate_choice(
    choice: ChoiceValidation<'_>,
    snapshot: &OwnershipSnapshot,
    depth: usize,
    context: &mut NodeValidationContext<'_, '_>,
) -> Result<(), Vec<ChoreographyDiagnostic>> {
    if !role_declared(context.roles, choice.decider) {
        return Err(issue(
            DiagnosticCode::UndeclaredRole,
            choice.span,
            "choice names an undeclared decider",
        ));
    }
    if choice.branches.is_empty() {
        return Err(issue(
            DiagnosticCode::EmptyChoice,
            choice.span,
            "choice has no branches",
        ));
    }
    context.count.branches = context.count.branches.saturating_add(choice.branches.len());
    if choice.branches.len() > MAX_BRANCHES || context.count.branches > MAX_BRANCHES {
        return Err(issue(
            DiagnosticCode::BranchLimitExceeded,
            choice.span,
            "choice branch count exceeds the reviewed limit",
        ));
    }
    for (index, branch) in choice.branches.iter().enumerate() {
        validate_span(branch.span)?;
        if index > 0 {
            let previous = &choice.branches[index.saturating_sub(1)];
            if previous.label == branch.label {
                return Err(duplicate_issue(
                    DiagnosticCode::DuplicateBranchLabel,
                    branch.span,
                    previous.span,
                    "choice branch label is duplicated",
                ));
            }
            if previous.label > branch.label {
                return Err(duplicate_issue(
                    DiagnosticCode::UnsortedBranchLabel,
                    branch.span,
                    previous.span,
                    "choice branch labels are not in strict order",
                ));
            }
        }
        validate_node(&branch.body, snapshot, depth.saturating_add(1), context)?;
    }
    debug_assert!(context.count.branches <= MAX_BRANCHES);
    debug_assert!(choice.branches.is_sorted_by_key(|branch| branch.label));
    Ok(())
}

fn validate_node(
    node: &ChoreographyNode,
    snapshot: &OwnershipSnapshot,
    depth: usize,
    context: &mut NodeValidationContext<'_, '_>,
) -> Result<(), Vec<ChoreographyDiagnostic>> {
    let span = node.span();
    validate_span(span)?;
    if depth > MAX_NESTING_DEPTH {
        return Err(issue(
            DiagnosticCode::NestingLimitExceeded,
            span,
            "protocol nesting exceeds the reviewed limit",
        ));
    }
    context.count.steps = context.count.steps.saturating_add(1);
    if context.count.steps > MAX_STEPS {
        return Err(issue(
            DiagnosticCode::StepLimitExceeded,
            span,
            "protocol step count exceeds the reviewed limit",
        ));
    }
    debug_assert!(span_is_valid(span));
    debug_assert!(depth <= MAX_NESTING_DEPTH);
    match node {
        ChoreographyNode::End { .. } => Ok(()),
        ChoreographyNode::Communication { from, to, next, .. } => {
            validate_distinct_participants(context.roles, *from, *to, span)?;
            validate_node(next, snapshot, depth.saturating_add(1), context)
        }
        ChoreographyNode::Transfer {
            from,
            to,
            payload_tag,
            value,
            next,
            ..
        } => {
            let next_snapshot = validate_transfer(TransferValidation {
                from: *from,
                to: *to,
                payload_tag: *payload_tag,
                value: *value,
                span,
                roles: context.roles,
                values: context.values,
                snapshot,
            })?;
            validate_node(next, &next_snapshot, depth.saturating_add(1), context)
        }
        ChoreographyNode::Choice {
            decider, branches, ..
        } => validate_choice(
            ChoiceValidation {
                decider: *decider,
                branches,
                span,
            },
            snapshot,
            depth,
            context,
        ),
    }
}

fn lower_node(node: &ChoreographyNode) -> GlobalChoreo {
    match node {
        ChoreographyNode::End { .. } => GlobalChoreo::End,
        ChoreographyNode::Communication {
            from,
            to,
            label,
            payload_tag,
            next,
            ..
        } => GlobalChoreo::Comm {
            from: from.0,
            to: to.0,
            label: label.0,
            payload_tag: PayloadTag { raw: payload_tag.0 },
            next: Box::new(lower_node(next)),
        },
        ChoreographyNode::Transfer {
            from,
            to,
            label,
            payload_tag,
            value,
            next,
            ..
        } => GlobalChoreo::OwnedTransfer {
            from: from.0,
            to: to.0,
            label: label.0,
            payload_tag: PayloadTag { raw: payload_tag.0 },
            value: verified_logic::choreography_ownership::ValueId { raw: value.0 },
            next: Box::new(lower_node(next)),
        },
        ChoreographyNode::Choice {
            decider, branches, ..
        } => GlobalChoreo::Choice {
            decider: decider.0,
            branches: branches
                .iter()
                .map(|branch| GlobalBranch {
                    label: branch.label.0,
                    body: lower_node(&branch.body),
                })
                .collect(),
        },
    }
}

pub fn parse_evaluated_json(
    input: &[u8],
) -> Result<RoleChoreographySource, Vec<ChoreographyDiagnostic>> {
    serde_json::from_slice(input).map_err(|_| {
        issue(
            DiagnosticCode::MalformedEvaluatedData,
            SourceSpan { start: 0, end: 0 },
            "evaluated choreography data is malformed or has unknown fields",
        )
    })
}

pub fn admit_source(
    mut source: RoleChoreographySource,
) -> Result<AdmittedChoreography, Vec<ChoreographyDiagnostic>> {
    let root = source.protocol.span();
    if source.schema != ROLE_CHOREOGRAPHY_SCHEMA {
        return Err(issue(
            DiagnosticCode::UnsupportedSchema,
            root,
            "role choreography schema is unsupported",
        ));
    }
    if source.limits != ChoreographyLimits::reviewed() {
        return Err(issue(
            DiagnosticCode::LimitsMismatch,
            root,
            "source limits differ from the reviewed fixed limits",
        ));
    }
    validate_imports(&source.imports, root)?;
    validate_roles(&source.roles, root)?;
    validate_values(&source.values, &source.roles, root)?;
    validate_non_claims(&source)?;

    let snapshot = initial_ownership(&source.values);
    trellis_state(&snapshot, root)?;
    let mut count = TraversalCount::default();
    let mut context = NodeValidationContext {
        roles: &source.roles,
        values: &source.values,
        count: &mut count,
    };
    validate_node(&source.protocol, &snapshot, 0, &mut context)?;
    debug_assert!(count.steps <= MAX_STEPS);
    debug_assert!(count.branches <= MAX_BRANCHES);

    source.roles.sort_unstable_by_key(|role| role.id);
    source.values.sort_unstable_by_key(|value| value.id);
    source.non_claims.sort_unstable();
    debug_assert!(source.roles.is_sorted_by_key(|role| role.id));
    debug_assert!(source.values.is_sorted_by_key(|value| value.id));
    debug_assert!(source.non_claims.is_sorted());
    let roles = source.roles.iter().map(|role| role.id).collect();
    let global = lower_node(&source.protocol);
    if !verified_logic::choreography_global::well_formed_global(&global) {
        return Err(issue(
            DiagnosticCode::TrellisGlobalRejected,
            root,
            "Trellis rejected global choreography well-formedness",
        ));
    }
    if !projectable(&global) {
        return Err(projection_issue(&source.protocol));
    }

    let admitted = AdmittedChoreography {
        source,
        global,
        roles,
    };
    let no_diagnostics: Vec<ChoreographyDiagnostic> = Vec::new();
    if !diagnostics_are_safe(&no_diagnostics) {
        return Err(issue(
            DiagnosticCode::UnsafeDiagnostic,
            root,
            "diagnostic safety invariant failed",
        ));
    }
    Ok(admitted)
}
