#![no_std]
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the adapter keeps the exact reviewed Universal Weave boundary vocabulary"
)]

//! Optional bounded mutable topology derived from immutable history facts.

extern crate alloc;

use alloc::vec::Vec;
use core::hash::Hash;

use choregraph_history::{Digest, EventId, HistoryGraph, Projection};
use universal_weave::hashbrown::DefaultHashBuilder;
use universal_weave::independent::IndependentNode;
use universal_weave::indexmap::IndexSet;
use universal_weave::{BookmarkableWeave, IndependentContents, IndependentWeave, Node, Weave};

// r[impl choregraph.history.weave]
// r[impl choregraph.history.weave.valid]
// r[impl choregraph.history.weave.rejects]
// r[impl choregraph.history.source_adoption]
// r[impl choregraph.history.source_adoption.parity]
// r[verify choregraph.history.source_adoption.rejects_product]
const POLICY_DOMAIN: &[u8] = b"choregraph.weave.policy.v1";
const TOPOLOGY_DOMAIN: &[u8] = b"choregraph.weave.topology.v1";
const RECEIPT_DOMAIN: &[u8] = b"choregraph.weave.receipt.v1";
const ADD_OPERATION_TAG: u8 = 0;
const MOVE_OPERATION_TAG: u8 = 1;
const SET_ACTIVE_OPERATION_TAG: u8 = 2;
const SET_BOOKMARKED_OPERATION_TAG: u8 = 3;
const ADJACENT_PARENT_COUNT: usize = 2;
const LENGTH_U64_TAG: u8 = 0;
const LENGTH_NATIVE_TAG: u8 = 1;

/// A generic ordered key with a stable canonical identity projection.
pub trait CanonicalKey: Copy + Eq + Hash + Ord {
    /// Adds this key to a BLAKE3 framing context.
    fn update_identity(self, hasher: &mut blake3::Hasher);
}

impl CanonicalKey for EventId {
    fn update_identity(self, hasher: &mut blake3::Hasher) {
        hasher.update(&self.bytes());
    }
}

/// Named positive topology bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeavePolicy {
    max_nodes: u32,
    max_parent_fan_in: u32,
    max_nodes_native: usize,
}

impl WeavePolicy {
    /// Creates a valid policy.
    pub fn new(max_nodes: u32, max_parent_fan_in: u32) -> Result<Self, WeaveError> {
        if max_nodes == 0 {
            return Err(WeaveError::InvalidPolicy { field: "max_nodes" });
        }
        if max_parent_fan_in == 0 {
            return Err(WeaveError::InvalidPolicy {
                field: "max_parent_fan_in",
            });
        }
        let max_nodes_native = usize::try_from(max_nodes)
            .map_err(|_| WeaveError::InvalidPolicy { field: "max_nodes" })?;
        Ok(Self {
            max_nodes,
            max_parent_fan_in,
            max_nodes_native,
        })
    }

    /// Returns the maximum admitted topology nodes.
    #[must_use]
    pub const fn max_nodes(self) -> u32 {
        self.max_nodes
    }

    /// Returns the maximum parents for one topology node.
    #[must_use]
    pub const fn max_parent_fan_in(self) -> u32 {
        self.max_parent_fan_in
    }

    /// Returns the exact policy identity.
    #[must_use]
    pub fn identity(self) -> Digest {
        let mut hasher = blake3::Hasher::new();
        update_bytes(&mut hasher, POLICY_DOMAIN);
        update_count(&mut hasher, self.max_nodes);
        update_count(&mut hasher, self.max_parent_fan_in);
        Digest::from_bytes(*hasher.finalize().as_bytes())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TopologyContent {
    identity: Digest,
}

impl IndependentContents for TopologyContent {}

type TopologyDag<K> =
    universal_weave::independent::IndependentWeave<K, TopologyContent, (), DefaultHashBuilder>;
type TopologyNode<K> = IndependentNode<K, TopologyContent, DefaultHashBuilder>;

/// One bounded derived-topology operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WeaveOperation<K> {
    Add {
        key: K,
        parents: Vec<K>,
        content_identity: Digest,
    },
    Move {
        key: K,
        parents: Vec<K>,
    },
    SetActive {
        key: K,
        active: bool,
    },
    SetBookmarked {
        key: K,
        bookmarked: bool,
    },
}

impl<K: Copy> WeaveOperation<K> {
    const fn key(&self) -> K {
        match self {
            Self::Add { key, .. }
            | Self::Move { key, .. }
            | Self::SetActive { key, .. }
            | Self::SetBookmarked { key, .. } => *key,
        }
    }

