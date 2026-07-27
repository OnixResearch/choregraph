## Why

Single-step preservation does not by itself state that every accepted event sequence remains safe. Choregraph also needs a trace theorem and a deterministic replay API that never turns historical effect intents into new dispatch authority.

Consumers need exact failure positions and prefix state when replay encounters malformed or invalid history. Choregraph must provide those pure decisions without owning storage or recovery policy.

## What Changes

- Define bounded traces, trace prefixes, replay decisions, and indexed diagnostics.
- Prove invariant preservation across finite traces by induction from the single-step theorem.
- Prove deterministic replay for equal machine, initial state, trace, and explicit limits.
- Ensure replay computes state only and never emits dispatchable effect intents.
- Preserve the last valid prefix and exact failing event index on invalid history.
- Add positive, negative, boundary, and property-based trace fixtures.

## Success Criteria

- Every successfully replayed bounded trace ends in an invariant state.
- Replay of historical events cannot authorize or execute effects.
- Equal replay inputs produce equal state and diagnostics.
- Invalid history reports the exact first invalid event without inventing later state.

## Non-Goals

- Durable storage, log completeness, external anchoring, crash recovery, liveness, fairness, or exactly-once effects.
- Proving that an observed event is truthful or authorized.

## Impact

- **Core**: trace fold, replay mode, indexed diagnostics, and bounded prefix handling.
- **Proofs**: induction theorem and supporting prefix lemmas.
- **Fixtures**: empty, valid, denied, truncated, malformed, oversized, and reordered traces.
