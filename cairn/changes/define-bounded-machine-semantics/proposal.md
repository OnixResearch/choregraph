## Why

Choregraph needs one canonical, bounded, parameterized machine model before it can prove behavior or lower Nickel graphs. The current execution-graph core separates DAG and local lifecycle helpers, but it does not define one aggregate transition relation with explicit effect intents and denial semantics.

Enumerating every global product state would grow exponentially and would make practical workflow graphs unusable. The aggregate machine must instead represent node state as bounded data and apply local transition rules under graph invariants.

## What Changes

- Define the canonical machine IR, admission limits, immutable admitted machine, aggregate state, event vocabulary, decision type, diagnostics, and effect-intent type.
- Define `WellFormed`, `Invariant`, `EffectsAllowed`, readiness, and the pure `step` relation.
- Reject malformed, oversized, cyclic, unknown, premature, and terminal-mutation inputs deterministically.
- Preserve state and emit no intents on every denied transition.
- Emit only declared, bounded effect intents; no effect is executed by the core.
- Import the Lattice diamond graph and malformed variants as initial fixtures.

## Success Criteria

- `choregraph-core` has one deterministic transition implementation usable without Nickel or a runtime shell.
- Aggregate state is parameterized by admitted nodes rather than an enumerated product-state table.
- Accepted transitions preserve executable invariants at the test-oracle level before formal proof.
- Denied transitions expose stable reasons, preserve state, and emit no intents.

## Non-Goals

- Formal proof discharge, Nickel evaluation, Kamacite encoding, or Lattice cutover.
- Executing, authorizing, retrying, persisting, or scheduling effects.
- Proving liveness, fairness, external success, or exactly-once behavior.

## Impact

- **Core**: canonical types, admission, invariants, transition relation, readiness, and replay-neutral decisions.
- **Fixtures**: positive diamond and negative malformed, bounds, readiness, and terminal cases.
- **Compatibility**: mapping inventory for every current execution-graph public behavior.
