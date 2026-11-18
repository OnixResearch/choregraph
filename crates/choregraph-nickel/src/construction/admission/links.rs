//! Typed data/control links and diagnostic interface maps.

pub(super) struct LinkIndex {
    pub dependencies: std::collections::BTreeSet<(u64, u64)>,
    pub inputs: std::collections::BTreeMap<u64, usize>,
}
pub(super) fn validate(
    projection: &crate::construction::model::projection::LatticeV3,
    indexes: &crate::construction::admission::Indexes<'_>,
    uses: &mut std::collections::BTreeMap<u64, usize>,
    limits: crate::construction::model::profile::Limits,
) -> Result<LinkIndex, crate::construction::Failure> {
    let mut links = LinkIndex {
        dependencies: std::collections::BTreeSet::new(),
        inputs: std::collections::BTreeMap::new(),
    };
    let mut data_edges = std::collections::BTreeSet::new();
    for edge in &projection.edges {
        if data_edges.len() >= limits.max_edges || !data_edges.insert(edge) {
            return Err(crate::construction::Failure::DuplicateKey);
        }
        data_edge(edge, indexes, &mut links, uses)?;
    }
    control_edges(projection, indexes, &mut links, limits)?;
    Ok(links)
}
fn data_edge(
    edge: &crate::construction::model::projection::DataEdge,
    indexes: &crate::construction::admission::Indexes<'_>,
    links: &mut LinkIndex,
    uses: &mut std::collections::BTreeMap<u64, usize>,
) -> Result<(), crate::construction::Failure> {
    let (source_node, source) = indexes
        .ports
        .get(&edge.source)
        .ok_or(crate::construction::Failure::UnknownBoundary)?;
    let (target_node, target) = indexes
        .ports
        .get(&edge.target)
        .ok_or(crate::construction::Failure::UnknownBoundary)?;
    compatible(source, target)?;
    crate::construction::admission::count_use(uses, source.id)?;
    crate::construction::admission::count_use(&mut links.inputs, target.id)?;
    links.dependencies.insert((*source_node, *target_node));
    Ok(())
}
fn compatible(
    source: &crate::construction::model::projection::ExpandedPort,
    target: &crate::construction::model::projection::ExpandedPort,
) -> Result<(), crate::construction::Failure> {
    if source.direction != crate::construction::model::interface::Direction::Output
        || target.direction != crate::construction::model::interface::Direction::Input
    {
        return Err(crate::construction::Failure::WrongDirection);
    }
    if source.value_type != target.value_type {
        return Err(crate::construction::Failure::IncompatibleType);
    }
    Ok(())
}
fn control_edges(
    projection: &crate::construction::model::projection::LatticeV3,
    indexes: &crate::construction::admission::Indexes<'_>,
    links: &mut LinkIndex,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    let mut control_edges = std::collections::BTreeSet::new();
    for edge in &projection.control_edges {
        if !indexes.nodes.contains_key(&edge.source) || !indexes.nodes.contains_key(&edge.target) {
            return Err(crate::construction::Failure::InvalidReference);
        }
        if control_edges.len() >= limits.max_edges || !control_edges.insert(edge) {
            return Err(crate::construction::Failure::DuplicateKey);
        }
        links.dependencies.insert((edge.source, edge.target));
    }
    Ok(())
}
#[derive(Clone, Copy)]
struct Counts {
    boundaries: usize,
    mapped: usize,
}
impl Counts {
    fn take(
        &mut self,
        boundary: &crate::construction::model::projection::ExpandedBoundary,
        limits: crate::construction::model::profile::Limits,
    ) -> Result<(), crate::construction::Failure> {
        self.boundaries = self
            .boundaries
            .checked_add(1)
            .ok_or(crate::construction::Failure::LimitExceeded)?;
        self.mapped = self
            .mapped
            .checked_add(boundary.ports.len())
            .ok_or(crate::construction::Failure::LimitExceeded)?;
        if self.boundaries > limits.max_boundaries || self.mapped > limits.max_boundaries {
            return Err(crate::construction::Failure::LimitExceeded);
        }
        Ok(())
    }
}
pub(super) fn interfaces(
    projection: &crate::construction::model::projection::LatticeV3,
    selected: &crate::construction::registry::Index<'_>,
    ports: &crate::construction::admission::PortIndex<'_>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    if projection.interfaces.len() > limits.max_instances {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    let mut scopes = std::collections::BTreeSet::new();
    let mut counts = Counts {
        boundaries: 0,
        mapped: 0,
    };
    for interface in &projection.interfaces {
        interface_scope(interface, &mut scopes, limits)?;
        interface_groups(interface, selected, ports, &mut counts, limits)?;
    }
    Ok(())
}
fn interface_scope<'a>(
    interface: &'a crate::construction::model::projection::ExpandedInterface,
    scopes: &mut std::collections::BTreeSet<&'a str>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    crate::construction::identifier(&interface.scope, limits)?;
    if scopes.len() >= limits.max_instances || !scopes.insert(&interface.scope) {
        return Err(crate::construction::Failure::DuplicateKey);
    }
    Ok(())
}
struct Group<'a> {
    boundaries: &'a [crate::construction::model::projection::ExpandedBoundary],
    direction: crate::construction::model::interface::Direction,
}
fn interface_groups(
    interface: &crate::construction::model::projection::ExpandedInterface,
    selected: &crate::construction::registry::Index<'_>,
    ports: &crate::construction::admission::PortIndex<'_>,
    counts: &mut Counts,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    for (boundaries, direction) in [
        (
            &interface.inputs,
            crate::construction::model::interface::Direction::Input,
        ),
        (
            &interface.outputs,
            crate::construction::model::interface::Direction::Output,
        ),
    ] {
        boundary_group(
            Group {
                boundaries,
                direction,
            },
            selected,
            ports,
            counts,
            limits,
        )?;
    }
    debug_assert!(
        counts.boundaries <= limits.max_boundaries,
        "admitted boundary count stays within its ceiling"
    );
    debug_assert!(
        counts.mapped <= limits.max_boundaries,
        "admitted mapped-port count stays within its ceiling"
    );
    Ok(())
}
fn boundary_group(
    group: Group<'_>,
    selected: &crate::construction::registry::Index<'_>,
    ports: &crate::construction::admission::PortIndex<'_>,
    counts: &mut Counts,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    let mut names = std::collections::BTreeSet::new();
    for boundary in group.boundaries {
        counts.take(boundary, limits)?;
        crate::construction::identifier(&boundary.name, limits)?;
        if !selected.types.contains_key(boundary.value_type.as_str())
            || !names.insert(&boundary.name)
        {
            return Err(crate::construction::Failure::InvalidReference);
        }
        boundary_ports(boundary, ports, group.direction)?;
    }
    Ok(())
}
fn boundary_ports(
    boundary: &crate::construction::model::projection::ExpandedBoundary,
    ports: &crate::construction::admission::PortIndex<'_>,
    direction: crate::construction::model::interface::Direction,
) -> Result<(), crate::construction::Failure> {
    let mut seen = std::collections::BTreeSet::new();
    for id in &boundary.ports {
        let (_, port) = ports
            .get(id)
            .ok_or(crate::construction::Failure::UnknownBoundary)?;
        if seen.len() >= boundary.ports.len() || !seen.insert(id) {
            return Err(crate::construction::Failure::DuplicateKey);
        }
        boundary_type(port, boundary, direction)?;
    }
    Ok(())
}
fn boundary_type(
    port: &crate::construction::model::projection::ExpandedPort,
    boundary: &crate::construction::model::projection::ExpandedBoundary,
    direction: crate::construction::model::interface::Direction,
) -> Result<(), crate::construction::Failure> {
    if port.direction != direction {
        return Err(crate::construction::Failure::WrongDirection);
    }
    if port.value_type != boundary.value_type {
        return Err(crate::construction::Failure::IncompatibleType);
    }
    Ok(())
}
