#![allow(
    unknown_lints,
    assertion_density,
    explicit_defaults,
    clippy::expect_used,
    function_length,
    no_panic,
    no_unwrap,
    non_trait_imports,
    numeric_units,
    reason = "bounded source-parity fixtures require direct setup and external adapter types"
)]

// r[verify choregraph.history.weave]
// r[verify choregraph.history.weave.valid]
// r[verify choregraph.history.weave.rejects]
// r[verify choregraph.history.source_adoption]
// r[verify choregraph.history.source_adoption.parity]
use std::collections::BTreeMap;

use choregraph_history::{
    DIGEST_BYTES, Digest, EventClass, EventDraft, HistoryGraph, HistoryLimits, ParentLink,
    linear_lineage,
};
use choregraph_weave::{
    CanonicalKey, WeaveAdapter, WeaveError, WeaveOperation, WeaveOperationKind, WeavePolicy,
};

const MAX_EVENTS: u32 = 16;
const MAX_PARENTS: u32 = 4;
const MAX_METADATA: u32 = 4;
const MAX_LABEL: u32 = 32;
const MAX_PROJECTION: u32 = 16;
const MAX_NODES: u32 = 8;
const MAX_FAN_IN: u32 = 2;
const ROOT_KEY: u64 = 10;
const CHILD_KEY: u64 = 20;
const OTHER_KEY: u64 = 30;
const MISSING_KEY: u64 = 99;
const ROOT_SEED: u8 = 61;
const CHILD_SEED: u8 = 62;
const SCHEMA_SEED: u8 = 63;
const CURRENT_GENERATION: u64 = 0;
const NEXT_GENERATION: u64 = 1;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct TestKey(u64);

impl CanonicalKey for TestKey {
    fn update_identity(self, hasher: &mut blake3::Hasher) {
        hasher.update(&self.0.to_le_bytes());
    }
}

fn digest(seed: u8) -> Digest {
    Digest::from_bytes([seed; DIGEST_BYTES])
}

fn policy() -> WeavePolicy {
    WeavePolicy::new(MAX_NODES, MAX_FAN_IN).expect("positive weave policy")
}

#[test]
fn generic_add_move_activate_and_bookmark_match_reviewed_structural_behavior() {
    let adapter = WeaveAdapter::<TestKey>::new(policy());
    let root = adapter
        .apply(
            CURRENT_GENERATION,
            WeaveOperation::Add {
                key: TestKey(ROOT_KEY),
                parents: Vec::new(),
                content_identity: digest(ROOT_SEED),
            },
        )
        .expect("root add");
    assert_eq!(root.receipt.operation, WeaveOperationKind::Add);
    assert_eq!(root.receipt.generation, NEXT_GENERATION);

    let child = root
        .adapter
        .apply(
            root.receipt.generation,
            WeaveOperation::Add {
                key: TestKey(CHILD_KEY),
                parents: vec![TestKey(ROOT_KEY)],
                content_identity: digest(CHILD_SEED),
            },
        )
        .expect("child add");
    let activated = child
        .adapter
        .apply(
            child.receipt.generation,
            WeaveOperation::SetActive {
                key: TestKey(CHILD_KEY),
                active: true,
            },
        )
        .expect("activate");
    let bookmarked = activated
        .adapter
        .apply(
            activated.receipt.generation,
            WeaveOperation::SetBookmarked {
                key: TestKey(CHILD_KEY),
                bookmarked: true,
            },
        )
        .expect("bookmark");
    let other = bookmarked
        .adapter
        .apply(
            bookmarked.receipt.generation,
            WeaveOperation::Add {
                key: TestKey(OTHER_KEY),
                parents: Vec::new(),
                content_identity: digest(SCHEMA_SEED),
            },
        )
        .expect("other root");
    let moved = other
        .adapter
        .apply(
            other.receipt.generation,
            WeaveOperation::Move {
                key: TestKey(CHILD_KEY),
                parents: vec![TestKey(OTHER_KEY)],
            },
        )
        .expect("move");
    let snapshot = moved.adapter.snapshot().expect("moved snapshot");
    let child_entry = snapshot
        .entries
        .iter()
        .find(|entry| entry.key == TestKey(CHILD_KEY))
        .expect("child entry");
    assert_eq!(child_entry.parent_count, 1);
    assert!(child_entry.active);
    assert!(child_entry.bookmarked);
    assert_eq!(child_entry.content_identity, digest(CHILD_SEED));
}

