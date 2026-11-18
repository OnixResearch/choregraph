struct Frame<'a> {
    fragment: &'a crate::construction::model::program::Fragment,
    template: &'a str,
    parameters: Option<usize>,
    parameter_base: usize,
    depth: usize,
    scope: String,
    children: std::collections::BTreeMap<&'a str, crate::construction::expand::Open>,
    remaining: &'a [crate::construction::model::program::Fragment],
    child_template: &'a str,
    child_parameters: Option<usize>,
    entered: bool,
}

struct ExposureTarget<'a> {
    consumed: &'a mut std::collections::BTreeSet<String>,
    result: &'a mut crate::construction::expand::Open,
}

impl<'a> Frame<'a> {
    fn new(
        fragment: &'a crate::construction::model::program::Fragment,
        template: &'a str,
        parameters: Option<usize>,
        depth: usize,
    ) -> Self {
        Self {
            fragment,
            template,
            parameters,
            depth,
            parameter_base: 0,
            scope: String::new(),
            children: std::collections::BTreeMap::new(),
            remaining: &[],
            child_template: template,
            child_parameters: parameters,
            entered: false,
        }
    }
}

impl<'a> crate::construction::expand::Expansion<'a> {
    pub(super) fn fragment(
        &mut self,
    ) -> Result<crate::construction::expand::Open, crate::construction::Failure> {
        let candidate = self.candidate;
        let mut stack =
            Vec::with_capacity(crate::construction::expand::deeper(self.limits.max_depth)?);
        let mut parameters: Vec<crate::construction::expand::Parameters> = Vec::new();
        let mut path = vec![candidate.root.clone()];
        stack.push(Frame::new(&candidate.body, "root", None, 0));
        while let Some(mut frame) = stack.pop() {
            debug_assert!(stack.len() <= self.limits.max_depth);
            if !frame.entered
                && let Some(result) = self.start(&mut frame, &mut path, &mut parameters)?
            {
                if let Some(root) =
                    self.complete(frame, result, &mut stack, &mut path, &mut parameters)?
                {
                    debug_assert_eq!(path.len(), 1);
                    return Ok(root);
                }
                continue;
            }
            if let Some((child, remaining)) = frame.remaining.split_first() {
                if frame.children.contains_key(child.key.as_str()) {
                    return Err(crate::construction::Failure::DuplicateKey);
                }
                frame.remaining = remaining;
                let next = Frame::new(
                    child,
                    frame.child_template,
                    frame.child_parameters,
                    crate::construction::expand::deeper(frame.depth)?,
                );
                if stack.len() >= self.limits.max_depth {
                    return Err(crate::construction::Failure::LimitExceeded);
                }
                stack.push(frame);
                stack.push(next);
                continue;
            }
            let result = self.compose(&mut frame, &path)?;
            if let Some(root) =
                self.complete(frame, result, &mut stack, &mut path, &mut parameters)?
            {
                return Ok(root);
            }
        }
        Err(crate::construction::Failure::InvalidSchema)
    }
    fn start(
        &mut self,
        frame: &mut Frame<'a>,
        path: &mut Vec<String>,
        parameters: &mut Vec<crate::construction::expand::Parameters>,
    ) -> Result<Option<crate::construction::expand::Open>, crate::construction::Failure> {
        let candidate = self.candidate;
        frame.parameter_base = parameters.len();
        debug_assert!(frame.parameter_base <= self.limits.max_instances);
        self.enter(frame, path)?;
        let actual = if let Some(index) = frame.parameters {
            parameters
                .get(index)
                .ok_or(crate::construction::Failure::InvalidParameters)?
        } else {
            &candidate.parameters
        };
        debug_assert!(frame.depth <= self.limits.max_depth);
        if let crate::construction::model::program::FragmentKind::Operation { .. } =
            &frame.fragment.kind
        {
            return self
                .operation(frame.fragment, path, actual, frame.template)
                .map(Some);
        }
        if let crate::construction::model::program::FragmentKind::Instantiate {
            template,
            arguments,
        } = &frame.fragment.kind
        {
            let definition = *self
                .templates
                .get(template.as_str())
                .ok_or(crate::construction::Failure::UnknownTemplate)?;
            let actual = self.arguments(definition, arguments, actual)?;
            if parameters.len() >= self.limits.max_instances {
                return Err(crate::construction::Failure::LimitExceeded);
            }
            frame.child_parameters = Some(parameters.len());
            debug_assert!(parameters.len() < self.limits.max_instances);
            parameters.push(actual);
            frame.child_template = &definition.name;
            frame.remaining = std::slice::from_ref(&definition.body);
        } else {
            frame.remaining = self.parts(frame.fragment)?;
        }
        Ok(None)
    }

