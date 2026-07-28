# Execution-graph compatibility baseline

The reviewed execution-graph revision is `c415ddf2c8252dd9ba6cb267a4e6346158069823`.

The reviewed Lattice revision is `c8d56ad83a57287064edb85f58808f53e4bf4536`.

## Baseline results

These commands passed before the new transition relation was added:

```text
cd ../execution-graph && nix develop -c cargo test
cd ../lattice && nix develop -c cargo test --test test_runner_core
```

Execution-graph passed all seven unit tests. The focused Lattice runner passed all 21 tests.

## Behavior inventory

| Current behavior | Choregraph disposition |
|---|---|
| Graph and node constructors | `choregraph-seaglass` converts values into `MachineSpec`. |
| Empty graph admission | Preserved as an empty admitted machine. |
| Duplicate and empty node diagnostics | Preserved with typed Choregraph diagnostic codes. |
| Duplicate, missing, and self-loop edge diagnostics | Preserved with typed Choregraph diagnostic codes. |
| Cycle rejection | Preserved with lexicographic bounded admission. |
| Graph size rejection | Replaced by caller-selected named limits. |
| Lexicographic topological order | Preserved. |
| Direct dependencies and dependents | Preserved through immutable admitted indexes. |
| Parallel dependency levels | Preserved. |
| Critical-path tie breaking | Preserved by path length and lexicographic order. |
| Bottleneck ranking | Preserved by dependent count and node identifier. |
| Ready-node selection | Preserved after an adapter expands sparse legacy state into exact aggregate state. |
| Local lifecycle transitions | Preserved for pending, ready, running, completed, failed, and cancelled states. |
| Terminal mutation rejection | Preserved through denial without replacement state or intents. |
| Resume decisions | Preserved for admitted nodes and exact state. Unknown nodes now fail closed. |
| Cycle probe for a proposed edge | Preserved through `would_create_cycle`. |
| Serde field representation | Deferred to the Kamacite and compatibility changes. |
| Diagnostic display text | Stable compatibility text is derived from typed Choregraph admission codes. |

## Parity result

The adapter dual-runs the exact execution-graph source at revision `03736f1ec46c377ff86b451260ad68aa70ff3b0b`. Eight integration tests cover the complete selected matrix. The legacy source is a test-only dependency.

Production adapter decisions call Choregraph admission, indexes, analytics, lifecycle, readiness, resume, aggregate step, and cycle-probe functions. The adapter does not contain graph traversal or transition algorithms.

## Intentional strictness

The current execution-graph helper treats missing state entries as pending. Choregraph requires exact state coverage.

The Lattice adapter expands its sparse persisted map before it calls Choregraph. Unknown, extra, malformed, or dependency-inconsistent entries fail closed.

The legacy helper accepted some non-empty identifiers outside the Choregraph profile and treated an unknown resume node as an implicit pending root. Choregraph rejects those cases with `InvalidNodeId` or `UnknownNode`.

Effect declarations are permissions for matching event requests. A declaration does not require every matching event to request that effect.

## Claim boundary

This inventory records expected compatibility. It does not establish full parity or authorize consumer cutover.
