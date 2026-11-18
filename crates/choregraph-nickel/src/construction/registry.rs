//! Native, host-selected registry validation. Author descriptors confer no traits.

pub(super) struct Index<'a> {
    pub operations: std::collections::BTreeMap<
        &'a str,
        &'a crate::construction::model::host::OperationSignature,
    >,
    pub types: std::collections::BTreeMap<&'a str, &'a crate::construction::model::host::ValueType>,
}

pub(super) fn validate(
    registry: &crate::construction::model::host::TargetRegistry,
    limits: crate::construction::model::profile::Limits,
) -> Result<Index<'_>, crate::construction::Failure> {
    crate::construction::bounded_json(registry, limits.max_candidate_bytes)?;
    header(registry, limits)?;
    let types = types(registry, limits)?;
    let operations = operations(registry, &types, limits)?;
    Ok(Index { operations, types })
}
fn header(
    registry: &crate::construction::model::host::TargetRegistry,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    if registry.schema != crate::construction::model::profile::REGISTRY_SCHEMA
        || registry.operations.len() > limits.max_nodes
        || registry.types.len() > limits.max_boundaries
    {
        return Err(crate::construction::Failure::InvalidRegistry);
    }
    for value in [
        &registry.target,
        &registry.profile,
        &registry.implementation,
    ] {
        crate::construction::identifier(value, limits)?;
    }
    Ok(())
}
fn types(
    registry: &crate::construction::model::host::TargetRegistry,
    limits: crate::construction::model::profile::Limits,
) -> Result<
    std::collections::BTreeMap<&str, &crate::construction::model::host::ValueType>,
    crate::construction::Failure,
> {
    let mut types = std::collections::BTreeMap::new();
    for ty in &registry.types {
        crate::construction::identifier(&ty.name, limits)?;
        if types.len() >= limits.max_boundaries || types.insert(ty.name.as_str(), ty).is_some() {
            return Err(crate::construction::Failure::InvalidRegistry);
        }
    }
    if !types.contains_key("Unit") {
        return Err(crate::construction::Failure::InvalidRegistry);
    }
    Ok(types)
}
fn operations<'a>(
    registry: &'a crate::construction::model::host::TargetRegistry,
    types: &std::collections::BTreeMap<&str, &crate::construction::model::host::ValueType>,
    limits: crate::construction::model::profile::Limits,
) -> Result<
    std::collections::BTreeMap<&'a str, &'a crate::construction::model::host::OperationSignature>,
    crate::construction::Failure,
> {
    let mut operations = std::collections::BTreeMap::new();
    let mut total_ports = 0usize;
    let mut total_effects = 0usize;
    for operation in &registry.operations {
        operation_entry(operation, &mut operations, limits)?;
        for group in [&operation.inputs, &operation.outputs, &operation.invocation] {
            boundaries(group, types, &mut total_ports, limits)?;
        }
        configuration(operation, limits)?;
        effects(&operation.effects, &mut total_effects, limits)?;
    }
    Ok(operations)
}
fn operation_entry<'a>(
    operation: &'a crate::construction::model::host::OperationSignature,
    operations: &mut std::collections::BTreeMap<
        &'a str,
        &'a crate::construction::model::host::OperationSignature,
    >,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    crate::construction::identifier(&operation.operation, limits)?;
    crate::construction::identifier(&operation.implementation, limits)?;
    if operations.len() >= limits.max_nodes
        || operations
            .insert(operation.operation.as_str(), operation)
            .is_some()
    {
        return Err(crate::construction::Failure::InvalidRegistry);
    }
    Ok(())
}
fn boundaries(
    boundaries: &[crate::construction::model::interface::Boundary],
    types: &std::collections::BTreeMap<&str, &crate::construction::model::host::ValueType>,
    total: &mut usize,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    let mut names = std::collections::BTreeSet::new();
    for boundary in boundaries {
        boundary_entry(boundary, types, &mut names, limits)?;
        *total = total
            .checked_add(1)
            .ok_or(crate::construction::Failure::LimitExceeded)?;
        if *total > limits.max_ports {
            return Err(crate::construction::Failure::LimitExceeded);
        }
    }
    Ok(())
}
fn boundary_entry<'a>(
    boundary: &'a crate::construction::model::interface::Boundary,
    types: &std::collections::BTreeMap<&str, &crate::construction::model::host::ValueType>,
    names: &mut std::collections::BTreeSet<&'a str>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    crate::construction::identifier(&boundary.name, limits)?;
    if boundary.name.contains('/')
        || !types.contains_key(boundary.value_type.as_str())
        || !names.insert(&boundary.name)
    {
        return Err(crate::construction::Failure::InvalidRegistry);
    }
    Ok(())
}
fn configuration(
    operation: &crate::construction::model::host::OperationSignature,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    let mut names = std::collections::BTreeSet::new();
    if operation.configuration.len() > limits.max_boundaries {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    for field in &operation.configuration {
        crate::construction::identifier(&field.name, limits)?;
        if field.name.contains('/')
            || !names.insert(&field.name)
            || !literal_name(&field.value_type)
        {
            return Err(crate::construction::Failure::InvalidRegistry);
        }
        operators(field, limits)?;
    }
    Ok(())
}
fn literal_name(name: &str) -> bool {
    matches!(
        name,
        "null" | "bool" | "number" | "string" | "array" | "object"
    )
}
fn operators(
    field: &crate::construction::model::host::ConfigurationField,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    let mut operators = std::collections::BTreeSet::new();
    for op in &field.operators {
        crate::construction::identifier(op, limits)?;
        if operators.len() >= field.operators.len() || !operators.insert(op) {
            return Err(crate::construction::Failure::InvalidRegistry);
        }
    }
    Ok(())
}
fn effects(
    effects: &[crate::construction::model::host::EffectRule],
    total: &mut usize,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    let mut rules = std::collections::BTreeSet::new();
    for effect in effects {
        effect_entry(effect, &mut rules, limits)?;
        *total = total
            .checked_add(1)
            .ok_or(crate::construction::Failure::LimitExceeded)?;
        if *total > limits.max_effects {
            return Err(crate::construction::Failure::LimitExceeded);
        }
    }
    Ok(())
}
fn effect_entry<'a>(
    effect: &'a crate::construction::model::host::EffectRule,
    rules: &mut std::collections::BTreeSet<&'a crate::construction::model::host::EffectRule>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    for value in [
        &effect.from_state,
        &effect.action,
        &effect.resource,
        &effect.ability,
    ] {
        crate::construction::identifier(value, limits)?;
    }
    if !rules.insert(effect) {
        return Err(crate::construction::Failure::InvalidRegistry);
    }
    Ok(())
}

pub(super) fn literal_type(value: &serde_json::Value, expected: &str) -> bool {
    matches!(
        (value, expected),
        (serde_json::Value::Null, "null")
            | (serde_json::Value::Bool(_), "bool")
            | (serde_json::Value::Number(_), "number")
            | (serde_json::Value::String(_), "string")
            | (serde_json::Value::Array(_), "array")
            | (serde_json::Value::Object(_), "object")
    )
}
