# Concrete Effect Intents Specification

## ADDED Requirements

### Requirement: Versioned concrete-effect schema
r[choregraph.effect_v2.schema] Choregraph MUST add a distinct v2 machine schema for runtime-bound concrete intents and MUST preserve the meaning of existing v1 schemas.

#### Scenario: Existing v1 machine is loaded
r[choregraph.effect_v2.schema.scenario.v1]
- GIVEN a valid v1 machine artifact
- WHEN the versioned frontend and admission path process it
- THEN its accepted behavior and canonical artifact identity MUST remain unchanged.

### Requirement: Stable machine identity
r[choregraph.effect_v2.stable_machine] Runtime request and payload values MUST NOT change the identity of the admitted immutable workflow machine.

#### Scenario: Two runs resolve different payloads
r[choregraph.effect_v2.stable_machine.scenario.runs]
- GIVEN one admitted v2 machine and two runs with different concrete payload references
- WHEN each run requests its declared effect
- THEN both requests MUST use the same machine identity and distinct concrete intent content identities.

### Requirement: Content and occurrence identities are separate
r[choregraph.effect_v2.occurrence_identity] A Choregraph concrete-intent identity MUST identify canonical intent content and MUST NOT be treated as a run occurrence, attempt, replay, or deduplication identity.

#### Scenario: Two runs resolve equal intent content
r[choregraph.effect_v2.occurrence_identity.scenario.equal-content]
- GIVEN two runs that produce equal canonical Choregraph intent content
- WHEN content and downstream occurrence identities are derived
- THEN the Choregraph content identities MAY be equal while the Lattice-bound run and attempt identities MUST differ.

### Requirement: Static effect declarations
r[choregraph.effect_v2.declarations] Each v2 effect declaration MUST have one bounded unique declaration key, node, transition, operation, resource, ability, request binding, and payload binding.

#### Scenario: Declaration key is duplicated
r[choregraph.effect_v2.declarations.scenario.duplicate]
- GIVEN two declarations with the same declaration key
- WHEN machine admission runs
- THEN no admitted machine MUST be produced.

### Requirement: Exact and slot bindings
r[choregraph.effect_v2.bindings] Request and payload bindings MUST be explicit exact-reference or named runtime-slot bindings with bounded declared source references and MUST fail closed for unknown modes, slots, or sources.

#### Scenario: Event selects the wrong payload slot
r[choregraph.effect_v2.bindings.scenario.wrong-slot]
- GIVEN a declaration with one named payload slot and source reference
- WHEN an event supplies a concrete reference under another slot or source
- THEN the transition MUST be denied without an intent.

### Requirement: Canonical concrete references
r[choregraph.effect_v2.references] Concrete request and payload references MUST name the selected canonical packed Preserves profile and one domain-separated BLAKE3 identity over the complete versioned Preserves constructor.

#### Scenario: Non-canonical payload projection is supplied
r[choregraph.effect_v2.references.scenario.non-canonical]
- GIVEN JSON or text bytes that do not match the canonical packed Preserves identity
- WHEN the canonical adapter checks the reference
- THEN the reference MUST be rejected before it becomes a checked intent artifact.

### Requirement: Concrete intent production
r[choregraph.effect_v2.step] An accepted v2 transition MUST emit only concrete intents that preserve the matched declaration fields and the exact request and payload references supplied by the event.

#### Scenario: Declared runtime effect is requested
r[choregraph.effect_v2.step.scenario.accepted]
- GIVEN an invariant state, a matching declaration, and valid concrete references
- WHEN the production transition accepts the event
- THEN its intent MUST contain the matched declaration key, route, authority fields, slots, and exact references.

### Requirement: Bounded deterministic intent handling
r[choregraph.effect_v2.determinism] Admission and transition handling MUST enforce named bounds, permit each declaration at most once per event, reject duplicate concrete requests, and return deterministic intent order independent of source order.

#### Scenario: Equal requests use different source order
r[choregraph.effect_v2.determinism.scenario.order]
- GIVEN two semantically equal machines and events with reordered declarations and requests
- WHEN admission and transition processing run
- THEN diagnostics, state decisions, and concrete intent order MUST be identical.

### Requirement: Fail-closed denial
r[choregraph.effect_v2.denial] A malformed, undeclared, mismatched, duplicate, or oversized concrete request MUST produce no replacement state and no intent.

#### Scenario: Event names an unknown declaration
r[choregraph.effect_v2.denial.scenario.unknown]
- GIVEN a valid machine state and an event with an unknown declaration key
- WHEN the production transition processes the event
- THEN the decision MUST deny the event without replacement state or intents.

### Requirement: Safe deterministic diagnostics
r[choregraph.effect_v2.diagnostics] V2 admission and transition failures MUST use stable bounded diagnostic codes and MUST NOT expose raw request bytes, payload bytes, credentials, environment values, or host paths.

#### Scenario: Rejected request contains sensitive bytes
r[choregraph.effect_v2.diagnostics.scenario.redaction]
- GIVEN a rejected concrete request whose source data contains sensitive bytes
- WHEN safe diagnostics are rendered
- THEN persisted and operator-safe output MUST contain a stable code without the sensitive bytes.

