# Seaglass Lattice Conformance Specification

## Purpose

Defines the `seaglass-lattice-conformance` capability.

## Requirements

### Requirement: Immutable consumer sources

r[choregraph.conformance.sources] Conformance MUST bind exact execution-graph, Seaglass, and Lattice source revisions, licenses, package identities, and acquisition adapters.

#### Scenario: Consumer source is floating
- GIVEN a conformance input selected by a floating branch or ambient fallback
- WHEN source admission runs
- THEN conformance MUST fail closed.

### Requirement: Complete behavior inventory

r[choregraph.conformance.inventory] The migration MUST inventory every current admission, ordering, dependency, group, readiness, lifecycle, resume, critical-path, bottleneck, state, and event entry point.

#### Scenario: Entry point is missing
- GIVEN a current production behavior without a dual-run case
- WHEN migration readiness is checked
- THEN readiness MUST fail.

### Requirement: Conversion-only adapter

r[choregraph.conformance.adapter] The Seaglass adapter MUST delegate semantic decisions to Choregraph and MUST NOT implement independent graph or transition algorithms.

#### Scenario: Adapter computes readiness itself
- GIVEN adapter code with an independent readiness algorithm
- WHEN architecture checks run
- THEN the adapter MUST be rejected.

### Requirement: Deterministic parity matrix

r[choregraph.conformance.parity] Every inventoried behavior MUST dual-run against the immutable baseline, and every difference MUST block migration or have an explicit reviewed semantic disposition.

#### Scenario: Readiness output differs
- GIVEN equal baseline and Choregraph inputs
- WHEN readiness results differ
- THEN cutover MUST remain blocked.

### Requirement: Lattice first integration

r[choregraph.conformance.lattice] The first end-to-end integration MUST lower the reviewed Lattice diamond Nickel graph and advance it through Choregraph machine decisions.

#### Scenario: Diamond graph completes
- GIVEN the reviewed diamond source and valid result events
- WHEN the integration fixture runs
- THEN Choregraph MUST produce the expected ready sets, terminal state, and declared intent sequence.

### Requirement: Negative consumer coverage

r[choregraph.conformance.negative] Conformance MUST include malformed graph, invalid lifecycle, replay, effect, authority-result, source, receipt, and fallback failures.

#### Scenario: Premature node start is requested
- GIVEN an incomplete dependency
- WHEN the consumer requests a start transition
- THEN Choregraph MUST deny the transition without an effect intent.

### Requirement: Disposable downstream check

r[choregraph.conformance.downstream] Choregraph MUST compile and test a disposable adapter against one exact reviewed Lattice revision before cutover.

#### Scenario: Lattice API changes
- GIVEN a reviewed Lattice revision that no longer compiles with the adapter
- WHEN the downstream check runs
- THEN conformance MUST fail.

### Requirement: Single implementation after cutover

r[choregraph.conformance.single_implementation] Production cutover MUST remove independent transition logic from execution-graph, Seaglass, Lattice adapters, and temporary dual-run code.

#### Scenario: Legacy implementation remains callable
- GIVEN completed cutover with a callable legacy transition path
- WHEN duplicate detection runs
- THEN release readiness MUST fail.

### Requirement: Ordered cutover and rollback

r[choregraph.conformance.cutover] Cutover MUST require proof, interchange, frontend, parity, source-alignment, duplicate-removal, and rollback evidence in dependency order.

#### Scenario: Proof evidence is absent
- GIVEN passing parity without current proof evidence
- WHEN cutover is requested
- THEN cutover MUST remain blocked.

### Requirement: Consumer authority remains local

r[choregraph.conformance.authority] Lattice MUST retain runtimes, Basalt calls, capability roots, file and process adapters, retries, schedules, triggers, persistence, diagnostics, and product evidence.

#### Scenario: Choregraph decision is accepted
- GIVEN an accepted Choregraph effect intent
- WHEN the Lattice shell handles it
- THEN Choregraph MUST NOT claim that authorization or execution succeeded.

### Requirement: Conformance validation

r[choregraph.conformance.validation] Maintained checks MUST run parity, downstream, negative, source, proof-linkage, and duplicate-implementation gates.

#### Scenario: Dual-run checks are skipped
- GIVEN a migration candidate without current parity evidence
- WHEN readiness is evaluated
- THEN migration MUST fail.
