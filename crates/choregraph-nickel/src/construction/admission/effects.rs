//! Concrete effect declarations and registry-selected usage rules.

type Rules = std::collections::BTreeMap<
    u64,
    std::collections::BTreeSet<crate::construction::model::host::EffectRule>,
>;
struct Declarations<'a> {
    keys: std::collections::BTreeSet<&'a str>,
    rules: Rules,
}
pub(super) fn lower(
    projection: &crate::construction::model::projection::LatticeV3,
    indexes: &crate::construction::admission::Indexes<'_>,
    uses: &mut std::collections::BTreeMap<u64, usize>,
    spec: &mut choregraph_core::concrete::machine::MachineSpecV2,
    limits: crate::construction::model::profile::Limits,
) -> Result<Rules, crate::construction::Failure> {
    let mut declarations = Declarations::new();
    spec.effects.reserve(projection.effects.len());
    for effect in &projection.effects {
        let node = *indexes
            .nodes
            .get(&effect.node_id)
            .ok_or(crate::construction::Failure::InvalidEffect)?;
        crate::construction::identifier(&effect.declaration_key, limits)?;
        declarations.register(effect, node)?;
        spec.effects.push(declaration(effect, node, uses, limits)?);
    }
    Ok(declarations.rules)
}
impl<'a> Declarations<'a> {
    const fn new() -> Self {
        Self {
            keys: std::collections::BTreeSet::new(),
            rules: std::collections::BTreeMap::new(),
        }
    }
    fn register(
        &mut self,
        effect: &'a crate::construction::model::projection::ExpandedEffect,
        node: &crate::construction::model::projection::ExpandedNode,
    ) -> Result<(), crate::construction::Failure> {
        if !effect
            .declaration_key
            .starts_with(&format!("{}/", node.scoped_id))
            || !self.keys.insert(&effect.declaration_key)
        {
            return Err(crate::construction::Failure::InvalidEffect);
        }
        let rule = effect_rule(effect);
        if !self.rules.entry(node.id).or_default().insert(rule) {
            return Err(crate::construction::Failure::InvalidEffect);
        }
        Ok(())
    }
}
fn effect_rule(
    effect: &crate::construction::model::projection::ExpandedEffect,
) -> crate::construction::model::host::EffectRule {
    crate::construction::model::host::EffectRule {
        from_state: effect.from_state.clone(),
        action: effect.action.clone(),
        resource: effect.resource.clone(),
        ability: effect.ability.clone(),
    }
}
fn declaration(
    effect: &crate::construction::model::projection::ExpandedEffect,
    node: &crate::construction::model::projection::ExpandedNode,
    uses: &mut std::collections::BTreeMap<u64, usize>,
    limits: crate::construction::model::profile::Limits,
) -> Result<
    choregraph_core::concrete::effect::ConcreteEffectDeclarationSpec,
    crate::construction::Failure,
