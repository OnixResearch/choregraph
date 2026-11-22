#![allow(
    unknown_lints,
    assertion_density,
    explicit_defaults,
    clippy::expect_used,
    function_length,
    missing_const_fn,
    no_unwrap,
    non_trait_imports,
    numeric_units,
    unbounded_collection_growth,
    reason = "bounded positive and negative replay fixtures require direct setup and event accumulation"
)]

// r[verify choregraph.history.machine_replay]
// r[verify choregraph.history.machine_replay.valid]
// r[verify choregraph.history.machine_replay.rejects]
use std::collections::BTreeMap;

use choregraph_core::{
    concrete::machine::MachineSpecV2, model::automaton::AdmittedMachine,
    model::automaton::MachineSpec, model::capacity::AdmissionLimits, model::dispatch::Event,
    model::graph::NodeSpec, model::history::ConsumedCount, model::history::ReplayBindings,
    model::history::ReplayDecision, model::history::ReplayLimits, model::history::TraceBinding,
    model::semantics::Action,
};
use choregraph_history::{
    dag::Graph,
    identity::{DIGEST_BYTES, Digest},
    model::{EventClass, EventDraft, Limits, ParentLink},
    traversal::linear_lineage,
};
use choregraph_history_machine::{
    BoundEventV1, BoundEventV2, Error, ReplayInputV1, ReplayInputV2,
    concrete_event_payload_identity, event_payload_identity, replay, replay_concrete,
};

const MAX_EVENTS: u32 = 16;
const MAX_PARENTS: u32 = 4;
const MAX_METADATA: u32 = 4;
const MAX_LABEL: u32 = 32;
const MAX_PROJECTION: u32 = 16;
const NODE: &str = "node";
const MACHINE_SEED: u8 = 31;
const STALE_MACHINE_SEED: u8 = 32;
const SCHEMA_V1_SEED: u8 = 41;
const SCHEMA_V2_SEED: u8 = 42;
const STALE_SCHEMA_SEED: u8 = 43;
const MACHINE_BINDING_BYTE: u8 = 0x51;
const STATE_BINDING_BYTE: u8 = 0x52;
const STALE_BINDING_BYTE: u8 = 0x53;
const TRACE_EVENT_LIMIT: u64 = 16;
const COMPLETE_EVENT_COUNT: u64 = 3;

fn digest(seed: u8) -> Digest {
    Digest::from_bytes([seed; DIGEST_BYTES])
}

fn limits() -> Limits {
    Limits {
        max_events: MAX_EVENTS,
        max_parents: MAX_PARENTS,
        max_metadata_entries: MAX_METADATA,
        max_label_bytes: MAX_LABEL,
        max_projection_events: MAX_PROJECTION,
    }
}

fn bindings() -> ReplayBindings {
    ReplayBindings::new(
        TraceBinding::from_bytes(
            [MACHINE_BINDING_BYTE; choregraph_core::model::history::TRACE_BINDING_BYTES],
        ),
        TraceBinding::from_bytes(
            [STATE_BINDING_BYTE; choregraph_core::model::history::TRACE_BINDING_BYTES],
        ),
    )
}

fn stale_bindings() -> ReplayBindings {
    ReplayBindings::new(
        TraceBinding::from_bytes(
            [STALE_BINDING_BYTE; choregraph_core::model::history::TRACE_BINDING_BYTES],
        ),
        bindings().initial_state,
    )
}

#[allow(no_panic, reason = "this test helper reports failed fixture setup")]
fn machine_v1() -> AdmittedMachine {
    let spec = MachineSpec::new(vec![NodeSpec::new(NODE, Vec::new())], Vec::new());
    match AdmittedMachine::admit(spec, AdmissionLimits::default()) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("v1 admission failed: {diagnostics:?}"),
    }
}