    fn enter(
        &mut self,
        frame: &mut Frame<'_>,
        path: &mut Vec<String>,
    ) -> Result<(), crate::construction::Failure> {
        if frame.depth > self.limits.max_depth {
            return Err(crate::construction::Failure::LimitExceeded);
        }
        crate::construction::expand::take(
            &mut self.instances,
            1,
            crate::construction::expand::Ceiling(self.limits.max_instances),
        )?;
        crate::construction::identifier(&frame.fragment.key, self.limits)?;
        if frame.fragment.key.contains('/') {
            return Err(crate::construction::Failure::InvalidIdentifier);
        }
        path.push(frame.fragment.key.clone());
        frame.scope = crate::construction::expand::scoped(path, "fragment")?;
        if frame.scope.len() > self.limits.max_identifier_bytes {
            return Err(crate::construction::Failure::LimitExceeded);
        }
        debug_assert!(path.len() > 1);
        debug_assert!(self.instances <= self.limits.max_instances);
        crate::construction::expand::attribution::origin(
            self,
            frame.scope.clone(),
            path,
            crate::construction::expand::OriginLabel {
                local: &frame.fragment.key,
                template: frame.template,
            },
            frame.fragment.origin.as_ref(),
        )?;
        frame.entered = true;
        Ok(())
    }

    fn parts(
        &self,
        fragment: &'a crate::construction::model::program::Fragment,
    ) -> Result<&'a [crate::construction::model::program::Fragment], crate::construction::Failure>
    {
        let parts = if let crate::construction::model::program::FragmentKind::Sequence { parts }
        | crate::construction::model::program::FragmentKind::Parallel { parts } =
            &fragment.kind
        {
            parts
        } else if let crate::construction::model::program::FragmentKind::Wire {
            parts,
            connections,
            ..
        } = &fragment.kind
        {
            if connections.len() > self.limits.max_edges {
                return Err(crate::construction::Failure::LimitExceeded);
            }
            parts
        } else {
            return Ok(&[]);
        };
        if parts.len() > self.limits.max_instances {
            return Err(crate::construction::Failure::LimitExceeded);
        }
        debug_assert!(parts.len() <= self.limits.max_instances);
        debug_assert!(matches!(
            &fragment.kind,
            crate::construction::model::program::FragmentKind::Sequence { .. }
                | crate::construction::model::program::FragmentKind::Parallel { .. }
                | crate::construction::model::program::FragmentKind::Wire { .. }
        ));
        Ok(parts)
    }

    fn arguments(
        &mut self,
        definition: &crate::construction::model::program::Template,
        arguments: &std::collections::BTreeMap<
            String,
            crate::construction::model::term::Expression,
        >,
        parameters: &crate::construction::expand::Parameters,
    ) -> Result<crate::construction::expand::Parameters, crate::construction::Failure> {
        if definition.parameters.len() != arguments.len() {
            return Err(crate::construction::Failure::InvalidParameters);
        }
        let mut actual = crate::construction::expand::Parameters::new();
        debug_assert!(definition.parameters.len() <= self.limits.max_boundaries);
        for name in &definition.parameters {
            let expression = crate::construction::expand::terms::expression(
                self,
                arguments
                    .get(name)
                    .ok_or(crate::construction::Failure::InvalidParameters)?,
                parameters,
            )?;
            let crate::construction::model::term::Expression::Literal { value } = expression else {
                return Err(crate::construction::Failure::InvalidParameters);
            };
            if actual.len() >= self.limits.max_boundaries {
                return Err(crate::construction::Failure::LimitExceeded);
            }
            actual.insert(name.clone(), value);
        }
        crate::construction::bounded_json(&actual, self.limits.max_parameter_bytes)?;
        debug_assert_eq!(actual.len(), definition.parameters.len());
        Ok(actual)
    }

