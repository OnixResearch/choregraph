## Why

Lattice cannot link `choregraph-nickel` into its existing dependency graph because Choregraph requires exact BLAKE3 and Serde JSON patch versions. Lattice also cannot run its accepted Octet source because Choregraph declares Rust 1.97.1 while the pinned Octet compiler is Rust 1.96. The source compiles under that compiler, but Choregraph currently hides the mismatch with `--ignore-rust-version`.

## What Changes

- Declare Rust 1.96.0 as the tested package compiler floor while retaining Rust 1.97.1 as the release toolchain.
- Remove the Octet `--ignore-rust-version` bypass.
- Replace exact BLAKE3 and Serde JSON patch requirements with compatible semver requirements.
- Update the generated Cargo lock to the versions already selected by Lattice.
- Add positive and negative foundation checks for the compiler floor and public dependency requirements.
- Rebuild affected release evidence before a new publication candidate is authorized.

## Impact

- **Files**: Cargo manifests and lock, foundation checker and fixtures, Octet policy documentation, Nix policy checks, Cairn lifecycle evidence, and generated release evidence
- **Testing**: workspace tests and Clippy, foundation validation, Octet lint execution under Rust 1.96, Lattice consumer resolution, and `nix flake check -L`
