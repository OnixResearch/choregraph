## Tasks

- [x] [serial] Inventory Lattice execution-graph Nickel shapes, imports, contracts, diagnostics, evaluator cohort, and current malformed fixtures. r[choregraph.nickel.lattice_profile]
- [x] [serial] Define typed Nickel contracts for bounded Choregraph and compatible Lattice graph source data. r[choregraph.nickel.contracts]
- [x] [serial] Define the restricted evaluator and content-addressed import-closure profile with explicit limits and forbidden authority. r[choregraph.nickel.profile]
- [x] [depends:nickel-contracts] Implement pure Rust revalidation and lowering from evaluated values into the canonical Choregraph descriptor and admission path. r[choregraph.nickel.lowerer]
- [x] [depends:nickel-profile] Implement source, import, evaluator, output, lowerer, IR, proof-set, and toolchain BLAKE3 bindings. r[choregraph.nickel.identity]
- [ ] [depends:lowerer] Implement a thin CLI shell with operator-opened roots, `cap-root` lexical admission, `cap-std` I/O, bounded diagnostics, and no ambient graph-selected paths. r[choregraph.nickel.capabilities] r[choregraph.nickel.diagnostics]
- [x] [parallel] Add positive Lattice diamond, minimal graph, declared import, and source-order fixtures. r[choregraph.nickel.fixtures]
- [x] [parallel] Add negative contract, unknown-field, duplicate, cycle, endpoint, bound, undeclared-import, hash, path, stale-evaluator, and redaction fixtures. r[choregraph.nickel.fixtures]
- [x] [serial] Emit source and evaluator receipts with an explicit no-source-equivalence claim. r[choregraph.nickel.claim_boundary]
- [ ] [serial] Run contract checks, lowerer tests, capability tests, diagnostic checks, Clippy, Cairn gates, and relevant Nix checks. r[choregraph.nickel.validation]

The capability-shell and final-validation tasks are blocked until `cap-root` has a published immutable revision with aligned Cargo and Nix source identity. See `evidence/cap-root-blocker-2026-07-27.md`.
