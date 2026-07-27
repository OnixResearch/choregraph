## Tasks

- [ ] [serial] Define bounded trace, replay input, replay decision, failure index, consumed-count, and last-valid-state types. r[choregraph.proof.trace.model]
- [ ] [depends:trace-model] Implement a pure replay fold over the production `step` relation that returns no dispatchable intents. r[choregraph.proof.trace.replay]
- [ ] [depends:trace-model] Prove the empty-trace base case and supporting prefix, length, state-coverage, and first-failure lemmas. r[choregraph.proof.trace.lemmas]
- [ ] [depends:single-step-preservation] Prove finite trace invariant preservation by induction. r[choregraph.proof.trace.preservation]
- [ ] [depends:replay-fold] Prove replay determinism and exact first-failure reporting. r[choregraph.proof.trace.determinism] r[choregraph.proof.trace.failure]
- [ ] [parallel] Add positive empty, linear, diamond, terminal, and denied-ending trace fixtures. r[choregraph.proof.trace.fixtures]
- [ ] [parallel] Add negative malformed, unknown-node, invalid-transition, terminal-mutation, oversized, stale-machine, and reordered trace fixtures. r[choregraph.proof.trace.fixtures]
- [ ] [parallel] Add property tests that compare replay state with repeated `step` and assert replay emits no dispatch authority. r[choregraph.proof.trace.replay]
- [ ] [serial] Extend proof and trusted-surface manifests with trace theorems, limits, and explicit liveness and storage non-claims. r[choregraph.proof.trace.evidence]
- [ ] [serial] Run trace proofs, expected-failure checks, tests, Clippy, no-std checks, Cairn gates, and relevant Nix checks. r[choregraph.proof.trace.validation]
