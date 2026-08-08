# Role Choreography Specification Delta

## ADDED Requirements

### Requirement: Role choreography source is typed and bounded

r[choregraph.role_choreography.source] Choregraph MUST define a typed Nickel role-choreography profile for schema version, declared roles, role kinds, owned values, initial owners, transfers, choices, termination, named limits, and required non-claims.

#### Scenario: Complete role protocol is admitted by Nickel

r[choregraph.role_choreography.source.valid]
- GIVEN a source declares every role, value, owner, transfer, branch, limit, and non-claim required by the selected profile
- WHEN Nickel contract evaluation runs
- THEN the evaluated value MUST contain the complete typed protocol input for Rust revalidation.

#### Scenario: Source contains an undeclared role

r[choregraph.role_choreography.source.undeclared]
- GIVEN a transfer or choice names a role absent from the declaration set
- WHEN contract evaluation or Rust revalidation runs
- THEN admission MUST fail before global or local artifact construction.

### Requirement: Role choreography enforces named limits

r[choregraph.role_choreography.bounds] Admission MUST enforce named limits for role count, value count, step count, branch count, nesting depth, identifier size, import size, diagnostic size, and canonical output size without caller-controlled widening.

#### Scenario: Protocol exceeds one selected limit

r[choregraph.role_choreography.bounds.exceeded]
- GIVEN a source exceeds any selected role-choreography limit
- WHEN admission runs
- THEN it MUST stop before the next semantic stage
- AND it MUST return one bounded deterministic limit diagnostic.

### Requirement: Rust revalidates evaluated choreography data

r[choregraph.role_choreography.admission] A pure Rust core MUST reject unknown fields, duplicate or empty identities, self-transfer, stale ownership, duplicate or unsorted labels, malformed continuations, invalid role kinds, and every value outside selected limits before projection.

#### Scenario: Evaluator output changes a transfer owner

r[choregraph.role_choreography.admission.stale_owner]
- GIVEN evaluated data names a source that does not own the selected value
- WHEN Rust admission runs
- THEN no admitted global choreography MUST be produced.

### Requirement: Role kinds remain declarative

r[choregraph.role_choreography.roles] Every declared role MUST have exactly one native or external kind. An external role MUST bind exactly one immutable contract identity and input and output schema identities. Role declarations MUST NOT grant host, credential, capability, adapter, transport, or effect authority.

#### Scenario: External role has one exact contract

r[choregraph.role_choreography.roles.external]
- GIVEN an external role names one supported immutable contract and exact schemas
- WHEN role admission runs
- THEN the role MAY be admitted as external protocol data
- AND no runtime authority MUST be inferred.

#### Scenario: Role mixes native and external settings

r[choregraph.role_choreography.roles.mixed]
- GIVEN one role declares both native and external kinds or omits its external contract
- WHEN role admission runs
- THEN admission MUST fail with a stable role-kind diagnostic.

### Requirement: Lowering uses the selected Trellis model

r[choregraph.role_choreography.lowering] The pure choreography adapter MUST lower admitted role, value, transfer, choice, and termination data into the exact selected Trellis global choreography and ownership model without dropping or synthesizing semantic fields.

#### Scenario: Admitted source lowers completely

r[choregraph.role_choreography.lowering.valid]
- GIVEN strict Rust admission succeeds
- WHEN pure lowering runs
- THEN every declared role, value, owner, transfer, label, payload tag, choice, and continuation MUST appear in the exact Trellis input.

#### Scenario: Lowering filters a required child

r[choregraph.role_choreography.lowering.dropped]
- GIVEN lowering would omit a transfer, branch, value, role, or continuation
- WHEN lowering validation runs
- THEN the result MUST fail before projectability evaluation.

### Requirement: Trellis semantics are pinned and revalidated

r[choregraph.role_choreography.trellis] Choregraph MUST bind one exact Trellis revision, selected source identities, theorem set, assumptions, toolchain, and license. Directly reused or adapted semantics MUST pass Choregraph-local conformance or proof checks.

#### Scenario: Trellis theorem is cited without local linkage

r[choregraph.role_choreography.trellis.unlinked]
- GIVEN a projection result cites a Trellis theorem without current source identity and Choregraph-local evidence
- WHEN projection evidence is validated
- THEN the theorem reference MUST NOT satisfy the Choregraph projection claim.

### Requirement: Endpoint projection is exact

r[choregraph.role_choreography.projection] For every admitted projectable global choreography, Choregraph MUST use the selected Trellis projectability and endpoint projection semantics to derive each local role protocol. Local peers, labels, payload tags, value identities, branches, and continuations MUST match exactly.

#### Scenario: Projectable choice emits exact local views

r[choregraph.role_choreography.projection.valid]
- GIVEN an admitted projectable choice with one decider and multiple roles
- WHEN endpoint projection runs
- THEN the decider and every other role MUST receive the exact Trellis local view for that role.

#### Scenario: Branches cannot merge

r[choregraph.role_choreography.projection.conflict]
- GIVEN one non-decider has incompatible projected branch behavior
- WHEN projectability runs
- THEN projection MUST fail
- AND no canonical local artifact set MUST be emitted.

### Requirement: Projection output is complete

