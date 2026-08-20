## Why

Choregraph proves readiness soundness. A reported ready node has complete dependencies.

The current proof does not establish the reverse direction. It does not prove that every pending node with complete dependencies appears in the production ready frontier.

Lattice needs this exact semantic boundary before it replaces serial workflow execution with dependency-driven dispatch. The upstream proof must define graph readiness without taking runtime scheduling, capacity, authority, or effect ownership from Lattice.

## What Changes

- Define one mathematical dependency-readiness predicate over admitted production graph and state views.
- Prove exact equivalence between that predicate and membership in the production `ready_nodes` result.
- Prove that the result contains each eligible node once in admitted topological order.
- Prove an effect-free `MarkReady` event is accepted exactly when its target is pending and dependency-ready, under all other valid premises.
- Add expected-failure fixtures for omitted eligible nodes, extra ineligible nodes, duplicate frontier members, and premature transitions.
- Bind the new theorem set, production sources, assumptions, negative fixtures, and toolchain into current BLAKE3 proof evidence.

## Success Criteria

- Production readiness has a proved soundness and completeness theorem.
- The compatibility adapter delegates to the same production readiness implementation.
- A proof of a copied readiness implementation cannot satisfy the gate.
- Downstream consumers can distinguish semantic readiness from runtime dispatchability.

## Non-Goals

- Scheduling resources or executing effects.
- Proving fairness, runtime liveness, eventual dispatch, persistence, authorization, external success, or exactly-once behavior.
- Adding cyclic graphs, structured loops, semaphore counters, or Trainium-specific allocation.
- Treating the referenced accelerator paper as code or proof evidence.

## Impact

- **Core**: proof-compatible readiness specifications and production-linked helper postconditions.
- **Proofs**: exact frontier membership, order, and transition theorems.
- **Evidence**: updated proof manifest, negative fixtures, and claim boundaries.
- **Downstream**: current Lattice compatibility and source-link validation.
