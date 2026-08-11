# Wasm Kernel Adapter Specification

## Purpose

Defines the `wasm-kernel-adapter` capability.

## Requirements

### Requirement: WASM delegates to the production kernel

r[choregraph.wasm.module] Choregraph MUST publish one reusable WASM component that delegates every supported machine operation to existing production entrypoints.

#### Scenario: Component evaluates a supported call

r[choregraph.wasm.module.valid]
- GIVEN a supported canonical machine call and one publication component
- WHEN the component evaluates the call
- THEN it MUST use the production admission, step, concrete-step, or replay entrypoint selected by the call
- AND it MUST return the result without another lifecycle table or transition implementation.

#### Scenario: Adapter duplicates machine semantics

r[choregraph.wasm.module.duplicate]
- GIVEN the adapter contains copied transition logic, another lifecycle table, persistent semantic handles, or graph-specific generated WASM
- WHEN foundation and source-link checks run
- THEN they MUST reject the adapter
- AND no publication component MUST be produced.

### Requirement: WASM uses a canonical versioned ABI

r[choregraph.wasm.abi] The adapter MUST expose one versioned WIT byte-call interface with canonical packed Preserves requests and responses.

#### Scenario: Canonical call uses a supported operation

r[choregraph.wasm.abi.valid]
- GIVEN canonical call bytes select admission, `step`, `step_concrete`, `replay`, or `replay_concrete`
- WHEN the adapter decodes the call
- THEN it MUST check the ABI version, operation, artifact schemas, declared identities, state, and event or trace
- AND it MUST return one canonical accepted, denied, or adapter-error response.

#### Scenario: ABI input is unsupported or non-canonical

r[choregraph.wasm.abi.invalid]
- GIVEN call bytes contain JSON, malformed Preserves, an unsupported ABI version, an unknown operation, unknown fields, or non-canonical ordering
- WHEN the adapter decodes the call
- THEN it MUST return one bounded adapter error
- AND it MUST NOT invoke a production transition or emit an effect intent.

### Requirement: WASM calls are bounded and stateless

r[choregraph.wasm.bounds] The reference host and guest adapter MUST enforce named call limits, and the guest MUST retain no semantic state between calls.

#### Scenario: Call stays within all selected limits

r[choregraph.wasm.bounds.valid]
- GIVEN a canonical call satisfies the outer byte limits and all admitted machine limits
- WHEN the reference host and guest process the call
- THEN each layer MUST apply its explicit limits
- AND a fresh component instance MUST produce the same canonical result as a reused instance.

#### Scenario: Call exceeds an outer or machine limit

r[choregraph.wasm.bounds.invalid]
- GIVEN a call exceeds a request, result, trace, machine, event, intent, identifier, or reference limit
- WHEN the reference host or guest observes the excess
- THEN the observing layer MUST stop before the next semantic stage
- AND it MUST NOT widen a limit from call-controlled data.

### Requirement: Native and WASM results have exact parity

r[choregraph.wasm.parity] Maintained checks MUST compare exact canonical result bytes from the native dispatcher and a pinned WASM engine.

#### Scenario: Native and WASM calls agree

r[choregraph.wasm.parity.valid]
- GIVEN the same canonical call, pinned component, explicit engine limits, and supported machine profile
- WHEN native and WASM conformance paths run
- THEN accepted state, intents, denials, diagnostics, ordering, and packed result bytes MUST be equal
- AND repeated equal calls MUST remain equal across fresh component instances.

#### Scenario: Native and WASM calls differ

r[choregraph.wasm.parity.mismatch]
- GIVEN any fixture produces different native and WASM decision classes, state, intents, diagnostics, ordering, or bytes
- WHEN conformance runs
- THEN the check MUST fail with the fixture identity and mismatch class
- AND release evidence MUST NOT report WASM parity.

### Requirement: Published WASM has reproducible identity

r[choregraph.wasm.identity] Publication evidence MUST bind reproducible component bytes and every reviewed source, schema, proof-reference, toolchain, engine, and corpus input with BLAKE3.

#### Scenario: Equal pinned inputs rebuild the component

r[choregraph.wasm.identity.valid]
- GIVEN equal reviewed source, WIT, schemas, toolchain, Nix closure, and build arguments
- WHEN the publication component is rebuilt
- THEN the component bytes and domain-separated BLAKE3 identity MUST be equal
- AND evidence MUST bind the exact parity corpus and conformance engine identity.

#### Scenario: Publication material is stale or non-reproducible

r[choregraph.wasm.identity.invalid]
- GIVEN component bytes differ or any bound source, schema, proof reference, toolchain, engine, or corpus identity is stale
- WHEN publication checks run
- THEN they MUST fail closed
- AND no prior identity MUST be reused for the changed component.

### Requirement: WASM does not gain runtime authority

r[choregraph.wasm.boundary] The component MUST declare no host imports and MUST NOT claim runtime isolation, authorization, persistence, scheduling, or effect execution.

#### Scenario: Export-only component returns intents

r[choregraph.wasm.boundary.valid]
- GIVEN a checked component has the selected export and no imports
- WHEN a valid machine call returns state and effect intents
- THEN the result MUST remain data for a downstream host decision
- AND the host MUST retain engine, resource, authorization, persistence, scheduling, and dispatch authority.

#### Scenario: Host failure or overclaim crosses the boundary

r[choregraph.wasm.boundary.invalid]
- GIVEN the component adds a host import or evidence treats a trap, timeout, fuel limit, or matching intent as machine success
- WHEN import and claim-boundary checks run
- THEN they MUST reject the component or report
- AND passing source proofs or parity checks MUST NOT establish compiler, engine, sandbox, effect, production, or release correctness.
