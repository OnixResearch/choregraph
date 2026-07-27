# Bounded Machine Semantics

## ADDED Requirements

### Requirement: Canonical bounded IR

r[choregraph.machine.ir] The core MUST define one canonical typed IR for immutable graph topology, local lifecycle rules, aggregate state, events, decisions, and effect intents.

#### Scenario: Machine is instantiated
- GIVEN a valid canonical machine specification
- WHEN admission succeeds
- THEN the resulting machine MUST expose immutable normalized facts for the transition core.

### Requirement: Explicit admission limits

r[choregraph.machine.bounds] Admission MUST enforce named limits for graph size, identifier size, event size, intent size, and canonical reference size before unsafe resource growth.

#### Scenario: Limit is exceeded
- GIVEN a graph or event that exceeds one selected limit
- WHEN admission or transition validation runs
- THEN the input MUST be rejected with a deterministic limit diagnostic.

### Requirement: Fail-closed graph admission

r[choregraph.machine.admission] Admission MUST reject empty or duplicate identifiers, duplicate edges, missing endpoints, self-loops, cycles, unsupported lifecycle profiles, malformed references, and undeclared operations.

#### Scenario: Malformed graph is supplied
- GIVEN malformed graph facts
- WHEN admission runs
- THEN no admitted machine MUST be produced.

### Requirement: Executable invariants

r[choregraph.machine.invariants] The core MUST define executable `WellFormed` and `Invariant` predicates over the same production types used by `step`.

#### Scenario: State has an unknown node
- GIVEN an admitted machine and a state with missing or extra node entries
- WHEN invariant validation runs
- THEN the state MUST be rejected.

### Requirement: Pure transition relation

r[choregraph.machine.step] The `step` relation MUST depend only on the admitted machine, current state, event, and explicit bounded facts.

#### Scenario: Ambient state changes
- GIVEN equal explicit transition inputs under different ambient process state
- WHEN `step` runs
- THEN both decisions MUST be identical.

### Requirement: Deterministic normalization

r[choregraph.machine.determinism] Admission and transition results MUST use stable normalization and ordering independent of source order.

#### Scenario: Equivalent inputs are reordered
- GIVEN semantically equal machine inputs in different source order
- WHEN admission and `step` run
- THEN diagnostics, next state, and intent ordering MUST be identical.

### Requirement: Denial has no effects

r[choregraph.machine.denial] A denied transition MUST preserve the caller-held state and MUST emit no effect intents.

#### Scenario: Terminal node is mutated
- GIVEN a node in a terminal state
- WHEN a later mutation event is supplied
- THEN `step` MUST deny the event without replacement state or intents.

### Requirement: Effect intents are declared data

r[choregraph.machine.effect_intents] Every emitted effect intent MUST match an exact bounded operation, resource, ability, request identity, and payload-reference declaration in the admitted machine.

#### Scenario: Transition requests an undeclared operation
- GIVEN a transition without a matching effect declaration
- WHEN `step` evaluates the event
- THEN the transition MUST emit no product effect intent.

### Requirement: Parameterized aggregate state

r[choregraph.machine.parameterized_state] The aggregate machine MUST represent state as bounded per-node data and MUST NOT require enumeration of the complete global product-state space.

#### Scenario: Graph gains another node
- GIVEN an admitted graph with a supported additional node
- WHEN machine state is constructed
- THEN representation size SHOULD grow with declared graph data rather than every theoretical state combination.

### Requirement: Compatibility baseline

r[choregraph.machine.compatibility_baseline] The implementation MUST inventory existing execution-graph admission, ordering, readiness, lifecycle, resume, critical-path, and bottleneck behavior before migration.

#### Scenario: Existing behavior differs
- GIVEN one inventoried compatibility fixture
- WHEN old and new decisions differ
- THEN the difference MUST be rejected or reviewed as an explicit semantic change.

### Requirement: Positive and negative validation

r[choregraph.machine.fixtures] Machine tests MUST include accepted behavior and malformed, bounded, premature, unknown, undeclared, and terminal failure behavior.

#### Scenario: Validation corpus runs
- GIVEN the reviewed fixture corpus
- WHEN maintained tests run
- THEN valid inputs MUST pass and invalid inputs MUST fail deterministically.

### Requirement: Pre-proof claim boundary

r[choregraph.machine.validation] Passing executable tests MUST NOT be reported as formal invariant or trace proof.

#### Scenario: Core tests pass
- GIVEN all executable tests pass
- WHEN evidence is emitted
- THEN evidence MUST label formal proof obligations as not yet discharged.
