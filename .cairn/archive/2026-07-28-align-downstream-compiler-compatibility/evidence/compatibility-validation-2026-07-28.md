# Downstream compiler compatibility validation

Date: 2026-07-28

## Baseline and resolver diagnosis

The unchanged Choregraph parent revision passed:

```text
nix develop -c cargo test --workspace --all-targets
```

An isolated Lattice worktree at `676e0961950e54298f731e521d1843f4a0281ebb` first selected a Choregraph copy with only the BLAKE3 exact requirement removed. `cargo check --lib` then failed because `choregraph-nickel` required exact `serde_json =1.0.149` while Animus required `^1.0.150`.

After Choregraph used compatible requirements for both libraries, the same isolated Lattice worktree selected:

- BLAKE3 1.8.5;
- Serde JSON 1.0.151;
- the local `choregraph-core`, `choregraph-nickel`, and `choregraph-seaglass` packages.

This command passed in 41.05 seconds:

```text
nix develop -c cargo check --lib
```

The isolated worktrees were test inputs only. They did not change Lattice or publish Choregraph.

## Compiler floor and Octet

Choregraph now declares Rust 1.96.0 as its package compiler floor. Rust 1.97.1 remains the release toolchain in `rust-toolchain.toml`, Nickel toolchain policy, and Nix.

The Octet Cargo arguments are `--workspace --all-targets`. They do not include `--ignore-rust-version`.

This command passed under Octet's pinned `nightly-2026-03-21` compiler:

```text
nix run .#octet-deny-all
```

Observed result:

```text
Status: clean
Findings: 0
Warnings: 0
Errors: 0
Toolchain: nightly-2026-03-21-x86_64-unknown-linux-gnu
Profile: workspace-metadata
```

This result is maintained lint execution. It is not an integration-failure result with zero findings.

## Foundation regression checks

The pure foundation validator imports Cargo dependency requirements into its normalized model. It accepts the reviewed `^1.8.2` BLAKE3 and `^1.0.149` Serde JSON requirements. It rejects exact or otherwise unreviewed requirements in consumer library packages.

The focused foundation suite passed 13 tests. The suite includes:

- accepted compatible BLAKE3 and Serde JSON requirements;
- rejected exact BLAKE3 requirements;
- rejected exact Serde JSON requirements;
- rejected compiler-floor drift;
- the existing positive architecture boundary;
- the existing negative authority, source, path, and dependency-direction cases.

The workspace test, Clippy, and live foundation rails passed:

```text
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- --deny warnings
nix develop -c cargo run -p choregraph-foundation-check -- --root .
```

## Identity regeneration

The generated Cargo lock selects BLAKE3 1.8.5 and Serde JSON 1.0.151. The dependency change also updates `constant_time_eq` to 0.4.2 and adds `cpufeatures` 0.3.0 as BLAKE3 dependencies.

The proof manifests now bind the current Cargo lock and `flake.nix` BLAKE3 values. Regenerated manifest identities are:

- single-step proof JSON: `ef2820597bf41f3d741c0baa1dfce64ba4211f4e51c34227926ef2c895e5dd24`;
- trace-replay proof JSON: `84051015036cc037670d0e32570d7215e2773d0da47460f5d4ef2fa55216e6fd`.

The trace proof passed with `65 verified, 0 errors`. All four trace negative fixtures failed with their expected postcondition diagnostics. The single-step and trace proof Nix derivations passed after regeneration.

The canonical Kamacite corpus was regenerated after the toolchain and proof identities changed.

The rebuilt release bundle passed independent verification:

```text
status=verified
bundle-blake3=6912c485313908411040fa533bd75a1cfd2611d7589605268ae111b3ba1472bc
artifacts=30
artifact-bytes=126940
manifest-bytes=8369
```

The manifest-file BLAKE3 is `ac364a1954ff65294ad7c293a05a474441fa9f9bd6c090cd5a3a7a93b793a61b`.

## Broad validation

The final broad gate passed with one local Nix job:

```text
nix flake check -L --max-jobs 1 --cores 2 --builders ''
```

Nix reported `all checks passed!`. The single-job limit avoids concurrent Verus builder resource contention. Cache timeout warnings and an unavailable optional SSH builder did not change local check results.

Cairn validation and the proposal, design, and task gates passed.

## Local publication candidate

The immutable compatibility source commit is `81b260da2256147ec82cb5dd44c0a0c1dcb7500d`.

A deterministic `git archive` produced:

- source archive BLAKE3: `41adfbdae4bc5b688ef92ab11cd39ed9c8f572bf6214f522d9b4a1290e5f84d5`;
- source archive bytes: `1853440`.

Independent candidate validation reopened and hashed that archive. It also rebuilt and verified the current release bundle from current artifact bytes.

The resulting local candidate is `c493ef88a648eb30b621e74fa5c0f7294df9048501a379395403ad3dcd8a4c65`. Its rollback revision is the published source revision `fc47c5f4ecbb7b4341af690fa42199f25d57f54c`.

The compatibility candidate has `publication_authority = 'none` and `publication_status = 'not_published`. The existing published candidate and publication receipt remain unchanged.

## Claim boundary

These checks establish dependency resolution, source compilation at the declared floor, selected static-lint execution, deterministic identity regeneration, and the named local gates. They do not publish a new source revision, grant consumer acceptance, prove dependency implementation equivalence, or grant runtime or effect authority.
