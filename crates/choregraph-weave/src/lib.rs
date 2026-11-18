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

use choregraph_history::{
    dag::Graph,
    identity::{Digest, EventId},
    traversal::Projection,
};
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

/// Explicit caller-selected topology limits, validated by `Policy::new`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub max_nodes: u32,
    pub max_parent_fan_in: u32,
}

impl Limits {
    fn validate(self) -> Result<(), Error> {
        if self.max_nodes == 0 {
            return Err(Error::InvalidPolicy { field: "max_nodes" });
        }
        if self.max_parent_fan_in == 0 {
            return Err(Error::InvalidPolicy {
                field: "max_parent_fan_in",
            });
        }
        Ok(())
    }
}

/// Named positive topology bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Policy {
    max_nodes: u32,
    max_parent_fan_in: u32,
    max_nodes_native: usize,
}

impl Policy {
    /// Creates a valid policy.
    pub fn new(limits: Limits) -> Result<Self, Error> {
        limits.validate()?;
        let Limits {
            max_nodes,
            max_parent_fan_in,
        } = limits;
        let max_nodes_native =
            usize::try_from(max_nodes).map_err(|_| Error::InvalidPolicy { field: "max_nodes" })?;
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
pub enum Operation<K> {
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

impl<K: Copy> Operation<K> {
    const fn descriptor(&self) -> Result<(K, u8, OperationKind), Error> {
        if let Self::Add { key, .. } = self {
            return Ok((*key, ADD_OPERATION_TAG, OperationKind::Add));
        }
        if let Self::Move { key, .. } = self {
            return Ok((*key, MOVE_OPERATION_TAG, OperationKind::Move));
        }
        if let Self::SetActive { key, .. } = self {
            return Ok((*key, SET_ACTIVE_OPERATION_TAG, OperationKind::SetActive));
        }
        if let Self::SetBookmarked { key, .. } = self {
            return Ok((
                *key,
                SET_BOOKMARKED_OPERATION_TAG,
                OperationKind::SetBookmarked,
            ));
        }
        Err(Error::InvalidParentage)
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
pub struct Receipt<K> {
    pub operation: OperationKind,
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
pub enum OperationKind {
    Add,
    Move,
    SetActive,
    SetBookmarked,
}

/// A new adapter value and its bounded receipt.
pub struct Mutation<K: CanonicalKey> {
    pub adapter: Adapter<K>,
    pub receipt: Receipt<K>,
}

/// Fail-closed derived-topology errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
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
pub struct Adapter<K: CanonicalKey> {
    policy: Policy,
    generation: u64,
    node_count: u32,
    source_history_identity: Option<Digest>,
    weave: TopologyDag<K>,
}

impl<K: CanonicalKey> Adapter<K> {
    /// Creates empty derived topology.
    #[must_use]
    pub fn new(policy: Policy) -> Self {
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
        operation: Operation<K>,
    ) -> Result<Mutation<K>, Error> {
        if expected_generation != self.generation {
            return Err(Error::StaleGeneration {
                expected: self.generation,
                observed: expected_generation,
            });
        }
        let before = self.snapshot()?.identity;
        let (target, operation_tag, operation_kind) = operation.descriptor()?;
        let mut candidate = self.clone();
        candidate.apply_to_candidate(operation)?;
        debug_assert!(candidate.node_count <= candidate.policy.max_nodes);
        debug_assert!(candidate.weave.contains(&target));
        let changed_topology = candidate.snapshot()?.identity;
        if before == changed_topology {
            return Err(Error::NoChange);
        }
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(Error::GenerationOverflow)?;
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
        Ok(Mutation {
            adapter: candidate,
            receipt: Receipt {
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
    pub fn snapshot(&self) -> Result<TopologySnapshot<K>, Error> {
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
        debug_assert_eq!(u32::try_from(entries.len()), Ok(self.node_count));
        debug_assert!(entries.len() <= self.policy.max_nodes_native);
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
    fn apply_to_candidate(&mut self, operation: Operation<K>) -> Result<(), Error> {
        if let Operation::Add {
            key,
            parents,
            content_identity,
        } = &operation
        {
            return self.add(*key, parents, *content_identity);
        }
        self.apply_update(operation)
    }

    fn apply_update(&mut self, operation: Operation<K>) -> Result<(), Error> {
        if let Operation::Move { key, parents } = &operation {
            return self.move_node(*key, parents);
        }
        if let Operation::SetActive { key, active } = &operation {
            return self.set_active(*key, *active);
        }
        if let Operation::SetBookmarked { key, bookmarked } = &operation {
            return self.set_bookmarked(*key, *bookmarked);
        }
        Err(Error::InvalidParentage)
    }

    #[allow(
        explicit_defaults,
        reason = "the selected Universal Weave IndexSet exposes no empty constructor for its fixed hasher"
    )]
    fn add(&mut self, key: K, parents: &[K], content_identity: Digest) -> Result<(), Error> {
        debug_assert!(self.node_count <= self.policy.max_nodes);
        if self.node_count >= self.policy.max_nodes {
            return Err(Error::NodeLimitExceeded {
                maximum: self.policy.max_nodes,
            });
        }
        let next_node_count = self.node_count.checked_add(1).ok_or(Error::CountOverflow {
            field: "topology_nodes",
        })?;
        self.validate_parents(parents)?;
        if self.weave.contains(&key) {
            return Err(Error::NodeExists);
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
            return Err(Error::InvalidParentage);
        }
        self.node_count = next_node_count;
        debug_assert!(
            self.weave
                .get(&key)
                .is_some_and(|node| node.from().len() == parents.len())
        );
        Ok(())
    }

    fn move_node(&mut self, key: K, parents: &[K]) -> Result<(), Error> {
        if !self.weave.contains(&key) {
            return Err(Error::NodeMissing);
        }
        self.validate_parents(parents)?;
        if !self.weave.move_to(&key, parents) {
            return Err(Error::InvalidParentage);
        }
        Ok(())
    }

    fn set_active(&mut self, key: K, active: bool) -> Result<(), Error> {
        let node = self.weave.get(&key).ok_or(Error::NodeMissing)?;
        if node.is_active() == active {
            return Err(Error::NoChange);
        }
        if !self.weave.set_active(&key, active) {
            return Err(Error::InvalidParentage);
        }
        Ok(())
    }

    #[allow(
        missing_const_fn,
        reason = "Universal Weave bookmark trait calls are not const operations"
    )]
    fn set_bookmarked(&mut self, key: K, bookmarked: bool) -> Result<(), Error> {
        if !self.weave.contains(&key) {
            return Err(Error::NodeMissing);
        }
        if self.weave.contains_bookmark(&key) == bookmarked {
            return Err(Error::NoChange);
        }
        if !self.weave.set_bookmarked(&key, bookmarked) {
            return Err(Error::InvalidParentage);
        }
        Ok(())
    }

    #[allow(
        missing_const_fn,
        reason = "Universal Weave membership trait calls are not const operations"
    )]
    fn validate_parents(&self, parents: &[K]) -> Result<(), Error> {
        let parent_count = count_from_length(parents.len(), "topology_parents")?;
        if parent_count > self.policy.max_parent_fan_in {
            return Err(Error::ParentLimitExceeded {
                observed: parent_count,
                maximum: self.policy.max_parent_fan_in,
            });
        }
        if !parents
            .windows(ADJACENT_PARENT_COUNT)
            .all(|pair| pair[0] < pair[1])
        {
            return Err(Error::InvalidParentage);
        }
        if parents.iter().any(|parent| !self.weave.contains(parent)) {
            return Err(Error::ParentMissing);
        }
        Ok(())
    }
}

impl Adapter<EventId> {
    /// Projects admitted immutable history into a mutable metadata-only topology.
    pub fn from_history(
        graph: &Graph,
        projection: &Projection,
        policy: Policy,
    ) -> Result<Self, Error> {
        let mut adapter = Self::new(policy);
        adapter.source_history_identity = Some(graph.identity());
        for identity in projection.events() {
            let event = graph
                .event(*identity)
                .ok_or(Error::ProjectionEventMissing { event: *identity })?;
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
                Operation::Add {
                    key: *identity,
                    parents,
                    content_identity: event.content().payload_identity,
                },
            )?;
            adapter = mutation.adapter;
        }
        debug_assert_eq!(
            usize::try_from(adapter.node_count),
            Ok(projection.events().len())
        );
        debug_assert_eq!(adapter.generation, u64::from(adapter.node_count));
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
fn update_length(hasher: &mut blake3::Hasher, item_count: usize) {
    match u64::try_from(item_count) {
        Ok(framed) => {
            hasher.update(&[LENGTH_U64_TAG]);
            hasher.update(&framed.to_le_bytes());
        }
        Err(_) => {
            hasher.update(&[LENGTH_NATIVE_TAG]);
            hasher.update(&item_count.to_le_bytes());
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

fn count_from_length(item_count: usize, field: &'static str) -> Result<u32, Error> {
    u32::try_from(item_count).map_err(|_| Error::CountOverflow { field })
}