    fn complete(
        &mut self,
        frame: Frame<'a>,
        result: crate::construction::expand::Open,
        stack: &mut [Frame<'a>],
        path: &mut Vec<String>,
        parameters: &mut Vec<crate::construction::expand::Parameters>,
    ) -> Result<Option<crate::construction::expand::Open>, crate::construction::Failure> {
        let count = result
            .inputs
            .len()
            .checked_add(result.outputs.len())
            .ok_or(crate::construction::Failure::LimitExceeded)?;
        crate::construction::expand::take(
            &mut self.boundaries,
            count,
            crate::construction::expand::Ceiling(self.limits.max_boundaries),
        )?;
        debug_assert!(self.boundaries <= self.limits.max_boundaries);
        debug_assert!(frame.parameter_base <= parameters.len());
        self.interfaces
            .push((frame.scope, result.inputs.clone(), result.outputs.clone()));
        path.pop();
        parameters.truncate(frame.parameter_base);
        if let Some(parent) = stack.last_mut() {
            if parent.children.len() >= self.limits.max_instances {
                return Err(crate::construction::Failure::LimitExceeded);
            }
            parent.children.insert(frame.fragment.key.as_str(), result);
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    fn compose(
        &mut self,
        frame: &mut Frame<'a>,
        path: &[String],
    ) -> Result<crate::construction::expand::Open, crate::construction::Failure> {
        let children = std::mem::take(&mut frame.children);
        debug_assert!(children.len() <= self.limits.max_instances);
        debug_assert!(frame.entered);
        if let crate::construction::model::program::FragmentKind::Instantiate { .. } =
            &frame.fragment.kind
        {
            children
                .into_values()
                .next()
                .ok_or(crate::construction::Failure::InvalidSchema)
        } else if let crate::construction::model::program::FragmentKind::Identity { boundaries } =
            &frame.fragment.kind
        {
            self.identity(frame, path, boundaries)
        } else if let crate::construction::model::program::FragmentKind::Sequence { parts } =
            &frame.fragment.kind
        {
            self.sequence(parts, children)
        } else if let crate::construction::model::program::FragmentKind::Parallel { parts } =
            &frame.fragment.kind
        {
            self.parallel(parts, children)
        } else if let crate::construction::model::program::FragmentKind::Wire { .. } =
            &frame.fragment.kind
        {
            self.wire(frame, path, children)
        } else {
            Err(crate::construction::Failure::InvalidSchema)
        }
    }

    fn identity(
        &mut self,
        frame: &Frame<'_>,
        path: &[String],
        boundaries: &[crate::construction::model::interface::Boundary],
    ) -> Result<crate::construction::expand::Open, crate::construction::Failure> {
        let fragment = frame.fragment;
        let template = frame.template;
        let mut result = crate::construction::expand::Open::default();
        debug_assert!(self.boundaries <= self.limits.max_boundaries);
        let mut names = std::collections::BTreeSet::new();
        for boundary in boundaries {
            crate::construction::identifier(&boundary.name, self.limits)?;
            if !names.insert(&boundary.name) {
                return Err(crate::construction::Failure::DuplicateKey);
            }
            crate::construction::expand::take(
                &mut self.boundaries,
                2,
                crate::construction::expand::Ceiling(self.limits.max_boundaries),
            )?;
            let endpoint = crate::construction::expand::attribution::endpoint(
                self,
                path,
                crate::construction::expand::EndpointLabel {
                    local: &format!("identity/{}", boundary.name),
                    value_type: &boundary.value_type,
                },
                None,
            )?;
            crate::construction::expand::attribution::origin(
                self,
                endpoint.clone(),
                path,
                crate::construction::expand::OriginLabel {
                    local: &boundary.name,
                    template,
                },
                fragment.origin.as_ref(),
            )?;
            let port = crate::construction::expand::Port {
                name: boundary.name.clone(),
                value_type: boundary.value_type.clone(),
                endpoint,
            };
            result.inputs.push(port.clone());
            result.outputs.push(port);
        }
        debug_assert_eq!(result.inputs.len(), result.outputs.len());
        Ok(result)
    }

    fn sequence(
        &mut self,
        parts: &[crate::construction::model::program::Fragment],
        mut children: std::collections::BTreeMap<&str, crate::construction::expand::Open>,
    ) -> Result<crate::construction::expand::Open, crate::construction::Failure> {
        let mut result = crate::construction::expand::Open::default();
        debug_assert!(children.len() <= self.limits.max_instances);
        let mut previous: Option<crate::construction::expand::Open> = None;
        let mut previous_exits: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        for part in parts {
            let child = children
                .remove(part.key.as_str())
                .ok_or(crate::construction::Failure::InvalidHandle)?;
            let (entries, exits) =
                crate::construction::expand::routing::frontier(self, &child.nodes)?;
            debug_assert!(result.nodes.len() <= self.nodes.len());
            for source in &previous_exits {
                for target in &entries {
                    crate::construction::expand::routing::control(
                        self,
                        crate::construction::expand::Route { source, target },
                    )?;
                }
            }
            if !child.nodes.is_empty() {
                previous_exits = exits;
            }
            if let Some(previous) = previous {
                if previous.outputs.len() != child.inputs.len() {
                    return Err(crate::construction::Failure::OpenBoundary);
                }
                for (source, target) in previous.outputs.iter().zip(&child.inputs) {
                    crate::construction::expand::routing::arc(
                        self,
                        crate::construction::expand::Route {
                            source: &source.endpoint,
                            target: &target.endpoint,
                        },
                    )?;
                }
            } else {
                result.inputs = child.inputs.clone();
            }
            result.nodes.extend(child.nodes.iter().cloned());
            previous = Some(child);
        }
        if let Some(previous) = previous {
            result.outputs = previous.outputs;
        }
        Ok(result)
    }

    fn parallel(
        &mut self,
        parts: &[crate::construction::model::program::Fragment],
        mut children: std::collections::BTreeMap<&str, crate::construction::expand::Open>,
    ) -> Result<crate::construction::expand::Open, crate::construction::Failure> {
        let mut result = crate::construction::expand::Open::default();
        debug_assert!(children.len() <= self.limits.max_instances);
        for part in parts {
            let child = children
                .remove(part.key.as_str())
                .ok_or(crate::construction::Failure::InvalidHandle)?;
            result.nodes.extend(child.nodes);
            for (ports, destination) in [
                (child.inputs, &mut result.inputs),
                (child.outputs, &mut result.outputs),
            ] {
                for mut port in ports {
                    port.name = format!("{}/{}", part.key, port.name);
                    crate::construction::identifier(&port.name, self.limits)?;
                    destination.push(port);
                }
            }
        }
        debug_assert!(children.is_empty());
        Ok(result)
    }

    fn wire(
        &mut self,
        frame: &Frame<'_>,
        path: &[String],
        children: std::collections::BTreeMap<&str, crate::construction::expand::Open>,
    ) -> Result<crate::construction::expand::Open, crate::construction::Failure> {
        let fragment = frame.fragment;
        let crate::construction::model::program::FragmentKind::Wire {
            connections,
            inputs,
            outputs,
            ..
        } = &fragment.kind
        else {
            return Err(crate::construction::Failure::InvalidSchema);
        };
        let mut result = crate::construction::expand::Open::default();
        debug_assert!(children.len() <= self.limits.max_instances);
        for child in children.values() {
            result.nodes.extend(child.nodes.iter().cloned());
        }
        debug_assert!(result.nodes.len() <= self.nodes.len());
        let mut consumed = std::collections::BTreeSet::new();
        for connection in connections {
            let source = self.handle(
                &children,
                &connection.source,
                crate::construction::model::interface::Direction::Output,
            )?;
            let target = self.handle(
                &children,
                &connection.target,
                crate::construction::model::interface::Direction::Input,
            )?;
            if !consumed.insert(target.endpoint.clone()) {
                return Err(crate::construction::Failure::UsageViolation);
            }
            crate::construction::expand::routing::arc(
                self,
                crate::construction::expand::Route {
                    source: &source.endpoint,
                    target: &target.endpoint,
                },
            )?;
        }
        for (exposures, direction) in [
            (
                inputs,
                crate::construction::model::interface::Direction::Input,
            ),
            (
                outputs,
                crate::construction::model::interface::Direction::Output,
            ),
        ] {
            self.expose(
                frame,
                path,
                &children,
                ExposureTarget {
                    consumed: &mut consumed,
                    result: &mut result,
                },
                (exposures, direction),
            )?;
        }
        if children
            .values()
            .flat_map(|child| &child.inputs)
            .any(|port| !consumed.contains(&port.endpoint))
        {
            return Err(crate::construction::Failure::OpenBoundary);
        }
        Ok(result)
    }

    fn expose(
        &mut self,
        frame: &Frame<'_>,
        path: &[String],
        children: &std::collections::BTreeMap<&str, crate::construction::expand::Open>,
        target: ExposureTarget<'_>,
        (exposures, direction): (
            &[crate::construction::model::interface::Exposure],
            crate::construction::model::interface::Direction,
        ),
    ) -> Result<(), crate::construction::Failure> {
        let mut names = std::collections::BTreeSet::new();
        for exposure in exposures {
            crate::construction::expand::take(
                &mut self.boundaries,
                1,
                crate::construction::expand::Ceiling(self.limits.max_boundaries),
            )?;
            crate::construction::identifier(&exposure.name, self.limits)?;
            if !names.insert(&exposure.name) {
                return Err(crate::construction::Failure::DuplicateKey);
            }
            let port = self.handle(children, &exposure.handle, direction)?;
            if direction == crate::construction::model::interface::Direction::Input
                && !target.consumed.insert(port.endpoint.clone())
            {
                return Err(crate::construction::Failure::UsageViolation);
            }
            let public = crate::construction::expand::Port {
                name: exposure.name.clone(),
                value_type: port.value_type.clone(),
                endpoint: port.endpoint.clone(),
            };
            crate::construction::expand::attribution::origin(
                self,
                crate::construction::expand::scoped(
                    path,
                    &format!("boundary/{direction:?}/{}", exposure.name),
                )?,
                path,
                crate::construction::expand::OriginLabel {
                    local: &exposure.name,
                    template: frame.template,
                },
                frame.fragment.origin.as_ref(),
            )?;
            if direction == crate::construction::model::interface::Direction::Input {
                target.result.inputs.push(public);
            } else {
                target.result.outputs.push(public);
            }
        }
        debug_assert_eq!(names.len(), exposures.len());
        debug_assert!(self.boundaries <= self.limits.max_boundaries);
        Ok(())
    }

    fn handle<'b>(
        &self,
        children: &'b std::collections::BTreeMap<&str, crate::construction::expand::Open>,
        handle: &crate::construction::model::interface::Handle,
        direction: crate::construction::model::interface::Direction,
    ) -> Result<&'b crate::construction::expand::Port, crate::construction::Failure> {
        debug_assert!(children.len() <= self.limits.max_instances);
        for value in [&handle.root, &handle.instance, &handle.boundary] {
            crate::construction::identifier(value, self.limits)?;
        }
        if handle.root != self.candidate.root {
            return Err(crate::construction::Failure::InvalidHandle);
        }
        debug_assert_eq!(handle.root, self.candidate.root);
        if handle.direction != direction {
            return Err(crate::construction::Failure::WrongDirection);
        }
        let child = children
            .get(handle.instance.as_str())
            .ok_or(crate::construction::Failure::InvalidHandle)?;
        let ports = if direction == crate::construction::model::interface::Direction::Input {
            &child.inputs
        } else {
            &child.outputs
        };
        ports
            .iter()
            .find(|port| port.name == handle.boundary)
            .ok_or(crate::construction::Failure::UnknownBoundary)
    }
}
