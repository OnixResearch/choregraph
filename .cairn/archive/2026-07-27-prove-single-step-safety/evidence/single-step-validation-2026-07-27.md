# Single-step verification evidence

Date: 2026-07-27

## Result

The production-linked single-step proof rail passed.

The positive Verus proof reported `79 verified, 0 errors`.

All six maintained negative fixtures failed with the required postcondition diagnostic.

## Production linkage

`AdmittedMachine::admit` builds the indexed graph views.

`well_formed` calls `verified_topology::admitted_graph_valid`.

`step` calls `verified::indexed_step` for the precheck and final indexed decision. The public shell does not contain a second lifecycle table.

The maintained Nix gate checks these links and rejects unrecorded proof trust escapes.

## Validation

These commands passed:

```text
nix develop -c octet-production-verus --crate-type=lib verus/proofs/single_step.rs -o /tmp/choregraph-single-step.rlib
nix build .#checks.x86_64-linux.single-step-verus --no-link -L
nix develop -c cargo test --workspace
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo check -p choregraph-core --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-core --target thumbv7em-none-eabihf
nix develop -c cargo fmt --all -- --check
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal prove-single-step-safety --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design prove-single-step-safety --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks prove-single-step-safety --root .
nix flake check -L
```

The full Nix check ended with `all checks passed!`.

## Traceability note

All 14 accepted single-step requirements have implementation or verification markers.

The repository-wide Tracey command reported 31 referenced requirements from 42 accepted requirements. It still fails for 11 earlier requirements outside this change.

This existing repository-wide coverage debt does not change the passing result of the three change gates.

The Spectrum binary cache timed out during some commands. Nix used available local or alternate inputs, and each listed command completed successfully.

## Evidence identities

The reviewed proof manifest is `evidence/single-step-proof.ncl`.

The generated manifest is `evidence/generated/single-step-proof.json`. Its BLAKE3 value is stored in `evidence/generated/single-step-proof.blake3`.

The manifest binds the production sources, proof roots, negative fixtures, toolchain, Nix closure, and selected Trellis sources.

## Trust boundary

The project proof core has no trusted declaration, assumption, or axiom.

The evidence trusts the selected Verus binary, Z3, rustc, vstd contracts, allocation semantics, and target platform.

The proof starts at admitted indexed views. It does not prove identifier parsing or the `BTreeMap` conversion adapters.

The proof does not establish Nickel semantics, runtime authorization, effect execution, persistence, scheduling, liveness, or trace safety.
