# Release manifest and shell validation — 2026-07-27

## Result

The release manifest and capability shell pass the focused checks.

- Bundle identity: `a2ac1336679542fa9573b356daddc18266fdda8147ddbe28f85aa37d135030d9`
- Manifest file BLAKE3: `7eeb1983c3dc0e8be1f8e6450b28477f3f9de999537a019e9c22c498925e8f1f`
- Required artifacts: 30
- Observed artifact bytes: 126701
- Manifest bytes: 8369

## Artifact shell

`choregraph-cli` reads 30 fixed repository-relative paths through an ambient-authority `cap-std::fs::Dir`. It enforces per-artifact, total-input, and manifest bounds. It publishes one local manifest without replacement.

The shell passes only in-memory observations to `choregraph-release`. The pure crate defines roles, statuses, schemas, normalization, identity, linkage verification, Valence projection, and candidate policy.

The verifier reopens the artifact and manifest roots. It rebuilds the current expected bundle and requires exact manifest-byte equality. It does not trust the checked-in bundle identity by itself.

## Commands

```text
nix develop -c cargo test -p choregraph-cli
nix develop -c cargo clippy -p choregraph-cli --all-targets -- -D warnings
nix run .#octet-deny-all
nix develop -c cargo run -q -p choregraph-cli -- write-release-bundle "$PWD" "$PWD/evidence/release/generated" choregraph.release.bundle
nix develop -c cargo run -q -p choregraph-cli -- verify-release-bundle "$PWD" "$PWD/evidence/release/generated" choregraph.release.bundle
```

Results:

```text
choregraph-cli: 14 passed, 0 failed
Clippy: passed
Octet: 0 findings
status=verified
bundle-blake3=a2ac1336679542fa9573b356daddc18266fdda8147ddbe28f85aa37d135030d9
artifacts=30
artifact-bytes=126701
manifest-bytes=8369
```

## Nickel observations

All ten positive observation files export through `evidence/release/observation-contract.ncl`.

The incomplete-toolchain and failed-gate fixtures fail the Nickel contract as intended. The stale-source fixture is structurally valid. Pure linkage tests reject its changed digest.

## Negative behavior

Focused Rust tests reject missing roles, duplicate roles, stale digests, skipped proof results, malformed paths, schema mismatches, unsupported statuses, undeclared roles, changed Valence projections, overclaim text, and candidate identity mismatches.

CLI tests also reject relative ambient roots, missing artifacts, output replacement, and a stale artifact after manifest creation.

## Claim boundary

This evidence proves deterministic linkage for the selected local bytes. It does not prove the verifier, compiler, evaluator, solver, filesystem, remote, or downstream runtime correct.
