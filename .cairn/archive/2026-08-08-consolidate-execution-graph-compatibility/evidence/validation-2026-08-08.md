# Execution graph compatibility validation

## Passed checks

- `cargo test -p execution-graph-core`: 10 tests passed.
- `cargo test --workspace --all-targets`: the full Rust workspace passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed with an isolated Cargo target directory.
- `cargo check -p execution-graph-core --target wasm32-unknown-unknown`: passed.
- `cargo check -p execution-graph-core --target thumbv7em-none-eabihf`: passed.
- `choregraph-foundation-check --root .`: passed.
- The foundation checker passed 13 positive and negative library tests and 2 path-classification tests.
- The selected Nix checks for workspace tests, workspace Clippy, no-std targets, foundation boundaries, and Cairn policy passed.
- Cairn validation and the proposal, design, and tasks gates passed with the current Cairn default policy.
- The production Verus trace command reported `95 verified, 0 errors`.

## Bounded baseline blockers

The full `nix flake check path:$PWD -L` does not pass on the inherited durable-suspension work. The trace negative fixture `verus/trace-failure-fixtures/empty_trace_mutation.rs` calls `replay_final_states_from_spec` with 4 arguments, but the inherited function now requires 6 arguments. This compatibility change does not edit that fixture or the suspension proof API.

The inherited workspace also has Rust formatting drift outside `execution-graph-core` and the foundation checker. Focused formatting for the changed Rust packages passed.

The repository-owned generated Cairn policy predates the current Cairn workflow artifact shape. The current CLI rejects it because workflow artifacts do not contain `output_path`. Lifecycle validation therefore used `/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json` explicitly. This validates the change package, but it does not claim that the repository-owned policy migration is complete.

## Non-claims

These checks prove only the selected compatibility source, delegation, bounds, and lifecycle package structure. They do not prove the inherited suspension change, the current Cairn policy migration, whole-workspace proof freshness, runtime execution, product authority, or release eligibility.