    const fn tag(&self) -> u8 {
        match self {
            Self::Add { .. } => ADD_OPERATION_TAG,
            Self::Move { .. } => MOVE_OPERATION_TAG,
            Self::SetActive { .. } => SET_ACTIVE_OPERATION_TAG,
            Self::SetBookmarked { .. } => SET_BOOKMARKED_OPERATION_TAG,
        }
    }

    const fn kind(&self) -> WeaveOperationKind {
        if matches!(self, Self::Add { .. }) {
            WeaveOperationKind::Add
        } else if matches!(self, Self::Move { .. }) {
            WeaveOperationKind::Move
        } else if matches!(self, Self::SetActive { .. }) {
            WeaveOperationKind::SetActive
        } else {
            debug_assert!(matches!(self, Self::SetBookmarked { .. }));
            WeaveOperationKind::SetBookmarked
        }
    }
}

/// Metadata-only topology entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyEntry<K> {
    pub key: K,
    pub parent_count: u32,
    pub child_count: u32,
    pub active: bool,
    pub bookmarked: bool,
    pub content_identity: Digest,
}

/// Deterministic metadata-only topology snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologySnapshot<K> {
    pub generation: u64,
    pub policy_identity: Digest,
    pub source_history_identity: Option<Digest>,
    pub entries: Vec<TopologyEntry<K>>,
    pub identity: Digest,
}

/// One admitted derived-topology operation receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeaveReceipt<K> {
    pub operation: WeaveOperationKind,
    pub target: K,
    pub prior_generation: u64,
    pub generation: u64,
    pub prior_topology_identity: Digest,
    pub topology_identity: Digest,
    pub policy_identity: Digest,
    pub identity: Digest,
}

/// Stable operation class in receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeaveOperationKind {
    Add,
    Move,
    SetActive,
    SetBookmarked,
}

/// A new adapter value and its bounded receipt.
pub struct WeaveMutation<K: CanonicalKey> {
    pub adapter: WeaveAdapter<K>,
    pub receipt: WeaveReceipt<K>,
}

