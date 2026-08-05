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

## Durable suspension profile

The opt-in `choregraph.lifecycle.durable-suspension.v1` profile adds one non-terminal `Suspended` local state. Machines select the profile by exact string at admission. No adapter infers the profile from node names or product configuration.

A suspended node carries one bounded suspension class and one canonical BLAKE3 reference. The class shares the identifier byte limit. The reference shares the reference byte limit. Both are opaque: the kernel compares bytes and never parses approvers, policies, credentials, clocks, or payloads.

The durable profile accepts these additional transitions:

```text
Ready -> Suspended (suspend)
Running -> Suspended (suspend)
Suspended -> Ready (resume-signal)
```

Suspend and resume-signal events carry the exact class and reference. Both actions always reject effect requests. A resume signal must match the recorded class and reference byte for byte. Resume also requires completed dependencies. Duplicate, stale, wrong-node, wrong-class, wrong-reference, nested, premature, malformed, and oversized cases fail closed with one deterministic denial reason.

A suspended node is never ready. `resume_decision` returns `AwaitSignal` for suspended nodes even when dependencies are complete. Dependency completion alone never releases a suspension.

`AdmittedMachine::declared_suspension_state` reconstructs a recorded wait after a process restart. Admission validates each declared suspension: known node, bounded class, canonical reference, no duplicates, durable profile required. Consumers can also build a restart state with `MachineState::with_suspensions`; the executable invariant accepts the state only when records exist exactly for suspended nodes.

Replay binds suspension facts through the production `step` relation. A trace can end validly in suspended state. Replay of the same admitted trace produces the same state, decisions, and trace identity.

The canonical Kamacite path versions durable artifacts explicitly. Linear machines keep the version-1 descriptor with unchanged bytes. Durable machines use the version-2 descriptor with suspension declarations and the durable FSM artifact. The version-1 descriptor rejects durable machines, and the version-2 descriptor rejects linear machines.

Suspension conformance does not establish approval authority, persistence durability, signal truth, timeout correctness, or downstream effect safety. See `SUSPENSION_NON_CLAIMS` in `choregraph-core`.

## Migration

Existing v1 machines keep `LINEAR_LIFECYCLE_V1` behavior, canonical bytes, and decisions. Consumers that need durable waits re-admit their machine with the durable profile string and declare suspension facts explicitly. A v1 artifact never gains suspension fields.

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
