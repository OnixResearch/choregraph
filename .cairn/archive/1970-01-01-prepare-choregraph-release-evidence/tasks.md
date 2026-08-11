## Tasks

- [x] [serial] Define stable release-bundle roles, required artifacts, supported schema profiles, BLAKE3 identity rules, and explicit non-claims. r[choregraph.release.bundle]
- [x] [serial] Generate deterministic source, license, schema, production, proof, trust, toolchain, Kamacite, fixture, parity, downstream, Cairn, Octet, Nix, and Valence artifact entries. r[choregraph.release.manifest]
- [x] [depends:release-manifest] Implement pure bundle linkage verification and a thin capability-relative artifact shell. r[choregraph.release.verification]
- [x] [parallel] Add positive complete and deterministic-repeat bundle fixtures. r[choregraph.release.fixtures]
- [x] [parallel] Add negative missing, stale, skipped, malformed, mismatched, unsupported, undeclared, and overclaim bundle fixtures. r[choregraph.release.fixtures]
- [x] [serial] Add Valence-friendly projections that preserve proof, canonicalization, lifecycle, and runtime role boundaries. r[choregraph.release.valence]
- [x] [serial] Define a local immutable Radicle publication candidate with exact source, archive, license, package, evidence, acquisition, and rollback identities. r[choregraph.release.publication_candidate]
- [x] [serial] Define execution-graph and Lattice handoff requirements for exact source and evidence identities. r[choregraph.release.downstream]
- [x] [serial] Run formatting, tests, property tests, expected failures, Clippy, no-std, Verus, Nickel, canonical corpus, adapter, parity, downstream, Octet, Cairn, traceability, and Nix checks. r[choregraph.release.gates]
- [x] [serial] Verify release commands perform no publication, push, signing, or remote authority mutation without explicit operator action. r[choregraph.release.authority]
