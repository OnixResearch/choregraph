# Foundation validation evidence

Date: 2026-07-27

## Baseline

The pre-change Cairn validation and proposal, design, and tasks gates passed. No Rust workspace or Rust test suite existed.

## Source identity

`evidence/source-baseline.ncl` records committed revisions and BLAKE3 identities for every required reference. The calculation used committed archives, not sibling working trees.

The checked-in generated JSON has its own BLAKE3 identity. The Nix evidence rail validates both files and rejects a missing revision.

## Maintained validation

These commands passed:

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- --deny warnings
cargo check -p choregraph-core --target wasm32-unknown-unknown
cargo check -p choregraph-core --target thumbv7em-none-eabihf
cargo run -p choregraph-foundation-check -- --root .
nix build .#choregraph --no-link -L
nix flake check -L
```

The Cargo commands ran with the pinned Rust toolchain from the Nix environment. The full flake validation passed on `x86_64-linux`.

Nix reported an unavailable optional remote builder and continued locally. Nix omitted systems that differ from the current host.

Positive and negative Nickel fixtures passed their expected rails. The negative rails cover forbidden dependencies, source tokens, paths, reverse edges, malformed data, and missing provenance.

## Claim boundary

This evidence validates repository structure, declared dependency direction, source identities, tool identities, and the `no_std` build boundary.

It does not validate graph semantics, transition safety, lowering correctness, verifier soundness, runtime behavior, consumer behavior, or release readiness.
