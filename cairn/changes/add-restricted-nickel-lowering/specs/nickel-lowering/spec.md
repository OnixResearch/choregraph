# Restricted Nickel Lowering

## ADDED Requirements

### Requirement: Lattice authoring profile

r[choregraph.nickel.lattice_profile] The frontend MUST inventory and support the selected bounded Lattice execution-graph source shapes required by the first consumer fixtures.

#### Scenario: Lattice diamond graph is loaded
- GIVEN the reviewed Lattice diamond source
- WHEN the selected frontend runs
- THEN the source MUST lower to the expected canonical machine descriptor.

### Requirement: Typed Nickel contracts

r[choregraph.nickel.contracts] Nickel source data MUST satisfy typed contracts for schema version, nodes, edges, lifecycle profile, events, effects, bounds, and imports before Rust lowering.

#### Scenario: Contract field is malformed
- GIVEN a source value with an invalid required field
- WHEN contract evaluation runs
- THEN evaluation MUST fail before machine admission.

### Requirement: Restricted evaluator profile

r[choregraph.nickel.profile] Maintained evaluation MUST use explicit bounded inputs and MUST exclude network, process, clock, hidden environment, undeclared import, and ambient path authority.

#### Scenario: Source requests an undeclared import
- GIVEN a source import outside the declared closure
- WHEN evaluation is requested
- THEN the frontend MUST reject the import.

### Requirement: Content-addressed import closure

r[choregraph.nickel.imports] Every allowed import MUST have a safe relative path, selected media class, byte limit, and expected BLAKE3 identity.

#### Scenario: Import bytes change
- GIVEN a declared import whose observed BLAKE3 differs from its expected identity
- WHEN the source closure is checked
- THEN evaluation MUST fail closed.

### Requirement: Pure Rust lowerer

r[choregraph.nickel.lowerer] Rust MUST revalidate every evaluated value and lower it through the normal canonical descriptor and machine-admission path.

#### Scenario: Evaluator emits malformed graph data
- GIVEN evaluated output that passed no valid machine contract
- WHEN Rust lowering runs
- THEN no admitted machine MUST be produced.

### Requirement: Translation identity chain

r[choregraph.nickel.identity] Translation evidence MUST bind source, import closure, evaluator, evaluated output, lowerer, canonical descriptor, admitted IR, proof set, and toolchain identities with BLAKE3.

#### Scenario: Lowerer source changes
- GIVEN a translation receipt for an earlier lowerer identity
- WHEN current linkage is checked
- THEN the receipt MUST be stale.

### Requirement: Capability-relative CLI

r[choregraph.nickel.capabilities] The CLI MUST open operator-declared roots at the shell boundary and MUST use `cap-root` plus `cap-std` for relative source and output access.

#### Scenario: Graph selects an ambient path
- GIVEN a graph-provided absolute or escaping path
- WHEN path admission runs
- THEN access MUST be rejected before file I/O.

### Requirement: Safe deterministic diagnostics

r[choregraph.nickel.diagnostics] Frontend diagnostics MUST use stable issue codes and bounded summaries without raw source, imported values, credentials, environment values, or host paths.

#### Scenario: Evaluator error contains a host path
- GIVEN a raw evaluator error with sensitive source context
- WHEN safe diagnostics are rendered
- THEN persisted and operator-safe output MUST omit that context.

### Requirement: Nickel proof boundary

r[choregraph.nickel.claim_boundary] Source and evaluator receipts MUST state that they bind observed inputs and outputs without proving arbitrary Nickel semantic equivalence.

#### Scenario: Translation checks pass
- GIVEN valid source, output, and canonical linkage
- WHEN evidence is reported
- THEN evidence MUST NOT claim that Nickel source meaning was formally proved.

### Requirement: Frontend fixture coverage

r[choregraph.nickel.fixtures] Tests MUST include valid Lattice and minimal sources plus malformed, bounded, import, hash, path, evaluator, graph, and redaction failures.

#### Scenario: Frontend corpus runs
- GIVEN the reviewed positive and negative source corpus
- WHEN maintained checks run
- THEN valid sources MUST pass and invalid sources MUST fail deterministically.

### Requirement: Maintained frontend validation

r[choregraph.nickel.validation] Release checks MUST run contract, lowering, capability, diagnostic, and identity-linkage checks.

#### Scenario: Frontend validation is skipped
- GIVEN a candidate without current frontend checks
- WHEN release readiness runs
- THEN readiness MUST fail.
