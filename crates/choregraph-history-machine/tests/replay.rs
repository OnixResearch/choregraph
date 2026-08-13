#![allow(
    unknown_lints,
    assertion_density,
    explicit_defaults,
    clippy::expect_used,
    function_length,
    missing_const_fn,
    no_panic,
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
    Action, AdmissionLimits, AdmittedMachine, ConsumedCount, Event, MachineSpec, MachineSpecV2,
    NodeSpec, ReplayBindings, ReplayDecision, ReplayLimits, TraceBinding,
};
use choregraph_history::{
    DIGEST_BYTES, Digest, EventClass, EventDraft, HistoryGraph, HistoryLimits, ParentLink,
    linear_lineage,
};
use choregraph_history_machine::{
    BoundEventV1, BoundEventV2, MachineHistoryError, ReplayHistoryInputV1, ReplayHistoryInputV2,
    concrete_event_payload_identity, machine_event_payload_identity, replay_history,
    replay_history_concrete,
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

fn limits() -> HistoryLimits {
    match HistoryLimits::new(
        MAX_EVENTS,
        MAX_PARENTS,
        MAX_METADATA,
        MAX_LABEL,
        MAX_PROJECTION,
    ) {
        Ok(value) => value,
        Err(error) => panic!("history limits failed: {error:?}"),
    }
}

fn bindings() -> ReplayBindings {
    ReplayBindings::new(
        TraceBinding::from_bytes([MACHINE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
        TraceBinding::from_bytes([STATE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
    )
}

fn stale_bindings() -> ReplayBindings {
    ReplayBindings::new(
        TraceBinding::from_bytes([STALE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
        bindings().initial_state,
    )
}

fn machine_v1() -> AdmittedMachine {
    let spec = MachineSpec::new(vec![NodeSpec::new(NODE, Vec::new())], Vec::new());
    match AdmittedMachine::admit(spec, AdmissionLimits::default()) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("v1 admission failed: {diagnostics:?}"),
    }
}

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

fn history_v1(events: &[Event], schema: Digest) -> (HistoryGraph, Vec<BoundEventV1>) {
    let mut graph = match HistoryGraph::new(limits()) {
        Ok(value) => value,
        Err(error) => panic!("history graph failed: {error:?}"),
    };
    let mut parent = None;
    let mut bound = Vec::new();
    for event in events {
        let parents = parent
            .map(|event| {
                vec![ParentLink {
                    event,
                    role: "previous".to_owned(),
                }]
            })
            .unwrap_or_default();
        let draft = EventDraft::v1(
            EventClass::MachineV1,
            machine_event_payload_identity(event),
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
    let decision = replay_history(ReplayHistoryInputV1 {
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
        replay_history(ReplayHistoryInputV1 {
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
        Err(MachineHistoryError::StaleMachineIdentity)
    );
    assert!(matches!(
        base(digest(MACHINE_SEED), digest(STALE_SCHEMA_SEED), bindings()),
        Err(MachineHistoryError::StaleSchemaIdentity { .. })
    ));
    assert_eq!(
        base(digest(MACHINE_SEED), schema, stale_bindings()),
        Err(MachineHistoryError::StaleBinding)
    );

    let mut changed = bound.clone();
    changed[0].event.node_id = "other".to_owned();
    let payload_result = replay_history(ReplayHistoryInputV1 {
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
        Err(MachineHistoryError::PayloadIdentityMismatch { .. })
    ));

    let application_graph = HistoryGraph::new(limits()).expect("application graph");
    let application = application_graph
        .admit(EventDraft::v1(
            EventClass::Application,
            machine_event_payload_identity(&events[0]),
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
    let unsupported = replay_history(ReplayHistoryInputV1 {
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
        Err(MachineHistoryError::UnsupportedEventClass { .. })
    ));
}

#[test]
fn concrete_lineage_delegates_to_production_concrete_replay() {
    let machine = machine_v2();
    let initial_state = machine.initial_state();
    let schema = digest(SCHEMA_V2_SEED);
    let event = choregraph_core::ConcreteEvent::new(NODE, Action::MarkReady);
    let graph = HistoryGraph::new(limits()).expect("graph");
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
    let result = replay_history_concrete(ReplayHistoryInputV2 {
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
