//! Ordered registry signatures, numeric indexes, and configuration dependencies.

pub(super) fn validate<'a>(
    projection: &'a crate::construction::model::projection::LatticeV3,
    selected: &crate::construction::registry::Index<'_>,
    limits: crate::construction::model::profile::Limits,
    uses: &mut std::collections::BTreeMap<u64, usize>,
) -> Result<crate::construction::admission::Indexes<'a>, crate::construction::Failure> {
    let mut indexes = crate::construction::admission::Indexes {
        nodes: std::collections::BTreeMap::new(),
        ports: std::collections::BTreeMap::new(),
    };
    let mut scoped = std::collections::BTreeSet::new();
    let mut invocation_count = 0usize;
    for node in &projection.nodes {
        debug_assert_eq!(
            indexes.nodes.len(),
            scoped.len(),
            "scope and node indexes advance together"
        );
        crate::construction::identifier(&node.scoped_id, limits)?;
        if !scoped.insert(&node.scoped_id) || indexes.nodes.insert(node.id, node).is_some() {
            return Err(crate::construction::Failure::DuplicateKey);
        }
        let signature = selected
            .operations
            .get(node.operation.as_str())
            .ok_or(crate::construction::Failure::UnknownOperation)?;
        signature_matches(node, signature)?;
        invocation_count = invocation_count
            .checked_add(node.invocation.len())
            .ok_or(crate::construction::Failure::LimitExceeded)?;
        if invocation_count > limits.max_boundaries {
            return Err(crate::construction::Failure::LimitExceeded);
        }
        register_ports(node, signature, &mut indexes.ports, limits)?;
        fields(node, signature, uses, limits)?;
        acyclic_configuration(node, limits)?;
    }
    debug_assert!(
        indexes.ports.len() <= limits.max_ports,
        "global port index remains bounded"
    );
    Ok(indexes)
}
fn signature_matches(
    node: &crate::construction::model::projection::ExpandedNode,
    signature: &crate::construction::model::host::OperationSignature,
) -> Result<(), crate::construction::Failure> {
    if node.implementation != signature.implementation || node.invocation != signature.invocation {
        return Err(crate::construction::Failure::InvalidRegistry);
    }
    let ports = signature
        .inputs
        .len()
        .checked_add(signature.outputs.len())
        .ok_or(crate::construction::Failure::LimitExceeded)?;
    if node.ports.len() != ports || node.configuration.len() != signature.configuration.len() {
        return Err(crate::construction::Failure::InvalidRegistry);
    }
    Ok(())
}
fn register_ports<'a>(
    node: &'a crate::construction::model::projection::ExpandedNode,
    signature: &crate::construction::model::host::OperationSignature,
    ports: &mut crate::construction::admission::PortIndex<'a>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    for (port, (boundary, direction)) in node.ports.iter().zip(ordered_ports(signature)) {
        port_matches(port, boundary, direction)?;
        if ports.len() >= limits.max_ports {
            return Err(crate::construction::Failure::LimitExceeded);
        }
        if ports.insert(port.id, (node.id, port)).is_some() {
            return Err(crate::construction::Failure::DuplicateKey);
        }
    }
    Ok(())
}
fn ordered_ports(
    signature: &crate::construction::model::host::OperationSignature,
) -> impl Iterator<
    Item = (
        &crate::construction::model::interface::Boundary,
        crate::construction::model::interface::Direction,
    ),
> {
    signature
        .inputs
        .iter()
        .map(|boundary| {
            (
                boundary,
                crate::construction::model::interface::Direction::Input,
            )
        })
        .chain(signature.outputs.iter().map(|boundary| {
            (
                boundary,
                crate::construction::model::interface::Direction::Output,
            )
        }))
}
fn port_matches(
    port: &crate::construction::model::projection::ExpandedPort,
    boundary: &crate::construction::model::interface::Boundary,
    direction: crate::construction::model::interface::Direction,
) -> Result<(), crate::construction::Failure> {
    if port.direction != direction {
        return Err(crate::construction::Failure::WrongDirection);
    }
    if port.name != boundary.name {
        return Err(crate::construction::Failure::UnknownBoundary);
    }
    if port.value_type != boundary.value_type {
        return Err(crate::construction::Failure::IncompatibleType);
    }
    Ok(())
}
fn fields(
    node: &crate::construction::model::projection::ExpandedNode,
    signature: &crate::construction::model::host::OperationSignature,
    uses: &mut std::collections::BTreeMap<u64, usize>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    for field in &signature.configuration {
        let input = node
            .configuration
            .get(&field.name)
            .ok_or(crate::construction::Failure::InvalidConfiguration)?;
        crate::construction::admission::configuration::validate(
            crate::construction::admission::configuration::Input {
                expression: input,
                node,
                field,
                declarations: &signature.configuration,
                limits,
            },
            uses,
        )?;
    }
    Ok(())
}
fn ready_fields<'a>(
    node: &'a crate::construction::model::projection::ExpandedNode,
    pending: &std::collections::BTreeSet<&'a str>,
    resolved: &std::collections::BTreeSet<&str>,
    limits: crate::construction::model::profile::Limits,
) -> Result<Vec<&'a str>, crate::construction::Failure> {
    let mut ready = Vec::with_capacity(pending.len());
    for name in pending {
        if crate::construction::admission::configuration::resolved(
            &node.configuration[*name],
            resolved,
            limits,
        )? {
            ready.push(*name);
        }
    }
    Ok(ready)
}
fn acyclic_configuration(
    node: &crate::construction::model::projection::ExpandedNode,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    let mut pending = node
        .configuration
        .keys()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let mut resolved = std::collections::BTreeSet::new();
    while !pending.is_empty() {
        let ready = ready_fields(node, &pending, &resolved, limits)?;
        debug_assert!(
            ready.iter().all(|name| pending.contains(name)),
            "only pending fields become ready"
        );
        if ready.is_empty() {
            return Err(crate::construction::Failure::InvalidReference);
        }
        for name in ready {
            pending.remove(name);
            if resolved.len() >= node.configuration.len() {
                return Err(crate::construction::Failure::LimitExceeded);
            }
            resolved.insert(name);
        }
    }
    debug_assert_eq!(
        resolved.len(),
        node.configuration.len(),
        "every configured field was resolved"
    );
    Ok(())
}
