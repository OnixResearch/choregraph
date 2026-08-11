# WASM kernel adapter

`choregraph-wasm` publishes one stateless WebAssembly component. It calls the existing Choregraph production kernel.

## Interface

The versioned WIT source is `crates/choregraph-wasm/wit/kernel.wit`.

```wit
package onix:choregraph@1.0.0;

world kernel {
  export call: func(request: list<u8>) -> list<u8>;
}
```

The world has no imports. The component has no WASI adapter and no JavaScript glue.

The request and result lists contain canonical packed Preserves. JSON and Preserves text are not accepted. The reviewed serde schema is in `crates/choregraph-wasm/src/schema.rs`.

The request root is `choregraph-wasm-call-v1`. The result root is `choregraph-wasm-result-v1`. Both envelopes contain ABI version `1`.

## Operations

The call envelope supports these operations:

- `admit-machine`
- `step`
- `step-concrete`
- `replay`
- `replay-concrete`

Each operation carries all required state. The guest does not retain admitted machines, state, traces, handles, or product data between calls.

Linear and durable v1 calls carry canonical machine, lifecycle, lifecycle-identity, profile-identity, and machine-identity fields. Concrete v2 calls carry canonical machine bytes and their machine identity. Extra or missing artifact fields fail closed.

A valid replay carries separate expected and trace bindings. This structure preserves the production replay mismatch checks.

## Limits

The guest uses these fixed limits:

| Limit | Value |
| --- | ---: |
| Outer request | 1,048,576 bytes |
| Outer result | 2,097,152 bytes |
| Nodes | 4,096 nodes |
| Edges | 16,384 edges |
| Effect declarations | 16,384 declarations |
| State entries | 4,096 entries |
| Effects per event | 64 effects |
| Trace events | 65,536 events |

The machine artifact keeps its admitted operation, identifier, reference, and other structural limits. Call data cannot widen any guest limit.

A reference host must apply the outer request limit before component invocation. WIT list lowering can allocate transport memory before guest Rust runs. The host must also set explicit engine memory, fuel, instance, and deadline limits.

## Results and errors

A canonical result is one of these classes:

- admitted
- accepted
- denied
- replay-complete
- replay-failed
- adapter-error

Adapter errors contain only a stable code and stage. They do not include input bytes or codec diagnostics. Engine traps, fuel exhaustion, memory rejection, and host timeouts are host failures. A host must not convert them to accepted or denied machine results.

Effect intents are data only. A host must make a separate authorization and dispatch decision.

## Identity and publication

Publication evidence uses domain-separated BLAKE3 identities. It binds these inputs:

- final component bytes
- WIT source
- request and result schema source
- adapter source
- production-core source
- Verus proof references
- Rust and Nix toolchains
- component tooling
- Wasmtime conformance engine
- parity corpus
- conformance tool source

Equal pinned inputs must produce byte-equal component output. A changed input requires new evidence and a new component identity.

The reviewed evidence is `evidence/wasm-kernel-adapter.ncl`. Its deterministic export is `evidence/generated/wasm-kernel-adapter.json`.

Build the component and run its identity-linked parity check with these commands:

```console
nix build .#choregraph-wasm-component
nix build .#checks.x86_64-linux.wasm-kernel-evidence
```

The parity corpus covers empty, linear, diamond, reordered, durable-suspension, and concrete-v2 artifacts. It also covers stale identities, wrong schemas, invalid states, undeclared and duplicate effects, replay mismatches, malformed input, and version rejection. Each fixture must produce equal native and component bytes in fresh and reused Wasmtime instances.

## Host duties

The host owns all of these controls:

- engine selection and configuration
- memory, fuel, deadline, and process limits
- tenant and isolation boundaries
- machine and artifact source authorization
- persistence and recovery
- scheduling and retry policy
- effect authorization and dispatch
- effect outcome recording

## Non-claims

Native and WASM parity is bounded evidence for the selected corpus and engine. It does not prove rustc, component lowering, Wasmtime, another engine, sandbox isolation, authorization, persistence, effect success, workflow correctness, production readiness, or release eligibility.