/// Fail-closed derived-topology errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WeaveError {
    InvalidPolicy { field: &'static str },
    StaleGeneration { expected: u64, observed: u64 },
    NodeLimitExceeded { maximum: u32 },
    ParentLimitExceeded { observed: u32, maximum: u32 },
    CountOverflow { field: &'static str },
    NodeExists,
    NodeMissing,
    ParentMissing,
    InvalidParentage,
    NoChange,
    GenerationOverflow,
    ProjectionEventMissing { event: EventId },
}

/// A pure bounded adapter over Universal Weave `0.4.1`.
#[derive(Clone)]
pub struct WeaveAdapter<K: CanonicalKey> {
    policy: WeavePolicy,
    generation: u64,
    node_count: u32,
    source_history_identity: Option<Digest>,
    weave: TopologyDag<K>,
}

impl<K: CanonicalKey> WeaveAdapter<K> {
    /// Creates empty derived topology.
    #[must_use]
    pub fn new(policy: WeavePolicy) -> Self {
        Self {
            policy,
            generation: 0,
            node_count: 0,
            source_history_identity: None,
            weave: TopologyDag::with_capacity(policy.max_nodes_native, ()),
        }
    }

    /// Returns the current generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns the current node count.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.node_count
    }

    /// Returns true when no topology nodes exist.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.weave.is_empty()
    }

    /// Applies one operation to a clone, preserving this value on every denial.
    pub fn apply(
        &self,
        expected_generation: u64,
        operation: WeaveOperation<K>,
    ) -> Result<WeaveMutation<K>, WeaveError> {
        if expected_generation != self.generation {
            return Err(WeaveError::StaleGeneration {
                expected: self.generation,
                observed: expected_generation,
            });
        }
        let before = self.snapshot()?.identity;
        let target = operation.key();
        let operation_tag = operation.tag();
        let operation_kind = operation.kind();
        let mut candidate = self.clone();
        candidate.apply_to_candidate(operation)?;
        let changed_topology = candidate.snapshot()?.identity;
        if before == changed_topology {
            return Err(WeaveError::NoChange);
        }
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(WeaveError::GenerationOverflow)?;
        candidate.generation = next_generation;
        let after = candidate.snapshot()?.identity;
        let policy_identity = self.policy.identity();
        let identity = receipt_identity(&ReceiptIdentityMaterial {
            operation_tag,
            target,
            prior_generation: self.generation,
            generation: next_generation,
            before,
            after,
            policy: policy_identity,
        });
        Ok(WeaveMutation {
            adapter: candidate,
            receipt: WeaveReceipt {
                operation: operation_kind,
                target,
                prior_generation: self.generation,
                generation: next_generation,
                prior_topology_identity: before,
                topology_identity: after,
                policy_identity,
                identity,
            },
        })
    }

    /// Returns a deterministic metadata-only snapshot.
    pub fn snapshot(&self) -> Result<TopologySnapshot<K>, WeaveError> {
        let mut weave = self.weave.clone();
        let mut keys = Vec::new();
        weave.get_ordered_identifiers(&mut keys);
        let mut entries = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(node) = weave.get(&key) {
                entries.push(TopologyEntry {
                    key,
                    parent_count: count_from_length(node.from().len(), "topology_parents")?,
                    child_count: count_from_length(node.to().len(), "topology_children")?,
                    active: node.is_active(),
                    bookmarked: weave.contains_bookmark(&key),
                    content_identity: node.contents().identity,
                });
            }
        }
        let policy_identity = self.policy.identity();
        let identity = topology_identity(
            self.generation,
            policy_identity,
            self.source_history_identity,
            &entries,
        );
        Ok(TopologySnapshot {
            generation: self.generation,
            policy_identity,
            source_history_identity: self.source_history_identity,
            entries,
            identity,
        })
    }

    #[allow(
        missing_const_fn,
        reason = "Universal Weave mutation trait calls are not const operations"
    )]
    fn apply_to_candidate(&mut self, operation: WeaveOperation<K>) -> Result<(), WeaveError> {
        if let WeaveOperation::Add {
            key,
            parents,
            content_identity,
        } = &operation
        {
            return self.add(*key, parents, *content_identity);
        }
        if let WeaveOperation::Move { key, parents } = &operation {
            return self.move_node(*key, parents);
        }
        if let WeaveOperation::SetActive { key, active } = &operation {
            return self.set_active(*key, *active);
        }
        if let WeaveOperation::SetBookmarked { key, bookmarked } = &operation {
            return self.set_bookmarked(*key, *bookmarked);
        }
        Err(WeaveError::InvalidParentage)
    }

    #[allow(
        explicit_defaults,
        reason = "the selected Universal Weave IndexSet exposes no empty constructor for its fixed hasher"
    )]
    fn add(&mut self, key: K, parents: &[K], content_identity: Digest) -> Result<(), WeaveError> {
        if self.node_count >= self.policy.max_nodes {
            return Err(WeaveError::NodeLimitExceeded {
                maximum: self.policy.max_nodes,
            });
        }
        let next_node_count = self
            .node_count
            .checked_add(1)
            .ok_or(WeaveError::CountOverflow {
                field: "topology_nodes",
            })?;
        self.validate_parents(parents)?;
        if self.weave.contains(&key) {
            return Err(WeaveError::NodeExists);
        }
        let node = TopologyNode {
            id: key,
            from: parents.iter().copied().collect(),
            to: IndexSet::default(),
            active: false,
            bookmarked: false,
            contents: TopologyContent {
                identity: content_identity,
            },
        };
        if !self.weave.insert(node) {
            return Err(WeaveError::InvalidParentage);
        }
        self.node_count = next_node_count;
        Ok(())
    }

    fn move_node(&mut self, key: K, parents: &[K]) -> Result<(), WeaveError> {
        if !self.weave.contains(&key) {
            return Err(WeaveError::NodeMissing);
        }
        self.validate_parents(parents)?;
        if !self.weave.move_to(&key, parents) {
            return Err(WeaveError::InvalidParentage);
        }
        Ok(())
    }

    fn set_active(&mut self, key: K, active: bool) -> Result<(), WeaveError> {
        let node = self.weave.get(&key).ok_or(WeaveError::NodeMissing)?;
        if node.is_active() == active {
            return Err(WeaveError::NoChange);
        }
        if !self.weave.set_active(&key, active) {
            return Err(WeaveError::InvalidParentage);
        }
        Ok(())
    }

    #[allow(
        missing_const_fn,
        reason = "Universal Weave bookmark trait calls are not const operations"
    )]
    fn set_bookmarked(&mut self, key: K, bookmarked: bool) -> Result<(), WeaveError> {
        if !self.weave.contains(&key) {
            return Err(WeaveError::NodeMissing);
        }
        if self.weave.contains_bookmark(&key) == bookmarked {
            return Err(WeaveError::NoChange);
        }
        if !self.weave.set_bookmarked(&key, bookmarked) {
            return Err(WeaveError::InvalidParentage);
        }
        Ok(())
    }

    #[allow(
        missing_const_fn,
        reason = "Universal Weave membership trait calls are not const operations"
    )]
    fn validate_parents(&self, parents: &[K]) -> Result<(), WeaveError> {
        let parent_count = count_from_length(parents.len(), "topology_parents")?;
        if parent_count > self.policy.max_parent_fan_in {
            return Err(WeaveError::ParentLimitExceeded {
                observed: parent_count,
                maximum: self.policy.max_parent_fan_in,
            });
        }
        if !parents
            .windows(ADJACENT_PARENT_COUNT)
            .all(|pair| pair[0] < pair[1])
        {
            return Err(WeaveError::InvalidParentage);
        }
        if parents.iter().any(|parent| !self.weave.contains(parent)) {
            return Err(WeaveError::ParentMissing);
        }
        Ok(())
    }
}

