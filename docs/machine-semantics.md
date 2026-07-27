# Bounded machine semantics

`choregraph-core` now provides one pure aggregate transition relation.

## Admission

`AdmittedMachine::admit` validates an untrusted `MachineSpec` under explicit `AdmissionLimits`.

Admission validates counts before normalized allocation. It rejects malformed identifiers, missing endpoints, duplicate facts, self-loops, cycles, unsupported profiles, and invalid effects.

An admitted machine stores immutable nodes, edges, dependency indexes, a topological order, operations, and exact effect declarations.

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

An effect declaration binds one node, transition, operation, resource, ability, request, and canonical BLAKE3 payload reference.

Declarations permit exact event requests. An event can request a subset of its matching declarations.

An accepted decision returns sorted `EffectIntent` data. The core does not authorize, dispatch, retry, persist, or execute an intent.

## Executable predicates

- `well_formed` validates normalized immutable machine facts.
- `invariant` validates aggregate state.
- `terminal_states_preserved` compares two aggregate states.
- `effects_allowed` validates exact event, declaration, and intent linkage.

## Current evidence

Positive and negative tests cover empty, linear, diamond, lifecycle, bounds, malformed graph, readiness, terminal, and effect behavior.

A bounded property test covers every node and edge input order for the Lattice diamond graph.

The single-step Verus rail proves named properties for the production-called indexed kernel. See [`single-step-verification.md`](single-step-verification.md).

The finite-trace rail proves invariant preservation for the indexed replay fold. The public replay API calls production `step`, discards historical intents, and has executable parity tests. Verus does not prove the identifier-map adapter or caller-supplied identity derivation.
