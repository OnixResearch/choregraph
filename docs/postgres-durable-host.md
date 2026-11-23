# Reference Postgres durable host adapter

`examples/postgres-durable-host/` is a standalone reference host crate outside Choregraph's verified `no_std + alloc` kernel (`choregraph-core`). It demonstrates the "the log is the agent" durable execution pattern using a single PostgreSQL table (`workflow_log`).

## Single-table design (`workflow_log`)

One table holds both:

- **Per-node claim slots** (`entry_kind = 'step'`, negative `seq`): track the monotonic `epoch`, current `claimed_by` worker ID, and `lease_until` timestamp for each admitted node.
- **Append-only kernel event log** (`entry_kind = 'event'`, positive `seq`): stores every accepted kernel `Event` as JSONB plus the resulting canonical `state_hash`.

The worker loop has four operations:

1. **Replay (`replay_workflow`):** reads `entry_kind = 'event'` rows in `seq` order and folds them through `choregraph_core::transition::replay` to reconstruct the exact `MachineState`. Restart requires no checkpoint table or worker memory.
2. **Claim (`claim_next_step`):** replays the log to find runnable nodes (`analytics::ready_nodes` plus non-terminal `Ready`/`Running` nodes with expired leases), locks one row with `SELECT ... FOR UPDATE SKIP LOCKED`, increments `epoch`, and records a lease.
3. **Append (`append_fenced_event`):** inside one transaction, locks the node's step slot (`FOR UPDATE`), reads the authoritative `current_epoch`, replays the log, calls `choregraph_core::ownership::step_epoch_fenced`, and appends the event row only if both the epoch fence and kernel transition admit.
4. **Release (`release_step`):** clears `claimed_by` and `lease_until` only if `epoch` still matches the worker's claim so a stale worker can never clear a newer worker's lease.

## Boundary: outside the verified kernel

`examples/postgres-durable-host` is intentionally an independent crate (`[workspace]` in its own `Cargo.toml`) outside `crates/choregraph-core`. `choregraph-core` retains zero database, network, clock, process, or `std` dependencies.

## Proved kernel properties relied upon

This host relies on four Verus-verified kernel properties:

1. **Epoch-fenced decision (`verus/proofs/epoch_fence.rs`):** `step_epoch_fenced` admits a mutation only when the entity exists, the event targets that entity, `offered_epoch == current_epoch`, and the node is non-terminal; stale epochs (`EpochMismatch`) and post-terminal retries (`TerminalMutation`) produce no replacement state and no effect intents.
2. **Single-step invariant preservation and readiness completeness (`verus/proofs/single_step.rs`):** `ready_nodes` returns exactly the dependency-complete `Pending` frontier, terminal states never mutate, and denied transitions emit zero intents.
3. **Deterministic, effect-free trace replay (`verus/proofs/trace_replay.rs`):** replaying an accepted event log reproduces the exact kernel state deterministically and never returns historical effect intents for re-dispatch.
4. **Bounded DAG liveness (`verus/proofs/liveness.rs`):** under fair claiming and completion delivery, every accepted completion strictly decreases `non_terminal_count_spec` until all nodes reach a terminal state.

## Host and Postgres responsibilities (not proved by Choregraph)

Choregraph's proofs cover the pure kernel decisions over supplied facts only. The host and PostgreSQL remain responsible for:

- transactional row locking (`SELECT ... FOR UPDATE SKIP LOCKED` and `FOR UPDATE`) and isolation;
- strictly monotonic epoch allocation inside Postgres;
- lease duration tuning and clock consistency for `lease_until <= NOW()`;
- disk durability (`fsync`, WAL persistence, backups); and
- idempotent or outbox-coordinated external effect execution when a worker crashes between executing an external effect and committing `Complete`.

## Running the real Postgres integration tests

```sh
nix shell 'github:NixOS/nixpkgs/20535e48e12c86043b577b8518234ff5dbb26957#postgresql' \
  -c nix develop --builders '' \
  -c cargo test --manifest-path examples/postgres-durable-host/Cargo.toml
```

The integration suite starts a real ephemeral PostgreSQL cluster via `initdb` and `pg_ctl` over a Unix socket in `$TMPDIR`, kills a spawned worker subprocess mid-step with `SIGKILL`, verifies stale-epoch rejection and single-application on resume, checks `SKIP LOCKED` concurrency, exercises durable suspension/resume, and asserts byte-for-byte replay from `workflow_log` alone.
