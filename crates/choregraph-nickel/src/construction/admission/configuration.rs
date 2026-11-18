//! Iterative, depth-bounded configuration traversal and local references.

struct Walk<'a> {
    current: std::slice::Iter<'a, crate::construction::model::projection::ExpandedExpression>,
    parents: Vec<std::slice::Iter<'a, crate::construction::model::projection::ExpandedExpression>>,
    max_depth: usize,
}
impl<'a> Walk<'a> {
    fn new(
        input: &'a crate::construction::model::projection::ExpandedExpression,
        limits: crate::construction::model::profile::Limits,
    ) -> Self {
        Self {
            current: std::slice::from_ref(input).iter(),
            parents: Vec::new(),
            max_depth: limits.max_depth,
        }
    }
    fn next(&mut self) -> Option<&'a crate::construction::model::projection::ExpandedExpression> {
        loop {
            if let Some(input) = self.current.next() {
                return Some(input);
            }
            self.current = self.parents.pop()?;
        }
    }
    fn descend(
        &mut self,
        arguments: &'a [crate::construction::model::projection::ExpandedExpression],
    ) -> Result<(), crate::construction::Failure> {
        if !arguments.is_empty() {
            if self.parents.len() >= self.max_depth {
                return Err(crate::construction::Failure::LimitExceeded);
            }
            self.parents.reserve(1);
            self.parents
                .push(std::mem::replace(&mut self.current, arguments.iter()));
        }
        Ok(())
    }
}
pub(super) struct Input<'a> {
    pub expression: &'a crate::construction::model::projection::ExpandedExpression,
    pub node: &'a crate::construction::model::projection::ExpandedNode,
    pub field: &'a crate::construction::model::host::ConfigurationField,
    pub declarations: &'a [crate::construction::model::host::ConfigurationField],
    pub limits: crate::construction::model::profile::Limits,
}

#[derive(Clone, Copy)]
struct Reference<'a> {
    owner: u64,
    direction: crate::construction::model::term::ReferenceDirection,
    name: &'a str,
    port: Option<u64>,
}

