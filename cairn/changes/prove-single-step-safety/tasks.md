## Tasks

- [ ] [serial] Record passing core tests and the unresolved proof obligations before proof-directed refactoring. r[choregraph.proof.single_step.baseline]
- [ ] [serial] Pin the Verus, Rust, Trellis, solver, and Nix identities used by the maintained proof rail. r[choregraph.proof.single_step.toolchain]
- [ ] [serial] Define mathematical specifications for admission, ordering, readiness, invariants, transitions, denial, and allowed intents over production types. r[choregraph.proof.single_step.specifications]
- [ ] [depends:proof-specifications] Refactor production functions into proof-compatible pure cores with executable refinement postconditions. r[choregraph.proof.single_step.production_linkage]
- [ ] [depends:proof-specifications] Import or locally adapt exact Trellis lemmas with source identities and downstream proof obligations. r[choregraph.proof.single_step.trellis]
- [ ] [depends:production-refinement] Prove graph-admission closure, topological edge order, readiness soundness, and transition determinism. r[choregraph.proof.single_step.graph] r[choregraph.proof.single_step.determinism]
- [ ] [depends:production-refinement] Prove accepted-step invariant preservation and `EffectsAllowed`. r[choregraph.proof.single_step.preservation]
- [ ] [depends:production-refinement] Prove denial returns no next state and no intents. r[choregraph.proof.single_step.denial]
- [ ] [parallel] Add expected-failure fixtures for premature readiness, terminal mutation, undeclared intents, state-map mismatch, and denial with intents. r[choregraph.proof.single_step.negative]
- [ ] [serial] Generate deterministic proof, theorem, toolchain, source, and trusted-surface manifests with BLAKE3 identities. r[choregraph.proof.single_step.evidence] r[choregraph.proof.single_step.trust]
- [ ] [serial] Run proofs, expected-failure checks, Cargo tests, Clippy, no-std checks, Cairn gates, and relevant Nix checks. r[choregraph.proof.single_step.validation]
