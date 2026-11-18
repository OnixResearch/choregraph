type Ids = std::collections::BTreeMap<String, u64>;
type Materialized = (
    Vec<crate::construction::model::projection::ExpandedNode>,
    Vec<crate::construction::model::projection::ExpandedEffect>,
);

impl crate::construction::expand::Expansion<'_> {
    pub(in crate::construction) fn finish(
        mut self,
    ) -> Result<
        (
            crate::construction::model::projection::LatticeV3,
            Vec<crate::construction::model::projection::Origin>,
        ),
        crate::construction::Failure,
    > {
        crate::construction::bounded_json(
            &self.candidate.parameters,
            self.limits.max_parameter_bytes,
        )?;
        let root = self.fragment()?;
        if !root.inputs.is_empty() || !root.outputs.is_empty() {
            return Err(crate::construction::Failure::OpenBoundary);
        }
        debug_assert!(root.inputs.is_empty() && root.outputs.is_empty());
        let routes = crate::construction::expand::routing::data_routes(&self)?;
        self.usage(&routes)?;
        self.acyclic()?;
        let (node_ids, port_ids) = self.identities()?;
        let (nodes, mut effects) = self.materialize(&node_ids, &port_ids)?;
        debug_assert_eq!(effects.len(), self.effect_count);
        let edges = routes
            .into_iter()
            .map(
                |(source, target)| crate::construction::model::projection::DataEdge {
                    source: port_ids[source],
                    target: port_ids[target],
                },
            )
            .collect();
        let control_edges = self
            .controls
            .iter()
            .map(
                |(source, target)| crate::construction::model::projection::ControlEdge {
                    source: node_ids[source],
                    target: node_ids[target],
                },
            )
            .collect();
        self.rewrite_origins(&node_ids, &port_ids)?;
        effects.sort_by(|left, right| left.declaration_key.cmp(&right.declaration_key));
        self.origins.sort_by(|left, right| {
            left.element
                .cmp(&right.element)
                .then(left.local.cmp(&right.local))
        });
        let interfaces =
            crate::construction::expand::routing::expanded_interfaces(&self, &port_ids)?;
        Ok((
            crate::construction::model::projection::LatticeV3 {
                schema: crate::construction::model::profile::PROJECTION_SCHEMA.to_owned(),
                target: self.candidate.target.clone(),
                registry: self.candidate.registry.clone(),
                nodes,
                edges,
                control_edges,
                effects,
                interfaces,
            },
            self.origins,
        ))
    }

    fn usage(
        &self,
        routes: &std::collections::BTreeSet<(&str, &str)>,
    ) -> Result<(), crate::construction::Failure> {
        debug_assert!(routes.len() <= self.limits.max_edges);
        let mut inputs = std::collections::BTreeMap::<&str, usize>::new();
        let mut outputs = std::collections::BTreeMap::<&str, usize>::new();
        for (source, target) in routes {
            crate::construction::expand::take(
                outputs.entry(*source).or_default(),
                1,
                crate::construction::expand::Ceiling(usize::MAX),
            )?;
            crate::construction::expand::take(
                inputs.entry(*target).or_default(),
                1,
                crate::construction::expand::Ceiling(usize::MAX),
            )?;
        }
        for (endpoint, uses) in &self.config_uses {
            crate::construction::expand::take(
                outputs.entry(endpoint.as_str()).or_default(),
                *uses,
                crate::construction::expand::Ceiling(usize::MAX),
            )?;
        }
        debug_assert!(inputs.len() <= self.endpoints.len());
        for (id, endpoint) in &self.endpoints {
            if let Some((_, direction, _)) = &endpoint.real {
                if *direction == crate::construction::model::interface::Direction::Input {
                    if inputs.get(id.as_str()).copied().unwrap_or(0) != 1 {
                        return Err(crate::construction::Failure::UsageViolation);
                    }
                } else {
                    let count = outputs.get(id.as_str()).copied().unwrap_or(0);
                    let ty = self
                        .registry
                        .types
                        .get(endpoint.value_type.as_str())
                        .ok_or(crate::construction::Failure::IncompatibleType)?;
                    if (count == 0 && !ty.droppable) || (count > 1 && !ty.copyable) {
                        return Err(crate::construction::Failure::UsageViolation);
                    }
                }
            }
        }
        Ok(())
    }

    fn acyclic(&self) -> Result<(), crate::construction::Failure> {
        // Include every symbolic junction, even components disconnected from real ports.
        let mut indegree = self
            .endpoints
            .keys()
            .map(|endpoint| (endpoint.as_str(), 0usize))
            .collect::<std::collections::BTreeMap<_, _>>();
        debug_assert!(indegree.len() <= self.limits.max_ports);
        let mut outgoing = std::collections::BTreeMap::<&str, Vec<&str>>::new();
        for (source, target) in &self.arcs {
            let degree = indegree
                .get_mut(target.as_str())
                .ok_or(crate::construction::Failure::UnknownBoundary)?;
            *degree = degree
                .checked_add(1)
                .ok_or(crate::construction::Failure::LimitExceeded)?;
            outgoing
                .entry(source.as_str())
                .or_default()
                .push(target.as_str());
        }
        let mut ready = indegree
            .iter()
            .filter_map(|(endpoint, degree)| (*degree == 0).then_some(*endpoint))
            .collect::<std::collections::BTreeSet<_>>();
        let mut visited = 0usize;
        while let Some(endpoint) = ready.pop_first() {
            visited = visited
                .checked_add(1)
                .ok_or(crate::construction::Failure::LimitExceeded)?;
            if let Some(next) = outgoing.get(endpoint) {
                for target in next {
                    let degree = indegree
                        .get_mut(*target)
                        .ok_or(crate::construction::Failure::UnknownBoundary)?;
                    *degree = degree
                        .checked_sub(1)
                        .ok_or(crate::construction::Failure::AdmissionRejected)?;
                    if *degree == 0 {
                        ready.insert(*target);
                    }
                }
            }
        }
        debug_assert!(visited <= indegree.len());
        if visited != indegree.len() {
            return Err(crate::construction::Failure::AdmissionRejected);
        }
        Ok(())
    }

    fn identities(&self) -> Result<(Ids, Ids), crate::construction::Failure> {
        let node_ids = self
            .nodes
            .keys()
            .enumerate()
            .map(|(index, key)| {
                Ok((
                    key.clone(),
                    u64::try_from(crate::construction::expand::deeper(index)?)
                        .map_err(|_| crate::construction::Failure::LimitExceeded)?,
                ))
            })
            .collect::<Result<std::collections::BTreeMap<_, _>, crate::construction::Failure>>()?;
        let port_ids = self
            .endpoints
            .iter()
            .filter(|(_, endpoint)| endpoint.real.is_some())
            .enumerate()
            .map(|(index, (key, _))| {
                Ok((
                    key.clone(),
                    u64::try_from(crate::construction::expand::deeper(index)?)
                        .map_err(|_| crate::construction::Failure::LimitExceeded)?,
                ))
            })
            .collect::<Result<std::collections::BTreeMap<_, _>, crate::construction::Failure>>()?;
        debug_assert!(
            node_ids.len() <= self.limits.max_nodes && port_ids.len() <= self.limits.max_ports
        );
        debug_assert_eq!(node_ids.len(), self.nodes.len());
        Ok((node_ids, port_ids))
    }

    fn materialize(
        &self,
        node_ids: &std::collections::BTreeMap<String, u64>,
        port_ids: &std::collections::BTreeMap<String, u64>,
    ) -> Result<Materialized, crate::construction::Failure> {
        let mut nodes = Vec::with_capacity(self.nodes.len());
        debug_assert!(self.nodes.len() <= self.limits.max_nodes);
        let mut effects = Vec::with_capacity(self.effect_count);
        for node in self.nodes.values() {
            let id = node_ids[&node.scoped];
            let ports = node
                .ports
                .iter()
                .map(|(name, direction, value_type, endpoint)| {
                    crate::construction::model::projection::ExpandedPort {
                        id: port_ids[endpoint],
                        name: name.clone(),
                        direction: *direction,
                        value_type: value_type.clone(),
                    }
                })
                .collect();
            let configuration = node
                .configuration
                .iter()
                .map(|(name, expression)| {
                    Ok((
                        name.clone(),
                        crate::construction::expand::terms::expanded_expression(
                            self, node, expression, node_ids, port_ids,
                        )?,
                    ))
                })
                .collect::<Result<_, crate::construction::Failure>>()?;
            nodes.push(crate::construction::model::projection::ExpandedNode {
                id,
                scoped_id: node.scoped.clone(),
                operation: node.operation.clone(),
                implementation: node.implementation.clone(),
                ports,
                invocation: node.invocation.clone(),
                configuration,
            });
            for effect in &node.effects {
                let declaration_key = format!("{}/{}", node.scoped, effect.key);
                crate::construction::identifier(&declaration_key, self.limits)?;
                effects.push(crate::construction::model::projection::ExpandedEffect {
                    declaration_key,
                    node_id: id,
                    from_state: effect.from_state.clone(),
                    action: effect.action.clone(),
                    resource: effect.resource.clone(),
                    ability: effect.ability.clone(),
                    request_binding: crate::construction::expand::terms::expanded_binding(
                        self,
                        node,
                        &effect.request,
                        id,
                    )?,
                    payload_binding: crate::construction::expand::terms::expanded_binding(
                        self,
                        node,
                        &effect.payload,
                        id,
                    )?,
                });
            }
        }
        debug_assert!(
            nodes.len() <= self.limits.max_nodes && effects.len() <= self.limits.max_effects
        );
        Ok((nodes, effects))
    }

    fn rewrite_origins(
        &mut self,
        node_ids: &std::collections::BTreeMap<String, u64>,
        port_ids: &std::collections::BTreeMap<String, u64>,
    ) -> Result<(), crate::construction::Failure> {
        debug_assert!(node_ids.len() <= self.limits.max_nodes);
        debug_assert!(port_ids.len() <= self.limits.max_ports);
        for origin in &mut self.origins {
            if let Some(id) = node_ids.get(&origin.element) {
                origin.element = format!("node/{id}");
            } else if let Some(id) = port_ids.get(&origin.element) {
                origin.element = format!("port/{id}");
            } else {
                let owner = crate::construction::expand::scoped(&origin.instance_path, "node")?;
                if let Some(id) = node_ids.get(&owner) {
                    if origin.element
                        == crate::construction::expand::scoped(
                            &origin.instance_path,
                            &format!("config/{}", origin.local),
                        )?
                    {
                        origin.element = format!("node/{id}/config/{}", origin.local);
                    } else if origin.element
                        == crate::construction::expand::scoped(
                            &origin.instance_path,
                            &format!("invocation/{}", origin.local),
                        )?
                    {
                        origin.element = format!("node/{id}/invocation/{}", origin.local);
                    } else if origin.element
                        == crate::construction::expand::scoped(
                            &origin.instance_path,
                            &format!("effect/{}", origin.local),
                        )?
                    {
                        origin.element = format!("effect/{owner}/{}", origin.local);
                    } else if origin.element
                        == crate::construction::expand::scoped(
                            &origin.instance_path,
                            &format!("slot/{}", origin.local),
                        )?
                    {
                        origin.element = format!("slot/{owner}/{}", origin.local);
                    }
                }
            }
        }
        Ok(())
    }
}
