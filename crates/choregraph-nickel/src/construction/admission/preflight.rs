//! Streaming resource preflight for the closed projection DTO. The typed parser remains
//! authoritative for field types and values; this pass only admits collection sizes.
use serde::Deserialize;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};

#[derive(Clone, Copy)]
enum Shape {
    Projection,
    Node,
    Interface,
    Boundary,
    Expression,
    Configuration,
    Nodes,
    Ports,
    Invocation,
    Edges,
    Controls,
    Effects,
    Interfaces,
    Boundaries,
    MappedPorts,
    Arguments,
}
#[derive(Clone, Copy)]
enum Collection {
    Nodes,
    Ports,
    Invocation,
    Edges,
    Controls,
    Effects,
    Interfaces,
    Boundaries,
    MappedPorts,
    Configuration,
    Arguments,
}

#[derive(Default)]
struct Counts {
    nodes: usize,
    ports: usize,
    invocation: usize,
    edges: usize,
    effects: usize,
    interfaces: usize,
    boundaries: usize,
    mapped: usize,
    exceeded: bool,
}

struct Scan {
    limits: crate::construction::model::profile::Limits,
    counts: Counts,
}

impl Scan {
    fn take<E: serde::de::Error>(
        &mut self,
        collection: Collection,
        local: &mut usize,
    ) -> Result<(), E> {
        let (counter, ceiling) = match collection {
            Collection::Nodes => (&mut self.counts.nodes, self.limits.max_nodes),
            Collection::Ports => (&mut self.counts.ports, self.limits.max_ports),
            Collection::Invocation => (&mut self.counts.invocation, self.limits.max_boundaries),
            Collection::Edges | Collection::Controls => {
                (&mut self.counts.edges, self.limits.max_edges)
            }
            Collection::Effects => (&mut self.counts.effects, self.limits.max_effects),
            Collection::Interfaces => (&mut self.counts.interfaces, self.limits.max_instances),
            Collection::Boundaries => (&mut self.counts.boundaries, self.limits.max_boundaries),
            Collection::MappedPorts => (&mut self.counts.mapped, self.limits.max_boundaries),
            Collection::Configuration => (local, self.limits.max_boundaries),
            _ => {
                if !matches!(collection, Collection::Arguments) {
                    return Err(E::custom("unsupported projection collection"));
                }
                (local, self.limits.max_boundaries)
            }
        };
        if *counter >= ceiling {
            self.counts.exceeded = true;
            return Err(E::custom("projection resource ceiling"));
        }
        *counter = counter.checked_add(1).ok_or_else(|| {
            self.counts.exceeded = true;
            E::custom("projection resource ceiling")
        })?;
        debug_assert!(
            *counter <= ceiling,
            "collection count remains within its ceiling"
        );
        debug_assert!(
            !self.counts.exceeded,
            "successful count does not signal a ceiling violation"
        );
        Ok(())
    }
}

struct Seed<'a> {
    scan: &'a mut Scan,
    shape: Shape,
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = ();

    fn deserialize<D: serde::de::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Seed<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a projection object or collection")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        let Seed { scan, shape, depth } = self;
        let mut local = 0;
        let mut fields = 0u16;
        let mut configuration_names = std::collections::BTreeSet::new();
        while let Some(key) = map.next_key::<std::borrow::Cow<'de, str>>()? {
            let (bit, child) = if matches!(shape, Shape::Configuration) {
                if configuration_names.insert(key) {
                    scan.take::<M::Error>(Collection::Configuration, &mut local)?;
                }
                (0, Some(Shape::Expression))
            } else {
                field::<M::Error>(shape, key.as_ref())?
            };
            if bit != 0 && fields & bit != 0 {
                return Err(serde::de::Error::custom("duplicate projection field"));
            }
            fields |= bit;
            if let Some(child) = child {
                map.next_value_seed(Seed {
                    scan: &mut *scan,
                    shape: child,
                    depth,
                })?;
            } else {
                map.next_value::<serde::de::IgnoredAny>()?;
            }
        }
        debug_assert_eq!(
            configuration_names.len(),
            local,
            "counted each distinct configuration key"
        );
        debug_assert!(
            local <= scan.limits.max_boundaries,
            "distinct configuration keys stay bounded"
        );
        Ok(())
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut sequence: S) -> Result<(), S::Error> {
        let Seed { scan, shape, depth } = self;
        let (collection, child) = match shape {
            Shape::Nodes => (Collection::Nodes, Some(Shape::Node)),
            Shape::Interfaces => (Collection::Interfaces, Some(Shape::Interface)),
            Shape::Boundaries => (Collection::Boundaries, Some(Shape::Boundary)),
            Shape::Arguments => (Collection::Arguments, Some(Shape::Expression)),
            Shape::Ports => (Collection::Ports, None),
            Shape::Invocation => (Collection::Invocation, None),
            Shape::Edges => (Collection::Edges, None),
            Shape::Controls => (Collection::Controls, None),
            Shape::Effects => (Collection::Effects, None),
            Shape::MappedPorts => (Collection::MappedPorts, None),
            _ => return Err(serde::de::Error::custom("expected projection object")),
        };
        let mut local = 0;
        loop {
            // SeqAccess confirms an element before the seed counts it, without allocation.
            let next = sequence.next_element_seed(Element {
                scan: &mut *scan,
                collection,
                child,
                depth,
                local: &mut local,
            })?;
            if next.is_none() {
                debug_assert!(
                    local <= scan.limits.max_boundaries,
                    "argument collections stay bounded"
                );
                debug_assert!(
                    !scan.counts.exceeded,
                    "complete sequence did not exceed a ceiling"
                );
                return Ok(());
            }
        }
    }
}

