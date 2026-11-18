struct Frame<'a, T> {
    operator: &'a str,
    remaining: std::slice::Iter<'a, crate::construction::model::term::Expression>,
    values: Vec<T>,
}

impl<'a, T> Frame<'a, T> {
    fn new(
        operator: &'a str,
        arguments: &'a [crate::construction::model::term::Expression],
    ) -> Self {
        Self {
            operator,
            remaining: arguments.iter(),
            values: Vec::with_capacity(arguments.len()),
        }
    }
}

// One continuation per open apply; siblings remain borrowed, never queued or copied.
fn rewrite<T>(
    input: &crate::construction::model::term::Expression,
    limits: crate::construction::model::profile::Limits,
    mut leaf: impl FnMut(
        &crate::construction::model::term::Expression,
    ) -> Result<T, crate::construction::Failure>,
    apply: impl Fn(&str, Vec<T>) -> T,
) -> Result<T, crate::construction::Failure> {
    let mut frames: Vec<Frame<'_, T>> = Vec::with_capacity(limits.max_depth);
    let mut current = input;
    loop {
        debug_assert!(frames.len() <= limits.max_depth);
        let mut value = if let crate::construction::model::term::Expression::Apply {
            operator,
            arguments,
        } = current
        {
            if arguments.len() > limits.max_boundaries {
                return Err(crate::construction::Failure::LimitExceeded);
            }
            if !arguments.is_empty() && frames.len() >= limits.max_depth {
                return Err(crate::construction::Failure::LimitExceeded);
            }
            let mut frame = Frame::new(operator, arguments);
            if let Some(first) = frame.remaining.next() {
                frames.push(frame);
                current = first;
                continue;
            }
            apply(operator, frame.values)
        } else {
            leaf(current)?
        };
        loop {
            let Some(frame) = frames.last_mut() else {
                return Ok(value);
            };
            debug_assert!(frame.values.len() < frame.values.capacity());
            frame.values.push(value);
            if let Some(next) = frame.remaining.next() {
                current = next;
                break;
            }
            let frame = frames
                .pop()
                .ok_or(crate::construction::Failure::InvalidSchema)?;
            value = apply(frame.operator, frame.values);
        }
    }
}

