#![allow(
    unknown_lints,
    assertion_density,
    clippy::expect_used,
    explicit_defaults,
    function_length,
    no_panic,
    no_unwrap,
    non_trait_imports,
    numeric_units,
    raw_arithmetic_overflow,
    reason = "bounded positive and negative history fixtures require direct setup and failure assertions"
)]

// r[verify choregraph.history.event_dag]
// r[verify choregraph.history.event_dag.append]
// r[verify choregraph.history.event_dag.rejects]
// r[verify choregraph.history.branch_refs]
// r[verify choregraph.history.branch_refs.advance]
// r[verify choregraph.history.branch_refs.stale]
// r[verify choregraph.history.lineage]
// r[verify choregraph.history.lineage.linear]
// r[verify choregraph.history.lineage.ambiguous]
// r[verify choregraph.history.merge]
// r[verify choregraph.history.merge.valid]
// r[verify choregraph.history.merge.rejects_implicit]
// r[verify choregraph.history.verification]
use std::collections::BTreeMap;

use choregraph_history::{
    dag::Graph,
    identity::{DIGEST_BYTES, Digest, EventId},
    model::{Error, EventClass, EventDraft, Limits, MergeEnvelope, MetadataIdentity, ParentLink},
    refs::{BranchRef, Cursor, Snapshot, plan_branch_advance, validate_cursor},
    traversal::{ancestor_closure, linear_lineage},
};

const MAX_EVENTS: u32 = 32;
const MAX_PARENTS: u32 = 4;
const MAX_METADATA: u32 = 4;
const MAX_LABEL_BYTES: u32 = 32;
const MAX_PROJECTION: u32 = 32;
const ROOT_SEED: u8 = 11;
const LEFT_SEED: u8 = 12;
const RIGHT_SEED: u8 = 13;
const MERGE_SEED: u8 = 14;
const SCHEMA_SEED: u8 = 21;
const POLICY_SEED: u8 = 22;
const METADATA_SEED: u8 = 23;
const UNKNOWN_SEED: u8 = 99;
const INITIAL_GENERATION: u64 = 7;
const SUCCESSOR_GENERATION: u64 = INITIAL_GENERATION + 1;
const CURSOR_POSITION: u32 = 2;

fn digest(seed: u8) -> Digest {
    Digest::from_bytes([seed; DIGEST_BYTES])
}

fn limits() -> Limits {
    Limits {
        max_events: MAX_EVENTS,
        max_parents: MAX_PARENTS,
        max_metadata_entries: MAX_METADATA,
        max_label_bytes: MAX_LABEL_BYTES,
        max_projection_events: MAX_PROJECTION,
    }
}

fn root_draft(seed: u8) -> EventDraft {
    EventDraft::v1(
        EventClass::Application,
        digest(seed),
        Vec::new(),
        digest(SCHEMA_SEED),
    )
}

fn child_draft(seed: u8, role: &str, parent: EventId) -> EventDraft {
    EventDraft::v1(
        EventClass::Application,
        digest(seed),
        vec![ParentLink {
            event: parent,
            role: role.to_owned(),
        }],
        digest(SCHEMA_SEED),
    )
}

fn admit(graph: &Graph, draft: EventDraft) -> (Graph, EventId) {
    let admission = graph.admit(draft).expect("event admission");
    (admission.graph, admission.event)
}

fn fork_graph() -> (Graph, EventId, EventId, EventId) {
    let graph = Graph::new(limits()).expect("empty graph");
    let (graph, root) = admit(&graph, root_draft(ROOT_SEED));
    let (graph, left) = admit(&graph, child_draft(LEFT_SEED, "main", root));
    let (graph, right) = admit(&graph, child_draft(RIGHT_SEED, "main", root));
    (graph, root, left, right)
}

