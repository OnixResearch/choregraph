// Canonical IR is partitioned by semantic owner; wire values remain unchanged.
#![allow(
    unknown_lints,
    fragile_exhaustive_enum_match,
    non_trait_imports,
    reason = "canonical IR modules retain explicit value imports and closed schema matches"
)]

// r[impl choregraph.machine.ir]
// r[impl choregraph.machine.bounds]
// r[impl choregraph.machine.parameterized_state]
// r[impl choregraph.machine.effect_intents]
// r[impl choregraph.machine.denial]
// r[impl choregraph.machine.suspension_profile]
// r[impl choregraph.machine.suspension_boundary]
pub mod analysis;
pub mod automaton;
pub mod capacity;
pub mod dispatch;
pub mod graph;
pub mod history;
pub mod outcome;
pub mod pause;
pub mod semantics;