type Field = (u16, Option<Shape>);

fn field<E: serde::de::Error>(shape: Shape, key: &str) -> Result<Field, E> {
    if matches!(shape, Shape::Projection) {
        projection_field(key)
    } else if matches!(shape, Shape::Node) {
        node_field(key)
    } else if matches!(shape, Shape::Interface) {
        interface_field(key)
    } else if matches!(shape, Shape::Boundary) {
        boundary_field(key)
    } else if matches!(shape, Shape::Expression) {
        expression_field(key)
    } else {
        Err(E::custom("expected projection collection"))
    }
}

fn projection_field<E: serde::de::Error>(key: &str) -> Result<Field, E> {
    Ok(match key {
        "schema" => (1, None),
        "target" => (2, None),
        "registry" => (4, None),
        "nodes" => (8, Some(Shape::Nodes)),
        "edges" => (16, Some(Shape::Edges)),
        "control_edges" => (32, Some(Shape::Controls)),
        "effects" => (64, Some(Shape::Effects)),
        "interfaces" => (128, Some(Shape::Interfaces)),
        _ => return Err(E::custom("unknown projection field")),
    })
}

fn node_field<E: serde::de::Error>(key: &str) -> Result<Field, E> {
    Ok(match key {
        "id" => (1, None),
        "scoped_id" => (2, None),
        "operation" => (4, None),
        "implementation" => (8, None),
        "ports" => (16, Some(Shape::Ports)),
        "invocation" => (32, Some(Shape::Invocation)),
        "configuration" => (64, Some(Shape::Configuration)),
        _ => return Err(E::custom("unknown node field")),
    })
}

fn interface_field<E: serde::de::Error>(key: &str) -> Result<Field, E> {
    Ok(match key {
        "scope" => (1, None),
        "inputs" => (2, Some(Shape::Boundaries)),
        "outputs" => (4, Some(Shape::Boundaries)),
        _ => return Err(E::custom("unknown interface field")),
    })
}

fn boundary_field<E: serde::de::Error>(key: &str) -> Result<Field, E> {
    Ok(match key {
        "name" => (1, None),
        "value_type" => (2, None),
        "ports" => (4, Some(Shape::MappedPorts)),
        _ => return Err(E::custom("unknown boundary field")),
    })
}

fn expression_field<E: serde::de::Error>(key: &str) -> Result<Field, E> {
    Ok(match key {
        "tag" => (1, None),
        "value" => (2, None),
        "node" => (4, None),
        "direction" => (8, None),
        "name" => (16, None),
        "port" => (32, None),
        "operator" => (64, None),
        "arguments" => (128, Some(Shape::Arguments)),
        _ => return Err(E::custom("unknown expression field")),
    })
}

struct Element<'a, 'b> {
    scan: &'a mut Scan,
    collection: Collection,
    child: Option<Shape>,
    depth: usize,
    local: &'b mut usize,
}

impl<'de> DeserializeSeed<'de> for Element<'_, '_> {
    type Value = ();

    fn deserialize<D: serde::de::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        self.scan.take::<D::Error>(self.collection, self.local)?;
        debug_assert!(
            self.depth <= self.scan.limits.max_depth,
            "visited expression depth is admitted"
        );
        if matches!(self.collection, Collection::Arguments)
            && self.depth >= self.scan.limits.max_depth
        {
            self.scan.counts.exceeded = true;
            return Err(serde::de::Error::custom("projection expression depth"));
        }
        if let Some(shape) = self.child {
            let depth = if matches!(self.collection, Collection::Arguments) {
                self.depth.checked_add(1).ok_or_else(|| {
                    self.scan.counts.exceeded = true;
                    serde::de::Error::custom("projection expression depth")
                })?
            } else {
                self.depth
            };
            debug_assert!(
                depth <= self.scan.limits.max_depth,
                "child expression remains within the depth ceiling"
            );
            Seed {
                scan: self.scan,
                shape,
                depth,
            }
            .deserialize(deserializer)
        } else {
            serde::de::IgnoredAny::deserialize(deserializer).map(|_| ())
        }
    }
}

pub(super) fn scan_projection(
    bytes: &[u8],
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    let mut scan = Scan {
        limits,
        counts: Counts::default(),
    };
    let mut parser = serde_json::Deserializer::from_slice(bytes);
    let result = Seed {
        scan: &mut scan,
        shape: Shape::Projection,
        depth: 0,
    }
    .deserialize(&mut parser)
    .and_then(|()| parser.end());
    match result {
        Ok(()) => Ok(()),
        Err(_) if scan.counts.exceeded => Err(crate::construction::Failure::LimitExceeded),
        Err(_) => Err(crate::construction::Failure::InvalidJson),
    }
}
