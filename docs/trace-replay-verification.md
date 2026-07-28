# Trace replay verification

Choregraph replays bounded v1 and v2 event slices through the production transition wrappers. Replay returns state, a consumed count, or exact failure information.

`replay` calls `step`. `replay_concrete` calls `step_concrete`. Neither replay type returns effect intents.

## Public replay model

`ReplayInput` binds these explicit values:

- one admitted machine;
- one invariant initial state;
- expected machine and initial-state bindings;
- one ordered `BoundedTrace` with its own bindings;
- one `ReplayLimits` value.

`TraceBinding` contains 32 caller-supplied BLAKE3 bytes. Choregraph compares the expected and presented bytes before it processes an event. The core does not compute, authorize, or attest these identities.

The default maximum is `65_536` events. Consumers can select a smaller explicit limit. Replay rejects an over-limit trace before it calls `step`.

## Replay decisions

A complete replay returns the final state and consumed count.

A failed replay returns:

- the first denied event index, when failure occurs during the fold;
- the accepted-event count;
- the stable replay or transition reason;
- the last valid state.

Binding, limit, and initial-state failures have no event index and consume no events.

The `ReplayDecision` type has no intent field. The fold still lets production `step` validate every accepted intent. It discards those intents before it processes the next event.

## Production linkage

The public folds are in `crates/choregraph-core/src/transition.rs`. Each fold calls the same exported transition wrapper used for live decisions.

The Verus trace specifications are in `crates/choregraph-core/src/verified/trace.rs`. They recursively apply `indexed_step_accepts_spec` and `indexed_step_next_spec` from the production-called kernel.

The positive proof root includes both production sources by path:

```text
verus/proofs/trace_replay.rs
```

Executable property tests compare public replay with repeated production `step` calls. This comparison covers the public identifier and collection adapter. It is regression evidence, not a formal proof of that adapter.

## Proved properties

The maintained Verus rail proves these properties:

1. Empty replay returns the initial state.
2. The accepted prefix length does not exceed the remaining trace length.
3. Every finite replay prefix preserves the indexed invariant.
4. Equal indexed inputs produce equal final state, consumed count, and failure index.
5. Failure identifies the first denied event and its last valid prefix state.
6. Indexed replay returns an empty dispatch-intent sequence.

The preservation theorem is stronger than success-only preservation. A denied trace also returns an invariant last valid state.

## Expected failures

The proof rail rejects false claims about:

- empty-trace state mutation;
- safety without an invariant initial state;
- skipping the first denied event;
- dispatch authority from replay.

Executable tests also cover malformed node identifiers, unknown nodes, invalid transitions, terminal mutation, reordered dependencies, stale bindings, and oversized traces.

## Trust boundary

The trace theorem starts at admitted indexed machine, state, and event views. It relies on the selected Verus binary, Z3, rustc, vstd contracts, allocator, Nix closure, and target platform.

The public replay adapter and caller-supplied binding derivation remain executable Rust. They are not Verus-proved. Tests compare this adapter with repeated production steps.

Trellis sources provide reviewed trace-bound and induction proof shapes. Choregraph adapts these shapes and verifies its own theorem locally.

## Non-claims

Trace proof success does not prove:

- event truth or authorization;
- history completeness or canonical ordering outside the supplied slice;
- storage durability or crash recovery;
- liveness, fairness, or scheduler progress;
- external effect success or exactly-once behavior;
- BLAKE3 binding derivation by a consumer.

A historical concrete intent records an earlier pure decision. It is not a fresh Basalt authorization, capability grant, or execution request.

Replay can recheck concrete references, but it discards every accepted concrete intent before the next event.

## Validation

Run the focused rail:

```text
nix build .#checks.x86_64-linux.trace-replay-verus --no-link -L
```

Run the executable and target rails:

```text
nix develop -c cargo test -p choregraph-core --all-targets
nix develop -c cargo clippy -p choregraph-core --all-targets -- --deny warnings
nix develop -c cargo check -p choregraph-core --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-core --target thumbv7em-none-eabihf
nix build .#checks.x86_64-linux.octet-deny-all --no-link -L
nix flake check -L
```