### Requirement: Replay without dispatch authority
r[choregraph.effect_v2.replay] Replay MUST use the production transition relation and MUST discard every historical concrete intent from its output.

#### Scenario: Historical concrete intent is replayed
r[choregraph.effect_v2.replay.scenario.history]
- GIVEN a valid trace that originally emitted one concrete intent
- WHEN replay reconstructs machine state
- THEN replay MAY validate the historical references but MUST NOT return an intent or dispatch authority.

### Requirement: Single normalized transition kernel
r[choregraph.effect_v2.kernel] V1 and v2 public paths MUST normalize into one production transition kernel and MUST NOT keep separate machine-decision algorithms.

#### Scenario: V1 wrapper is inspected
r[choregraph.effect_v2.kernel.scenario.wrapper]
- GIVEN the maintained v1 compatibility path
- WHEN architecture and production-linkage checks run
- THEN the path MUST delegate to the same normalized kernel used by v2.

### Requirement: Typed Lattice v2 lowering
r[choregraph.effect_v2.nickel] The Nickel frontend MUST support a typed `lattice-workflow-v2` profile with explicit effect declarations and MUST keep `lattice-workflow-v1` behavior unchanged.

#### Scenario: Effectful v2 node omits its declaration
r[choregraph.effect_v2.nickel.scenario.missing]
- GIVEN an effectful Lattice v2 node without a required effect declaration
- WHEN contract evaluation and pure Rust lowering run
- THEN no admitted v2 machine MUST be produced.

### Requirement: Canonical interchange artifacts
r[choregraph.effect_v2.kamacite] Kamacite adapters MUST encode and decode canonical packed v2 machine and intent artifacts with exact BLAKE3 identity and linkage receipts.

#### Scenario: Intent bytes change after identity generation
r[choregraph.effect_v2.kamacite.scenario.stale]
- GIVEN a concrete intent receipt for earlier canonical bytes
- WHEN current artifact linkage is checked
- THEN the stale identity MUST fail closed.

### Requirement: Production-linked formal proof
r[choregraph.effect_v2.proof] Formal verification MUST prove declaration matching, exact reference preservation, bounds, determinism, invariant preservation, denial safety, and replay suppression over the production-called kernel.

#### Scenario: Proof intent changes the payload reference
r[choregraph.effect_v2.proof.scenario.mutation]
- GIVEN a negative proof fixture that emits a payload reference different from the accepted event
- WHEN the maintained Verus rail runs
- THEN verification MUST fail for the concrete-binding obligation.

### Requirement: Conversion-only Seaglass handoff
r[choregraph.effect_v2.seaglass] Seaglass MUST convert v2 consumer data and expose concrete intents without authorization, persistence, policy, filesystem, process, network, clock, or dispatch authority.

#### Scenario: Adapter adds an authorization dependency
r[choregraph.effect_v2.seaglass.scenario.authority]
- GIVEN a Seaglass source or manifest that imports Basalt, UCAN, or a runtime authority
- WHEN the architecture boundary check runs
- THEN the check MUST fail.

### Requirement: Exact Lattice handoff
r[choregraph.effect_v2.lattice_handoff] Choregraph MUST provide an exact source, schema, content-identity, source-reference, fixture, and non-claim handoff that lets Lattice bind workflow, run, policy, authority, persistence, and attempt facts downstream.

#### Scenario: Downstream invents an intent
r[choregraph.effect_v2.lattice_handoff.scenario.synthetic]
- GIVEN a Lattice path that builds authority data without a Choregraph concrete intent
- WHEN the downstream compatibility check runs
- THEN the path MUST NOT satisfy the v2 handoff contract.

### Requirement: Positive and negative fixture corpus
r[choregraph.effect_v2.fixtures] Maintained tests MUST include accepted exact, slot, mixed, ordered, replayed, and v1 cases plus malformed, duplicate, mismatched, oversized, stale, non-canonical, authority-import, and mutation failures.

#### Scenario: Only accepted requests are tested
r[choregraph.effect_v2.fixtures.scenario.negative]
- GIVEN a candidate without current negative concrete-intent fixtures
- WHEN release readiness is checked
- THEN the candidate MUST fail.

### Requirement: Exact release evidence
r[choregraph.effect_v2.release] Release evidence MUST bind current schema, source, canonical artifacts, proofs, negative fixtures, toolchains, adapters, downstream checks, and non-claims through exact BLAKE3 identities.

#### Scenario: Proof or downstream evidence is stale
r[choregraph.effect_v2.release.scenario.stale]
- GIVEN a candidate whose source differs from its proof or downstream evidence identity
- WHEN release verification runs
- THEN release readiness MUST fail closed.

### Requirement: Effect claim boundary
r[choregraph.effect_v2.non_claims] Passing checks MUST NOT claim request truth, payload availability, digest truth inside the pure core, arbitrary Nickel equivalence, authorization, persistence, dispatch, external success, exactly-once behavior, downstream correctness, or publication authority.

#### Scenario: Concrete intent artifact passes
r[choregraph.effect_v2.non_claims.scenario.scope]
- GIVEN one valid concrete intent artifact and passing local proof evidence
- WHEN results are reported
- THEN reporting MUST remain limited to the checked local declaration, reference, transition, artifact, and replay relations.