#[test]
fn linear_fork_projection_reference_snapshot_cursor_and_merge_are_explicit() {
    let (graph, root, left, right) = fork_graph();
    let closure = ancestor_closure(&graph, left, MAX_PROJECTION).expect("closure");
    assert_eq!(closure.events(), [root, left]);

    let merge = MergeEnvelope {
        parents: vec![
            ParentLink {
                event: left,
                role: "left".to_owned(),
            },
            ParentLink {
                event: right,
                role: "right".to_owned(),
            },
        ],
        resolution_identity: Some(digest(MERGE_SEED)),
        schema_identity: digest(SCHEMA_SEED),
        merge_policy_identity: digest(POLICY_SEED),
        metadata: vec![MetadataIdentity {
            key: "safe".to_owned(),
            value: digest(METADATA_SEED),
        }],
    };
    let admission = graph.admit_merge(merge).expect("supplied merge");
    let graph = admission.graph;
    let merge_id = admission.event;
    let ambiguous = linear_lineage(&graph, merge_id, &BTreeMap::new(), MAX_PROJECTION);
    assert_eq!(ambiguous, Err(Error::AmbiguousLineage { event: merge_id }));

    let choices = BTreeMap::from([(merge_id, left)]);
    let lineage =
        linear_lineage(&graph, merge_id, &choices, MAX_PROJECTION).expect("selected lineage");
    assert_eq!(lineage.events(), [root, left, merge_id]);

    let branch = BranchRef {
        name: "main".to_owned(),
        head: left,
        generation: INITIAL_GENERATION,
    };
    let plan = plan_branch_advance(&graph, &branch, INITIAL_GENERATION, left, merge_id)
        .expect("reference plan");
    assert_eq!(plan.before, branch);
    assert_eq!(plan.after.head, merge_id);
    assert_eq!(plan.after.generation, SUCCESSOR_GENERATION);

    let snapshot =
        Snapshot::derive(&graph, &plan.after, &lineage, limits().identity()).expect("snapshot");
    let cursor = Cursor::derive(&snapshot, CURSOR_POSITION).expect("cursor");
    assert_eq!(validate_cursor(&cursor, &snapshot), Ok(()));
    assert_eq!(
        graph
            .event(merge_id)
            .expect("merge")
            .content()
            .payload_identity,
        digest(MERGE_SEED)
    );
}

#[test]
fn malformed_missing_duplicate_cycle_stale_ambiguous_and_bound_cases_fail_closed() {
    let graph = Graph::new(limits()).expect("empty graph");
    let missing = EventId::new(digest(UNKNOWN_SEED));
    assert_eq!(
        graph.admit(child_draft(LEFT_SEED, "main", missing)),
        Err(Error::MissingParent { event: missing })
    );
    assert!(graph.is_empty());

    let (graph, root) = admit(&graph, root_draft(ROOT_SEED));
    assert_eq!(graph.admit(root_draft(ROOT_SEED)), Err(Error::NoChange));
    assert_eq!(
        graph.admit_claimed(root, root_draft(LEFT_SEED)),
        Err(Error::ConflictingDuplicate { event: root })
    );
    assert_eq!(
        graph.admit_claimed(root, child_draft(RIGHT_SEED, "self", root)),
        Err(Error::Cycle { event: root })
    );

    let noncanonical = EventDraft::v1(
        EventClass::Application,
        digest(RIGHT_SEED),
        vec![
            ParentLink {
                event: root,
                role: "z".to_owned(),
            },
            ParentLink {
                event: root,
                role: "a".to_owned(),
            },
        ],
        digest(SCHEMA_SEED),
    );
    assert!(matches!(
        graph.admit(noncanonical),
        Err(Error::NonCanonicalParents | Error::DuplicateParent { .. })
    ));

    let branch = BranchRef {
        name: "main".to_owned(),
        head: root,
        generation: INITIAL_GENERATION,
    };
    assert_eq!(
        plan_branch_advance(&graph, &branch, SUCCESSOR_GENERATION, root, root),
        Err(Error::StaleGeneration {
            expected: INITIAL_GENERATION,
            observed: SUCCESSOR_GENERATION,
        })
    );
    assert_eq!(
        plan_branch_advance(&graph, &branch, INITIAL_GENERATION, missing, root),
        Err(Error::StaleHead)
    );
    assert_eq!(
        plan_branch_advance(&graph, &branch, INITIAL_GENERATION, root, root),
        Err(Error::NoChange)
    );
    assert!(matches!(
        ancestor_closure(&graph, root, 0),
        Err(Error::ProjectionLimitExceeded { .. })
    ));
    assert_eq!(graph.len(), 1);

    let one_event_limits = Limits {
        max_events: 1,
        ..limits()
    };
    let bounded = Graph::new(one_event_limits).expect("bounded graph");
    let admitted = bounded.admit(root_draft(ROOT_SEED)).expect("bounded root");
    assert!(matches!(
        admitted
            .graph
            .admit(child_draft(LEFT_SEED, "main", admitted.event)),
        Err(Error::EventLimitExceeded { .. })
    ));
    assert_eq!(admitted.graph.len(), 1);
}

