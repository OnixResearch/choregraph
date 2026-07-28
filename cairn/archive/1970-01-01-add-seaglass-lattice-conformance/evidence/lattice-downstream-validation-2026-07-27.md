# Disposable Lattice downstream validation — 2026-07-27

## Result

The exact reviewed Lattice revision compiled and passed its 21 focused runner tests with `choregraph-seaglass` substituted for the legacy Seaglass package.

- Lattice revision: `c8d56ad83a57287064edb85f58808f53e4bf4536`
- Test target: `test_runner_core`
- Result: 21 passed, 0 failed
- Source fallback: none

## Maintained check

`tools/check-lattice-downstream.rs` requires an explicit Lattice Git repository and Choregraph root. It performs these steps:

1. Resolve and compare the exact Lattice commit object.
2. Archive that commit into a new temporary directory.
3. Replace the exact reviewed `seaglass-execution-graph` dependency line with a package alias to `choregraph-seaglass`.
4. Remove the archived host-specific linker selection and keep only Git CLI source fetching.
5. Run `cargo test --test test_runner_core` with a pinned Rust toolchain.
6. Remove the disposable source directory.

The check reads the selected Git object. It does not copy files from the modified Lattice working tree.

The script self-test includes positive replacement coverage. It also rejects a missing dependency line, duplicate dependency lines, an unsafe adapter path, and a floating revision.

## Compatibility findings

The first disposable compile found four source-level incompatibilities:

- ready-node queries returned `Result` instead of the legacy vector;
- bottleneck queries returned `Result` instead of the legacy vector;
- `GraphDiagnostic` did not implement `Display`;
- the diagnostic enum had extra variants that broke an exhaustive Lattice match.

The adapter now keeps the exact legacy signatures and closed diagnostic vocabulary. Checked companion APIs retain explicit conversion failures for new consumers. All semantic decisions still delegate to Choregraph.

## Toolchain note

The reviewed Lattice shell selected Rust 1.96. Choregraph requires Rust 1.97.1. The disposable check therefore uses pinned Fenix revision `4e49ef62eedaa5c149d62b63b3f53844e1cc45d7`, which supplies Rust 1.99 nightly.

This is a compatibility test toolchain. It does not change either repository's declared release toolchain.

## Working-tree preservation

The Lattice working tree retained its pre-existing modified and untracked files. The check did not edit, reset, stash, or commit that working tree.

## Validation

These commands passed:

```text
./tools/check-lattice-downstream.rs --self-test
./tools/check-lattice-downstream.rs ../lattice .
nix develop -c cargo test -p choregraph-seaglass
nix develop -c cargo clippy -p choregraph-seaglass --all-targets -- -D warnings
nix run .#octet-deny-all
```

Octet reported zero findings, warnings, and errors.

## Claim boundary

This result proves compilation and the selected 21 tests for one exact Lattice revision. It does not prove arbitrary Lattice compatibility, formal correctness, runtime authorization, external effect behavior, default-branch cutover, or release eligibility.
