{
  links = [
    # r[verify choregraph.foundation.workspace]
    { requirement = "choregraph.foundation.workspace", evidence = "Cargo.toml and tools/choregraph-foundation-check" },
    # r[verify choregraph.foundation.boundaries]
    { requirement = "choregraph.foundation.boundaries", evidence = "crates/choregraph-core and foundation negative fixtures" },
    # r[verify choregraph.foundation.single_implementation]
    { requirement = "choregraph.foundation.single_implementation", evidence = "crates/choregraph-core and crates/choregraph-seaglass/tests/architecture-boundary.rs" },
    # r[verify choregraph.foundation.license]
    { requirement = "choregraph.foundation.license", evidence = "LICENSE and Cargo workspace package metadata" },
    # r[verify choregraph.foundation.provenance]
    { requirement = "choregraph.foundation.provenance", evidence = "evidence/source-baseline.ncl and vendor source manifests" },
    # r[verify choregraph.foundation.toolchain]
    { requirement = "choregraph.foundation.toolchain", evidence = "rust-toolchain.toml, flake.lock, and config/toolchain.ncl" },
    # r[verify choregraph.foundation.validation]
    { requirement = "choregraph.foundation.validation", evidence = "tools/choregraph-foundation-check/src/tests.rs and fixtures/foundation" },
    # r[verify choregraph.interchange.kamacite.source]
    { requirement = "choregraph.interchange.kamacite.source", evidence = "vendor/kamacite-core-selected/SOURCE.ncl and docs/kamacite-profile.md" },
    # r[verify choregraph.interchange.kamacite.validation]
    { requirement = "choregraph.interchange.kamacite.validation", evidence = "crates/choregraph-kamacite/src/tests.rs and fixtures/kamacite" },
    # r[verify choregraph.machine.compatibility_baseline]
    { requirement = "choregraph.machine.compatibility_baseline", evidence = "docs/execution-graph-compatibility-baseline.md and crates/choregraph-seaglass/tests/parity.rs" },
    # r[verify choregraph.machine.validation]
    { requirement = "choregraph.machine.validation", evidence = "crates/choregraph-core/tests and docs/machine-semantics.md" },
    # r[verify choregraph.nickel.capabilities]
    { requirement = "choregraph.nickel.capabilities", evidence = "crates/choregraph-cli/src/capability.rs and capability tests" },
    # r[verify choregraph.nickel.validation]
    { requirement = "choregraph.nickel.validation", evidence = "crates/choregraph-nickel/src/tests.rs and restricted-nickel-contracts Nix check" },
    # r[impl choregraph.release.bundle]
    { requirement = "choregraph.release.bundle", evidence = "crates/choregraph-release/src/lib.rs" },
    # r[impl choregraph.release.manifest]
    { requirement = "choregraph.release.manifest", evidence = "crates/choregraph-cli/src/release.rs and evidence/release/generated/choregraph.release.bundle" },
    # r[impl choregraph.release.verification]
    { requirement = "choregraph.release.verification", evidence = "choregraph-release verifier and choregraph-cli capability shell" },
    # r[verify choregraph.release.fixtures]
    { requirement = "choregraph.release.fixtures", evidence = "choregraph-release tests and evidence/release/negative" },
    # r[impl choregraph.release.valence]
    { requirement = "choregraph.release.valence", evidence = "choregraph-release Valence projection" },
    # r[impl choregraph.release.publication_candidate]
    { requirement = "choregraph.release.publication_candidate", evidence = "evidence/release/publication-candidate.ncl" },
    # r[impl choregraph.release.downstream]
    { requirement = "choregraph.release.downstream", evidence = "docs/lattice-cutover-readiness.md" },
    # r[verify choregraph.release.gates]
    { requirement = "choregraph.release.gates", evidence = "release validation evidence and flake checks" },
    # r[verify choregraph.release.authority]
    { requirement = "choregraph.release.authority", evidence = "release authority audit" },
    # r[impl choregraph.release.claim_boundary]
    { requirement = "choregraph.release.claim_boundary", evidence = "REQUIRED_NON_CLAIMS and docs/release-evidence.md" },
    # r[impl choregraph.release.publication]
    { requirement = "choregraph.release.publication", evidence = "evidence/release/publication-receipt.ncl" },
    # r[verify choregraph.release.acquisition]
    { requirement = "choregraph.release.acquisition", evidence = "record-choregraph-publication evidence" },
    # r[verify choregraph.release.publication_validation]
    { requirement = "choregraph.release.publication_validation", evidence = "publication receipt contract and negative fixture" },
  ],
}
