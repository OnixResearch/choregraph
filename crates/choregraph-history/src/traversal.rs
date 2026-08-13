// r[impl choregraph.history.lineage]
// r[impl choregraph.history.lineage.linear]
// r[impl choregraph.history.lineage.ambiguous]
const PROJECTION_IDENTITY_DOMAIN: &[u8] = b"choregraph.history.projection.v1";

/// Projection class. Parent order is never treated as a semantic choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionKind {
    AncestorClosure,
    LinearLineage,
}

impl ProjectionKind {
    const fn tag(self) -> u8 {
        if let Self::AncestorClosure = self {
            0
        } else {
            debug_assert!(matches!(self, Self::LinearLineage));
            1
        }
    }
}

/// Explicit parent choices keyed by each multi-parent event.
pub type ParentChoices =
    alloc::collections::BTreeMap<crate::identity::EventId, crate::identity::EventId>;

/// One bounded deterministic graph projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Projection {
    kind: ProjectionKind,
    head: crate::identity::EventId,
    events: alloc::vec::Vec<crate::identity::EventId>,
    identity: crate::identity::Digest,
}

impl Projection {
    /// Returns the selected projection class.
    #[must_use]
    pub const fn kind(&self) -> ProjectionKind {
        self.kind
    }

    /// Returns the selected immutable head.
    #[must_use]
    pub const fn head(&self) -> crate::identity::EventId {
        self.head
    }

    /// Returns deterministic projected event order.
    #[must_use]
    pub fn events(&self) -> &[crate::identity::EventId] {
        &self.events
    }

    /// Returns the exact graph-projection identity.
    #[must_use]
    pub const fn identity(&self) -> crate::identity::Digest {
        self.identity
    }
}

/// Returns the bounded ancestor closure of one head.
pub fn ancestor_closure(
    graph: &crate::dag::HistoryGraph,
    head: crate::identity::EventId,
    maximum: u32,
) -> Result<Projection, crate::model::HistoryError> {
    validate_bound(graph, maximum)?;
    debug_assert!(maximum > 0);
    debug_assert!(maximum <= graph.limits().max_projection_events);
    if !graph.contains(head) {
        return Err(crate::model::HistoryError::HeadMissing { event: head });
    }
    let mut state = ClosureState::new(graph, head, maximum)?;
    while let Some((current, is_exit)) = state.pending.pop() {
        if is_exit {
            let ordered_count = count_or_error(state.ordered.len(), "projection_events")?;
            if ordered_count >= maximum {
                return Err(crate::model::HistoryError::ProjectionLimitExceeded {
                    observed: ordered_count,
                    maximum,
                });
            }
            state.ordered.push(current);
            continue;
        }
        if !state.expanded.insert(current) {
            continue;
        }
        let expanded_count = count_or_error(state.expanded.len(), "projection_expanded")?;
        if expanded_count > maximum {
            return Err(crate::model::HistoryError::ProjectionLimitExceeded {
                observed: expanded_count,
                maximum,
            });
        }
        state.push(current, true)?;
        let event = graph
            .event(current)
            .ok_or(crate::model::HistoryError::HeadMissing { event: current })?;
        for parent in event.content().parents.iter().rev() {
            state.push(parent.event, false)?;
        }
    }
    Ok(build(
        graph,
        ProjectionKind::AncestorClosure,
        head,
        state.ordered,
    ))
}

/// Returns one explicit root-to-head lineage.
pub fn linear_lineage(
    graph: &crate::dag::HistoryGraph,
    head: crate::identity::EventId,
    choices: &ParentChoices,
    maximum: u32,
) -> Result<Projection, crate::model::HistoryError> {
    validate_bound(graph, maximum)?;
    debug_assert!(maximum > 0);
    debug_assert!(maximum <= graph.limits().max_projection_events);
    if !graph.contains(head) {
        return Err(crate::model::HistoryError::HeadMissing { event: head });
    }
    let reversed_reserve = output_reserve(graph, maximum)?;
    let mut reversed = alloc::vec::Vec::with_capacity(reversed_reserve);
    let mut visited = alloc::collections::BTreeSet::new();
    let mut current = head;
    loop {
        if !visited.insert(current) {
            return Err(crate::model::HistoryError::Cycle { event: current });
        }
        let visited_count = count_or_error(visited.len(), "lineage_visited")?;
        if visited_count > maximum {
            return Err(crate::model::HistoryError::ProjectionLimitExceeded {
                observed: visited_count,
                maximum,
            });
        }
        let reversed_count = count_or_error(reversed.len(), "lineage_events")?;
        if reversed_count >= maximum {
            return Err(crate::model::HistoryError::ProjectionLimitExceeded {
                observed: reversed_count,
                maximum,
            });
        }
        reversed.push(current);
        let event = graph
            .event(current)
            .ok_or(crate::model::HistoryError::HeadMissing { event: current })?;
        let parents = &event.content().parents;
        let selected = match parents.as_slice() {
            [] => break,
            [parent] => parent.event,
            _ => {
                let selected = choices
                    .get(&current)
                    .copied()
                    .ok_or(crate::model::HistoryError::AmbiguousLineage { event: current })?;
                if !parents.iter().any(|parent| parent.event == selected) {
                    return Err(crate::model::HistoryError::InvalidParentChoice {
                        event: current,
                        selected,
                    });
                }
                selected
            }
        };
        current = selected;
    }
    reversed.reverse();
    Ok(build(graph, ProjectionKind::LinearLineage, head, reversed))
}

