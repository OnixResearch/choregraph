# Single Step Verification Specification

## Purpose

Defines the `single-step-verification` capability.

## Requirements

### Requirement: Production proof linkage

r[choregraph.proof.single_step.production_linkage] Formal specifications and proofs MUST refine the production transition functions rather than a separate transition implementation.

#### Scenario: Proof model diverges from production
- GIVEN a proof-only transition implementation that consumers do not call
- WHEN proof linkage is checked
- THEN the verification gate MUST fail.

### Requirement: Explicit mathematical specifications

r[choregraph.proof.single_step.specifications] The proof core MUST define mathematical specifications for admission, ordering, readiness, invariants, transitions, denial, and allowed intents.

#### Scenario: Required specification is missing
- GIVEN one maintained production decision without a corresponding specification
- WHEN proof coverage is checked
- THEN proof readiness MUST fail.

### Requirement: Admitted graph properties

r[choregraph.proof.single_step.graph] The proof rail MUST prove that admitted topology is well formed and that its topological order contains each node once while respecting every edge.

#### Scenario: Edge order is violated
- GIVEN an admitted edge from one node to another
- WHEN the topological-order theorem is applied
- THEN the source node MUST precede the target node.

### Requirement: Readiness soundness

r[choregraph.proof.single_step.readiness] The proof rail MUST prove that every node reported ready has all declared dependencies in the required completed state.

#### Scenario: Dependency is incomplete
- GIVEN a node with an incomplete dependency
- WHEN readiness is evaluated
- THEN the node MUST NOT satisfy the readiness theorem.

### Requirement: Transition determinism

r[choregraph.proof.single_step.determinism] The proof rail MUST prove that equal admitted machines, states, events, and explicit facts produce equal decisions.

#### Scenario: Equal inputs are evaluated
- GIVEN two equal explicit transition inputs
- WHEN `step` is applied to each input
- THEN both decisions MUST be equal.

### Requirement: Accepted-step preservation

r[choregraph.proof.single_step.preservation] The proof rail MUST prove that an accepted step from a well-formed machine and invariant state preserves the invariant and emits only allowed intents.

#### Scenario: Accepted transition is proved
- GIVEN `WellFormed(machine)` and `Invariant(machine, state)`
- WHEN `step` returns an accepted next state and intents
- THEN `Invariant(machine, next)` and `EffectsAllowed` MUST hold.

### Requirement: Denial preservation

r[choregraph.proof.single_step.denial] The proof rail MUST prove that a denied step returns no replacement state and no effect intents.

#### Scenario: Invalid event is denied
- GIVEN a valid state and an invalid event
- WHEN `step` denies the event
- THEN no next state or intent MUST exist in the decision.

### Requirement: Trellis reuse is revalidated

r[choregraph.proof.single_step.trellis] Reused Trellis artifacts MUST have exact source identities, compatible claims, and successful Choregraph-local proof checks.

#### Scenario: Trellis reference is cited without checking
- GIVEN a Trellis theorem reference without local proof linkage
- WHEN proof evidence is validated
- THEN the reference MUST NOT satisfy the Choregraph theorem.

### Requirement: Trusted surface is explicit

r[choregraph.proof.single_step.trust] The proof evidence MUST list trusted declarations, external assumptions, verifier identity, solver identity, compiler boundary, and unsupported claims.

#### Scenario: Hidden assumption is introduced
- GIVEN a new trusted declaration or unrestricted assumption
- WHEN the trusted-surface check runs
- THEN the proof gate MUST fail until the assumption is reviewed and recorded.

### Requirement: Negative proof fixtures

r[choregraph.proof.single_step.negative] The maintained rail MUST include fixtures that fail for expected violated obligations.

#### Scenario: Invalid denial emits an intent
- GIVEN a proof fixture where denial emits an intent
- WHEN the negative proof rail runs
- THEN verification MUST fail for the denial obligation.

### Requirement: Deterministic proof evidence

r[choregraph.proof.single_step.evidence] Proof evidence MUST bind source, theorem set, toolchain, flags, assumptions, and result through project-owned BLAKE3 identities.

#### Scenario: Source changes after proof
- GIVEN proof evidence for an earlier source identity
- WHEN current source is checked
- THEN stale evidence MUST fail closed.

### Requirement: Maintained proof validation

r[choregraph.proof.single_step.validation] Release checks MUST run the selected positive proofs and expected negative proof fixtures.

#### Scenario: Proof execution is skipped
- GIVEN a release candidate without current proof execution
- WHEN proof readiness is evaluated
- THEN the candidate MUST fail the proof gate.

### Requirement: Baseline is preserved

r[choregraph.proof.single_step.baseline] Proof-directed refactoring MUST preserve the accepted executable behavior corpus unless a semantic change is separately reviewed.

#### Scenario: Proof refactor changes a fixture result
- GIVEN an accepted baseline fixture
- WHEN pre-proof and post-proof behavior differs
- THEN the change MUST be rejected or reviewed as a semantic change.

### Requirement: Toolchain identity is pinned

r[choregraph.proof.single_step.toolchain] The maintained proof rail MUST pin Verus, Rust, solver, Trellis, and Nix identities.

#### Scenario: Proof tool is floating
- GIVEN proof evidence without an exact verifier identity
- WHEN evidence is checked
- THEN the evidence MUST be rejected.