> {
    Ok(
        choregraph_core::concrete::effect::ConcreteEffectDeclarationSpec {
            declaration_key: effect.declaration_key.clone(),
            node_id: format!("lattice-node/{}", node.id),
            transition: choregraph_core::model::semantics::TransitionSelector {
                from: state(&effect.from_state)?,
                action: action(&effect.action)?,
            },
            operation: format!("lattice/{}", node.operation),
            resource: effect.resource.clone(),
            ability: effect.ability.clone(),
            request_binding: binding(&effect.request_binding, node, uses, limits)?,
            payload_binding: binding(&effect.payload_binding, node, uses, limits)?,
        },
    )
}
const fn state(
    from: &str,
) -> Result<choregraph_core::model::semantics::LocalState, crate::construction::Failure> {
    match from.as_bytes() {
        b"pending" => Ok(choregraph_core::model::semantics::LocalState::Pending),
        b"ready" => Ok(choregraph_core::model::semantics::LocalState::Ready),
        b"running" => Ok(choregraph_core::model::semantics::LocalState::Running),
        b"completed" => Ok(choregraph_core::model::semantics::LocalState::Completed),
        b"failed" => Ok(choregraph_core::model::semantics::LocalState::Failed),
        b"cancelled" => Ok(choregraph_core::model::semantics::LocalState::Cancelled),
        _ => Err(crate::construction::Failure::InvalidEffect),
    }
}
const fn action(
    action: &str,
) -> Result<choregraph_core::model::semantics::Action, crate::construction::Failure> {
    match action.as_bytes() {
        b"mark-ready" => Ok(choregraph_core::model::semantics::Action::MarkReady),
        b"start" => Ok(choregraph_core::model::semantics::Action::Start),
        b"complete" => Ok(choregraph_core::model::semantics::Action::Complete),
        b"fail" => Ok(choregraph_core::model::semantics::Action::Fail),
        b"cancel" => Ok(choregraph_core::model::semantics::Action::Cancel),
        _ => Err(crate::construction::Failure::InvalidEffect),
    }
}
fn binding(
    input: &crate::construction::model::projection::ExpandedBinding,
    node: &crate::construction::model::projection::ExpandedNode,
    uses: &mut std::collections::BTreeMap<u64, usize>,
    limits: crate::construction::model::profile::Limits,
) -> Result<choregraph_core::concrete::reference::ReferenceBindingSpec, crate::construction::Failure>
{
    if let crate::construction::model::projection::ExpandedBinding::Exact { reference } = input {
        Ok(
            choregraph_core::concrete::reference::ReferenceBindingSpec::Exact {
                reference: choregraph_core::concrete::reference::ConcreteReferenceSpec {
                    profile: reference.profile.clone(),
                    identity: reference.identity.clone(),
                },
            },
        )
    } else if let crate::construction::model::projection::ExpandedBinding::Slot {
        slot_id,
        source_ref,
    } = input
    {
        slot(
            Slot {
                slot_id,
                source_ref,
            },
            node,
            uses,
            limits,
        )
    } else {
        Err(crate::construction::Failure::InvalidReference)
    }
}
struct Slot<'a> {
    slot_id: &'a str,
    source_ref: &'a str,
}
fn slot(
    input: Slot<'_>,
    node: &crate::construction::model::projection::ExpandedNode,
    uses: &mut std::collections::BTreeMap<u64, usize>,
    limits: crate::construction::model::profile::Limits,
) -> Result<choregraph_core::concrete::reference::ReferenceBindingSpec, crate::construction::Failure>
{
    crate::construction::identifier(input.slot_id, limits)?;
    crate::construction::identifier(input.source_ref, limits)?;
    if !input.slot_id.starts_with(&format!("{}/", node.scoped_id)) {
        return Err(crate::construction::Failure::InvalidReference);
    }
    let local = input
        .source_ref
        .strip_prefix(&format!("node/{}/", node.id))
        .ok_or(crate::construction::Failure::InvalidReference)?;
    source(local, node, uses)?;
    Ok(
        choregraph_core::concrete::reference::ReferenceBindingSpec::Slot {
            slot_id: input.slot_id.to_owned(),
            source_ref: input.source_ref.to_owned(),
        },
    )
}
fn source(
    local: &str,
    node: &crate::construction::model::projection::ExpandedNode,
    uses: &mut std::collections::BTreeMap<u64, usize>,
) -> Result<(), crate::construction::Failure> {
    let (direction, name) = local
        .split_once('/')
        .ok_or(crate::construction::Failure::InvalidReference)?;
    let port_direction = match direction {
        "config" if node.configuration.contains_key(name) => return Ok(()),
        "invocation" if node.invocation.iter().any(|role| role.name == name) => return Ok(()),
        "input" => crate::construction::model::interface::Direction::Input,
        "output" => crate::construction::model::interface::Direction::Output,
        _ => return Err(crate::construction::Failure::InvalidReference),
    };
    source_port(port_direction, name, node, uses)
}
fn source_port(
    direction: crate::construction::model::interface::Direction,
    name: &str,
    node: &crate::construction::model::projection::ExpandedNode,
    uses: &mut std::collections::BTreeMap<u64, usize>,
) -> Result<(), crate::construction::Failure> {
    let port = node
        .ports
        .iter()
        .find(|port| port.direction == direction && port.name == name)
        .ok_or(crate::construction::Failure::InvalidReference)?;
    if direction == crate::construction::model::interface::Direction::Output {
        crate::construction::admission::count_use(uses, port.id)?;
    }
    Ok(())
}
pub(super) fn check_usage(
    projection: &crate::construction::model::projection::LatticeV3,
    selected: &crate::construction::registry::Index<'_>,
    mut rules: Rules,
    links: &crate::construction::admission::links::LinkIndex,
    uses: &std::collections::BTreeMap<u64, usize>,
) -> Result<(), crate::construction::Failure> {
    for node in &projection.nodes {
        let signature = selected.operations[node.operation.as_str()];
        let actual = rules.remove(&node.id).unwrap_or_default();
        if actual != signature.effects.iter().cloned().collect() {
            return Err(crate::construction::Failure::InvalidEffect);
        }
        for port in &node.ports {
            port_usage(port, selected, links, uses)?;
        }
    }
    Ok(())
}
fn port_usage(
    port: &crate::construction::model::projection::ExpandedPort,
    selected: &crate::construction::registry::Index<'_>,
    links: &crate::construction::admission::links::LinkIndex,
    uses: &std::collections::BTreeMap<u64, usize>,
) -> Result<(), crate::construction::Failure> {
    if port.direction == crate::construction::model::interface::Direction::Input {
        if links.inputs.get(&port.id).copied().unwrap_or(0) != 1 {
            return Err(crate::construction::Failure::UsageViolation);
        }
    } else {
        output_usage(port, selected, uses)?;
    }
    Ok(())
}
fn output_usage(
    port: &crate::construction::model::projection::ExpandedPort,
    selected: &crate::construction::registry::Index<'_>,
    uses: &std::collections::BTreeMap<u64, usize>,
) -> Result<(), crate::construction::Failure> {
    let count = uses.get(&port.id).copied().unwrap_or(0);
    let ty = selected.types[port.value_type.as_str()];
    if count == 0 && !ty.droppable {
        return Err(crate::construction::Failure::UsageViolation);
    }
    if count > 1 && !ty.copyable {
        return Err(crate::construction::Failure::UsageViolation);
    }
    Ok(())
}
