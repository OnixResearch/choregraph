# Seaglass adapter parity validation — 2026-07-27

## Result

The conversion-only Seaglass adapter passed the selected execution-graph parity matrix.

Production `choregraph-seaglass` depends only on `choregraph-core`. The exact legacy implementation is a test-only Git dependency at revision `03736f1ec46c377ff86b451260ad68aa70ff3b0b`.

## Approach registry

Three approach families were reviewed:

1. A legacy-shaped facade converts graph, state, action, and query values into Choregraph production values. This is the selected production mechanism.
2. A core-type-only bridge would avoid DTO conversion but would not preserve the selected consumer vocabulary. It was rejected for this migration.
3. A test-only comparator compiles the exact reviewed execution-graph source and compares results. It is retained only as validation support.

A wrapper with its own graph traversal, readiness logic, lifecycle table, or transition relation was excluded as false completion.

## Production delegation

The adapter calls these Choregraph production paths:

- `AdmittedMachine::admit` for graph admission and topological order;
- admitted dependency and dependent indexes for graph queries;
- `parallel_groups`, `critical_path`, and `bottlenecks` for analytics;
- `lifecycle_target` for local lifecycle compatibility;
- `validate_invariant`, `ready_nodes`, and `resume_decision` after exact sparse-state expansion;
- `step` for aggregate event decisions;
- `would_create_cycle` for proposed-edge probes.

The adapter source contains no `while` loop, recursive graph function, lifecycle table, or independent readiness calculation. Its only `for` loop expands supplied sparse state values into an already allocated exact state map.

## Parity matrix

Eight integration tests cover:

- empty graph admission;
- diamond graph admission and deterministic order;
- direct dependency and dependent queries;
- parallel groups;
- critical path and bottleneck ranking;
- malformed graph diagnostic classes;
- cycle rejection and proposed-edge probes;
- all 30 local state and action cells;
- ready sets and resume decisions for valid sparse states;
- aggregate event delegation with no effect intent;
- stricter identifier, unknown-node, and invalid-state failures.

The exact baseline passed seven tests. The focused Lattice runner baseline passed 21 tests. Choregraph passed the eight parity tests and two adapter-boundary tests.

## Reviewed semantic dispositions

Selected valid graph, lifecycle, readiness, resume, and analytics inputs match the baseline.

Three legacy behaviors are intentionally stricter:

- A non-empty identifier outside the Choregraph ASCII profile is rejected as `InvalidNodeId`.
- An unknown resume node returns `UnknownNode`; the legacy helper treated it as an implicit pending root.
- A sparse state that violates dependency invariants returns `InvalidState`; the legacy helper tolerated the inconsistent map.

These differences fail closed and remain visible in negative tests. They are not presented as exact legacy behavior.

Checked graph queries also return typed conversion errors instead of hiding malformed input behind an empty collection.

## Adversarial audit

The audit checked empty input, duplicate facts, missing endpoints, self-loops, cycles, unknown nodes, malformed identifiers, terminal mutation, failed dependencies, incomplete dependency state, level conversion, dependent-count conversion, and effect-intent leakage.

No accepted aggregate graph event emitted an intent because the compatibility machine declares no effects. An unexpected accepted intent becomes `AuthorityBoundaryViolation` and is not exposed for dispatch.

## Validation

These commands passed:

```text
nix develop -c cargo test -p choregraph-seaglass
nix develop -c cargo test --workspace
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo check -p choregraph-seaglass --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-seaglass --target thumbv7em-none-eabihf
nix run .#octet-deny-all
nix flake check -L
```

Octet reported zero findings, warnings, and errors. The full flake ended with `all checks passed!`.

## Claim boundary

This result validates the selected test matrix only. It does not prove arbitrary source equivalence, default-branch Lattice adoption, downstream compilation, runtime authorization, external effect behavior, duplicate production-code removal, or release eligibility.
