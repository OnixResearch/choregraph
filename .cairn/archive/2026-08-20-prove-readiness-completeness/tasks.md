## Tasks

- [x] [serial] Record the current core, proof, compatibility, and downstream readiness baseline before proof changes. r[choregraph.proof.single_step.readiness_completeness_baseline]
- [x] [serial] Define `dependency_ready` over admitted indexed production views and connect the executable helper to its specification. r[choregraph.proof.single_step.readiness_completeness]
- [x] [depends:choregraph.proof.single_step.readiness_completeness] Prove exact `ready_nodes` membership, uniqueness, and admitted topological order. r[choregraph.proof.single_step.readiness_frontier]
- [x] [depends:choregraph.proof.single_step.readiness_completeness] Prove effect-free `MarkReady` acceptance equivalence under explicit valid premises. r[choregraph.proof.single_step.readiness_transition]
- [x] [parallel] Add expected-failure fixtures for omission, extra membership, duplication, wrong order, premature acceptance, eligible rejection, copied logic, and unrestricted assumptions. r[choregraph.proof.single_step.readiness_completeness_negative]
- [x] [serial] Update proof documentation, trusted-surface records, Nickel evidence, generated identities, and bounded non-claims. r[choregraph.proof.single_step.readiness_completeness_evidence]
- [x] [serial] Run compatibility and Lattice downstream fixtures against the same production readiness path. r[choregraph.proof.single_step.readiness_completeness_compatibility]
- [x] [serial] Run focused proofs, negative fixtures, workspace tests, Clippy, no-std checks, Octet, Cairn gates, and relevant Nix checks. r[choregraph.proof.single_step.readiness_completeness_validation]