struct ClosureState {
    pending: alloc::vec::Vec<(crate::identity::EventId, bool)>,
    pending_max_entries: u32,
    expanded: alloc::collections::BTreeSet<crate::identity::EventId>,
    ordered: alloc::vec::Vec<crate::identity::EventId>,
}

impl ClosureState {
    fn new(
        graph: &crate::dag::HistoryGraph,
        head: crate::identity::EventId,
        maximum: u32,
    ) -> Result<Self, crate::model::HistoryError> {
        let parent_slots = graph.limits().max_parents.checked_add(1).ok_or(
            crate::model::HistoryError::CountOverflow {
                field: "projection_parent_slots",
            },
        )?;
        let pending_max_entries =
            maximum
                .checked_mul(parent_slots)
                .ok_or(crate::model::HistoryError::CountOverflow {
                    field: "projection_pending",
                })?;
        let ordered_reserve = output_reserve(graph, maximum)?;
        Ok(Self {
            pending: alloc::vec![(head, false)],
            pending_max_entries,
            expanded: alloc::collections::BTreeSet::new(),
            ordered: alloc::vec::Vec::with_capacity(ordered_reserve),
        })
    }

    fn push(
        &mut self,
        event: crate::identity::EventId,
        is_exit: bool,
    ) -> Result<(), crate::model::HistoryError> {
        let pending_count = count_or_error(self.pending.len(), "projection_pending")?;
        if pending_count >= self.pending_max_entries {
            return Err(crate::model::HistoryError::ProjectionLimitExceeded {
                observed: pending_count,
                maximum: self.pending_max_entries,
            });
        }
        self.pending.push((event, is_exit));
        Ok(())
    }
}

fn output_reserve(
    graph: &crate::dag::HistoryGraph,
    maximum: u32,
) -> Result<usize, crate::model::HistoryError> {
    usize::try_from(core::cmp::min(maximum, graph.len())).map_err(|_| {
        crate::model::HistoryError::CountOverflow {
            field: "projection_capacity",
        }
    })
}

const fn validate_bound(
    graph: &crate::dag::HistoryGraph,
    maximum: u32,
) -> Result<(), crate::model::HistoryError> {
    if maximum == 0 || maximum > graph.limits().max_projection_events {
        return Err(crate::model::HistoryError::ProjectionLimitExceeded {
            observed: maximum,
            maximum: graph.limits().max_projection_events,
        });
    }
    Ok(())
}

fn count_or_error(length: usize, field: &'static str) -> Result<u32, crate::model::HistoryError> {
    crate::identity::count_from_length(length)
        .ok_or(crate::model::HistoryError::CountOverflow { field })
}

fn build(
    graph: &crate::dag::HistoryGraph,
    kind: ProjectionKind,
    head: crate::identity::EventId,
    events: alloc::vec::Vec<crate::identity::EventId>,
) -> Projection {
    let mut hasher = blake3::Hasher::new();
    crate::identity::update_bytes(&mut hasher, PROJECTION_IDENTITY_DOMAIN);
    hasher.update(&[kind.tag()]);
    hasher.update(&graph.identity().bytes());
    hasher.update(&head.bytes());
    crate::identity::update_length(&mut hasher, events.len());
    for event in &events {
        hasher.update(&event.bytes());
    }
    Projection {
        kind,
        head,
        events,
        identity: crate::identity::Digest::from_bytes(*hasher.finalize().as_bytes()),
    }
}
