# Release core validation — 2026-07-27

## Result

The pure release-bundle core defines the closed role profile, BLAKE3 identity rules, exact non-claims, deterministic normalization, linkage verification, Valence projection, and publication-candidate validation.

## Coverage

Positive tests cover:

- one complete 30-role bundle;
- equal identity after reversed input order;
- exact current-artifact verification;
- exact Valence projection;
- an exact local publication candidate.

Negative tests cover:

- missing and duplicate roles;
- an extra undeclared entry;
- malformed relative paths;
- unsupported schemas;
- a skipped proof status;
- stale artifact identity;
- mismatched bundle identity;
- removed non-claims and promoted Valence claims;
- floating source revision;
- incomplete package set;
- unsupported acquisition adapter.

## Validation

These commands passed:

```text
nix develop -c cargo test -p choregraph-release
nix develop -c cargo clippy -p choregraph-release --all-targets -- -D warnings
nix develop -c cargo check -p choregraph-release --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-release --target thumbv7em-none-eabihf
nix run .#octet-deny-all
nix run path:../cairn#cairn -- validate --root .
```

Octet reported zero findings, warnings, and errors.

## Claim boundary

These tests verify pure normalization and linkage behavior. They do not yet provide a repository artifact manifest, capability-relative shell, current gate receipts, or immutable publication candidate.
