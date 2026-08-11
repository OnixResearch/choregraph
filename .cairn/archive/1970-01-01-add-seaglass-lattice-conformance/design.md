## Context

The execution-graph repository is already published through Radicle. Seaglass and Lattice consume its exact revision. Choregraph cannot replace this path based only on similar APIs or unit tests.

## Success Contract

Completion requires an immutable behavior baseline, complete dual-run inventory, Lattice end-to-end fixture, negative source and semantic checks, one implementation after cutover, and tested VCS rollback. A wrapper that leaves both transition implementations active is false completion.

## Compatibility Adapter

`choregraph-seaglass` maps existing graph, node, edge, lifecycle, transition, readiness, resume, and analytics inputs into Choregraph types. It maps stable output classes back where compatibility requires them.

The adapter owns conversion only. It contains no independent graph traversal, readiness, lifecycle, or transition algorithm.

## Dual-Run Matrix

Test-only comparison covers:

- admission diagnostics;
- cycle probes and topological order;
- dependency and dependent queries;
- parallel groups and readiness;
- lifecycle transitions and terminal rejection;
- resume decisions;
- critical path and bottleneck queries;
- canonical state and event mapping.

Each mismatch has one stable case identity. Unreviewed mismatches block migration.

## Lattice First Consumer

The first positive fixture starts from the reviewed Lattice diamond Nickel graph. It evaluates through the selected frontend, lowers to canonical IR, admits the machine, advances valid node events, and compares state and intent results with the baseline consumer behavior.

Negative fixtures cover cycles, missing endpoints, duplicate IDs, premature readiness, terminal mutation, undeclared effects, authorization denial results, stale source identities, mismatched receipts, and replay suffix rejection.

The downstream check uses an exact reviewed Lattice revision in a disposable workspace. Ambient sibling paths are local test inputs only and do not become release source declarations.

## Authority Boundary

Choregraph returns pure decisions and effect intents. Lattice retains runtime selection, Basalt calls, configured roots, `cap-std`, process and network adapters, retries, schedules, triggers, persistence, diagnostics, and evidence.

The integration proves no external effect behavior.

## Cutover Order

1. Choregraph proof, interchange, and frontend gates pass.
2. Choregraph parity and Lattice disposable integration pass.
3. Choregraph release evidence identifies one immutable source.
4. `execution-graph-core` becomes a compatibility facade over that source.
5. Lattice dual-runs, then adopts Choregraph as its production machine path.
6. Temporary duplicate and dual-run code is removed.

Rollback is a reviewed VCS operation to recorded source states. No hidden runtime or dependency fallback is added.

## Claim Boundary

Passing conformance proves selected behavior equality and adapter linkage only. It does not prove formal theorem soundness, Lattice workflow correctness, runtime authorization, external effects, or release readiness.
