# Phase 1 baseline record

r[choregraph.machine.suspension_proof]

## S1 dependency check

The local change `bind-concrete-effect-intents` is archived at
`cairn/archive/2026-07-28-bind-concrete-effect-intents`. Concrete effect
intents run through the one production `step` relation. The transition
shell calls `crate::verified::indexed_step(` at exactly two sites, and
`crate::verified::lifecycle_target(` supplies the only lifecycle table.
No independent transition table exists in the shell.

## S2 baseline runs

All commands ran in the `choregraph-suspension` worktree at parent
commit `1d4865b` (`make execution and proof authority easier to
distinguish`) before any suspension edit.

| Check | Command | Result |
| --- | --- | --- |
| Workspace tests | `nix develop -c cargo test --workspace --all-targets` | pass |
| Clippy deny-warnings | `nix develop -c cargo clippy --workspace --all-targets -- --deny warnings` | pass |
| Foundation rail | `nix develop -c cargo run -p choregraph-foundation-check -- --root .` | pass (`foundation-boundary: passed`) |
| Cairn validate | `cairn validate --root .` (pinned `70db8fe`) | PASS |
| Gate proposal | `cairn gate proposal model-durable-suspension-transitions --root .` | PASS |
| Gate design | `cairn gate design model-durable-suspension-transitions --root .` | PASS |
| Gate tasks | `cairn gate tasks model-durable-suspension-transitions --root .` | PASS |

One environment note: the first Clippy run failed with a stale
incremental-cache error under the shared `~/.cargo-target`. Removing the
stale `choregraph_core-*` incremental directories fixed it. This was an
environment fault, not a source failure.

The broad Verus, canonical-corpus, no-std, Octet, and Nix rails run
again as closeout evidence in C1.

## S3 match inventory

Exhaustive `LocalState` matches and lifecycle tables that the new
`Suspended` state touches:

- `crates/choregraph-core/src/model.rs`: `LocalState` definition,
  `MachineState` entries, `initial_state`.
- `crates/choregraph-core/src/transition.rs`: `local_state_code`,
  `local_state_from_code`, `lifecycle_target`, `step`, `step_concrete`,
  `replay`, `replay_concrete`.
- `crates/choregraph-core/src/verified.rs`: `STATE_*` codes,
  `ACTION_*` codes, `lifecycle_target_spec`, `lifecycle_target`,
  `local_state_valid_spec`, `terminal_state_spec`,
  `indexed_step` and its spec functions, single-step lemmas.
- `crates/choregraph-core/src/verified/trace.rs`: replay spec fold over
  `indexed_step`, trace lemmas.
- `crates/choregraph-core/src/invariant.rs`: `validate_invariant`,
  `terminal_states_preserved`, `well_formed` profile check.
- `crates/choregraph-core/src/analytics/mod.rs`: `ready_nodes`,
  `resume_decision`, `dependencies_complete`, `has_failed_dependency`.
- `crates/choregraph-core/src/admission.rs` and `concrete.rs`:
  lifecycle-profile admission for schema v1 and v2.
- `crates/choregraph-kamacite/src/lifecycle.rs` and `aggregate.rs`:
  `STATES` table, `state_id`, `state_from_id`, `row_guards`.
- `crates/choregraph-nickel/src/lower.rs`: `state` string conversion,
  lifecycle-profile passthrough.
- `crates/choregraph-seaglass/src/lib.rs`: `core_state` and
  `compatibility_state` conversions.
- `crates/choregraph-cli/src/corpus.rs`: canonical corpus descriptors
  with lifecycle-profile identity.
- `verus/proofs/single_step.rs` and `trace_replay.rs`: proof roots that
  include the production `verified.rs` and `verified/trace.rs`.
- `verus/failure-fixtures/` and `verus/trace-failure-fixtures/`:
  negative proof fixtures with pinned expectations in `flake.nix`.
- `evidence/single-step-proof.ncl` and `evidence/trace-replay-proof.ncl`:
  proof manifests with pinned source BLAKE3 identities, theorem counts,
  and fixture counts. `flake.nix` pins those counts (`theorems == 7`,
  `negative_fixtures == 7`) and per-source hashes.

## S4 Swamp reference record

Swamp provides the product reference for durable manual gates: the
process exits, persisted state remains suspended, and a later invocation
resumes from recorded facts.

Adopt/reject decisions under the AGPL clean-room boundary:

- Adopt the mechanism shape only: a non-terminal durable wait state,
  one exact external signal as the resume condition, and restart from
  recorded facts. These are behavioral requirements restated from the
  product description, not copied code.
- Reject any Swamp source, schema, identifier, or runtime dependency.
  Swamp is AGPL; Choregraph copies no code and links no artifact.
- Reject product semantics: approvers, identities, credentials, clocks,
  persistence engines, notifications, and authorization stay outside
  Choregraph.
- Choregraph treats the suspension class and reference as opaque bounded
  data. A matching reference proves reference equality only.
