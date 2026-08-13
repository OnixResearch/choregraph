# Agent Notes

Read `README.md` before changing this repository.

Choregraph owns a pure verified machine kernel and compiler adapters. It does not own product runtime authority.

## Core boundary

Keep `choregraph-core` and all history crates `no_std` plus `alloc`. These crates must not read files, inspect the environment, use clocks, spawn processes, access networks, persist state, evaluate Nickel, verify UCAN tokens, or execute effects.

The machine-history adapter must call the production `replay` or `replay_concrete` entrypoint. It must not add a transition table.

Use a functional core and thin imperative shells. Production transitions and proof targets must use one implementation.

## Verification

Use Verus specifications and proofs beside production Rust. Reuse Trellis primitives where their exact claims and licenses fit. Record every imported assumption and proof boundary.

Every behavior requires positive and negative tests. Include malformed input, bounds, denial, terminal mutation, stale identity, and mismatched receipt cases.

Run the focused foundation rail before broad Nix validation:

```text
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- --deny warnings
nix develop -c cargo run -p choregraph-foundation-check -- --root .
nix flake check -L
```

Use named constants for non-trivial numeric values. Use BLAKE3 for project-owned identities.

## Lifecycle and VCS

Use native `.cairn/` lifecycle packages. Use `jj` for version-control operations.

Do not edit `flake.lock` manually. Nix must generate lock changes.

Do not publish, push, or change repository authority without an explicit user request.
