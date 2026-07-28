## Phase 1: Dependencies and baseline

- [ ] [depends:bind-concrete-effect-intents] D1 Start WASM work only after concrete effect intents use the final production transition path. r[choregraph.wasm.module]
- [ ] [depends:model-durable-suspension-transitions] D2 Start ABI work only after the durable-suspension profile and canonical forms are accepted. r[choregraph.wasm.parity]
- [ ] [serial] B1 Record current native tests, no-std targets, canonical corpus identities, proof identities, Octet results, Cairn results, and Nix results. r[choregraph.wasm.identity]
- [ ] [serial] B2 Inventory public admission, `step`, `step_concrete`, `replay`, `replay_concrete`, codec, diagnostic, limit, and identity entrypoints. r[choregraph.wasm.module] r[choregraph.wasm.abi]
- [ ] [serial] B3 Build a minimal export-only component with the selected WIT toolchain. Stop if it adds WASI or other imports. r[choregraph.wasm.module] r[choregraph.wasm.boundary]

## Phase 2: Canonical ABI and pure dispatcher

- [ ] [serial] A1 Add a versioned WIT world with one `call(list<u8>) -> list<u8>` export and no imported interfaces. r[choregraph.wasm.abi] r[choregraph.wasm.boundary]
- [ ] [serial] A2 Add reviewed Preserves schemas for calls, results, states, events, traces, stable errors, operation selectors, and identity bindings. r[choregraph.wasm.abi]
- [ ] [serial] A3 Define named outer-request, outer-result, trace, and conformance-engine limits without permitting caller-selected widening. r[choregraph.wasm.bounds]
- [ ] [serial] A4 Implement a pure in-memory dispatcher for admission, `step`, `step_concrete`, `replay`, and `replay_concrete`. r[choregraph.wasm.module] r[choregraph.wasm.abi]
- [ ] [serial] A5 Reconstruct admitted machines from checked canonical artifacts on each call and retain no semantic state between calls. r[choregraph.wasm.module] r[choregraph.wasm.bounds]
- [ ] [parallel] A6 Add positive native tests for each operation, schema, lifecycle profile, accepted decision, intent form, and replay result. r[choregraph.wasm.abi] r[choregraph.wasm.parity]
- [ ] [parallel] A7 Add negative native tests for malformed bytes, JSON, unknown operations, unsupported versions, stale identities, mismatches, bounds, denial, and redaction. r[choregraph.wasm.bounds] r[choregraph.wasm.parity]

## Phase 3: Component shell and artifact identity

- [ ] [serial] M1 Add `choregraph-wasm` as a thin component shell around the pure dispatcher without transition tables or graph-specific code generation. r[choregraph.wasm.module]
- [ ] [serial] M2 Build the final component from pinned Rust and component tooling, then inspect its imports, exports, target features, and custom sections. r[choregraph.wasm.module] r[choregraph.wasm.boundary]
- [ ] [serial] M3 Add foundation failures for host imports, WASI use, JavaScript glue, ambient capabilities, copied transition logic, extra exports, or persistent handles. r[choregraph.wasm.module] r[choregraph.wasm.boundary]
- [ ] [serial] M4 Add domain-separated BLAKE3 identities for WIT, schemas, adapter source, component bytes, core source, proof references, toolchain, engine, and corpus. r[choregraph.wasm.identity]
- [ ] [parallel] M5 Add stale-source, stale-schema, stale-proof-reference, stale-toolchain, stale-engine, stale-component, and non-reproducible-build failures. r[choregraph.wasm.identity]

## Phase 4: Native-to-WASM conformance

- [ ] [serial] C1 Add a private conformance tool that runs canonical calls natively and through one pinned WASM engine under explicit limits. r[choregraph.wasm.parity] r[choregraph.wasm.bounds]
- [ ] [serial] C2 Compare exact packed result bytes for admission, accepted steps, denied steps, concrete intents, replay success, and first-denial replay. r[choregraph.wasm.parity]
- [ ] [parallel] C3 Add reordered-input, repeated-call, fresh-instance, maximum-bound, empty-graph, linear-graph, diamond-graph, and accepted-profile parity fixtures. r[choregraph.wasm.parity] r[choregraph.wasm.bounds]
- [ ] [parallel] C4 Add malformed, oversized, stale, wrong-schema, wrong-machine, invalid-state, terminal, undeclared-effect, duplicate-effect, and trace-bound failures. r[choregraph.wasm.parity] r[choregraph.wasm.bounds]
- [ ] [serial] C5 Keep engine traps, fuel exhaustion, memory rejection, and host timeouts distinct from canonical machine decisions. r[choregraph.wasm.boundary]
- [ ] [serial] C6 Rebuild the publication component from equal pinned inputs and require byte-equal output before evidence generation. r[choregraph.wasm.identity]

## Phase 5: Proof boundary, documentation, and release evidence

- [ ] [serial] E1 Add source-link checks that require delegation to production entrypoints and reject a proof-only or adapter-only transition path. r[choregraph.wasm.module]
- [ ] [serial] E2 Run existing production Verus rails and bind their identities as proof references without claiming compiled-module or engine proof. r[choregraph.wasm.identity] r[choregraph.wasm.boundary]
- [ ] [serial] E3 Document the WIT ABI, Preserves envelopes, limits, host duties, error classes, import policy, identity chain, migration, and non-claims. r[choregraph.wasm.abi] r[choregraph.wasm.boundary]
- [ ] [parallel] E4 Add report failures for sandbox, authorization, persistence, engine-correctness, compiler-correctness, effect-success, or release-eligibility overclaims. r[choregraph.wasm.boundary]
- [ ] [serial] E5 Extend release evidence with exact component, ABI, schema, source, proof-reference, toolchain, engine, corpus, and conformance identities. r[choregraph.wasm.identity]

## Phase 6: Closeout

- [ ] [serial] X1 Run workspace tests, Clippy, no-std targets, component checks, conformance, foundation checks, Verus, Octet, Cairn, Tracey, and `nix flake check -L`. r[choregraph.wasm.parity] r[choregraph.wasm.identity]
- [ ] [serial] X2 Review the adversarial cases and block publication for any native mismatch, unexpected import, stale identity, non-reproducible bytes, or overclaim. r[choregraph.wasm.parity] r[choregraph.wasm.boundary]
- [ ] [serial] X3 Review dry-run sync and archive plans, make sure that every requirement reaches `wasm-kernel-adapter`, and archive only after current evidence passes. r[choregraph.wasm.identity]
