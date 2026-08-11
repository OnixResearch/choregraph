## Context

`choregraph-core` is a pure `no_std` plus `alloc` kernel. Public transition wrappers call one production-linked indexed implementation.

Canonical packed Preserves bytes already carry machine identity. The workspace also checks that the core compiles for `wasm32-unknown-unknown`.

Compilation alone does not provide a callable ABI. It also does not compare native decisions with decisions from a WASM engine.

The adapter must preserve the current proof and authority boundaries. WASM is a deployment form, not a new semantic source.

## Success Contract

The accepted result is one reusable, export-only WASM component. It consumes bounded canonical bytes and delegates to current production entrypoints.

Completion evidence includes byte-equal native and WASM results over positive and negative fixtures. It also includes import checks and reproducible module identity.

A bounded blocker is an acceptable result. The change must stop if the selected component toolchain requires undeclared host imports.

## Approach Registry

| Family | Mechanism | State |
| --- | --- | --- |
| Reusable WASM component | Wrap the existing kernel behind one canonical byte call | selected |
| Raw pointer ABI | Export manual allocation and pointer functions from a core module | rejected |
| Graph-specific WASM | Compile each admitted graph into separate executable code | rejected |
| Custom bytecode VM | Add an instruction set and a second interpreter | rejected |
| Compile-only target | Keep the current target check without a callable artifact | insufficient |

The selected mechanism keeps the transition relation in Rust. The Component Model supplies typed byte transport without a project-owned pointer protocol.

## Decisions

### Decision: publish one export-only component

**Choice:** Add `choregraph-wasm` as a reusable component adapter. It will expose one WIT function:

```wit
package onix:choregraph@1.0.0;

world kernel {
  export call: func(request: list<u8>) -> list<u8>;
}
```

The WIT world will declare no imports. The final component import section must remain empty.

The initial build will wrap `wasm32-unknown-unknown` output. It must not add WASI adapters or JavaScript glue.

**Rationale:** One export keeps the host contract small. Generated canonical-ABI glue avoids a project-owned unsafe pointer protocol.

### Decision: use canonical packed Preserves inside WIT bytes

**Choice:** WIT carries opaque byte lists. Choregraph defines their meaning with versioned packed Preserves schemas.

The call envelope will select one supported operation:

- `admit-machine`.
- `step`.
- `step-concrete`.
- `replay`.
- `replay-concrete`.

The envelope will contain the required canonical machine and lifecycle artifacts. It will also contain their declared BLAKE3 identities.

Transition calls will contain canonical state and event data. Replay calls will contain canonical state, bindings, limits, and trace data.

The result envelope will distinguish these classes:

- accepted machine decision.
- denied machine decision.
- adapter input error.

Engine traps, fuel exhaustion, memory rejection, and host timeouts remain outside the result envelope.

JSON will not be accepted as an ABI format. Text projections can exist for review fixtures only.

**Rationale:** Preserves keeps one authoritative semantic encoding. WIT defines transport and ownership only.

### Decision: keep a functional core and thin export shell

**Choice:** The adapter will split into two parts.

A pure in-memory dispatcher will decode one call, check bounds and identities, invoke current public core functions, and encode one result.

The generated component export will pass the byte list to that dispatcher. It will own no machine semantics or persistent session state.

Every call will carry all semantic inputs. No call can depend on a prior guest invocation.

**Rationale:** Native tests can exercise the dispatcher without an engine. Component checks can exercise the same dispatcher through WIT.

### Decision: re-admit canonical machine artifacts on each call

**Choice:** Each call will check canonical machine bytes, lifecycle bytes, declared identities, schema versions, and admission limits.

The first version will not retain admitted-machine handles between calls. It will not expose guest pointers or mutable sessions.

**Rationale:** Stateless calls remove stale-handle, reentrancy, teardown, and cross-tenant state risks. Measured evidence can justify a later cache design.

### Decision: enforce layered resource bounds

**Choice:** Named limits will cover the outer request, outer result, trace length, machine data, events, intents, identifiers, and references.

The reference host will reject an oversized request before component invocation. The guest dispatcher will repeat the request-length check before decoding.