#[allow(no_panic, reason = "this test helper reports failed fixture setup")]
fn machine_v2() -> AdmittedMachine {
    let spec = MachineSpecV2::new(vec![NodeSpec::new(NODE, Vec::new())], Vec::new());
    match AdmittedMachine::admit_v2(spec, AdmissionLimits::default()) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("v2 admission failed: {diagnostics:?}"),
    }
}

fn events_v1() -> Vec<Event> {
    vec![
        Event::new(NODE, Action::MarkReady),
        Event::new(NODE, Action::Start),
        Event::new(NODE, Action::Complete),
    ]
}

#[allow(no_panic, reason = "this test helper reports failed fixture setup")]
fn history_v1(events: &[Event], schema: Digest) -> (Graph, Vec<BoundEventV1>) {
    let mut graph = match Graph::new(limits()) {
        Ok(value) => value,
        Err(error) => panic!("history graph failed: {error:?}"),
    };
    let mut parent = None;
    let mut bound = Vec::new();
    let roles =
        std::iter::once(None).chain(events.iter().skip(1).map(|_| Some("previous".to_owned())));
    for (event, role) in events.iter().zip(roles) {
        let parents = parent
            .zip(role)
            .map(|(event, role)| vec![ParentLink { event, role }])
            .unwrap_or_default();
        let draft = EventDraft::v1(
            EventClass::MachineV1,
            event_payload_identity(event),
            parents,
            schema,
        );
        let admission = match graph.admit(draft) {
            Ok(value) => value,
            Err(error) => panic!("history admission failed: {error:?}"),
        };
        parent = Some(admission.event);
        bound.push(BoundEventV1 {
            history_event: admission.event,
            event: event.clone(),
        });
        graph = admission.graph;
    }
    (graph, bound)
}

#[test]
fn aggregate_lineage_delegates_to_production_replay_without_intents() {
    let machine = machine_v1();
    let initial_state = machine.initial_state();
    let events = events_v1();
    let schema = digest(SCHEMA_V1_SEED);
    let (graph, bound) = history_v1(&events, schema);
    let head = bound
        .last()
        .map(|event| event.history_event)
        .expect("non-empty bound events");
    let lineage = linear_lineage(&graph, head, &BTreeMap::new(), MAX_PROJECTION)
        .expect("linear machine lineage");
    let decision = replay(ReplayInputV1 {
        graph: &graph,
        lineage: &lineage,
        machine: &machine,
        initial_state: &initial_state,
        expected_machine_identity: digest(MACHINE_SEED),
        supplied_machine_identity: digest(MACHINE_SEED),
        expected_schema_identity: schema,
        expected_bindings: bindings(),
        trace_bindings: bindings(),
        events: &bound,
        limits: ReplayLimits {
            max_events: TRACE_EVENT_LIMIT,
        },
    });
    assert!(matches!(
        decision,
        Ok(ReplayDecision::Complete {
            consumed,
            ..
        }) if consumed == ConsumedCount::new(COMPLETE_EVENT_COUNT)
    ));
}

