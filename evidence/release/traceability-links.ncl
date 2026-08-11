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
    # r[impl choregraph.release.compatibility_publication]
    { requirement = "choregraph.release.compatibility_publication", evidence = "evidence/release/compatibility-publication-receipt.ncl" },
    # r[verify choregraph.release.compatibility_acquisition]
    { requirement = "choregraph.release.compatibility_acquisition", evidence = "record-compatibility-publication evidence" },
    # r[verify choregraph.execution_graph_compatibility.source]
    { requirement = "choregraph.execution_graph_compatibility.source", evidence = "Cargo.toml and crates/execution-graph-core" },
    # r[verify choregraph.machine.suspension_profile.valid]
    { requirement = "choregraph.machine.suspension_profile.valid", evidence = "crates/choregraph-core/tests/durable-suspension.rs" },
    # r[verify choregraph.machine.suspension_profile.invalid]
    { requirement = "choregraph.machine.suspension_profile.invalid", evidence = "crates/choregraph-core/tests/durable-suspension.rs" },
    # r[verify choregraph.machine.suspension_transition.suspend]
    { requirement = "choregraph.machine.suspension_transition.suspend", evidence = "crates/choregraph-core/tests/durable-suspension.rs" },
    # r[verify choregraph.machine.suspension_transition.resume]
    { requirement = "choregraph.machine.suspension_transition.resume", evidence = "crates/choregraph-core/tests/durable-suspension.rs" },
    # r[verify choregraph.machine.suspension_transition.denied]
    { requirement = "choregraph.machine.suspension_transition.denied", evidence = "crates/choregraph-core/tests/durable-suspension.rs" },
    # r[verify choregraph.machine.resume_signal.wait]
    { requirement = "choregraph.machine.resume_signal.wait", evidence = "crates/choregraph-core/tests/durable-suspension.rs" },
    # r[verify choregraph.machine.resume_signal.mismatch]
    { requirement = "choregraph.machine.resume_signal.mismatch", evidence = "crates/choregraph-core/tests/durable-suspension.rs" },
    # r[verify choregraph.machine.suspension_replay.valid]
    { requirement = "choregraph.machine.suspension_replay.valid", evidence = "crates/choregraph-core/tests/durable-suspension.rs and proofs/trace-replay" },
    # r[verify choregraph.machine.suspension_replay.invalid]
    { requirement = "choregraph.machine.suspension_replay.invalid", evidence = "crates/choregraph-core/tests/durable-suspension.rs and proofs/trace-replay" },
    # r[verify choregraph.machine.suspension_proof.valid]
    { requirement = "choregraph.machine.suspension_proof.valid", evidence = "proofs and Verus Nix checks" },
    # r[verify choregraph.machine.suspension_proof.invalid]
    { requirement = "choregraph.machine.suspension_proof.invalid", evidence = "proofs/negative and Verus Nix checks" },
    # r[verify choregraph.machine.suspension_boundary.valid]
    { requirement = "choregraph.machine.suspension_boundary.valid", evidence = "crates/choregraph-core/src/boundary.rs and durable-suspension tests" },
    # r[verify choregraph.machine.suspension_boundary.overclaim]
    { requirement = "choregraph.machine.suspension_boundary.overclaim", evidence = "crates/choregraph-core/src/boundary.rs and evidence non-claims" },
    # r[verify choregraph.validation_baselines.trace_fixtures]
    { requirement = "choregraph.validation_baselines.trace_fixtures", evidence = "proofs/negative and trace-replay-verus Nix check" },
    # r[verify choregraph.validation_baselines.cairn_policy]
    { requirement = "choregraph.validation_baselines.cairn_policy", evidence = "cairn-policy/default.ncl and generated/cairn-policy.json" },
    # r[verify choregraph.validation_baselines.policy_overrides]
    { requirement = "choregraph.validation_baselines.policy_overrides", evidence = "cairn-policy/default.ncl and upstream-default.ncl" },
    # r[verify choregraph.validation_baselines.negative_policy]
    { requirement = "choregraph.validation_baselines.negative_policy", evidence = "cairn-policy/fixtures and cairn-policy Nix check" },
    # r[verify choregraph.validation_baselines.source_quality]
    { requirement = "choregraph.validation_baselines.source_quality", evidence = "workspace format, Clippy, and Octet Nix checks" },
    # r[verify choregraph.wasm.module.valid]
    { requirement = "choregraph.wasm.module.valid", evidence = "crates/choregraph-wasm and wasm-kernel-conformance Nix check" },
    # r[verify choregraph.wasm.module.duplicate]
    { requirement = "choregraph.wasm.module.duplicate", evidence = "choregraph-foundation-check source-link failures" },
    # r[verify choregraph.wasm.abi.valid]
    { requirement = "choregraph.wasm.abi.valid", evidence = "crates/choregraph-wasm/tests/dispatcher.rs and WIT inspection" },
    # r[verify choregraph.wasm.abi.invalid]
    { requirement = "choregraph.wasm.abi.invalid", evidence = "crates/choregraph-wasm/tests/dispatcher.rs malformed and unsupported fixtures" },
    # r[verify choregraph.wasm.bounds.valid]
    { requirement = "choregraph.wasm.bounds.valid", evidence = "dispatcher boundary tests and conformance maximum-bound fixture" },
    # r[verify choregraph.wasm.bounds.invalid]
    { requirement = "choregraph.wasm.bounds.invalid", evidence = "dispatcher over-bound tests and replay-limit-rejected fixture" },
    # r[verify choregraph.wasm.parity.valid]
    { requirement = "choregraph.wasm.parity.valid", evidence = "wasm-kernel-conformance exact-byte corpus" },
    # r[verify choregraph.wasm.parity.mismatch]
    { requirement = "choregraph.wasm.parity.mismatch", evidence = "wasm-kernel-conformance mismatch failure path" },
    # r[verify choregraph.wasm.identity.valid]
    { requirement = "choregraph.wasm.identity.valid", evidence = "evidence/wasm-kernel-adapter.ncl and wasm-kernel-evidence Nix check" },
    # r[verify choregraph.wasm.identity.invalid]
    { requirement = "choregraph.wasm.identity.invalid", evidence = "wasm-kernel-evidence stale identity and duplicate-build failures" },
    # r[verify choregraph.wasm.boundary.valid]
    { requirement = "choregraph.wasm.boundary.valid", evidence = "foundation import/export checks and adapter non-claims" },
    # r[verify choregraph.wasm.boundary.invalid]
    { requirement = "choregraph.wasm.boundary.invalid", evidence = "foundation host-import and overclaim failures" },
  ],
}
