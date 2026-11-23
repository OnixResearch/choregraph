# Preference annotation validation, 2026-11-19

This record covers the uncommitted working tree over `main`
`c3577352ee1e39391f5204ee234d991c7dbeb151`. It is local execution evidence for that tree, not a release, a publication or a Lattice adoption record.

Host: x86_64-linux. The tools were Rust 1.97.1 and Nickel 1.17.0 from the locked development shell. Nix builds ran locally with `--builders ''`.

## Commands

| Command | Result |
| --- | --- |
| `nix develop -c cargo fmt --all -- --check` | exit 0 |
| `nix develop -c cargo test --workspace --all-targets` | exit 0; 307 passed, 0 failed, 1 ignored across 35 suites. `choregraph-choreography` passed 32 tests: 16 v1 tests and 16 `preference::tests`. |
| `nix develop -c cargo clippy --workspace --all-targets -- --deny warnings` | exit 0 |
| `nix develop -c cargo run -p choregraph-foundation-check -- --root .` | exit 0, `foundation-boundary: passed` |
| `octet-deny-all` runner (`nix build .#octet-deny-all`, workspace, all targets) | exit 0, no findings |
| `nix run path:../cairn#cairn -- validate --root .` (Cairn `9655914`) | `valid: true`, no substance findings |
| `nix flake check -L --keep-going --builders '' path:$PWD` | exit 0, `all checks passed!`, 21 `x86_64-linux` checks |

`nix flake check` used `path:` because the new files are untracked. This evidence file and the final `tasks.md` wording were written after that run; neither file is read by any check.

## Baseline identity preservation

Each tree was extracted with `git archive`. The pristine `main` tree and this tree both compiled the same two inputs with the baseline `compile_evaluated` and envelope `envelope`:

- the v1 `two-role-transfer.ncl` export; and
- the `base` of `annotated-two-branch-choice.json`.

Both trees produced byte-identical v1 projection receipts and local-set identities:

| Input | Receipt identity | Complete local set |
| --- | --- | --- |
| two-role transfer | `3900009107aa5b193d41488d8e1bb2c4e8674ff7032eb6e6d3d5b7799440dc89` | `30c5e5b54669ee4f304ff2d0e31b0c0ea2306bdef6a8ee35c8c91c091a0a2892` |
| two-branch base | `3e27e9e6e129284ab756cea442d2732709d9243a920fc4b6bfcd8748e839fd1d` | `aae9fbd4fefa129f8772f99225d6e0b2a67a311a2d61dd7bb66ee6697ce12194` |

## Evidence refresh

`flake.nix` gained the preference-contract steps. Three proof manifests record its BLAKE3: `single-step-proof.ncl`, `trace-replay-proof.ncl` and `wasm-kernel-adapter.ncl`. Those hashes were refreshed, and the exports and `.blake3` files were regenerated with the documented procedure. No proof source, theorem, fixture or claim changed.

A first attempt added a Kamacite `profile::preference` module. That changed the Kamacite adapter identity, and `checked_in_corpus_matches_the_current_profile_and_adapter` failed. The module was removed, and the encoders now call the existing codec directly. No Kamacite source differs from `main`.

## Not observed

- Lattice adoption of the annotated cohort.
- Publication or release.
- Any solver, game semantics, or self-interested soundness. These are non-claims, not pending checks.
