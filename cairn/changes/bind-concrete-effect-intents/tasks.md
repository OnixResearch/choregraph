## Phase 1: Baseline and Versioned Model

- [x] [serial] Record exact v1 machine, effect, replay, Preserves, proof, Seaglass, and Lattice fixture identities before changing semantics. r[choregraph.effect_v2.schema]
- [x] [serial] Define bounded declaration keys, exact and slot bindings, source references, canonical reference profiles, and v2 request and intent types in the pure core. r[choregraph.effect_v2.declarations] r[choregraph.effect_v2.bindings] r[choregraph.effect_v2.references]
- [x] [serial] Add positive and negative type, parsing, bounds, duplicate, malformed-reference, and source-order fixtures before production integration. r[choregraph.effect_v2.fixtures] r[choregraph.effect_v2.determinism]

## Phase 2: One Production Transition Kernel

- [x] [serial] Normalize v1 exact declarations and v2 declarations into one admitted immutable representation without changing v1 artifact identity. r[choregraph.effect_v2.schema] r[choregraph.effect_v2.stable_machine] r[choregraph.effect_v2.occurrence_identity] r[choregraph.effect_v2.kernel]
- [x] [serial] Implement pure admission for declaration routes, binding modes, slot and source selection, one-use cardinality, canonical-reference shape, uniqueness, and named limits. r[choregraph.effect_v2.declarations] r[choregraph.effect_v2.bindings] r[choregraph.effect_v2.determinism]
- [x] [serial] Extend the production-called transition kernel to resolve declared concrete requests and emit exact deterministic intents. r[choregraph.effect_v2.step] r[choregraph.effect_v2.kernel]
- [x] [serial] Add fail-closed malformed, undeclared, mismatched, duplicate, and oversized request behavior with no replacement state or intent. r[choregraph.effect_v2.denial]
- [x] [serial] Add stable bounded v2 diagnostics and redaction fixtures that exclude raw request, payload, credential, environment, and host-path data. r[choregraph.effect_v2.diagnostics]
- [x] [serial] Extend replay through the same kernel while keeping all historical concrete intents out of replay output. r[choregraph.effect_v2.replay] r[choregraph.effect_v2.kernel]
- [x] [serial] Prove executable v1 parity and stable machine identity across different run references. r[choregraph.effect_v2.schema] r[choregraph.effect_v2.stable_machine]

## Phase 3: Frontend, Interchange, Compatibility, and Proof

- [ ] [serial] Add typed `lattice-workflow-v2` Nickel contracts, pure Rust lowering, evaluated fixtures, and explicit v1 preservation. r[choregraph.effect_v2.nickel]
- [ ] [serial] Add canonical packed Preserves v2 machine and concrete-intent artifacts, domain-separated BLAKE3 identities, round trips, receipts, and stale-byte failures. r[choregraph.effect_v2.references] r[choregraph.effect_v2.kamacite] r[choregraph.effect_v2.occurrence_identity]
- [ ] [serial] Extend Seaglass as a conversion-only v2 adapter and strengthen architecture failures for policy or runtime authority imports. r[choregraph.effect_v2.seaglass]
- [ ] [serial] Extend the production-linked single-step proof for declaration matching, exact reference preservation, bounds, determinism, invariant preservation, and denial safety. r[choregraph.effect_v2.proof]
- [ ] [serial] Extend finite-trace proof and expected-failure fixtures for replay suppression, reference mutation, undeclared output, and denial with intent. r[choregraph.effect_v2.proof] r[choregraph.effect_v2.replay]

## Phase 4: Downstream Handoff and Release Evidence

- [ ] [serial] Add an exact disposable Lattice handoff fixture that requires a Choregraph concrete intent, preserves declared source references, separates content from occurrence identity, and leaves run, policy, authority, persistence, and attempts downstream. r[choregraph.effect_v2.lattice_handoff] r[choregraph.effect_v2.occurrence_identity]
- [ ] [serial] Update architecture, machine-semantics, proof, interchange, frontend, migration, and authority-boundary documents with the v2 claim limits. r[choregraph.effect_v2.non_claims]
- [ ] [serial] Run focused workspace tests, no-std checks, Clippy, foundation, Verus, Nickel, Kamacite, Seaglass, Octet, Cairn, Tracey, downstream, and Nix gates. r[choregraph.effect_v2.fixtures] r[choregraph.effect_v2.release]
- [ ] [serial] Regenerate proof identities, canonical corpora, manifests, and the release bundle from current bytes, then independently verify exact equality. r[choregraph.effect_v2.release]
- [ ] [serial] Record the exact Lattice handoff and publication blocker, and do not publish without explicit repository authority. r[choregraph.effect_v2.lattice_handoff] r[choregraph.effect_v2.non_claims]
- [ ] [serial] Sync the accepted concrete-effect-intents specification, confirm Tracey has no new gaps, archive the completed change, and rerun post-archive checks. r[choregraph.effect_v2.release]
