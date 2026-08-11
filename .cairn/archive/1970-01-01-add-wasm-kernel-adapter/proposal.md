## Why

Choregraph already compiles its pure core for `wasm32-unknown-unknown`. It does not publish a callable WASM artifact or stable host contract.

Cross-language consumers need a portable way to call machine admission, transitions, and replay. They must not reimplement the transition relation.

This change packages the existing verified kernel. It does not replace the kernel with generated WASM or a new bytecode interpreter.

## What Changes

- Add one reusable WASM component that delegates all machine decisions to current production entrypoints. r[choregraph.wasm.module]
- Add a versioned WIT interface with canonical packed Preserves request and response envelopes. r[choregraph.wasm.abi]
- Add named input, output, trace, and memory-boundary limits for the reference host and guest adapter. r[choregraph.wasm.bounds]
- Add exact native-to-WASM parity checks for accepted decisions, denials, diagnostics, state, intents, and replay. r[choregraph.wasm.parity]
- Bind the WIT contract, canonical schemas, module bytes, source, proof references, toolchain, and engine into BLAKE3 evidence. r[choregraph.wasm.identity]
- Keep engine policy, authorization, effect execution, persistence, clocks, and runtime isolation outside Choregraph. r[choregraph.wasm.boundary]

## Success Criteria

- One module consumes canonical machine artifacts. Choregraph does not generate separate WASM code for each graph.
- The component delegates to `AdmittedMachine` admission, `step`, `step_concrete`, `replay`, and `replay_concrete`.
- Equal canonical calls produce byte-equal canonical results in native and WASM checks.
- Malformed, unsupported, stale, mismatched, oversized, and non-canonical calls fail closed with bounded adapter errors.
- The final component has no imported host interfaces and cannot execute product effects.
- Repeated release builds from the same pinned inputs produce the same module bytes and BLAKE3 identity.
- Evidence does not claim compiler soundness, engine soundness, sandboxing, authorization, effect success, or release eligibility.

## False Completion

A second transition table inside the WASM adapter is not completion. Graph-specific WASM code generation is not completion.

A compile-only `wasm32-unknown-unknown` check is not callable-module parity. Native unit tests without execution through a pinned WASM engine are not completion.

JSON transport, unbounded envelopes, stale artifact identities, or a module with undeclared imports are not completion.

## Impact

- **Workspace**: add `choregraph-wasm` and a private WASM conformance tool.
- **Contracts**: add WIT and canonical packed Preserves call and result schemas.
- **Nix**: pin component tooling and one conformance engine. Nix must generate any lock update.
- **Evidence**: add exact module, ABI, schema, source, proof-reference, toolchain, and engine identities.
- **Documentation**: describe the ABI, host duties, parity evidence, and non-claims.
- **Testing**: add positive and negative native, component, parity, import, identity, Octet, Cairn, and Nix checks.

## Out of Scope

- Replacing the verified Rust kernel with WASM or custom bytecode.
- Compiling each graph into executable WASM instructions.
- Persistent guest handles, shared mutable sessions, asynchronous calls, callbacks, threads, or component imports.
- Browser JavaScript glue, Node-specific bindings, WASI filesystem access, sockets, clocks, random sources, or environment access.
- Host authorization, engine selection by product data, effect dispatch, persistence, retries, scheduling, or runtime sandbox claims.
- Formal proofs of rustc, component lowering, a WASM engine, the Component Model, or target hardware.

## Dependencies

- Local Cairn change `bind-concrete-effect-intents` must finish first.
- Local Cairn change `model-durable-suspension-transitions` must finish first.
- The adapter must cover every machine schema and lifecycle profile accepted when implementation starts.

## Affected Specs

- `wasm-kernel-adapter`: reusable component, canonical ABI, bounds, parity, identity, and authority boundary.
