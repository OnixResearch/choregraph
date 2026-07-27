## Tasks

- [x] [serial] Run and record the current execution-graph and Lattice graph fixtures before defining the new transition relation. r[choregraph.machine.compatibility_baseline]
- [x] [serial] Define bounded identifier, node, edge, lifecycle, event, effect declaration, state, decision, intent, diagnostic, and admission-limit types. r[choregraph.machine.ir] r[choregraph.machine.bounds]
- [x] [depends:machine-types] Implement pure deterministic machine admission and normalized immutable indexes. r[choregraph.machine.admission]
- [x] [depends:machine-types] Define executable `WellFormed`, `Invariant`, and `EffectsAllowed` predicates beside production types. r[choregraph.machine.invariants] r[choregraph.machine.effect_intents]
- [x] [depends:machine-admission] Implement one pure `step` relation with deterministic accepted decisions and denial without replacement state or intents. r[choregraph.machine.step] r[choregraph.machine.denial]
- [x] [parallel] Add positive empty, linear, diamond, readiness, lifecycle, and declared-intent fixtures. r[choregraph.machine.fixtures]
- [x] [parallel] Add negative malformed, cyclic, oversized, unknown-event, premature-readiness, undeclared-intent, and terminal-mutation fixtures. r[choregraph.machine.fixtures]
- [x] [parallel] Add input-order, boundary-limit, and property tests for deterministic normalized decisions. r[choregraph.machine.determinism]
- [x] [serial] Inventory every execution-graph behavior that later conformance must preserve or intentionally supersede. r[choregraph.machine.compatibility_baseline]
- [x] [serial] Run core tests, Clippy, no-std targets, architecture checks, and Cairn gates without making a formal proof claim. r[choregraph.machine.validation]