The selected engine check will set explicit memory and fuel limits. These host limits are conformance inputs, not machine facts.

The component call can allocate transport memory before guest Rust receives the list. Documentation must state this Component Model boundary.

**Rationale:** Guest checks alone cannot limit host-selected canonical-ABI allocation. Host and guest limits create explicit, separate boundaries.

### Decision: require exact native and WASM parity

**Choice:** A private conformance tool will run the same canonical call through the native dispatcher and one pinned WASM engine.

The tool will compare exact packed result bytes. A mismatch in decision class, state, intents, diagnostics, ordering, or bytes will fail the check.

The corpus will include every accepted machine schema and lifecycle profile. It will also include malformed and denied cases.

The check will inspect the final component for imports and expected exports. It will reject graph-specific transition tables in the adapter source.

**Rationale:** Parity detects adapter, lowering, and transport differences. It does not prove compiler or engine correctness.

### Decision: bind publication bytes and non-claims

**Choice:** Domain-separated BLAKE3 evidence will bind these materials:

- final component bytes.
- WIT source and canonical schemas.
- adapter and production-core source.
- machine, lifecycle, and proof-reference identities.
- Rust, component, Nix, and engine inputs.
- parity corpus and conformance tool source.

Repeated release builds from equal pinned inputs must produce byte-equal components. A stale identity or changed dependency must fail publication checks.

Evidence will call existing Verus material a proof reference. It will not claim that Verus proves rustc lowering or engine execution.

**Rationale:** Exact identities make the deployment artifact reviewable. Explicit non-claims prevent proof transfer beyond the proved Rust source.

### Decision: preserve runtime authority outside Choregraph

**Choice:** The component will return state and effect intents only. It will not authorize, dispatch, persist, retry, schedule, or report effect success.

The host selects the engine, memory, fuel, deadline, tenant boundary, and process isolation. Choregraph does not interpret those host choices as machine evidence.

A component trap must remain a host failure. The host must not convert it into an accepted or denied machine transition.

**Rationale:** WASM packaging does not transfer product authority into the machine kernel.

## ABI Sketch

```text
canonical call bytes
  -> WIT call(list<u8>)
  -> guest envelope-size check
  -> canonical decode and identity check
  -> admitted machine reconstruction
  -> production step or replay entrypoint
  -> canonical result encoding
  -> WIT list<u8>
```

The canonical result binds its ABI version, operation, machine identity, input identity, and decision body. Display text and engine logs are not identity material.

## Validation Strategy

Positive fixtures cover admission, accepted transitions, declared intents, concrete bindings, replay success, deterministic ordering, and every accepted lifecycle profile.

Negative fixtures cover malformed Preserves, JSON, unsupported ABI versions, unknown operations, stale identities, schema mismatch, oversized calls, invalid state, terminal mutation, undeclared effects, and denied replay.

Component checks cover missing exports, added imports, changed WIT, native-to-WASM byte mismatch, engine trap classification, stale module identity, and non-reproducible output.

The maintained rail will include workspace tests, Clippy, core no-std targets, component build, conformance checks, Octet, Cairn, and `nix flake check -L`.

## Adversarial Audit

The audit must try to show that:

- the adapter contains another lifecycle or transition implementation.
- one graph changes the published component bytes.
- a stale machine identity reaches the production kernel.
- an oversized envelope allocates through the reference host.
- equal native and WASM calls produce different canonical bytes.
- a module import grants ambient host access.
- a trap becomes a machine denial or acceptance.
- evidence promotes source proof into compiler or engine proof.

Any successful counterexample blocks publication.

## Risks and Trade-offs

- Component tooling adds a pinned build and inspection surface.
- Re-admission on every call adds bounded decoding and normalization cost.
- The WIT byte-list ABI can allocate guest transport memory before the dispatcher runs.
- One engine gives conformance evidence only. It does not establish all-engine equivalence.
- Active machine changes can widen the parity corpus before this change starts.

## Non-Claims

This design does not prove rustc, component lowering, the Component Model, a WASM engine, memory isolation, fuel policy, deadlines, host sandboxing, authorization, persistence, effect execution, workflow correctness, production readiness, or release eligibility.
