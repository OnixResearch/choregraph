## Context

The production `ready_nodes` function returns pending nodes whose direct dependencies are complete. The maintained proof rail currently proves only this implication:

```text
reported_ready(node) => dependencies_complete(node)
```

A dependency-driven consumer also needs the reverse implication. Without it, a runtime can cite Choregraph safety while an incomplete readiness implementation silently over-synchronizes work.

The design adapts the proof pattern from *A Barrier-Free Synchronization Algorithm for Multi-Engine AI Accelerators*. It does not adapt the paper's semaphore formulas or hardware model.

## Success Contract

Completion requires an exact production-linked equivalence theorem, deterministic frontier ordering, expected proof failures, current evidence identities, and downstream compatibility.

These results are false completion:

- proving a copied or proof-only readiness function;
- proving only that reported nodes have complete dependencies;
- labeling executable examples as formal proof;
- claiming runtime liveness from pure readiness completeness;
- moving scheduling or capacity policy into Choregraph.

## Semantic Predicate

Define one specification predicate over admitted indexed views:

```text
dependency_ready(machine, state, node)
  = node exists
  && state[node] == Pending
  && every declared dependency of node is Completed
```

Suspended, ready, running, completed, failed, and cancelled nodes do not satisfy this predicate. Durable resume behavior remains a separate transition concern.

## Required Theorems

The production-linked proof rail establishes these properties:

```text
node in ready_nodes(machine, state)
  <=> dependency_ready(machine, state, node)
```

The returned frontier contains each satisfying node exactly once. Its order follows the admitted topological order.

For an otherwise valid, effect-free `MarkReady` event:

```text
step accepts MarkReady(node)
  <=> dependency_ready(machine, state, node)
```

The transition theorem states every required premise. It does not hide profile, event-shape, state-invariant, or identifier assumptions.

## Production Linkage

The theorem root includes the production readiness and indexed transition sources by path. The source-link gate rejects a second readiness table or a proof-only frontier builder.

The public identifier adapter remains executable Rust. The proof can start from admitted indexed views, as the current single-step rail does. Evidence keeps the identifier-conversion non-claim.

## Trellis Reuse

Review the pinned Trellis dependency-scheduler and graph-frontier primitives before local proof work. Reuse only exact compatible lemmas.

Record the source revision, source identity, imported scope, and local proof obligation. A citation or vendored source name does not transfer proof.

## Negative Evidence

Expected-failure fixtures cover these defects:

- one eligible pending node is omitted;
- one ineligible node is included;
- one eligible node appears twice;
- frontier order differs from admitted topological order;
- `MarkReady` accepts with an incomplete dependency;
- `MarkReady` rejects an otherwise valid eligible target;
- a copied readiness implementation replaces the production call;
- an unrestricted assumption enters the proof root.

Each fixture must fail for its selected obligation.

## Evidence and Claim Boundary

Update the reviewed Nickel proof manifest and generated BLAKE3 identities. Bind theorem names, production sources, proof sources, negative fixtures, Verus, Z3, Rust, Nix, and any Trellis inputs.

The evidence proves exact pure readiness under recorded assumptions. It does not prove scheduling, capacity, fairness, persistence, authority, effect execution, external progress, or release eligibility.

## Downstream Boundary

Lattice can use the proved frontier as one dispatch input. Lattice remains responsible for resource capacity, authorization, attempt reservation, conflicts, persistence, retries, recovery, and effects.