#[test]
fn stale_machine_schema_binding_payload_and_event_class_fail_before_replay() {
    let machine = machine_v1();
    let initial_state = machine.initial_state();
    let events = events_v1();
    let schema = digest(SCHEMA_V1_SEED);
    let (graph, bound) = history_v1(&events, schema);
    let head = bound.last().map(|event| event.history_event).expect("head");
    let lineage = linear_lineage(&graph, head, &BTreeMap::new(), MAX_PROJECTION).expect("lineage");
    let base = |supplied_machine_identity, expected_schema_identity, trace_bindings| {
        replay(ReplayInputV1 {
            graph: &graph,
            lineage: &lineage,
            machine: &machine,
            initial_state: &initial_state,
            expected_machine_identity: digest(MACHINE_SEED),
            supplied_machine_identity,
            expected_schema_identity,
            expected_bindings: bindings(),
            trace_bindings,
            events: &bound,
            limits: ReplayLimits {
                max_events: TRACE_EVENT_LIMIT,
            },
        })
    };
    assert_eq!(
        base(digest(STALE_MACHINE_SEED), schema, bindings()),
        Err(Error::StaleMachineIdentity)
    );
    assert!(matches!(
        base(digest(MACHINE_SEED), digest(STALE_SCHEMA_SEED), bindings()),
        Err(Error::StaleSchemaIdentity { .. })
    ));
    assert_eq!(
        base(digest(MACHINE_SEED), schema, stale_bindings()),
        Err(Error::StaleBinding)
    );

    let mut changed = bound.clone();
    changed[0].event.node_id = "other".to_owned();
    let payload_result = replay(ReplayInputV1 {
        graph: &graph,
        lineage: &lineage,
        machine: &machine,
        initial_state: &initial_state,
        expected_machine_identity: digest(MACHINE_SEED),
        supplied_machine_identity: digest(MACHINE_SEED),
        expected_schema_identity: schema,
        expected_bindings: bindings(),
        trace_bindings: bindings(),
        events: &changed,
        limits: ReplayLimits {
            max_events: TRACE_EVENT_LIMIT,
        },
    });
    assert!(matches!(
        payload_result,
        Err(Error::PayloadIdentityMismatch { .. })
    ));

    let application_graph = Graph::new(limits()).expect("application graph");
    let application = application_graph
        .admit(EventDraft::v1(
            EventClass::Application,
            event_payload_identity(&events[0]),
            Vec::new(),
            schema,
        ))
        .expect("application event");
    let application_bound = [BoundEventV1 {
        history_event: application.event,
        event: events[0].clone(),
    }];
    let application_lineage = linear_lineage(
        &application.graph,
        application.event,
        &BTreeMap::new(),
        MAX_PROJECTION,
    )
    .expect("application lineage");
    let unsupported = replay(ReplayInputV1 {
        graph: &application.graph,
        lineage: &application_lineage,
        machine: &machine,
        initial_state: &initial_state,
        expected_machine_identity: digest(MACHINE_SEED),
        supplied_machine_identity: digest(MACHINE_SEED),
        expected_schema_identity: schema,
        expected_bindings: bindings(),
        trace_bindings: bindings(),
        events: &application_bound,
        limits: ReplayLimits {
            max_events: TRACE_EVENT_LIMIT,
        },
    });
    assert!(matches!(
        unsupported,
        Err(Error::UnsupportedEventClass { .. })
    ));
}

#[test]
fn concrete_lineage_delegates_to_production_concrete_replay() {
    let machine = machine_v2();
    let initial_state = machine.initial_state();
    let schema = digest(SCHEMA_V2_SEED);
    let event = choregraph_core::concrete::event::ConcreteEvent::new(NODE, Action::MarkReady);
    let graph = Graph::new(limits()).expect("graph");
    let admission = graph
        .admit(EventDraft::v1(
            EventClass::MachineV2,
            concrete_event_payload_identity(&event),
            Vec::new(),
            schema,
        ))
        .expect("history event");
    let graph = admission.graph;
    let bound = [BoundEventV2 {
        history_event: admission.event,
        event,
    }];
    let lineage =
        linear_lineage(&graph, admission.event, &BTreeMap::new(), MAX_PROJECTION).expect("lineage");
    let result = replay_concrete(ReplayInputV2 {
        graph: &graph,
        lineage: &lineage,
        machine: &machine,
        initial_state: &initial_state,
        expected_machine_identity: digest(MACHINE_SEED),
        supplied_machine_identity: digest(MACHINE_SEED),
        expected_schema_identity: schema,
        expected_bindings: bindings(),
        trace_bindings: bindings(),
        events: &bound,
        limits: ReplayLimits {
            max_events: TRACE_EVENT_LIMIT,
        },
    });
    assert!(matches!(result, Ok(ReplayDecision::Complete { .. })));
}
