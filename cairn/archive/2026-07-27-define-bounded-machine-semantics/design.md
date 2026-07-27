## Context

A workflow graph with multiple local node states has an exponential explicit global state space. Choregraph therefore needs a parameterized state map and a small transition relation, not a generated table containing every aggregate state.

## Success Contract

Completion requires one pure transition implementation, bounded admission, deterministic decisions, explicit invariants, declared effect intents, and positive and negative fixtures. A DTO-only model, an unbounded graph, or separate test and production transition functions is false completion.

## Canonical Model

`MachineSpec` contains a schema identity, bounded nodes, bounded edges, one selected local lifecycle profile, and bounded effect declarations. Stable identifiers use validated newtypes.

`AdmittedMachine` contains normalized immutable graph facts and indexes derived by admission. Callers cannot construct it without validation.

`MachineState` contains one local state for each admitted node. Missing, extra, or unknown state entries are invalid.

`Event` names one admitted node and one supported lifecycle action or observed outcome. Unknown nodes, unsupported events, and malformed payload references are denied.

`Decision` is either `Accept { next, intents }` or `Deny { reason }`. A denial carries no replacement state and no effect intents. This shape makes denial immutability explicit.

`EffectIntent` is bounded data that names an exact declared operation, resource, ability, request identity, and canonical payload reference. It is an authorization and dispatch request, not an executed effect.

## Admission and Invariants

Admission checks configured limits before allocation growth. It rejects duplicate or empty identifiers, duplicate edges, missing endpoints, self-loops, cycles, unsupported lifecycle profiles, undeclared operations, malformed references, and inconsistent bounds.

`WellFormed(machine)` covers immutable topology, identifier closure, acyclicity, limits, lifecycle closure, and effect-declaration closure.

`Invariant(machine, state)` covers exact node-state coverage, legal local states, dependency readiness, and terminal-state immutability.

`EffectsAllowed` requires each emitted intent to match the exact node, transition, operation, resource, ability, and request-binding declaration in the machine.

## Transition Relation

The pure `step` function validates the event, checks readiness and local lifecycle rules, derives one next state, and derives a deterministic ordered intent list. It does not consult ambient policy or execute an intent.

The implementation uses explicit control flow and named limits. Derived collections use deterministic ordering. No hidden defaults depend on input order.

## Compatibility

The first behavior corpus covers execution-graph admission diagnostics, topological ordering, direct dependencies and dependents, parallel groups, readiness, lifecycle transitions, resume decisions, critical path, and bottlenecks.

Temporary comparison code is test-only. Choregraph becomes the one implementation after parity and proof gates pass.

## Validation

Positive fixtures cover empty and diamond graphs, valid lifecycle paths, ready sets, and declared intents. Negative fixtures cover every admission code, each limit, unknown events, premature starts, undeclared intents, terminal mutation, and input-order variation.

## Claim Boundary

Passing executable tests establishes the selected examples only. Formal invariant preservation, trace safety, runtime authorization, and external effect behavior remain outside this change.
