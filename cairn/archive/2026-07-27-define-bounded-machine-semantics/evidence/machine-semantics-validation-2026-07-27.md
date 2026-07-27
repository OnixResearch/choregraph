# Machine semantics validation evidence

Date: 2026-07-27

## Compatibility baseline

Before implementation, execution-graph passed seven unit tests. Lattice passed all 21 focused runner-core tests.

The exact commands and behavior inventory are in [`../../../../docs/execution-graph-compatibility-baseline.md`](../../../../docs/execution-graph-compatibility-baseline.md).

## Implemented core

`choregraph-core` now has bounded typed identifiers, machine admission, immutable normalized indexes, aggregate state, executable invariants, lifecycle transitions, effect intents, readiness, resume, cycle probes, and graph analytics.

The machine represents one state entry per admitted node. It does not enumerate the complete product-state space.

Effect declarations permit exact event requests. They do not make every declared effect mandatory for every matching event.

## Adversarial audit

The audit examined effect omission, recursive readiness, terminal mutation, resource bounds, BTree ordering, and exact declaration matching.

The final corpus includes recursive dependency validation and effect-order normalization. External CPU, memory, filesystem, network, and authorization limits remain explicit non-claims.

The admitted graph is acyclic. The aggregate invariant checks each completed ancestor through every local state entry.

## Maintained validation

These commands passed:

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- --deny warnings
cargo check -p choregraph-core --target wasm32-unknown-unknown
cargo check -p choregraph-core --target thumbv7em-none-eabihf
cargo run -p choregraph-foundation-check -- --root .
nix flake check -L
```

The machine corpus has positive and negative cases. It includes all Lattice diamond node and edge input orders and all local state and action pairs.

The full flake validation passed on `x86_64-linux`. Nix omitted systems that differ from the current host.

## Claim boundary

These results validate executable examples and maintained bounds. They do not discharge formal single-step or trace proof obligations.

They do not prove evaluator behavior, authorization, effect execution, persistence, liveness, fairness, external success, or exactly-once behavior.
