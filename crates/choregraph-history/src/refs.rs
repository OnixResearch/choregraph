// r[impl choregraph.history.branch_refs]
// r[impl choregraph.history.branch_refs.advance]
// r[impl choregraph.history.branch_refs.stale]

const BRANCH_IDENTITY_DOMAIN: &[u8] = b"choregraph.history.branch.v1";
const SNAPSHOT_IDENTITY_DOMAIN: &[u8] = b"choregraph.history.snapshot.v1";
const CURSOR_IDENTITY_DOMAIN: &[u8] = b"choregraph.history.cursor.v1";

/// One mutable branch reference value supplied to the pure core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchRef {
    /// Bounded consumer-owned branch name.
    pub name: alloc::string::String,
    /// Immutable selected event head.
    pub head: crate::identity::EventId,
    /// Exact compare-and-set generation.
    pub generation: u64,
}

impl BranchRef {
    /// Returns an identity over the name, head, and generation.
    #[must_use]
    pub fn identity(&self) -> crate::identity::Digest {
        let mut hasher = blake3::Hasher::new();
        crate::identity::update_bytes(&mut hasher, BRANCH_IDENTITY_DOMAIN);
        crate::identity::update_bytes(&mut hasher, self.name.as_bytes());
        hasher.update(&self.head.bytes());
        hasher.update(&self.generation.to_le_bytes());
        crate::identity::Digest::from_bytes(*hasher.finalize().as_bytes())
    }
}

/// A deterministic compare-and-set plan. It performs no persistence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceMutationPlan {
    /// Exact current reference.
    pub before: BranchRef,
    /// Proposed successor reference.
    pub after: BranchRef,
}

/// Plans one generation-fenced branch advance.
pub fn plan_branch_advance(
    graph: &crate::dag::Graph,
    current: &BranchRef,
    expected_generation: u64,
    expected_head: crate::identity::EventId,
    proposed_head: crate::identity::EventId,
) -> Result<ReferenceMutationPlan, crate::model::Error> {
    validate_branch(graph, current)?;
    debug_assert!(!current.name.is_empty());
    debug_assert!(graph.contains(current.head));
    if current.generation != expected_generation {
        return Err(crate::model::Error::StaleGeneration {
            expected: current.generation,
            observed: expected_generation,
        });
    }
    if current.head != expected_head {
        return Err(crate::model::Error::StaleHead);
    }
    if !graph.contains(proposed_head) {
        return Err(crate::model::Error::HeadMissing {
            event: proposed_head,
        });
    }
    if proposed_head == current.head {
        return Err(crate::model::Error::NoChange);
    }
    let generation = current
        .generation
        .checked_add(1)
        .ok_or(crate::model::Error::GenerationOverflow)?;
    Ok(ReferenceMutationPlan {
        before: current.clone(),
        after: BranchRef {
            name: current.name.clone(),
            head: proposed_head,
            generation,
        },
    })
}

/// A graph-derived snapshot bound to reference and projection facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    pub graph_identity: crate::identity::Digest,
    pub branch_identity: crate::identity::Digest,
    pub generation: u64,
    pub policy_identity: crate::identity::Digest,
    pub projection_identity: crate::identity::Digest,
    pub head: crate::identity::EventId,
    pub event_count: u32,
    pub identity: crate::identity::Digest,
}

impl Snapshot {
    /// Derives a snapshot without granting storage authority.
    pub fn derive(
        graph: &crate::dag::Graph,
        branch: &BranchRef,
        projection: &crate::traversal::Projection,
        policy_identity: crate::identity::Digest,
    ) -> Result<Self, crate::model::Error> {
        validate_branch(graph, branch)?;
        if branch.head != projection.head() {
            return Err(crate::model::Error::StaleHead);
        }
        let material =
            SnapshotIdentityMaterial::from_inputs(graph, branch, projection, policy_identity)?;
        Ok(Self::from_material(material))
    }

    fn from_material(material: SnapshotIdentityMaterial) -> Self {
        let identity = derive_snapshot_identity(&material);
        Self {
            graph_identity: material.graph,
            branch_identity: material.branch,
            generation: material.generation,
            policy_identity: material.policy,
            projection_identity: material.projection,
            head: material.head,
            event_count: material.event_count,
            identity,
        }
    }
}

