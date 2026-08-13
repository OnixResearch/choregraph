#![allow(
    unknown_lints,
    assertion_density,
    explicit_defaults,
    clippy::expect_used,
    missing_const_fn,
    no_panic,
    no_unwrap,
    non_trait_imports,
    numeric_units,
    reason = "disposable consumer fixtures require direct setup and named boundary types"
)]

// r[verify choregraph.history.consumers]
use std::collections::BTreeMap;

use choregraph_history::{
    DIGEST_BYTES, Digest, EventClass, EventDraft, HistoryGraph, HistoryLimits, Projection,
    linear_lineage,
};

const MAX_EVENTS: u32 = 8;
const MAX_PARENTS: u32 = 2;
const MAX_METADATA: u32 = 2;
const MAX_LABEL: u32 = 32;
const MAX_PROJECTION: u32 = 8;
const PAYLOAD_SEED: u8 = 71;
const SCHEMA_SEED: u8 = 72;
const SESSION_SEED: u8 = 73;
const WORKFLOW_SEED: u8 = 74;
const MACHINE_SEED: u8 = 75;
const RUN_SEED: u8 = 76;
const STALE_HISTORY_SEED: u8 = 77;

#[derive(Clone, Debug, Eq, PartialEq)]
struct AnimusStructuralFacts {
    session_identity: Digest,
    projection: Projection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LatticeHistoryBinding {
    workflow_identity: Digest,
    machine_identity: Digest,
    run_identity: Digest,
    history_identity: Digest,
}

fn digest(seed: u8) -> Digest {
    Digest::from_bytes([seed; DIGEST_BYTES])
}

fn graph_and_projection() -> (HistoryGraph, Projection) {
    let limits = HistoryLimits::new(
        MAX_EVENTS,
        MAX_PARENTS,
        MAX_METADATA,
        MAX_LABEL,
        MAX_PROJECTION,
    )
    .expect("limits");
    let graph = HistoryGraph::new(limits).expect("graph");
    let admitted = graph
        .admit(EventDraft::v1(
            EventClass::Application,
            digest(PAYLOAD_SEED),
            Vec::new(),
            digest(SCHEMA_SEED),
        ))
        .expect("event");
    let projection = linear_lineage(
        &admitted.graph,
        admitted.event,
        &BTreeMap::new(),
        MAX_PROJECTION,
    )
    .expect("lineage");
    (admitted.graph, projection)
}

// r[verify choregraph.history.consumers.animus]
#[test]
fn animus_fixture_returns_only_opaque_structural_history() {
    let (_graph, projection) = graph_and_projection();
    let facts = AnimusStructuralFacts {
        session_identity: digest(SESSION_SEED),
        projection,
    };
    assert_eq!(facts.session_identity, digest(SESSION_SEED));
    assert_eq!(facts.projection.events().len(), 1);
    let debug = format!("{facts:?}");
    for forbidden in ["prompt", "provider", "compaction", "repair", "persistence"] {
        assert!(!debug.contains(forbidden));
    }
}

fn validate_lattice_binding(
    graph: &HistoryGraph,
    binding: LatticeHistoryBinding,
) -> Result<(), &'static str> {
    if binding.history_identity != graph.identity() {
        return Err("stale-history-identity");
    }
    Ok(())
}

// r[verify choregraph.history.consumers.lattice]
#[test]
fn lattice_fixture_binds_identities_without_runtime_or_storage_authority() {
    let (graph, _projection) = graph_and_projection();
    let binding = LatticeHistoryBinding {
        workflow_identity: digest(WORKFLOW_SEED),
        machine_identity: digest(MACHINE_SEED),
        run_identity: digest(RUN_SEED),
        history_identity: graph.identity(),
    };
    assert_eq!(validate_lattice_binding(&graph, binding), Ok(()));
    let stale = LatticeHistoryBinding {
        history_identity: digest(STALE_HISTORY_SEED),
        ..binding
    };
    assert_eq!(
        validate_lattice_binding(&graph, stale),
        Err("stale-history-identity")
    );
    let source = include_str!("../Cargo.toml");
    for forbidden in ["redb", "bounded-exec", "lattice", "animus", "tokio"] {
        assert!(!source.contains(forbidden));
    }
}
