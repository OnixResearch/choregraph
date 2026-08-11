# Seaglass and Lattice conformance validation — 2026-07-27

## Result

All maintained Choregraph conformance gates passed.

The result covers the selected execution-graph parity corpus, the reviewed Lattice diamond, the negative consumer matrix, the exact disposable Lattice runner, architecture boundaries, proof linkage, Cargo, Clippy, Octet, Cairn, cross-target checks, and the full Nix flake.

## Reproducible selected sources

Pure Nix validation initially exposed unavailable Radicle HTTPS mirrors. No alternate network source or ambient sibling fallback was added.

Choregraph now stores two reproducible selected snapshots:

- `vendor/execution-graph-core-selected` contains the exact test-only comparator at revision `03736f1ec46c377ff86b451260ad68aa70ff3b0b`.
- `vendor/bounded-exec-selected` contains the exact production process-mechanism crates at revision `29dac88ecded94457572db3fdfaaaab95fa91525`.

`tools/sync-execution-graph-oracle.rs` and `tools/sync-bounded-exec.rs` reproduce those files from the recorded Git objects. Nix checks each selected manifest, Rust source, and license against its recorded BLAKE3 value.

The execution-graph snapshot remains a dev dependency. The architecture gate confirms that `choregraph-seaglass` has only `choregraph-core` as a production dependency.

## Validation commands

These commands passed:

```text
nix develop -c cargo fmt --all -- --check
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo check -p choregraph-core --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-core --target thumbv7em-none-eabihf
nix develop -c cargo check -p choregraph-seaglass --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-seaglass --target thumbv7em-none-eabihf
./tools/check-lattice-downstream.rs --self-test
./tools/check-lattice-downstream.rs ../lattice .
nix run .#octet-deny-all
nix run path:../cairn#cairn -- policy export --check
nix run path:../cairn#cairn -- validate --root .
nix run path:../cairn#cairn -- gate proposal add-seaglass-lattice-conformance --root .
nix run path:../cairn#cairn -- gate tasks add-seaglass-lattice-conformance --root .
nix build .#checks.x86_64-linux.trace-replay-verus -L --no-link
nix build .#checks.x86_64-linux.single-step-verus -L --no-link
nix flake check -L
```

The full flake ended with `all checks passed!`. Octet reported zero findings, warnings, and errors. The disposable Lattice check passed 21 tests at exact revision `c8d56ad83a57287064edb85f58808f53e4bf4536`.

The Nix run reported unavailable optional binary caches, an unavailable remote builder, and ignored cgroup cleanup warnings. Local exact-source builds completed without source fallback.

## Traceability status

All 11 `choregraph.conformance.*` requirements have implementation or verification markers after accepted-spec sync.

Repository-wide coverage is 65 of 78. The 13 missing links belong to earlier accepted foundation, Kamacite, machine, and Nickel requirements. There are no dangling markers. This historical coverage debt is outside this conformance change and remains visible.

## Requirement evidence

- immutable sources: source inventory and selected-source metadata;
- complete inventory: source inventory and parity matrix;
- conversion-only adapter: adapter source plus architecture boundary test;
- deterministic parity: eight parity cases and reviewed fail-closed dispositions;
- Lattice integration: diamond source-to-machine fixture;
- negative coverage: consumer failure matrix;
- downstream check: exact disposable 21-test runner;
- single implementation and cutover: blocked-state duplicate-removal requirements;
- ordered rollback: recorded VCS revisions and consumer-owned rollback gates;
- retained authority: authority matrix, architecture test, and denial-result fixture;
- validation: the commands above.

## Remaining consumer work

Conformance completion does not cut over either consumer.

- Choregraph release evidence still needs an immutable published source.
- `execution-graph-core` still needs its facade migration and legacy removal.
- Lattice still needs production adoption, persisted-state migration, authorization integration, and duplicate removal.
- Basalt's authenticated UCAN source remains a release blocker.

## Claim boundary

This validation proves the selected source, structure, behavior, failure, and downstream checks only. It does not prove arbitrary Nickel semantics, formal-tool soundness, event truth, runtime authorization, external effect behavior, consumer cutover, or release eligibility.
