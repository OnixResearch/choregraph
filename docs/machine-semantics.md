# Bounded machine semantics

`choregraph-core` now provides one pure aggregate transition relation.

## Admission

`AdmittedMachine::admit` validates an untrusted v1 `MachineSpec` under explicit `AdmissionLimits`.

`AdmittedMachine::admit_v2` validates a v2 `MachineSpecV2`. Both functions produce one normalized `AdmittedMachine` type.

Admission validates counts before normalized allocation. It rejects malformed identifiers, missing endpoints, duplicate facts, self-loops, cycles, unsupported profiles, and invalid effects.

An admitted machine stores immutable nodes, edges, dependency indexes, a topological order, operations, and versioned effect declarations.

A v2 declaration has one unique key. Its request and payload bindings are either exact references or named runtime slots.

Admission also stores numeric graph views. The production proof gate verifies these views without changing the public identifier model.

## Aggregate state

`MachineState` stores one local state for each admitted node. State size grows with admitted nodes, not the complete theoretical product-state space.

The executable invariant requires exact node coverage. Ready, running, completed, and failed nodes require completed dependencies.

Cancelled and pending nodes do not require completed dependencies. This rule permits cancellation before execution.

## Transition relation

`step(machine, state, event)` depends only on its arguments.

The linear profile accepts these transitions:

```text
Pending -> Ready
Ready -> Running
Running -> Completed
Running -> Failed
Pending|Ready|Running -> Cancelled
```

Ready and start actions require completed dependencies. Terminal states reject all later actions.

A denial contains one reason. It contains no replacement state and no effect intents.

## Effect intents

A v1 declaration binds one node, transition, operation, resource, ability, request, and canonical BLAKE3 payload reference.

A v2 declaration keeps static authority fields in the machine. A concrete event supplies canonical request and payload references for declared slots.

`step` handles v1 events. `step_concrete` handles v2 events. Both wrappers call the same production indexed kernel.

Each v2 declaration can occur once in an event. Output order follows the declaration key, not event source order.

A concrete intent preserves the selected slot, declared source reference, and exact event reference. Runtime references do not change machine identity.

The core does not authorize, dispatch, retry, persist, or execute an intent.

## Executable predicates

- `well_formed` validates normalized immutable machine facts.
- `invariant` validates aggregate state.
- `terminal_states_preserved` compares two aggregate states.
- `effects_allowed` validates v1 event, declaration, and intent linkage.
- `concrete_effects_allowed` validates v2 key, binding, source, and exact-reference linkage.

## Current evidence

Positive and negative tests cover empty, linear, diamond, lifecycle, bounds, malformed graph, readiness, terminal, and effect behavior.

A bounded property test covers every node and edge input order for the Lattice diamond graph.

The single-step Verus rail proves named properties for the production-called indexed kernel. See [`single-step-verification.md`](single-step-verification.md).

The finite-trace rail proves invariant preservation for the indexed replay fold. The public replay API calls production `step`, discards historical intents, and has executable parity tests. Verus does not prove the identifier-map adapter or caller-supplied identity derivation.