#[test]
fn property_identity_acyclicity_projection_and_denial_state_are_deterministic() {
    let (graph, root, left, right) = fork_graph();
    assert_eq!(graph.identity(), graph.clone().identity());
    assert_eq!(
        Graph::expected_identity(&root_draft(ROOT_SEED)),
        Graph::expected_identity(&root_draft(ROOT_SEED))
    );
    for head in [root, left, right] {
        let first = ancestor_closure(&graph, head, MAX_PROJECTION).expect("first closure");
        let second = ancestor_closure(&graph, head, MAX_PROJECTION).expect("second closure");
        assert_eq!(first, second);
        assert_eq!(first.events().last(), Some(&head));
    }
    let before = graph.identity();
    let denial = graph.admit(child_draft(
        LEFT_SEED,
        "missing",
        EventId::new(digest(UNKNOWN_SEED)),
    ));
    assert!(matches!(denial, Err(Error::MissingParent { .. })));
    assert_eq!(before, graph.identity());
}

#[test]
fn claim_hygiene_keeps_product_authority_outside_history() {
    let non_claims = choregraph_history::HISTORY_NON_CLAIMS.join("|");
    for required in [
        "application merge correctness",
        "persistence durability",
        "effect execution or success",
        "authorization",
        "release eligibility",
    ] {
        assert!(non_claims.contains(required));
    }
    assert!(!choregraph_history::HISTORY_CLAIM.contains("correctness"));
    assert!(!choregraph_history::HISTORY_CLAIM.contains("authority"));
}

#[test]
fn implicit_merge_and_stale_cursor_are_rejected() {
    let (graph, _root, left, right) = fork_graph();
    let merge = MergeEnvelope {
        parents: vec![
            ParentLink {
                event: left,
                role: "left".to_owned(),
            },
            ParentLink {
                event: right,
                role: "right".to_owned(),
            },
        ],
        resolution_identity: None,
        schema_identity: digest(SCHEMA_SEED),
        merge_policy_identity: digest(POLICY_SEED),
        metadata: Vec::new(),
    };
    assert_eq!(graph.admit_merge(merge), Err(Error::MissingMergeResolution));
    let direct_merge = EventDraft::v1(
        EventClass::Merge,
        digest(MERGE_SEED),
        vec![
            ParentLink {
                event: left,
                role: "left".to_owned(),
            },
            ParentLink {
                event: right,
                role: "right".to_owned(),
            },
        ],
        digest(SCHEMA_SEED),
    );
    assert_eq!(graph.admit(direct_merge), Err(Error::MissingMergePolicy));

    let projection =
        linear_lineage(&graph, left, &BTreeMap::new(), MAX_PROJECTION).expect("lineage");
    let branch = BranchRef {
        name: "main".to_owned(),
        head: left,
        generation: INITIAL_GENERATION,
    };
    let snapshot =
        Snapshot::derive(&graph, &branch, &projection, limits().identity()).expect("snapshot");
    let mut cursor = Cursor::derive(&snapshot, CURSOR_POSITION).expect("cursor");
    cursor.generation = SUCCESSOR_GENERATION;
    assert_eq!(
        validate_cursor(&cursor, &snapshot),
        Err(Error::CursorMismatch)
    );
}

#[test]
fn graph_rejects_invalid_named_bounds_in_policy_order() {
    type LimitMutation = fn(&mut Limits);
    let fields: [(LimitMutation, &str); 5] = [
        (|value| value.max_events = 0, "max_events"),
        (|value| value.max_parents = 0, "max_parents"),
        (
            |value| value.max_metadata_entries = 0,
            "max_metadata_entries",
        ),
        (|value| value.max_label_bytes = 0, "max_label_bytes"),
        (
            |value| value.max_projection_events = 0,
            "max_projection_events",
        ),
    ];
    for (invalidate, field) in fields {
        let mut policy = limits();
        invalidate(&mut policy);
        assert_eq!(Graph::new(policy), Err(Error::InvalidLimits { field }));
    }
    let mut policy = limits();
    policy.max_parents = 0;
    policy.max_label_bytes = 0;
    assert_eq!(
        Graph::new(policy),
        Err(Error::InvalidLimits {
            field: "max_parents"
        })
    );
}
