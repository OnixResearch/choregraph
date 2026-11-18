struct Mapping<'a> {
    adjacency: &'a std::collections::BTreeMap<&'a str, Vec<&'a str>>,
    port_ids: &'a std::collections::BTreeMap<String, u64>,
    direction: crate::construction::model::interface::Direction,
}

fn enqueue<'a>(
    pending: &mut Vec<&'a str>,
    next: &[&'a str],
    ceiling: crate::construction::expand::Ceiling,
) -> Result<(), crate::construction::Failure> {
    let count = pending
        .len()
        .checked_add(next.len())
        .ok_or(crate::construction::Failure::LimitExceeded)?;
    if count > ceiling.0 {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    pending.extend(next.iter().copied());
    Ok(())
}

pub(super) fn arc(
    expansion: &mut crate::construction::expand::Expansion<'_>,
    route: crate::construction::expand::Route<'_>,
) -> Result<(), crate::construction::Failure> {
    let from = expansion
        .endpoints
        .get(route.source)
        .ok_or(crate::construction::Failure::UnknownBoundary)?;
    let to = expansion
        .endpoints
        .get(route.target)
        .ok_or(crate::construction::Failure::UnknownBoundary)?;
    if from.value_type != to.value_type {
        return Err(crate::construction::Failure::IncompatibleType);
    }
    debug_assert_eq!(from.value_type, to.value_type);
    if expansion
        .arcs
        .len()
        .checked_add(expansion.controls.len())
        .ok_or(crate::construction::Failure::LimitExceeded)?
        >= expansion.limits.max_edges
    {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    if !expansion
        .arcs
        .insert((route.source.to_owned(), route.target.to_owned()))
    {
        return Err(crate::construction::Failure::DuplicateKey);
    }
    debug_assert!(
        expansion
            .arcs
            .len()
            .saturating_add(expansion.controls.len())
            <= expansion.limits.max_edges
    );
    Ok(())
}

pub(super) fn control(
    expansion: &mut crate::construction::expand::Expansion<'_>,
    route: crate::construction::expand::Route<'_>,
) -> Result<(), crate::construction::Failure> {
    if expansion
        .controls
        .contains(&(route.source.to_owned(), route.target.to_owned()))
    {
        return Ok(());
    }
    if expansion
        .arcs
        .len()
        .checked_add(expansion.controls.len())
        .ok_or(crate::construction::Failure::LimitExceeded)?
        >= expansion.limits.max_edges
    {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    debug_assert!(expansion.controls.len() < expansion.limits.max_edges);
    expansion
        .controls
        .insert((route.source.to_owned(), route.target.to_owned()));
    debug_assert!(
        expansion
            .arcs
            .len()
            .saturating_add(expansion.controls.len())
            <= expansion.limits.max_edges
    );
    Ok(())
}
pub(super) fn data_routes<'a>(
    expansion: &'a crate::construction::expand::Expansion<'_>,
) -> Result<std::collections::BTreeSet<(&'a str, &'a str)>, crate::construction::Failure> {
    let mut outgoing: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    for (source, target) in &expansion.arcs {
        outgoing.entry(source).or_default().push(target);
    }
    let mut routes = std::collections::BTreeSet::new();
    let mut pending = Vec::with_capacity(expansion.arcs.len());
    let mut visited = std::collections::BTreeSet::new();
    for (source, endpoint) in &expansion.endpoints {
        if !matches!(
            endpoint.real,
            Some((
                _,
                crate::construction::model::interface::Direction::Output,
                _
            ))
        ) {
            continue;
        }
        pending.clear();
        visited.clear();
        if let Some(next) = outgoing.get(source.as_str()) {
            enqueue(
                &mut pending,
                next,
                crate::construction::expand::Ceiling(expansion.arcs.len()),
            )?;
        }
        while let Some(target) = pending.pop() {
            if !visited.insert(target) {
                return Err(crate::construction::Failure::AdmissionRejected);
            }
            let endpoint = expansion
                .endpoints
                .get(target)
                .ok_or(crate::construction::Failure::UnknownBoundary)?;
            if let Some((_, crate::construction::model::interface::Direction::Input, _)) =
                endpoint.real
            {
                if routes.len() >= expansion.limits.max_edges {
                    return Err(crate::construction::Failure::LimitExceeded);
                }
                if !routes.insert((source.as_str(), target)) {
                    return Err(crate::construction::Failure::DuplicateKey);
                }
            } else {
                if endpoint.real.is_some() {
                    return Err(crate::construction::Failure::WrongDirection);
                }
                if let Some(next) = outgoing.get(target) {
                    enqueue(
                        &mut pending,
                        next,
                        crate::construction::expand::Ceiling(expansion.arcs.len()),
                    )?;
                }
            }
        }
    }
    debug_assert!(visited.len() <= expansion.endpoints.len());
    debug_assert!(routes.len() <= expansion.limits.max_edges);
    Ok(routes)
}

pub(super) fn frontier(
    expansion: &crate::construction::expand::Expansion<'_>,
    nodes: &std::collections::BTreeSet<String>,
) -> Result<
    (
        std::collections::BTreeSet<String>,
        std::collections::BTreeSet<String>,
    ),
    crate::construction::Failure,
> {
    let mut entries = nodes.clone();
    let mut exits = nodes.clone();
    for (source, target) in expansion
        .controls
        .iter()
        .map(|(source, target)| (source.as_str(), target.as_str()))
        .chain(
            data_routes(expansion)?
                .into_iter()
                .filter_map(|(source, target)| {
                    let from = expansion.endpoints.get(source)?.real.as_ref()?.0.as_str();
                    let to = expansion.endpoints.get(target)?.real.as_ref()?.0.as_str();
                    Some((from, to))
                }),
        )
    {
        if nodes.contains(source) && nodes.contains(target) {
            entries.remove(target);
            exits.remove(source);
        }
    }
    debug_assert!(entries.is_subset(nodes) && exits.is_subset(nodes));
    debug_assert!(nodes.iter().all(|node| expansion.nodes.contains_key(node)));
    Ok((entries, exits))
}
pub(super) fn expanded_interfaces(
    expansion: &crate::construction::expand::Expansion<'_>,
    port_ids: &std::collections::BTreeMap<String, u64>,
) -> Result<
    Vec<crate::construction::model::projection::ExpandedInterface>,
    crate::construction::Failure,
> {
    let mut forward = std::collections::BTreeMap::<&str, Vec<&str>>::new();
    let mut reverse = std::collections::BTreeMap::<&str, Vec<&str>>::new();
    for (source, target) in &expansion.arcs {
        forward.entry(source).or_default().push(target);
        reverse.entry(target).or_default().push(source);
    }
    let mut interfaces = Vec::with_capacity(expansion.interfaces.len());
    let mut mapped = 0usize;
    for (scope, inputs, outputs) in &expansion.interfaces {
        let mut result = interface(scope, (inputs.len(), outputs.len()));
        for (boundaries, direction) in [
            (
                inputs,
                crate::construction::model::interface::Direction::Input,
            ),
            (
                outputs,
                crate::construction::model::interface::Direction::Output,
            ),
        ] {
            let adjacency = if direction == crate::construction::model::interface::Direction::Input
            {
                &forward
            } else {
                &reverse
            };
            let mapping = Mapping {
                adjacency,
                port_ids,
                direction,
            };
            for boundary in boundaries {
                let expanded = map_boundary(expansion, boundary, &mapping, &mut mapped)?;
                if direction == crate::construction::model::interface::Direction::Input {
                    result.inputs.push(expanded)
                } else {
                    result.outputs.push(expanded)
                }
            }
        }
        interfaces.push(result);
    }
    interfaces.sort_by(|left, right| left.scope.cmp(&right.scope));
    debug_assert!(interfaces.len() <= expansion.instances);
    debug_assert!(mapped <= expansion.limits.max_boundaries);
    Ok(interfaces)
}

fn map_boundary(
    expansion: &crate::construction::expand::Expansion<'_>,
    boundary: &crate::construction::expand::Port,
    mapping: &Mapping<'_>,
    mapped: &mut usize,
) -> Result<crate::construction::model::projection::ExpandedBoundary, crate::construction::Failure>
{
    let mut pending = vec![boundary.endpoint.as_str()];
    let mut visited = std::collections::BTreeSet::new();
    let mut ports = std::collections::BTreeSet::new();
    let ceiling = crate::construction::expand::deeper(expansion.arcs.len())?;
    while let Some(endpoint) = pending.pop() {
        if !visited.insert(endpoint) {
            continue;
        }
        let metadata = expansion
            .endpoints
            .get(endpoint)
            .ok_or(crate::construction::Failure::InvalidReference)?;
        if let Some((_, actual, _)) = &metadata.real {
            if *actual != mapping.direction {
                return Err(crate::construction::Failure::WrongDirection);
            }
            if ports.insert(mapping.port_ids[endpoint]) {
                crate::construction::expand::take(
                    mapped,
                    1,
                    crate::construction::expand::Ceiling(expansion.limits.max_boundaries),
                )?;
            }
        } else if let Some(next) = mapping.adjacency.get(endpoint) {
            enqueue(
                &mut pending,
                next,
                crate::construction::expand::Ceiling(ceiling),
            )?;
        }
    }
    debug_assert!(*mapped <= expansion.limits.max_boundaries);
    debug_assert!(visited.len() <= expansion.endpoints.len());
    Ok(crate::construction::model::projection::ExpandedBoundary {
        name: boundary.name.clone(),
        value_type: boundary.value_type.clone(),
        ports: ports.into_iter().collect(),
    })
}

fn interface(
    scope: &str,
    (inputs, outputs): (usize, usize),
) -> crate::construction::model::projection::ExpandedInterface {
    crate::construction::model::projection::ExpandedInterface {
        scope: scope.to_owned(),
        inputs: Vec::with_capacity(inputs),
        outputs: Vec::with_capacity(outputs),
    }
}
