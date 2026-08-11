# Restricted Nickel validation — 2026-07-27

## Result

The restricted Nickel frontend passed its maintained Cargo, capability, contract, Octet, Cairn, proof-linkage, and Nix checks.

The CLI now reads source and declared import bytes through operator-opened capability roots. It applies lexical path admission before `cap-std` I/O. It rejects symlink components, forbidden Nickel authority, over-limit data, malformed evaluated output, stale import identities, unsafe output paths, and existing output names.

## Source identities

The selected lexical-admission source is:

- Radicle RID: `rad:z3cWBCK7VPbEmbL5MAvKJWKjCE745`
- Revision: `8991e2572578c16e590c2f1c0915db6c46c5d9cb`
- Selected source BLAKE3: `60fbdb7eae53dff8c2c0d41bc15aa123cd3a362ec1611eb4bc892fd6be58f3cb`
- License: `AGPL-3.0-or-later`

`tools/sync-cap-root-core.rs` reproduced the selected vendor tree. Cargo and Nix use `vendor/cap-root-core-selected`.

Bounded evaluator process handling uses Bounded Exec revision `29dac88ecded94457572db3fdfaaaab95fa91525`.

## Positive coverage

Maintained checks accept:

- minimal direct graph source;
- the selected Lattice diamond projection;
- source-order variation;
- one declared, identity-bound import;
- capability-relative source, contract, scratch, output, and import roots;
- canonical artifact publication to a new output name;
- the reviewed `cap-root-core` vendor path in the foundation boundary.

## Negative coverage

Maintained checks reject:

- malformed and unknown fields;
- missing effects and duplicate events;
- excessive bounds;
- Nickel runtime import syntax and string interpolation;
- absolute, escaping, empty, and invalid relative paths;
- unknown and duplicate capability roots;
- symlinked source components;
- non-regular and oversized inputs;
- import hash, media, declaration, and closure mismatches;
- evaluator timeout, non-zero exit, and oversized output;
- output collisions and unsafe output paths;
- an unreviewed `cap-root-core` path;
- raw evaluator diagnostics that contain host paths or source text.

## Commands

The following commands passed:

```text
nix develop -c cargo fmt --all -- --check
nix develop -c cargo test --workspace
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo check -p choregraph-core --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-core --target thumbv7em-none-eabihf
nix build -L .#checks.x86_64-linux.foundation-boundaries
nix build -L .#checks.x86_64-linux.restricted-nickel-contracts
nix run .#octet-deny-all
nix run path:../cairn#cairn -- policy export --root . --check
nix run path:../cairn#cairn -- validate --root .
nix run path:../cairn#cairn -- gate proposal add-restricted-nickel-lowering --root .
nix run path:../cairn#cairn -- gate design add-restricted-nickel-lowering --root .
nix run path:../cairn#cairn -- gate tasks add-restricted-nickel-lowering --root .
nix flake check -L
```

The Octet result was clean with zero findings, warnings, and errors. The full flake result ended with `all checks passed!`.

## Claim boundary

This validation does not prove Nickel evaluator correctness, arbitrary Nickel semantic equivalence, filesystem confinement, network sandboxing, event truth, runtime authorization, effect success, directory-entry durability, indefinite Radicle replication, or release eligibility.