/// One bounded cursor bound to the complete snapshot context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cursor {
    pub snapshot_identity: crate::identity::Digest,
    pub graph_identity: crate::identity::Digest,
    pub branch_identity: crate::identity::Digest,
    pub generation: u64,
    pub policy_identity: crate::identity::Digest,
    pub projection_identity: crate::identity::Digest,
    pub head: crate::identity::EventId,
    pub position: u32,
    pub identity: crate::identity::Digest,
}

impl Cursor {
    /// Creates a cursor inside the selected snapshot.
    pub fn derive(snapshot: &Snapshot, position: u32) -> Result<Self, crate::model::Error> {
        if position > snapshot.event_count {
            return Err(crate::model::Error::CursorOutOfBounds {
                position,
                event_count: snapshot.event_count,
            });
        }
        Ok(Self::from_snapshot(snapshot, position))
    }

    fn from_snapshot(snapshot: &Snapshot, position: u32) -> Self {
        let identity = cursor_identity(snapshot.identity, position);
        Self {
            snapshot_identity: snapshot.identity,
            graph_identity: snapshot.graph_identity,
            branch_identity: snapshot.branch_identity,
            generation: snapshot.generation,
            policy_identity: snapshot.policy_identity,
            projection_identity: snapshot.projection_identity,
            head: snapshot.head,
            position,
            identity,
        }
    }
}

fn cursor_identity(
    snapshot_identity: crate::identity::Digest,
    position: u32,
) -> crate::identity::Digest {
    let mut hasher = blake3::Hasher::new();
    crate::identity::update_bytes(&mut hasher, CURSOR_IDENTITY_DOMAIN);
    crate::identity::update_digest(&mut hasher, snapshot_identity);
    crate::identity::update_count(&mut hasher, position);
    crate::identity::Digest::from_bytes(*hasher.finalize().as_bytes())
}

/// Validates that a cursor still belongs to the exact snapshot.
pub fn validate_cursor(cursor: &Cursor, snapshot: &Snapshot) -> Result<(), crate::model::Error> {
    let expected = Cursor::derive(snapshot, cursor.position)?;
    if expected != *cursor {
        return Err(crate::model::Error::CursorMismatch);
    }
    Ok(())
}

fn validate_branch(
    graph: &crate::dag::Graph,
    branch: &BranchRef,
) -> Result<(), crate::model::Error> {
    let name_bytes = crate::identity::count_from_length(branch.name.len()).ok_or(
        crate::model::Error::CountOverflow {
            field: "branch_name",
        },
    )?;
    if branch.name.is_empty() || name_bytes > graph.limits().max_label_bytes {
        return Err(crate::model::Error::InvalidBranchName);
    }
    if !graph.contains(branch.head) {
        return Err(crate::model::Error::HeadMissing { event: branch.head });
    }
    Ok(())
}

struct SnapshotIdentityMaterial {
    graph: crate::identity::Digest,
    branch: crate::identity::Digest,
    generation: u64,
    policy: crate::identity::Digest,
    projection: crate::identity::Digest,
    head: crate::identity::EventId,
    event_count: u32,
}

impl SnapshotIdentityMaterial {
    fn from_inputs(
        graph: &crate::dag::Graph,
        branch: &BranchRef,
        projection: &crate::traversal::Projection,
        policy: crate::identity::Digest,
    ) -> Result<Self, crate::model::Error> {
        let event_count = crate::identity::count_from_length(projection.events().len()).ok_or(
            crate::model::Error::CountOverflow {
                field: "snapshot_events",
            },
        )?;
        Ok(Self {
            graph: graph.identity(),
            branch: branch.identity(),
            generation: branch.generation,
            policy,
            projection: projection.identity(),
            head: branch.head,
            event_count,
        })
    }
}

fn derive_snapshot_identity(material: &SnapshotIdentityMaterial) -> crate::identity::Digest {
    let mut hasher = blake3::Hasher::new();
    crate::identity::update_bytes(&mut hasher, SNAPSHOT_IDENTITY_DOMAIN);
    for identity in [
        material.graph,
        material.branch,
        material.policy,
        material.projection,
    ] {
        crate::identity::update_digest(&mut hasher, identity);
    }
    hasher.update(&material.generation.to_le_bytes());
    hasher.update(&material.head.bytes());
    crate::identity::update_count(&mut hasher, material.event_count);
    crate::identity::Digest::from_bytes(*hasher.finalize().as_bytes())
}