pub(super) fn expression(
    expansion: &mut crate::construction::expand::Expansion<'_>,
    input: &crate::construction::model::term::Expression,
    parameters: &crate::construction::expand::Parameters,
) -> Result<crate::construction::model::term::Expression, crate::construction::Failure> {
    rewrite(
        input,
        expansion.limits,
        |expression| {
            let value = if let crate::construction::model::term::Expression::Literal { value } =
                expression
            {
                value
            } else if let crate::construction::model::term::Expression::Parameter { name } =
                expression
            {
                crate::construction::identifier(name, expansion.limits)?;
                parameters
                    .get(name)
                    .ok_or(crate::construction::Failure::InvalidParameters)?
            } else if let crate::construction::model::term::Expression::Reference { reference } =
                expression
            {
                return Ok(crate::construction::model::term::Expression::Reference {
                    reference: reference.clone(),
                });
            } else {
                return Err(crate::construction::Failure::InvalidSchema);
            };
            let bytes =
                crate::construction::bounded_json(value, expansion.limits.max_parameter_bytes)?
                    .len();
            crate::construction::expand::take(
                &mut expansion.expanded_bytes,
                bytes,
                crate::construction::expand::Ceiling(expansion.limits.max_output_bytes),
            )?;
            debug_assert!(expansion.expanded_bytes <= expansion.limits.max_output_bytes);
            Ok(crate::construction::model::term::Expression::Literal {
                value: value.clone(),
            })
        },
        |operator, arguments| crate::construction::model::term::Expression::Apply {
            operator: operator.to_owned(),
            arguments,
        },
    )
}
pub(super) fn local<'b>(
    expansion: &crate::construction::expand::Expansion<'_>,
    node: &'b crate::construction::expand::Node,
    reference: &crate::construction::model::term::LocalReference,
) -> Result<Option<&'b str>, crate::construction::Failure> {
    crate::construction::identifier(&reference.name, expansion.limits)?;
    debug_assert!(reference.name.len() <= expansion.limits.max_identifier_bytes);
    debug_assert!(node.ports.len() <= expansion.endpoints.len());
    if reference.direction == crate::construction::model::term::ReferenceDirection::Invocation {
        if !node
            .invocation
            .iter()
            .any(|role| role.name == reference.name)
        {
            return Err(crate::construction::Failure::InvalidReference);
        }
        Ok(None)
    } else if reference.direction == crate::construction::model::term::ReferenceDirection::Config {
        if !node.configuration.contains_key(&reference.name) {
            return Err(crate::construction::Failure::InvalidReference);
        }
        Ok(None)
    } else if matches!(
        reference.direction,
        crate::construction::model::term::ReferenceDirection::Input
            | crate::construction::model::term::ReferenceDirection::Output
    ) {
        let direction =
            if reference.direction == crate::construction::model::term::ReferenceDirection::Input {
                crate::construction::model::interface::Direction::Input
            } else {
                crate::construction::model::interface::Direction::Output
            };
        node.ports
            .iter()
            .find(|(name, actual, _, _)| name == &reference.name && *actual == direction)
            .map(|(_, _, _, endpoint)| Some(endpoint.as_str()))
            .ok_or(crate::construction::Failure::InvalidReference)
    } else {
        Err(crate::construction::Failure::InvalidReference)
    }
}
pub(super) fn check_expression(
    expansion: &mut crate::construction::expand::Expansion<'_>,
    node: &crate::construction::expand::Node,
    expression: &crate::construction::model::term::Expression,
    field: &crate::construction::model::host::ConfigurationField,
) -> Result<(), crate::construction::Failure> {
    let mut stack = Vec::with_capacity(expansion.limits.max_depth);
    let mut current = Some(expression);
    while let Some(expression) = current {
        debug_assert!(stack.len() <= expansion.limits.max_depth);
        if let crate::construction::model::term::Expression::Literal { value } = expression {
            if !crate::construction::registry::literal_type(value, &field.value_type) {
                return Err(crate::construction::Failure::InvalidConfiguration);
            }
        } else if let crate::construction::model::term::Expression::Parameter { .. } = expression {
            return Err(crate::construction::Failure::InvalidParameters);
        } else if let crate::construction::model::term::Expression::Reference { reference } =
            expression
        {
            if !field.references {
                return Err(crate::construction::Failure::InvalidReference);
            }
            if let Some(endpoint) = local(expansion, node, reference)?
                && reference.direction
                    == crate::construction::model::term::ReferenceDirection::Output
            {
                record_use(expansion, endpoint)?;
            }
        } else if let crate::construction::model::term::Expression::Apply {
            operator,
            arguments,
        } = expression
        {
            if !field.operators.contains(operator) {
                return Err(crate::construction::Failure::InvalidConfiguration);
            }
            if !arguments.is_empty() {
                if stack.len() >= expansion.limits.max_depth {
                    return Err(crate::construction::Failure::LimitExceeded);
                }
                stack.push(arguments.iter());
            }
        } else {
            return Err(crate::construction::Failure::InvalidSchema);
        }
        current = next_expression(&mut stack);
    }
    debug_assert!(expansion.config_uses.len() <= expansion.endpoints.len());
    Ok(())
}
pub(super) fn expanded_expression(
    expansion: &crate::construction::expand::Expansion<'_>,
    node: &crate::construction::expand::Node,
    input: &crate::construction::model::term::Expression,
    node_ids: &std::collections::BTreeMap<String, u64>,
    port_ids: &std::collections::BTreeMap<String, u64>,
) -> Result<crate::construction::model::projection::ExpandedExpression, crate::construction::Failure>
{
    rewrite(
        input,
        expansion.limits,
        |expression| {
            if let crate::construction::model::term::Expression::Literal { value } = expression {
                Ok(
                    crate::construction::model::projection::ExpandedExpression::Literal {
                        value: value.clone(),
                    },
                )
            } else if let crate::construction::model::term::Expression::Reference { reference } =
                expression
            {
                Ok(
                    crate::construction::model::projection::ExpandedExpression::Reference {
                        node: node_ids[&node.scoped],
                        direction: reference.direction,
                        name: reference.name.clone(),
                        port: local(expansion, node, reference)?.map(|endpoint| port_ids[endpoint]),
                    },
                )
            } else {
                Err(crate::construction::Failure::InvalidParameters)
            }
        },
        |operator, arguments| crate::construction::model::projection::ExpandedExpression::Apply {
            operator: operator.to_owned(),
            arguments,
        },
    )
}
pub(super) fn check_binding(
    expansion: &mut crate::construction::expand::Expansion<'_>,
    node: &crate::construction::expand::Node,
    binding: &crate::construction::model::term::Binding,
) -> Result<(), crate::construction::Failure> {
    if let crate::construction::model::term::Binding::Slot { slot, source } = binding {
        crate::construction::identifier(slot, expansion.limits)?;
        if let Some(endpoint) = local(expansion, node, source)?
            && source.direction == crate::construction::model::term::ReferenceDirection::Output
        {
            record_use(expansion, endpoint)?;
        }
    }
    Ok(())
}
pub(super) fn expanded_binding(
    expansion: &crate::construction::expand::Expansion<'_>,
    node: &crate::construction::expand::Node,
    binding: &crate::construction::model::term::Binding,
    node_id: u64,
) -> Result<crate::construction::model::projection::ExpandedBinding, crate::construction::Failure> {
    if let crate::construction::model::term::Binding::Exact { reference } = binding {
        return Ok(
            crate::construction::model::projection::ExpandedBinding::Exact {
                reference: reference.clone(),
            },
        );
    }
    let crate::construction::model::term::Binding::Slot { slot, source } = binding else {
        return Err(crate::construction::Failure::InvalidReference);
    };
    local(expansion, node, source)?;
    debug_assert!(expansion.nodes.contains_key(&node.scoped));
    let direction = if source.direction
        == crate::construction::model::term::ReferenceDirection::Input
    {
        "input"
    } else if source.direction == crate::construction::model::term::ReferenceDirection::Output {
        "output"
    } else if source.direction == crate::construction::model::term::ReferenceDirection::Config {
        "config"
    } else if source.direction == crate::construction::model::term::ReferenceDirection::Invocation {
        "invocation"
    } else {
        return Err(crate::construction::Failure::InvalidReference);
    };
    let slot_id = format!("{}/{slot}", node.scoped);
    crate::construction::identifier(&slot_id, expansion.limits)?;
    debug_assert!(slot.len() <= expansion.limits.max_identifier_bytes);
    Ok(
        crate::construction::model::projection::ExpandedBinding::Slot {
            slot_id,
            source_ref: format!("node/{node_id}/{direction}/{}", source.name),
        },
    )
}

