## Tasks

- [ ] [serial] Record the selected Kamacite revision, supported Preserves profiles, licenses, canonicalization rules, and receipt non-claims. r[choregraph.interchange.kamacite.source]
- [ ] [serial] Define the versioned Choregraph aggregate machine Preserves descriptor and its bounded identity material. r[choregraph.interchange.kamacite.aggregate]
- [ ] [serial] Define the exact supported Kamacite local lifecycle FSM profile without enumerating aggregate product states. r[choregraph.interchange.kamacite.lifecycle]
- [ ] [depends:artifact-schemas] Implement pure in-memory Choregraph-to-Kamacite conversion, canonical decoding, and validation. r[choregraph.interchange.kamacite.adapter]
- [ ] [depends:artifact-schemas] Implement BLAKE3 identity bindings for machine, lifecycle, admitted IR, proof set, generator, adapter, profile, and toolchain. r[choregraph.interchange.kamacite.identity]
- [ ] [depends:artifact-adapter] Validate admission, lifecycle, exhaustiveness, trace, source, and proof-reference receipt roles with explicit non-claims. r[choregraph.interchange.kamacite.receipts]
- [ ] [parallel] Add positive canonical round-trip, identity-stability, lifecycle-coverage, and exact-linkage fixtures. r[choregraph.interchange.kamacite.fixtures]
- [ ] [parallel] Add negative malformed, unsupported-profile, non-canonical, stale, mismatched, JSON-substitution, global-enumeration, and overclaim fixtures. r[choregraph.interchange.kamacite.fixtures]
- [ ] [serial] Run adapter tests, canonical corpus checks, Clippy, Cairn gates, and relevant Nix checks. r[choregraph.interchange.kamacite.validation]