#[test]
fn stale_missing_cycle_and_no_change_reject_without_topology_change() {
    let adapter = WeaveAdapter::<TestKey>::new(policy());
    let root = adapter
        .apply(
            CURRENT_GENERATION,
            WeaveOperation::Add {
                key: TestKey(ROOT_KEY),
                parents: Vec::new(),
                content_identity: digest(ROOT_SEED),
            },
        )
        .expect("root");
    let child = root
        .adapter
        .apply(
            root.receipt.generation,
            WeaveOperation::Add {
                key: TestKey(CHILD_KEY),
                parents: vec![TestKey(ROOT_KEY)],
                content_identity: digest(CHILD_SEED),
            },
        )
        .expect("child");
    let adapter = child.adapter;
    let before = adapter.snapshot().expect("before snapshot").identity;
    assert!(matches!(
        adapter.apply(
            adapter.generation(),
            WeaveOperation::SetActive {
                key: TestKey(ROOT_KEY),
                active: false,
            }
        ),
        Err(WeaveError::NoChange)
    ));
    assert!(matches!(
        adapter.apply(
            adapter.generation(),
            WeaveOperation::Move {
                key: TestKey(CHILD_KEY),
                parents: vec![TestKey(ROOT_KEY)],
            }
        ),
        Err(WeaveError::NoChange)
    ));
    assert!(matches!(
        adapter.apply(
            CURRENT_GENERATION,
            WeaveOperation::SetActive {
                key: TestKey(CHILD_KEY),
                active: true,
            }
        ),
        Err(WeaveError::StaleGeneration { .. })
    ));
    assert!(matches!(
        adapter.apply(
            adapter.generation(),
            WeaveOperation::SetActive {
                key: TestKey(MISSING_KEY),
                active: true,
            }
        ),
        Err(WeaveError::NodeMissing)
    ));
    assert!(matches!(
        adapter.apply(
            adapter.generation(),
            WeaveOperation::Move {
                key: TestKey(ROOT_KEY),
                parents: vec![TestKey(CHILD_KEY)],
            }
        ),
        Err(WeaveError::InvalidParentage)
    ));
    assert_eq!(
        before,
        adapter.snapshot().expect("unchanged snapshot").identity
    );

    let one_node_policy = WeavePolicy::new(1, MAX_FAN_IN).expect("one-node policy");
    let one_node = WeaveAdapter::<TestKey>::new(one_node_policy)
        .apply(
            CURRENT_GENERATION,
            WeaveOperation::Add {
                key: TestKey(ROOT_KEY),
                parents: Vec::new(),
                content_identity: digest(ROOT_SEED),
            },
        )
        .expect("one bounded node");
    assert!(matches!(
        one_node.adapter.apply(
            one_node.receipt.generation,
            WeaveOperation::Add {
                key: TestKey(CHILD_KEY),
                parents: vec![TestKey(ROOT_KEY)],
                content_identity: digest(CHILD_SEED),
            }
        ),
        Err(WeaveError::NodeLimitExceeded { .. })
    ));
}

#[test]
fn admitted_history_projects_to_metadata_only_derived_topology() {
    let limits = HistoryLimits::new(
        MAX_EVENTS,
        MAX_PARENTS,
        MAX_METADATA,
        MAX_LABEL,
        MAX_PROJECTION,
    )
    .expect("limits");
    let graph = HistoryGraph::new(limits).expect("graph");
    let root = graph
        .admit(EventDraft::v1(
            EventClass::Application,
            digest(ROOT_SEED),
            Vec::new(),
            digest(SCHEMA_SEED),
        ))
        .expect("root");
    let child = root
        .graph
        .admit(EventDraft::v1(
            EventClass::Application,
            digest(CHILD_SEED),
            vec![ParentLink {
                event: root.event,
                role: "previous".to_owned(),
            }],
            digest(SCHEMA_SEED),
        ))
        .expect("child");
    let lineage = linear_lineage(&child.graph, child.event, &BTreeMap::new(), MAX_PROJECTION)
        .expect("lineage");
    let adapter =
        WeaveAdapter::from_history(&child.graph, &lineage, policy()).expect("derived topology");
    let snapshot = adapter.snapshot().expect("derived snapshot");
    assert_eq!(
        snapshot.source_history_identity,
        Some(child.graph.identity())
    );
    assert_eq!(snapshot.entries.len(), lineage.events().len());
    assert_eq!(snapshot.entries[0].content_identity, digest(ROOT_SEED));
    assert_eq!(snapshot.entries[1].content_identity, digest(CHILD_SEED));
}
