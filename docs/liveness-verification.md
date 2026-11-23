# Workflow liveness verification

Choregraph ships production-linked Verus proofs for bounded workflow liveness in `verus/proofs/liveness_lemmas.rs`, verified through the proof root `verus/proofs/liveness.rs`. Because the proof root imports the exact production `crates/choregraph-core/src/verified.rs`, `verified/topology.rs`, and `verified/trace.rs` modules, the liveness theorems apply directly to the production indexed transition relation without an unverified translation gap.

## Ranking functions

Two natural-number ranking functions measure distance to global termination over an indexed state vector `states`:

- `non_terminal_count_spec(states)`: counts nodes whose state is not in `{Completed, Failed, Cancelled}`. It equals `0` if and only if every node is terminal (`all_terminal_spec(states)`), and is bounded above by `states.len()`.
- `linear_steps_remaining_spec(states)`: assigns rank `3` to `Pending`, `2` to `Ready` (or `Suspended`), `1` to `Running`, and `0` to terminal states, and sums across all nodes. It equals `0` if and only if every node is terminal, and is bounded above by `3 * states.len()`.

## Proved properties

Verified by `octet-production-verus --crate-type=lib verus/proofs/liveness.rs`:

1. **`non_terminal_count_zero_iff_all_terminal`** and **`linear_steps_remaining_bounded_and_zero_iff_terminal`**: Both ranking functions are bounded (`<= N` and `<= 3 * N`) and vanish if and only if every node is terminal.
2. **`accepted_step_never_increases_non_terminal_count`**: Every accepted transition of any action or profile preserves or decreases `non_terminal_count_spec` (terminal nodes never regress).
3. **`accepted_terminal_event_strictly_decreases_non_terminal_count`**: Every accepted `Complete`, `Fail`, or `Cancel` event strictly decreases `non_terminal_count_spec` by `1`.
4. **`accepted_linear_progress_strictly_decreases_steps_remaining`**: Every accepted linear progress action (`MarkReady`, `Start`, `Complete`, `Fail`, or `Cancel`) strictly decreases `linear_steps_remaining_spec`.
5. **`admitted_dag_has_enabled_progress_action`** (DAG deadlock-freedom / progress): For any admitted acyclic graph (`admitted_graph_spec` with `dependencies_in_edges_spec`) in an invariant linear state (`no_failed_cancelled_or_suspended_spec`) with at least one non-terminal node, there exists a node and a linear progress action (`MarkReady`, `Start`, or `Complete`) that is accepted by `indexed_step_accepts_spec` and strictly decreases `linear_steps_remaining_spec`.
6. **`ready_node_persistence_no_starvation`** (no starvation of ready nodes): Any node `u` that satisfies `dependency_ready_spec` accepts `ACTION_MARK_READY`, and any accepted step on any other node `v != u` preserves `u`'s enabledness (because completed dependencies are terminal and immutable). Under weak fairness, no ready node can be starved by concurrent work elsewhere in the DAG.
7. **`fair_linear_progress_trace_bounded_termination`**: Any accepted trace of linear progress events has length at most `linear_steps_remaining_spec(initial_states) <= 3 * states.len()`, and reaches `all_terminal_spec` whenever the remaining rank is exhausted.

## Expected failure fixtures

`verus/liveness-failure-fixtures/` confirms that Verus rejects false liveness claims:

- `non_terminal_count_decreases_on_start.rs`: falsely claims `ACTION_START` decreases `non_terminal_count_spec`.
- `suspended_dag_accepts_start_without_resume.rs`: falsely claims a `Suspended` node accepts `ACTION_START` without a matching `ResumeSignal`.
- `cycle_has_ready_pending_node.rs`: falsely claims a cyclic two-node pending graph has a node with completed dependencies.

## Runner assumptions

The kernel is pure and passive: it decides transitions when the host submits events. Reaching a terminal state in a live system therefore relies on the host runner satisfying these explicit assumptions:

- **Weak fairness:** Every persistently enabled progress action (`MarkReady`, `Start`, `Complete`, or cancellation of blocked descendants) is eventually submitted by the runner.
- **No lost events:** Completion events for started nodes (and matching `ResumeSignal` or v2 `Cancel` events for durable waits) are eventually delivered without silent loss.
- **Finite suspension waits:** Under `choregraph.lifecycle.durable-suspension.v1`/`.v2`, an arbitrary sequence of `Suspend` / `ResumeSignal` cycles does not decrease `linear_steps_remaining_spec`; termination requires the host to bound suspensions or cancel blocked waits.
- **Failure propagation by the host:** When a predecessor transitions to `Failed` or `Cancelled`, its downstream `Pending` nodes cannot satisfy `dependencies_complete_spec`; reaching global `all_terminal_spec` requires the host to submit `Cancel` for those blocked descendants.

## Trace validation for recorded Lattice runs

`crates/choregraph-seaglass/tests/lattice-trace-validation.rs` connects the specification to recorded Lattice-style runs by replaying complete diamond, durable wait/resume, and failure-plus-cancellation runs through both production `transition::step` and `transition::replay`. It checks that every recorded step is allowed, that `non_terminal_count` and `linear_steps_remaining` decrease as proved, and that injected premature readiness or post-terminal mutations fail at the exact offending step index.

## Non-claims

This proof rail does **not** prove:

- that an external host scheduler or worker pool is fair;
- that external effects terminate, succeed, or run at most once;
- that network messages, resumes, or completion events are delivered without loss;
- that storage or crash recovery preserves the runner's event queue.

## Validation

```sh
nix build --builders '' --no-link -L .#checks.x86_64-linux.liveness-verus
nix develop --builders '' -c cargo test -p choregraph-seaglass --test lattice-trace-validation
```
