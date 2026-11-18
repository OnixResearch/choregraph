# Graph Construction Delta

## ADDED Requirements

### Requirement: Captured modules and forced export

r[choregraph.construction.captured] The supported profile MUST evaluate an explicit immutable captured entry with exact shipped library and contract bytes at logical `choregraph/graph-construction-v1.ncl` and `choregraph/graph-construction-contracts-v1.ncl`. The `finish` result MUST be a contracted JSON candidate with schema `choregraph.graph-construction.v1`; no unused lazy check field MAY stand in for contract enforcement. Existing restricted CLI and Lattice v1/v2 meanings MUST remain unchanged.

#### Scenario: Contract or module changes
- GIVEN a missing contract module, malformed nested exported value or changed shipped bytes
- WHEN captured evaluation or native admission runs
- THEN no construction output MUST be accepted.

### Requirement: Finite typed composition

r[choregraph.construction.composition] The library MUST provide operation, template, instantiate, input, output, connect, identity, sequence, parallel, wire, bounded generate and finish constructors. A handle MUST select an immediate child in the same root; reusing it MUST NOT create a new instance. Native compilation MUST enforce exact ordered typed boundaries, explicit Unit, registry-selected copy/drop rules, direction, scope, arity, uniqueness and acyclicity.

#### Scenario: Two instances and one reused handle
- GIVEN a reusable template instantiated twice and a separately reused output handle
- WHEN elaboration runs
- THEN the instances MUST have disjoint scoped identities, while handle reuse MUST target only its existing child and MUST obey copyability.

#### Scenario: Sequence across Unit-only steps
- GIVEN closed nonempty operations with empty data boundaries and Unit ordering
- WHEN sequence lowers
- THEN all-to-all exit/entry control dependencies MUST remain in v3 and in machine readiness, with no runtime identity node.

### Requirement: Exact registry and expression semantics

r[choregraph.construction.registry] A host-selected versioned registry MUST bind target/profile, exact implementation and operation signatures, named types including Unit, configuration field value/reference/operator rules and exact effect-rule tuples. Author labels MUST NOT select registry authority. Expressions MUST be structured literal/parameter/reference/apply records; slot and effect references MUST be scoped and rewritten, and exact content references MUST remain exact. Lattice MUST revalidate actual expressions and effect routing under its selected implementation.

The registry MUST explicitly declare ordered typed invocation reference roles, separately from ordinary data ports. Native construction and materialized readmission MUST preserve and validate those roles; invocation references MUST name the owning node and a selected role without inventing data ports. Lattice MUST resolve approved dispatch-request/resolved-payload roles from actual host-generated canonical references, never echo expected identities. Exact external resource and ability identities MUST remain unchanged across instantiation.

#### Scenario: Mismatched operation effect or forbidden expression
- GIVEN a candidate with an undeclared effect tuple, disallowed config reference/operator or structured expression where the registry permits literal data only
- WHEN native admission runs
- THEN it MUST reject rather than assume purity or reinterpret literal data as source code. Lattice MUST independently parse any string its selected implementation interprets as an expression and reject references outside the structured remapping/binding set.

### Requirement: Bounded elaboration and native admission

r[choregraph.construction.bounds] Native compilation MUST bound candidate bytes, nodes, ports, edges, effects, instances/depth, boundaries, identifiers, origins, parameters and output bytes before expansion/allocation; MUST emit typed routed Lattice projection v3 with separate data and control edges; and MUST use existing machine admission and unchanged kernel transition behavior. Root boundaries MUST be closed.

#### Scenario: Expanding template exceeds a limit
- GIVEN compact repeated template data whose expansion exceeds an explicit native ceiling
- WHEN compilation runs
- THEN it MUST fail before excess allocation or artifact publication.

### Requirement: Full compilation binding and honest attribution

r[choregraph.construction.binding] The binding MUST associate exact captured module bytes and observed evaluator closure, candidate/parameters, shipped library/contracts, registry implementation, elaborator, expanded graph/projection, lowerer, admitted machine, limits, origins and artifact. Validation MUST recompute the complete association. Supplied logical source locations MUST be checked against exact captured UTF-8 bytes; unavailable locations MUST remain unavailable; diagnostic origins MUST NOT redefine semantic identity.

#### Scenario: Stale evaluation or false location
- GIVEN changed module bytes, registry implementation, graph/output or an out-of-range/non-boundary location
- WHEN validation or admission runs
- THEN the result MUST fail without treating an author location as verified provenance.

### Requirement: Conditional laws and evidence boundaries

r[choregraph.construction.laws] Observer comparisons MUST state closed, well-typed, pure/disjoint and matching-registry premises. Identity/association claims MUST preserve explicit typed boundaries and dependency edges. General sequence/parallel interchange MUST NOT be claimed when all-to-all barriers change dependencies. Existing kernel proofs apply only after admission; receipts MUST NOT claim Nickel source semantics, evaluator correctness, effect authority or Lattice product readiness.

#### Scenario: Barrier changes dependency structure
- GIVEN sequence regrouping that introduces a different all-to-all exit/entry relation
- WHEN a law control compares admitted outputs
- THEN it MUST report nonequivalence, not a general interchange theorem.

### Requirement: Maintained adoption evidence

r[choregraph.construction.validation] Positive/negative Nickel exports and native tests MUST cover nested instantiation, shared handles, generated fork/join, Unit readiness, limits, registry/copy/drop/effects/references, scope/direction/type/cycle, origins, stale bindings and pure-law premises. Lattice adoption MUST require separately observed captured evaluation, native checks and consumer-side revalidation; authoring this specification MUST NOT be described as executed evidence or publication.

#### Scenario: No current validation record
- GIVEN source changes without recorded current evaluator/native/downstream observations
- WHEN release or consumer adoption is considered
- THEN readiness MUST remain unclaimed.