fn record_use(
    expansion: &mut crate::construction::expand::Expansion<'_>,
    endpoint: &str,
) -> Result<(), crate::construction::Failure> {
    if let Some(count) = expansion.config_uses.get_mut(endpoint) {
        crate::construction::expand::take(
            count,
            1,
            crate::construction::expand::Ceiling(expansion.limits.max_edges),
        )
    } else {
        let mut count = 0;
        crate::construction::expand::take(
            &mut count,
            1,
            crate::construction::expand::Ceiling(expansion.limits.max_edges),
        )?;
        expansion.config_uses.insert(endpoint.to_owned(), count);
        Ok(())
    }
}

fn next_expression<'a>(
    stack: &mut Vec<std::slice::Iter<'a, crate::construction::model::term::Expression>>,
) -> Option<&'a crate::construction::model::term::Expression> {
    while let Some(remaining) = stack.last_mut() {
        if let Some(next) = remaining.next() {
            return Some(next);
        }
        stack.pop();
    }
    None
}

pub(super) fn references_resolved(
    expansion: &crate::construction::expand::Expansion<'_>,
    expression: &crate::construction::model::term::Expression,
    resolved: &std::collections::BTreeSet<&str>,
) -> Result<bool, crate::construction::Failure> {
    let mut stack = Vec::with_capacity(expansion.limits.max_depth);
    let mut current = Some(expression);
    while let Some(expression) = current {
        debug_assert!(stack.len() <= expansion.limits.max_depth);
        if let crate::construction::model::term::Expression::Reference { reference } = expression {
            if reference.direction == crate::construction::model::term::ReferenceDirection::Config
                && !resolved.contains(reference.name.as_str())
            {
                return Ok(false);
            }
        } else if let crate::construction::model::term::Expression::Apply { arguments, .. } =
            expression
            && !arguments.is_empty()
        {
            if stack.len() >= expansion.limits.max_depth {
                return Err(crate::construction::Failure::LimitExceeded);
            }
            stack.push(arguments.iter());
        }
        current = next_expression(&mut stack);
    }
    debug_assert!(stack.is_empty());
    Ok(true)
}