r[choregraph.role_choreography.complete_set] A successful projection MUST contain exactly one canonical local artifact for every declared role in normalized role-identity order. Missing, duplicate, extra, or reordered semantic entries MUST fail complete-set admission.

#### Scenario: One local role artifact is missing

r[choregraph.role_choreography.complete_set.missing]
- GIVEN a global protocol declares roles `A`, `B`, and `C`
- WHEN a candidate local set contains only `A` and `B`
- THEN complete-set admission MUST fail
- AND no passing projection receipt MUST be produced.

### Requirement: Choreography artifacts use canonical Preserves

r[choregraph.role_choreography.artifacts] Choregraph MUST define versioned canonical Preserves artifacts for the admitted global protocol, each local protocol, the ordered complete local set, and the projection receipt. JSON MUST remain a review or compatibility projection.

#### Scenario: Canonical local set is encoded

r[choregraph.role_choreography.artifacts.valid]
- GIVEN successful projection produced a complete normalized local set
- WHEN the Kamacite adapter encodes the set
- THEN canonical packed Preserves bytes MUST be the authority for its identity.

### Requirement: Projection identities bind every semantic stage

r[choregraph.role_choreography.identity] Domain-separated BLAKE3 identities MUST bind the source envelope, imports, evaluator output, strict DTO, global artifact, every local artifact, complete local set, Trellis inputs, theorem set, lowerer, schemas, bounds, diagnostics, toolchain, and non-claims.

#### Scenario: One local continuation changes

r[choregraph.role_choreography.identity.changed]
- GIVEN one local artifact continuation differs from the recorded projection
- WHEN identity linkage runs
- THEN the local and complete-set identities MUST differ
- AND the prior projection receipt MUST be stale.

### Requirement: Choreography diagnostics are structured and safe

r[choregraph.role_choreography.diagnostics] Core diagnostics MUST use stable issue codes, severity, one source-relative primary span, ordered source-relative secondary spans, bounded notes, bounded hints, and optional bounded safe replacements. Safe output MUST omit raw secrets, absolute host paths, environment values, evaluator internals, and unbounded source excerpts.

#### Scenario: Projection conflict has multiple source sites

r[choregraph.role_choreography.diagnostics.multispan]
- GIVEN incompatible branches originate at distinct source-relative spans
- WHEN the core reports the conflict
- THEN the diagnostic MUST identify one primary span and the relevant ordered secondary spans
- AND rendering choices MUST NOT change the semantic diagnostic identity.

#### Scenario: Evaluator diagnostic contains a host path

r[choregraph.role_choreography.diagnostics.redacted]
- GIVEN a raw evaluator failure contains an absolute path or imported secret text
- WHEN safe diagnostics are persisted or displayed
- THEN that content MUST be omitted or replaced with bounded non-sensitive context.

### Requirement: Wyzer use is selective and recorded

r[choregraph.role_choreography.reference] Choregraph MUST record the exact reviewed Wyzer revision, MIT license disposition confirmed by the author to the project operator, selected concepts, adopted source fragments, and rejected runtime components.

#### Scenario: Wyzer source fragment is adopted

r[choregraph.role_choreography.reference.adopted]
- GIVEN a reviewed MIT Wyzer source fragment is selected for reuse
- WHEN it enters Choregraph
- THEN provenance MUST identify the exact source and adaptation
- AND Choregraph tests, architecture checks, Octet, and claim-boundary checks MUST cover it.

### Requirement: Compiler authority remains bounded

r[choregraph.role_choreography.boundary] Passing source admission, projectability, projection, identity, or proof-reference checks MUST NOT claim runtime execution, host placement, transport delivery, persistence, authorization, external-role correctness, exactly-once transfer, deadlock freedom under faults, production readiness, or release eligibility.

#### Scenario: Projectable protocol is presented as delivered

r[choregraph.role_choreography.boundary.delivery]
- GIVEN every compiler and proof-reference check passes
- WHEN evidence is promoted to successful network delivery
- THEN the claim MUST be rejected as outside Choregraph authority.

### Requirement: Role choreography has positive and negative fixtures

r[choregraph.role_choreography.fixtures] Maintained fixtures MUST pair valid transfer, choice, offer, ownership, native-role, external-role, artifact, and identity cases with malformed, bounded, stale, mismatched, incomplete, leaking, and overclaim cases.

#### Scenario: Fixture corpus runs

r[choregraph.role_choreography.fixtures.coverage]
- GIVEN the reviewed role-choreography corpus
- WHEN maintained checks run
- THEN valid cases MUST produce exact global and complete local artifacts
- AND invalid cases MUST fail at their first required boundary.

### Requirement: Role choreography has maintained validation

r[choregraph.role_choreography.validation] Maintained checks MUST cover Nickel contracts, strict admission, Trellis linkage, projection conformance, canonical artifacts, identities, diagnostics, positive and negative fixtures, architecture boundaries, Octet, Cairn, Verus, and Nix.

#### Scenario: Projection conformance is stale

r[choregraph.role_choreography.validation.stale]
- GIVEN Trellis source, theorem set, lowerer, schema, or fixture identities change
- WHEN maintained validation runs
- THEN current projection evidence MUST fail until regenerated and revalidated.
