# Kamacite interchange validation — 2026-07-27

## Result

The selected Kamacite interchange change passed its maintained Rust, corpus, Nix, and lifecycle checks.

The result covers canonical encoding, bounded conversion, exact identity linkage, local FSM coverage, and receipt structure. It does not prove transition behavior, source equivalence, verifier soundness, runtime authorization, effect success, or release eligibility.

## Reviewed source

- Kamacite revision: `6245a52498f7de511df660eabec3f9b350338cb6`
- Selected codec source BLAKE3: `bbb09dd44cae9aaac5226d7298633600480ff1e1776717b05138c422167056a0`
- Preserves crate: `5.0.0-rc.7`
- Canonical codec: `kamacite-preserves-packed-canonical-v2-preserves-rs-5.0.0-rc.7`
- FSM schema: `kamacite.fsm-spec.v2`
- FSM receipt schema: `kamacite.fsm-receipt.v3`
- License: `MIT OR Apache-2.0`

The authenticated Kamacite Git source could not serve as an anonymous Cargo dependency. `tools/sync-kamacite-codec.rs` imported only the selected codec and license files from the exact Git object. Nix checks the exact BLAKE3 values. No sibling path is a release input.

## Maintained artifacts

- `schemas/preserves/choregraph-machine-v1.preserves`
- `schemas/preserves/choregraph-link-receipt-v1.preserves`
- `fixtures/kamacite/corpus.ncl`
- `fixtures/kamacite/positive/`
- `fixtures/kamacite/negative/`
- `vendor/kamacite-core-selected/SOURCE.ncl`

The corpus generator binds adapter source, generator source, Cargo lock data, and Rust toolchain data. Its test rebuilds every checked-in corpus entry and compares exact bytes.

## Commands

These commands passed:

```text
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo test -p choregraph-kamacite --all-targets
nix develop -c cargo test -p choregraph-cli --all-targets
nix develop -c cargo clippy --workspace --all-targets -- --deny warnings
nix develop -c cargo fmt --all -- --check
nix develop -c nickel export --format json fixtures/kamacite/corpus.ncl
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- policy export --root . --check
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal add-kamacite-machine-artifacts --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design add-kamacite-machine-artifacts --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks add-kamacite-machine-artifacts --root .
nix flake check -L
```

The workspace test rail passed 58 tests. The `choregraph-kamacite` rail passed 20 tests, including the seven selected upstream codec tests.

`nix flake check -L` passed workspace formatting, tests, Clippy, core `wasm32-unknown-unknown`, core `thumbv7em-none-eabihf`, foundation boundaries, source evidence, policy checks, and package builds.

Cairn validation, policy freshness, and the proposal, design, and tasks gates passed. All nine tasks were complete.

The configured Spectrum cache timed out during some commands. Nix continued with available inputs, and the requested checks passed.

## Positive coverage

- canonical packed Preserves round trips;
- permutation-stable normalized machine identity;
- separate machine, lifecycle, admitted IR, proof, generator, adapter, profile, and toolchain bindings;
- all 30 local lifecycle state and event cells;
- exact `deterministic` Kamacite policy;
- declared `no-side-effect` denial class;
- admission, lifecycle, source, proof-reference, exhaustiveness, and trace receipt roles;
- exact trace transition identities and positive guard evidence.

## Negative coverage

- malformed and non-canonical packed Preserves;
- JSON substitution;
- duplicate and unknown descriptor fields;
- unsupported lifecycle profiles;
- incomplete local lifecycle coverage;
- global product-state enumeration;
- lifecycle identity mismatch;
- empty identity material;
- stale receipt identity and dependency links;
- proof-reference mismatch;
- trace mismatch;
- semantic overclaim text.

## Review correction

A secondary invariant review was followed by direct inspection of the selected Kamacite source. This found three compatibility details that were then fixed and tested:

- the FSM determinism policy is exactly `deterministic`;
- receipt kinds are exactly `exhaustiveness` and `trace`;
- denial rows reference a declared side effect whose class is `no-side-effect`.

## Claim boundary

Kamacite artifacts and receipts bind bytes, identities, schema roles, coverage material, and presented trace data. Choregraph formal proof changes remain responsible for transition and replay theorems. Lattice remains responsible for runtime authority and effects.
