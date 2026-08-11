# Machine Semantics Specification

## Purpose

Defines the `machine-semantics` capability.

## Requirements

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

### Requirement: Durable suspension uses an explicit lifecycle profile

r[choregraph.machine.suspension_profile] Choregraph MUST provide an opt-in versioned lifecycle profile that represents bounded suspended local state while preserving existing lifecycle-profile behavior and canonical identity unless the new profile is selected explicitly.

#### Scenario: New profile admits bounded suspension data

r[choregraph.machine.suspension_profile.valid]
- GIVEN a machine explicitly selects the supported durable-suspension profile and each suspension declaration has a bounded class and canonical reference identity
- WHEN machine admission runs
- THEN Choregraph MUST produce one admitted machine with deterministic profile and suspension identities
- AND the profile MUST add no approver, credential, clock, persistence, notification, or product-policy authority.

#### Scenario: Legacy or malformed profile is widened

r[choregraph.machine.suspension_profile.invalid]
- GIVEN an adapter infers suspension for a legacy profile or input supplies an unsupported profile, malformed reference, oversized class, duplicate declaration, or product approval payload
- WHEN admission runs
- THEN Choregraph MUST reject the widened or malformed input
- AND existing legacy profile bytes and decisions MUST remain unchanged.

### Requirement: Suspend and resume-signal transitions are pure and effect-free

r[choregraph.machine.suspension_transition] The production `step` relation MUST process admitted suspend and exact matching resume-signal events deterministically and MUST emit no product effect intent for either transition.

#### Scenario: Eligible node becomes suspended

r[choregraph.machine.suspension_transition.suspend]
- GIVEN an admitted durable-suspension machine, invariant-preserving state, eligible non-terminal node, and valid suspension class and reference
- WHEN `step` receives the suspend event
- THEN it MUST return a next state where only that node is suspended with the exact bounded facts
- AND it MUST emit no product effect intent.

#### Scenario: Matching signal releases suspended state

r[choregraph.machine.suspension_transition.resume]
- GIVEN a suspended node retains one exact profile, class, and reference and its dependencies remain admissible for resumption
- WHEN `step` receives one matching resume-signal event
- THEN it MUST move that node to the profile-defined resumed state and preserve the invariant
- AND it MUST emit no product effect intent.

#### Scenario: Invalid suspension transition is denied

r[choregraph.machine.suspension_transition.denied]
- GIVEN a suspend or resume event is terminal, nested, premature, wrong-node, wrong-class, wrong-reference, stale, duplicate, malformed, or outside a named limit
- WHEN `step` evaluates the event
- THEN it MUST deny without replacement state or effect intents
- AND it MUST return a deterministic diagnostic for the exact failed binding.

### Requirement: Suspended nodes wait for an exact signal

r[choregraph.machine.resume_signal] Choregraph MUST exclude suspended nodes from readiness and MUST return an explicit wait decision until one matching admitted resume signal changes the local state.

#### Scenario: Dependencies complete while node remains suspended

r[choregraph.machine.resume_signal.wait]
- GIVEN all graph dependencies for a suspended node are complete but no matching resume transition occurred
- WHEN readiness and resume decisions run
- THEN the node MUST remain non-ready and the resume decision MUST report an exact wait class
- AND dependency completion alone MUST NOT authorize execution.

#### Scenario: Mismatched signal cannot make work ready

r[choregraph.machine.resume_signal.mismatch]
- GIVEN a signal names another node, class, reference, profile, or stale suspension state
- WHEN transition, readiness, or resume evaluation runs
- THEN the signal MUST NOT change state or make any suspended node executable
- AND no fallback reference or signal source MAY be selected.

### Requirement: Suspension replay is deterministic and bounded

r[choregraph.machine.suspension_replay] Choregraph MUST bind suspension facts and signals into canonical trace identity and MUST replay valid suspended traces or return the first deterministic denial.

#### Scenario: Valid trace ends suspended and later resumes

r[choregraph.machine.suspension_replay.valid]
- GIVEN one admitted trace reaches suspended state and a later trace extension contains the exact matching resume signal
- WHEN bounded replay runs over each trace
- THEN the first trace MUST end validly suspended and the extended trace MUST reach the same resumed state as direct stepping
- AND equal normalized traces MUST produce equal replay and trace identities.

#### Scenario: Trace contains stale or reordered signal

r[choregraph.machine.suspension_replay.invalid]
- GIVEN a trace signals resume before suspension, after terminal state, with stale facts, or with a duplicate or reordered binding
- WHEN replay runs
- THEN it MUST return the first exact denial and no dispatchable intents
- AND it MUST NOT reinterpret the trace as successful approval or execution.

### Requirement: Suspension properties use production-linked proofs

r[choregraph.machine.suspension_proof] Choregraph MUST extend executable invariants and production-linked Verus proofs for admitted suspension, matching resume, denied signals, terminal immutability, no-effect denial, and finite replay.

#### Scenario: Production suspension proofs pass

r[choregraph.machine.suspension_proof.valid]
- GIVEN the reviewed production transition source, executable invariants, Verus specifications, and exact proof cohort
- WHEN maintained proof and linkage rails run
- THEN they MUST establish the declared single-step and finite-trace suspension properties for that exact source
- AND the release evidence MUST bind source, profile, proof, and execution identities.

#### Scenario: Proof path diverges or omits suspension

r[choregraph.machine.suspension_proof.invalid]
- GIVEN proof code uses another transition implementation, omits a suspended state or denial case, adds an unrestricted assumption, cites stale source, or does not execute
- WHEN proof linkage and readiness run
- THEN the proof gate MUST fail
- AND executable tests alone MUST NOT be reported as formal suspension proof.

### Requirement: Product suspension authority remains downstream

r[choregraph.machine.suspension_boundary] Choregraph MUST treat suspension classes and references as opaque transition facts and MUST NOT claim approver identity, authorization, freshness, timeout, persistence, notification, credential, or downstream execution authority.

#### Scenario: Consumer supplies verified product facts

r[choregraph.machine.suspension_boundary.valid]
- GIVEN a consumer independently validates its product request, authority, persistence, and signal facts before constructing a matching event
- WHEN Choregraph accepts the transition
- THEN the accepted result MUST remain limited to transition shape, reference equality, invariant preservation, and declared effect-intent facts
- AND the consumer MUST retain every product interpretation and effect decision.

#### Scenario: Matching reference is promoted into authority

r[choregraph.machine.suspension_boundary.overclaim]
- GIVEN a report claims that matching opaque references prove human approval, authorization, persistence durability, callback truth, timeout correctness, or safe downstream effects
- WHEN boundary validation runs
- THEN the report MUST be rejected or narrowed
- AND passing Choregraph evidence MUST NOT satisfy product, lifecycle, production, or release authority by itself.
