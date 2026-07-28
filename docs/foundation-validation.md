# Foundation validation

Run these commands from the repository root.

```text
nix develop -c cargo fmt --all -- --check
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- --deny warnings
nix develop -c cargo check -p choregraph-core --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-core --target thumbv7em-none-eabihf
nix develop -c cargo run -p choregraph-foundation-check -- --root .
nix run path:../cairn#cairn -- validate --root .
nix flake check -L
```

Positive Nickel fixtures describe the accepted workspace direction. Negative fixtures cover I/O, evaluator, runtime, receipt, policy, path, source, and reverse-edge failures.

The workspace declares Rust 1.96.0 as its tested compiler floor. The reviewed release toolchain remains Rust 1.97.1. The Octet gate checks the compiler floor without `--ignore-rust-version`.

The foundation validator rejects exact BLAKE3 and Serde JSON patch requirements in consumer library packages. `Cargo.lock` still selects exact reviewed versions for standalone builds.

The source baseline check compares a fresh Nickel export with the checked-in JSON. It also validates the checked-in BLAKE3 value.

Passing these commands proves only the foundation boundary. It does not prove graph semantics, lowering correctness, proof soundness, runtime behavior, or release eligibility.
