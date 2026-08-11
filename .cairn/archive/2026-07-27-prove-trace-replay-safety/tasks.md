## Tasks

- [x] [serial] Define bounded trace, replay input, replay decision, failure index, consumed-count, and last-valid-state types. r[choregraph.proof.trace.model]
- [x] [depends:trace-model] Implement a pure replay fold over the production `step` relation that returns no dispatchable intents. r[choregraph.proof.trace.replay]
- [x] [depends:trace-model] Prove the empty-trace base case and supporting prefix, length, state-coverage, and first-failure lemmas. r[choregraph.proof.trace.lemmas]
- [x] [depends:single-step-preservation] Prove finite trace invariant preservation by induction. r[choregraph.proof.trace.preservation]
- [x] [depends:replay-fold] Prove replay determinism and exact first-failure reporting. r[choregraph.proof.trace.determinism] r[choregraph.proof.trace.failure]
- [x] [parallel] Add positive empty, linear, diamond, terminal, and denied-ending trace fixtures. r[choregraph.proof.trace.fixtures]
- [x] [parallel] Add negative malformed, unknown-node, invalid-transition, terminal-mutation, oversized, stale-machine, and reordered trace fixtures. r[choregraph.proof.trace.fixtures]
- [x] [parallel] Add property tests that compare replay state with repeated `step` and assert replay emits no dispatch authority. r[choregraph.proof.trace.replay]
- [x] [serial] Extend proof and trusted-surface manifests with trace theorems, limits, and explicit liveness and storage non-claims. r[choregraph.proof.trace.evidence]
- [x] [serial] Run trace proofs, expected-failure checks, tests, Clippy, no-std checks, Cairn gates, and relevant Nix checks. r[choregraph.proof.trace.validation]