impl WeaveAdapter<EventId> {
    /// Projects admitted immutable history into a mutable metadata-only topology.
    pub fn from_history(
        graph: &HistoryGraph,
        projection: &Projection,
        policy: WeavePolicy,
    ) -> Result<Self, WeaveError> {
        let mut adapter = Self::new(policy);
        adapter.source_history_identity = Some(graph.identity());
        for identity in projection.events() {
            let event = graph
                .event(*identity)
                .ok_or(WeaveError::ProjectionEventMissing { event: *identity })?;
            let mut parents: Vec<EventId> = event
                .content()
                .parents
                .iter()
                .map(|parent| parent.event)
                .filter(|parent| projection.events().contains(parent))
                .collect();
            parents.sort();
            let mutation = adapter.apply(
                adapter.generation,
                WeaveOperation::Add {
                    key: *identity,
                    parents,
                    content_identity: event.content().payload_identity,
                },
            )?;
            adapter = mutation.adapter;
        }
        Ok(adapter)
    }
}

fn topology_identity<K: CanonicalKey>(
    generation: u64,
    policy_identity: Digest,
    source_history_identity: Option<Digest>,
    entries: &[TopologyEntry<K>],
) -> Digest {
    let mut hasher = blake3::Hasher::new();
    update_bytes(&mut hasher, TOPOLOGY_DOMAIN);
    hasher.update(&generation.to_le_bytes());
    hasher.update(&policy_identity.bytes());
    update_optional_digest(&mut hasher, source_history_identity);
    update_length(&mut hasher, entries.len());
    for entry in entries {
        entry.key.update_identity(&mut hasher);
        update_count(&mut hasher, entry.parent_count);
        update_count(&mut hasher, entry.child_count);
        hasher.update(&[u8::from(entry.active), u8::from(entry.bookmarked)]);
        hasher.update(&entry.content_identity.bytes());
    }
    Digest::from_bytes(*hasher.finalize().as_bytes())
}

struct ReceiptIdentityMaterial<K> {
    operation_tag: u8,
    target: K,
    prior_generation: u64,
    generation: u64,
    before: Digest,
    after: Digest,
    policy: Digest,
}

fn receipt_identity<K: CanonicalKey>(material: &ReceiptIdentityMaterial<K>) -> Digest {
    let mut hasher = blake3::Hasher::new();
    update_bytes(&mut hasher, RECEIPT_DOMAIN);
    hasher.update(&[material.operation_tag]);
    material.target.update_identity(&mut hasher);
    hasher.update(&material.prior_generation.to_le_bytes());
    hasher.update(&material.generation.to_le_bytes());
    for identity in [material.before, material.after, material.policy] {
        hasher.update(&identity.bytes());
    }
    Digest::from_bytes(*hasher.finalize().as_bytes())
}

#[allow(
    missing_const_fn,
    reason = "BLAKE3 hasher mutation is not a const operation"
)]
fn update_optional_digest(hasher: &mut blake3::Hasher, value: Option<Digest>) {
    match value {
        Some(identity) => {
            hasher.update(&[1]);
            hasher.update(&identity.bytes());
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

#[allow(
    missing_const_fn,
    reason = "BLAKE3 hasher mutation is not a const operation"
)]
fn update_length(hasher: &mut blake3::Hasher, length: usize) {
    match u64::try_from(length) {
        Ok(framed) => {
            hasher.update(&[LENGTH_U64_TAG]);
            hasher.update(&framed.to_le_bytes());
        }
        Err(_) => {
            hasher.update(&[LENGTH_NATIVE_TAG]);
            hasher.update(&length.to_le_bytes());
        }
    }
}

fn update_count(hasher: &mut blake3::Hasher, count: u32) {
    hasher.update(&[LENGTH_U64_TAG]);
    hasher.update(&u64::from(count).to_le_bytes());
}

fn update_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    update_length(hasher, bytes.len());
    hasher.update(bytes);
}

fn count_from_length(length: usize, field: &'static str) -> Result<u32, WeaveError> {
    u32::try_from(length).map_err(|_| WeaveError::CountOverflow { field })
}
