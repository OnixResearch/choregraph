//! Bounded compile-time elaboration; execution always uses the existing admitted machine.
use std::fmt::Write;

type Parameters = std::collections::BTreeMap<String, serde_json::Value>;
#[derive(Clone)]
struct Port {
    name: String,
    value_type: String,
    endpoint: String,
}
#[derive(Default)]
struct Open {
    inputs: Vec<Port>,
    outputs: Vec<Port>,
    nodes: std::collections::BTreeSet<String>,
}
struct Endpoint {
    value_type: String,
    real: Option<(
        String,
        crate::construction::model::interface::Direction,
        String,
    )>,
}
struct Route<'a> {
    source: &'a str,
    target: &'a str,
}
struct OriginLabel<'a> {
    local: &'a str,
    template: &'a str,
}
struct EndpointLabel<'a> {
    local: &'a str,
    value_type: &'a str,
}
struct Node {
    scoped: String,
    operation: String,
    implementation: String,
    ports: Vec<(
        String,
        crate::construction::model::interface::Direction,
        String,
        String,
    )>,
    invocation: Vec<crate::construction::model::interface::Boundary>,
    configuration: std::collections::BTreeMap<String, crate::construction::model::term::Expression>,
    effects: Vec<crate::construction::model::term::Effect>,
}

pub(super) struct Expansion<'a> {
    candidate: &'a crate::construction::model::program::Candidate,
    registry: crate::construction::registry::Index<'a>,
    modules: &'a [crate::construction::association::CapturedModule<'a>],
    limits: crate::construction::model::profile::Limits,
    templates:
        std::collections::BTreeMap<&'a str, &'a crate::construction::model::program::Template>,
    nodes: std::collections::BTreeMap<String, Node>,
    endpoints: std::collections::BTreeMap<String, Endpoint>,
    arcs: std::collections::BTreeSet<(String, String)>,
    controls: std::collections::BTreeSet<(String, String)>,
    origins: Vec<crate::construction::model::projection::Origin>,
    origin_bytes: usize,
    instances: usize,
    boundaries: usize,
    effect_count: usize,
    expanded_bytes: usize,
    config_uses: std::collections::BTreeMap<String, usize>,
    interfaces: Vec<(String, Vec<Port>, Vec<Port>)>,
}

fn scoped(path: &[String], local: &str) -> Result<String, crate::construction::Failure> {
    let mut result = String::new();
    for segment in path
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(local))
    {
        write!(result, "{}:{segment}", segment.len())
            .map_err(|_| crate::construction::Failure::InvalidIdentifier)?;
    }
    Ok(result)
}
fn port_name(
    direction: crate::construction::model::interface::Direction,
    name: &str,
) -> Result<String, crate::construction::Failure> {
    let prefix = if direction == crate::construction::model::interface::Direction::Input {
        "input"
    } else if direction == crate::construction::model::interface::Direction::Output {
        "output"
    } else {
        return Err(crate::construction::Failure::WrongDirection);
    };
    Ok(format!("{prefix}/{name}"))
}
struct Ceiling(usize);

const fn take(
    counter: &mut usize,
    count: usize,
    ceiling: Ceiling,
) -> Result<(), crate::construction::Failure> {
    let Some(total) = counter.checked_add(count) else {
        return Err(crate::construction::Failure::LimitExceeded);
    };
    *counter = total;
    if total > ceiling.0 {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    Ok(())
}

fn deeper(depth: usize) -> Result<usize, crate::construction::Failure> {
    depth
        .checked_add(1)
        .ok_or(crate::construction::Failure::LimitExceeded)
}

mod attribution;
mod elaboration;
mod nodes;
mod output;
mod routing;
mod terms;

impl<'a> Expansion<'a> {
    pub(super) fn new(
        candidate: &'a crate::construction::model::program::Candidate,
        registry: crate::construction::registry::Index<'a>,
        modules: &'a [crate::construction::association::CapturedModule<'a>],
        limits: crate::construction::model::profile::Limits,
    ) -> Result<Self, crate::construction::Failure> {
        if candidate.templates.len() > limits.max_instances {
            return Err(crate::construction::Failure::LimitExceeded);
        }
        let mut templates = std::collections::BTreeMap::new();
        for template in &candidate.templates {
            crate::construction::identifier(&template.name, limits)?;
            if template.parameters.len() > limits.max_boundaries
                || templates.insert(template.name.as_str(), template).is_some()
            {
                return Err(crate::construction::Failure::DuplicateKey);
            }
            let mut names = std::collections::BTreeSet::new();
            for parameter in &template.parameters {
                crate::construction::identifier(parameter, limits)?;
                if !names.insert(parameter) {
                    return Err(crate::construction::Failure::InvalidParameters);
                }
            }
            debug_assert_eq!(names.len(), template.parameters.len());
        }
        debug_assert_eq!(templates.len(), candidate.templates.len());
        debug_assert!(templates.len() <= limits.max_instances);
        Ok(Self {
            candidate,
            registry,
            modules,
            limits,
            templates,
            nodes: std::collections::BTreeMap::new(),
            endpoints: std::collections::BTreeMap::new(),
            arcs: std::collections::BTreeSet::new(),
            controls: std::collections::BTreeSet::new(),
            origins: Vec::new(),
            origin_bytes: 0,
            instances: 0,
            boundaries: 0,
            effect_count: 0,
            expanded_bytes: 0,
            config_uses: std::collections::BTreeMap::new(),
            interfaces: Vec::new(),
        })
    }
}
