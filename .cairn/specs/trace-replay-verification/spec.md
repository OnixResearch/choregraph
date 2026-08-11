# Trace Replay Verification Specification

## Purpose

Defines the `trace-replay-verification` capability.

## Requirements

### Requirement: Bounded trace model

r[choregraph.proof.trace.model] The core MUST represent traces as bounded ordered canonical events with explicit machine and initial-state bindings.

#### Scenario: Trace exceeds its limit
- GIVEN a trace longer than the selected replay limit
- WHEN replay admission runs
- THEN replay MUST reject the trace before unbounded processing.

### Requirement: Replay uses the production transition relation

r[choregraph.proof.trace.replay] Replay MUST fold the production `step` relation and MUST NOT expose historical effect intents as dispatchable output.

#### Scenario: Historical start event is replayed
- GIVEN a trace event that originally produced an effect intent
- WHEN replay processes the event
- THEN replay MAY update pure state but MUST NOT return dispatch authority.

### Requirement: Trace preservation theorem

r[choregraph.proof.trace.preservation] The proof rail MUST prove that successful finite replay from a well-formed machine and invariant initial state produces an invariant final state.

#### Scenario: Valid trace is replayed
- GIVEN a well-formed machine, invariant initial state, and accepted finite trace
- WHEN replay completes
- THEN the final state MUST satisfy the invariant.

### Requirement: Induction lemmas

r[choregraph.proof.trace.lemmas] The proof rail MUST establish the empty-trace base case and the prefix facts required by the induction step.

#### Scenario: Empty trace is replayed
- GIVEN an invariant initial state and an empty trace
- WHEN the base theorem is applied
- THEN the final state MUST equal the initial state and remain invariant.

### Requirement: Replay determinism

r[choregraph.proof.trace.determinism] Equal admitted machines, initial states, traces, and explicit limits MUST produce equal replay decisions.

#### Scenario: Equal trace inputs are replayed
- GIVEN two equal replay inputs
- WHEN replay runs
- THEN final state, consumed count, or failure diagnostics MUST be equal.

### Requirement: Exact first failure

r[choregraph.proof.trace.failure] Invalid history MUST report the first invalid event index and the last valid state without applying later events.

#### Scenario: Middle event is invalid
- GIVEN a trace with a valid prefix, one invalid event, and later events
- WHEN replay runs
- THEN replay MUST stop at the invalid event and MUST NOT apply the suffix.

### Requirement: Positive and negative trace fixtures

r[choregraph.proof.trace.fixtures] Trace validation MUST include empty and valid traces plus malformed, oversized, stale, reordered, terminal-mutation, and invalid-transition traces.

#### Scenario: Trace fixture corpus runs
- GIVEN the reviewed positive and negative trace corpus
- WHEN maintained replay checks run
- THEN valid traces MUST pass and invalid traces MUST fail deterministically.

### Requirement: Trace evidence boundary

r[choregraph.proof.trace.evidence] Trace evidence MUST bind theorem, source, machine schema, limits, and toolchain identities and MUST state storage, event-truth, liveness, and effect non-claims.

#### Scenario: Trace proof passes
- GIVEN a passing finite trace proof
- WHEN evidence is reported
- THEN no log-completeness, storage-durability, event-truth, liveness, or exactly-once claim may be inferred.

### Requirement: Maintained trace validation

r[choregraph.proof.trace.validation] Release checks MUST run current trace proofs, expected failures, executable tests, and no-std checks.

#### Scenario: Trace proof is stale
- GIVEN a source or theorem identity that differs from the saved trace evidence
- WHEN release validation runs
- THEN validation MUST fail closed.
