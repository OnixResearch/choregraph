# Seaglass and Lattice source inventory — 2026-07-27

## Result

The selected compatibility surface is bounded to one execution-graph implementation and one Lattice consumer baseline.

## Source identities

### Seaglass extraction source

- Repository: `git@github.com:brittonr/seaglass.git`
- Revision: `9d8726556045cbdb990ec32024cf6145b4b6cdfc`
- Package: `seaglass-execution-graph`
- Source path: `seaglass-execution-graph/src/lib.rs`
- License: `AGPL-3.0-or-later`

### Standalone execution graph

- Radicle RID: `rad:z2oYsb9jGTyp68BKYhzpivY1eK58a`
- Reviewed consumer source revision: `03736f1ec46c377ff86b451260ad68aa70ff3b0b`
- Current metadata and lifecycle revision: `c415ddf2c8252dd9ba6cb267a4e6346158069823`
- Read-only adapter: `https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git`
- Current `src/lib.rs` BLAKE3: `cb41e1d36cab9cd2ceffeeada2a862e5bb680143c99ee65e5376a22c65043dc6`
- Code lines: 701

The current source adds ten traceability comments to the Lattice vendor copy. Both files have 701 Rust code lines and the same blank-line count.

### Lattice baseline

- Revision: `c8d56ad83a57287064edb85f58808f53e4bf4536`
- Package declaration: local `vendor-cargo-deps/seaglass-execution-graph`
- Vendor source BLAKE3: `91450b94f6256c71057507b6ae97f63e505dc57377198be38087530bedefccb4`
- Code lines: 701
- Focused runner baseline: 21 tests passed

Lattice remains on its default branch baseline. Its published Radicle cutover evidence is on a feature branch and does not prove default-branch integration.

### Choregraph baseline

- Revision: `ff23d8cff062`
- Role: verified production machine, restricted Nickel frontend, and pure conversion target
- Working copy before conformance work: clean

## Behavior entry points

The reviewed execution-graph API contains:

- `ExecutionNode`, `ExecutionEdge`, `ExecutionGraph`, and `ExecutionPlan`;
- `GraphDiagnosticCode` and `GraphDiagnostic`;
- graph admission and validation;
- topological order;
- direct dependencies and dependents;
- parallel groups;
- critical path and bottlenecks;
- sparse-state readiness and resume decisions;
- local lifecycle state and action transitions;
- terminal-state detection;
- proposed-edge cycle probes.

Lattice calls this surface from `src/engine/runner.rs`. It converts product node IDs and edges, expands runner state values, maps graph diagnostics, selects ready nodes, applies lifecycle transitions, and asks for resume decisions.

## Baseline validation

These commands passed before adapter implementation:

```text
cd ../execution-graph && nix develop -c cargo test
cd ../execution-graph && nix develop -c cargo clippy --all-targets -- -D warnings
cd ../lattice && nix develop -c cargo test --test test_runner_core
cd . && nix flake check -L
```

Execution graph passed seven tests. Lattice passed 21 focused runner tests. Choregraph passed all maintained flake checks.

## Rollback and worktree state

Rollback is a VCS operation to the revisions above. No runtime or dependency fallback is permitted.

The execution-graph working copy contains uncommitted `delegate-core-to-choregraph` Cairn files. The Lattice working copy contains unrelated modified tests, configuration, and active Cairn packages. Choregraph conformance work must not modify or discard those changes.

## Claim boundary

This inventory proves source selection and baseline observations only. It does not prove behavior parity, default-branch cutover, effect safety, runtime authorization, or release eligibility.
