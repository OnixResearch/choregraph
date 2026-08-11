# Octet deny-all validation

Date: 2026-07-27

## Pinned inputs

- Octet revision: `374bd16b26cee2af34211a29bfa531c016811f51`
- Cargo scope: `--workspace --all-targets --ignore-rust-version`
- Policy mode: `deny-all`
- Octet config hash: `b3:385cbcef32f459df2893c4f7c3eacd3c566893515c341cf308cf6c146fefea33`
- Octet profile hash: `b3:27716986d8caa0b58c3648c15eb9a9b6ebd05d79ed8b948586f73ab4f7433b49`

The policy check matched the local generated lint list with the pinned Octet deny-all hook.

## Octet result

The strict consumer run completed with this summary:

```text
Status: clean
Findings: 0
Warnings: 0
Errors: 0
Autofixable: 0
```

The rollout refactored the foundation checker and moved underscore-separated module files to searchable layouts. Bounded Octet thresholds cover co-located proof and canonical-codec sources. Maintained source-local exceptions have nearby reasons. `dylint.toml` has no disabled lint or warning-level override.

## Validation

These commands passed:

```text
nix build .#checks.x86_64-linux.octet-deny-all-policy --no-link -L
nix build .#checks.x86_64-linux.octet-deny-all --no-link -L
nix build .#checks.x86_64-linux.single-step-verus --no-link -L
nix develop -c cargo fmt --all -- --check
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo check -p choregraph-core --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-core --target thumbv7em-none-eabihf
nix run path:../cairn#cairn -- validate --root .
nix run path:../cairn#cairn -- gate proposal enforce-octet-deny-all --root .
nix run path:../cairn#cairn -- gate design enforce-octet-deny-all --root .
nix run path:../cairn#cairn -- gate tasks enforce-octet-deny-all --root .
nix flake check -L
```

The full flake check ended with `all checks passed!`.

## Non-claims

This evidence records static source-lint and build observations. It does not prove runtime correctness, Verus soundness, effect safety, authorization, or release eligibility.