pub(super) fn validate(
    context: Input<'_>,
    uses: &mut std::collections::BTreeMap<u64, usize>,
) -> Result<(), crate::construction::Failure> {
    let mut walk = Walk::new(context.expression, context.limits);
    while let Some(input) = walk.next() {
        check(input, &context, uses)?;
        if let crate::construction::model::projection::ExpandedExpression::Apply {
            arguments, ..
        } = input
        {
            walk.descend(arguments)?;
        }
    }
    Ok(())
}
fn check(
    input: &crate::construction::model::projection::ExpandedExpression,
    context: &Input<'_>,
    uses: &mut std::collections::BTreeMap<u64, usize>,
) -> Result<(), crate::construction::Failure> {
    match input {
        crate::construction::model::projection::ExpandedExpression::Literal { value } => {
            if crate::construction::registry::literal_type(value, &context.field.value_type) {
                Ok(())
            } else {
                Err(crate::construction::Failure::InvalidConfiguration)
            }
        }
        crate::construction::model::projection::ExpandedExpression::Reference {
            node,
            direction,
            name,
            port,
        } => reference(
            Reference {
                owner: *node,
                direction: *direction,
                name,
                port: *port,
            },
            context,
            uses,
        ),
        _ => application(input, context),
    }
}
fn application(
    input: &crate::construction::model::projection::ExpandedExpression,
    context: &Input<'_>,
) -> Result<(), crate::construction::Failure> {
    let crate::construction::model::projection::ExpandedExpression::Apply {
        operator,
        arguments,
    } = input
    else {
        return Err(crate::construction::Failure::InvalidConfiguration);
    };
    if arguments.len() > context.limits.max_boundaries
        || !context.field.operators.contains(operator)
    {
        return Err(crate::construction::Failure::InvalidConfiguration);
    }
    Ok(())
}
fn reference(
    input: Reference<'_>,
    context: &Input<'_>,
    uses: &mut std::collections::BTreeMap<u64, usize>,
) -> Result<(), crate::construction::Failure> {
    if !context.field.references || input.owner != context.node.id {
        return Err(crate::construction::Failure::InvalidReference);
    }
    debug_assert!(
        context.node.configuration.contains_key(&context.field.name),
        "validated field belongs to the node"
    );
    debug_assert!(
        context
            .declarations
            .iter()
            .any(|field| field.name == context.field.name),
        "validated field belongs to the signature"
    );
    crate::construction::identifier(input.name, context.limits)?;
    match input.direction {
        crate::construction::model::term::ReferenceDirection::Input
        | crate::construction::model::term::ReferenceDirection::Output => {
            port_reference(input, context.node, uses)
        }
        _ => named_reference(input, context),
    }
}
fn named_reference(
    input: Reference<'_>,
    context: &Input<'_>,
) -> Result<(), crate::construction::Failure> {
    if input.port.is_some() {
        return Err(crate::construction::Failure::InvalidReference);
    }
    debug_assert!(context.field.references, "reference-bearing field admitted");
    debug_assert_eq!(input.owner, context.node.id, "reference owner admitted");
    if matches!(
        input.direction,
        crate::construction::model::term::ReferenceDirection::Config
    ) {
        config_reference(input.name, context)
    } else if matches!(
        input.direction,
        crate::construction::model::term::ReferenceDirection::Invocation
    ) {
        context
            .node
            .invocation
            .iter()
            .any(|role| role.name == input.name)
            .then_some(())
            .ok_or(crate::construction::Failure::InvalidReference)
    } else {
        Err(crate::construction::Failure::InvalidReference)
    }
}
fn config_reference(name: &str, context: &Input<'_>) -> Result<(), crate::construction::Failure> {
    if !context.node.configuration.contains_key(name) {
        return Err(crate::construction::Failure::InvalidReference);
    }
    let selected = context
        .declarations
        .iter()
        .find(|field| field.name == name)
        .ok_or(crate::construction::Failure::InvalidReference)?;
    if selected.value_type != context.field.value_type {
        return Err(crate::construction::Failure::IncompatibleType);
    }
    Ok(())
}
fn port_reference(
    input: Reference<'_>,
    node: &crate::construction::model::projection::ExpandedNode,
    uses: &mut std::collections::BTreeMap<u64, usize>,
) -> Result<(), crate::construction::Failure> {
    let (selected, direction) = selected_port(input, node)?;
    if direction == crate::construction::model::interface::Direction::Output {
        crate::construction::admission::count_use(uses, selected.id)?;
    }
    Ok(())
}
fn selected_port<'a>(
    input: Reference<'_>,
    node: &'a crate::construction::model::projection::ExpandedNode,
) -> Result<
    (
        &'a crate::construction::model::projection::ExpandedPort,
        crate::construction::model::interface::Direction,
    ),
    crate::construction::Failure,
> {
    let direction = port_direction(input.direction)?;
    let selected = node
        .ports
        .iter()
        .find(|candidate| candidate.name == input.name && candidate.direction == direction)
        .ok_or(crate::construction::Failure::InvalidReference)?;
    if input.port != Some(selected.id) {
        return Err(crate::construction::Failure::InvalidReference);
    }
    Ok((selected, direction))
}
const fn port_direction(
    direction: crate::construction::model::term::ReferenceDirection,
) -> Result<crate::construction::model::interface::Direction, crate::construction::Failure> {
    if matches!(
        direction,
        crate::construction::model::term::ReferenceDirection::Input
    ) {
        Ok(crate::construction::model::interface::Direction::Input)
    } else if matches!(
        direction,
        crate::construction::model::term::ReferenceDirection::Output
    ) {
        Ok(crate::construction::model::interface::Direction::Output)
    } else {
        Err(crate::construction::Failure::InvalidReference)
    }
}
pub(super) fn resolved(
    input: &crate::construction::model::projection::ExpandedExpression,
    names: &std::collections::BTreeSet<&str>,
    limits: crate::construction::model::profile::Limits,
) -> Result<bool, crate::construction::Failure> {
    let mut walk = Walk::new(input, limits);
    while let Some(input) = walk.next() {
        if !reference_resolved(input, names) {
            return Ok(false);
        }
        if let crate::construction::model::projection::ExpandedExpression::Apply {
            arguments, ..
        } = input
        {
            walk.descend(arguments)?;
        }
    }
    Ok(true)
}
fn reference_resolved(
    input: &crate::construction::model::projection::ExpandedExpression,
    names: &std::collections::BTreeSet<&str>,
) -> bool {
    !matches!(input,
        crate::construction::model::projection::ExpandedExpression::Reference { direction: crate::construction::model::term::ReferenceDirection::Config, name, .. }
        if !names.contains(name.as_str()))
}
