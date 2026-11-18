struct Site<'a> {
    fragment: &'a crate::construction::model::program::Fragment,
    path: &'a [String],
    template: &'a str,
}

impl crate::construction::expand::Expansion<'_> {
    pub(super) fn operation(
        &mut self,
        fragment: &crate::construction::model::program::Fragment,
        path: &[String],
        parameters: &crate::construction::expand::Parameters,
        template: &str,
    ) -> Result<crate::construction::expand::Open, crate::construction::Failure> {
        let crate::construction::model::program::FragmentKind::Operation {
            operation,
            configuration,
            effects,
        } = &fragment.kind
        else {
            return Err(crate::construction::Failure::InvalidSchema);
        };
        if self.nodes.len() >= self.limits.max_nodes {
            return Err(crate::construction::Failure::LimitExceeded);
        }
        let signature = *self
            .registry
            .operations
            .get(operation.as_str())
            .ok_or(crate::construction::Failure::UnknownOperation)?;
        self.reserve_operation(signature, configuration, effects)?;
        debug_assert!(self.nodes.len() < self.limits.max_nodes);
        let node_id = crate::construction::expand::scoped(path, "node")?;
        if node_id.len() > self.limits.max_identifier_bytes {
            return Err(crate::construction::Failure::LimitExceeded);
        }
        let mut node = crate::construction::expand::Node {
            scoped: node_id.clone(),
            operation: operation.to_owned(),
            implementation: signature.implementation.clone(),
            ports: Vec::with_capacity(
                signature
                    .inputs
                    .len()
                    .checked_add(signature.outputs.len())
                    .ok_or(crate::construction::Failure::LimitExceeded)?,
            ),
            invocation: signature.invocation.clone(),
            configuration: std::collections::BTreeMap::new(),
            effects: effects.to_vec(),
        };
        let site = Site {
            fragment,
            path,
            template,
        };
        let mut open = self.operation_ports(&mut node, signature, &site)?;
        self.configuration(&mut node, signature, parameters, &site)?;
        self.rules(&node, signature, &site)?;
        crate::construction::expand::attribution::origin(
            self,
            node_id.clone(),
            path,
            crate::construction::expand::OriginLabel {
                local: "node",
                template,
            },
            fragment.origin.as_ref(),
        )?;
        open.nodes.insert(node_id.clone());
        debug_assert_eq!(open.nodes.len(), 1);
        if self.nodes.insert(node_id, node).is_some() {
            return Err(crate::construction::Failure::DuplicateKey);
        }
        Ok(open)
    }

    fn reserve_operation(
        &mut self,
        signature: &crate::construction::model::host::OperationSignature,
        configuration: &std::collections::BTreeMap<
            String,
            crate::construction::model::term::Expression,
        >,
        effects: &[crate::construction::model::term::Effect],
    ) -> Result<(), crate::construction::Failure> {
        if configuration.len() != signature.configuration.len() {
            return Err(crate::construction::Failure::InvalidConfiguration);
        }
        crate::construction::expand::take(
            &mut self.effect_count,
            effects.len(),
            crate::construction::expand::Ceiling(self.limits.max_effects),
        )?;
        let remaining = self
            .limits
            .max_output_bytes
            .saturating_sub(self.expanded_bytes);
        let charge =
            crate::construction::bounded_json(&(signature, configuration, effects), remaining)?
                .len();
        crate::construction::expand::take(
            &mut self.expanded_bytes,
            charge,
            crate::construction::expand::Ceiling(self.limits.max_output_bytes),
        )?;
        crate::construction::expand::take(
            &mut self.boundaries,
            signature.invocation.len(),
            crate::construction::expand::Ceiling(self.limits.max_boundaries),
        )?;
        debug_assert!(self.nodes.len() < self.limits.max_nodes);
        debug_assert!(self.effect_count <= self.limits.max_effects);
        Ok(())
    }
    fn operation_ports(
        &mut self,
        node: &mut crate::construction::expand::Node,
        signature: &crate::construction::model::host::OperationSignature,
        site: &Site<'_>,
    ) -> Result<crate::construction::expand::Open, crate::construction::Failure> {
        let mut open = crate::construction::expand::Open {
            inputs: Vec::with_capacity(signature.inputs.len()),
            outputs: Vec::with_capacity(signature.outputs.len()),
            nodes: std::collections::BTreeSet::new(),
        };
        for role in &signature.invocation {
            crate::construction::expand::attribution::origin(
                self,
                crate::construction::expand::scoped(
                    site.path,
                    &format!("invocation/{}", role.name),
                )?,
                site.path,
                crate::construction::expand::OriginLabel {
                    local: &role.name,
                    template: site.template,
                },
                site.fragment.origin.as_ref(),
            )?;
        }
        for (direction, boundaries) in [
            (
                crate::construction::model::interface::Direction::Input,
                &signature.inputs,
            ),
            (
                crate::construction::model::interface::Direction::Output,
                &signature.outputs,
            ),
        ] {
            self.append_ports(node, &mut open, boundaries, direction, site)?;
        }
        debug_assert_eq!(
            node.ports.len(),
            signature
                .inputs
                .len()
                .saturating_add(signature.outputs.len())
        );
        debug_assert_eq!(open.inputs.len(), signature.inputs.len());
        Ok(open)
    }

    fn append_ports(
        &mut self,
        node: &mut crate::construction::expand::Node,
        open: &mut crate::construction::expand::Open,
        boundaries: &[crate::construction::model::interface::Boundary],
        direction: crate::construction::model::interface::Direction,
        site: &Site<'_>,
    ) -> Result<(), crate::construction::Failure> {
        for boundary in boundaries {
            crate::construction::expand::take(
                &mut self.boundaries,
                1,
                crate::construction::expand::Ceiling(self.limits.max_boundaries),
            )?;
            let name = crate::construction::expand::port_name(direction, &boundary.name)?;
            let endpoint = crate::construction::expand::attribution::endpoint(
                self,
                site.path,
                crate::construction::expand::EndpointLabel {
                    local: &name,
                    value_type: &boundary.value_type,
                },
                Some((node.scoped.clone(), direction, boundary.name.clone())),
            )?;
            crate::construction::expand::attribution::origin(
                self,
                endpoint.clone(),
                site.path,
                crate::construction::expand::OriginLabel {
                    local: &name,
                    template: site.template,
                },
                site.fragment.origin.as_ref(),
            )?;
            node.ports.push((
                boundary.name.clone(),
                direction,
                boundary.value_type.clone(),
                endpoint.clone(),
            ));
            let port = crate::construction::expand::Port {
                name: boundary.name.clone(),
                value_type: boundary.value_type.clone(),
                endpoint,
            };
            if direction == crate::construction::model::interface::Direction::Input {
                open.inputs.push(port);
            } else {
                open.outputs.push(port);
            }
        }
        debug_assert!(node.ports.len() <= self.endpoints.len());
        debug_assert!(self.boundaries <= self.limits.max_boundaries);
        Ok(())
    }

    fn configuration(
        &mut self,
        node: &mut crate::construction::expand::Node,
        signature: &crate::construction::model::host::OperationSignature,
        parameters: &crate::construction::expand::Parameters,
        site: &Site<'_>,
    ) -> Result<(), crate::construction::Failure> {
        let crate::construction::model::program::FragmentKind::Operation { configuration, .. } =
            &site.fragment.kind
        else {
            return Err(crate::construction::Failure::InvalidSchema);
        };
        for field in &signature.configuration {
            let expression = configuration
                .get(&field.name)
                .ok_or(crate::construction::Failure::InvalidConfiguration)?;
            node.configuration.insert(
                field.name.clone(),
                crate::construction::expand::terms::expression(self, expression, parameters)?,
            );
        }
        debug_assert_eq!(node.configuration.len(), signature.configuration.len());
        debug_assert!(
            signature
                .configuration
                .iter()
                .all(|field| node.configuration.contains_key(&field.name))
        );
        for field in &signature.configuration {
            crate::construction::expand::terms::check_expression(
                self,
                node,
                &node.configuration[&field.name],
                field,
            )?;
            crate::construction::expand::attribution::origin(
                self,
                crate::construction::expand::scoped(site.path, &format!("config/{}", field.name))?,
                site.path,
                crate::construction::expand::OriginLabel {
                    local: &field.name,
                    template: site.template,
                },
                site.fragment.origin.as_ref(),
            )?;
        }
        self.configuration_order(node)
    }

    fn rules(
        &mut self,
        node: &crate::construction::expand::Node,
        signature: &crate::construction::model::host::OperationSignature,
        site: &Site<'_>,
    ) -> Result<(), crate::construction::Failure> {
        let effects = &node.effects;
        let expected = signature
            .effects
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        let mut actual = std::collections::BTreeSet::new();
        let mut keys = std::collections::BTreeSet::new();
        for effect in effects {
            crate::construction::identifier(&effect.key, self.limits)?;
            if !keys.insert(&effect.key) {
                return Err(crate::construction::Failure::DuplicateKey);
            }
            actual.insert(crate::construction::model::host::EffectRule {
                from_state: effect.from_state.clone(),
                action: effect.action.clone(),
                resource: effect.resource.clone(),
                ability: effect.ability.clone(),
            });
            crate::construction::expand::terms::check_binding(self, node, &effect.request)?;
            crate::construction::expand::terms::check_binding(self, node, &effect.payload)?;
            crate::construction::expand::attribution::origin(
                self,
                crate::construction::expand::scoped(site.path, &format!("effect/{}", effect.key))?,
                site.path,
                crate::construction::expand::OriginLabel {
                    local: &effect.key,
                    template: site.template,
                },
                site.fragment.origin.as_ref(),
            )?;
            for binding in [&effect.request, &effect.payload] {
                if let crate::construction::model::term::Binding::Slot { slot, source } = binding {
                    crate::construction::expand::attribution::origin(
                        self,
                        crate::construction::expand::scoped(site.path, &format!("slot/{slot}"))?,
                        site.path,
                        crate::construction::expand::OriginLabel {
                            local: slot,
                            template: site.template,
                        },
                        site.fragment.origin.as_ref(),
                    )?;
                    crate::construction::expand::attribution::origin(
                        self,
                        crate::construction::expand::scoped(
                            site.path,
                            &format!("reference/{slot}"),
                        )?,
                        site.path,
                        crate::construction::expand::OriginLabel {
                            local: &source.name,
                            template: site.template,
                        },
                        site.fragment.origin.as_ref(),
                    )?;
                }
            }
        }
        if effects.len() != expected.len()
            || actual.iter().collect::<std::collections::BTreeSet<_>>() != expected
        {
            return Err(crate::construction::Failure::InvalidEffect);
        }
        debug_assert_eq!(actual.len(), expected.len());
        debug_assert_eq!(keys.len(), effects.len());
        Ok(())
    }

    fn configuration_order(
        &self,
        node: &crate::construction::expand::Node,
    ) -> Result<(), crate::construction::Failure> {
        let mut pending = node
            .configuration
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        let mut resolved = std::collections::BTreeSet::new();
        let mut ready = Vec::with_capacity(pending.len());
        while !pending.is_empty() {
            ready.clear();
            debug_assert!(pending.is_disjoint(&resolved));
            for name in &pending {
                if crate::construction::expand::terms::references_resolved(
                    self,
                    &node.configuration[*name],
                    &resolved,
                )? {
                    ready.push(*name);
                }
            }
            if ready.is_empty() {
                return Err(crate::construction::Failure::InvalidReference);
            }
            for name in &ready {
                pending.remove(name);
                if resolved.len() >= node.configuration.len() {
                    return Err(crate::construction::Failure::LimitExceeded);
                }
                resolved.insert(*name);
            }
        }
        debug_assert_eq!(resolved.len(), node.configuration.len());
        Ok(())
    }
}
