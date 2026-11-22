## Goals

- Record a single-writer, epoch-fenced ownership invariant.
- Keep the invariant deterministic and in the `no_std` core.
- Use one implementation for production transitions and proof targets.
- Leave storage, scheduling, and runtime authority outside the core.

## Entity ownership

The epoch-fenced API models one writer for one entity. The caller supplies the current ownership epoch; Choregraph does not observe or authorize it.

The core accepts these values:

1. the current entity epoch;
2. the epoch on the incoming mutation;
3. the entity identity;
4. the event target and current local state.

The core returns an admission decision from these supplied values. It reads no storage, clock, or network. Consumers that claim fencing must use `step_epoch_fenced`; the compatibility `step` API remains authority-neutral.

## Decision rules

The pure core returns one of two decisions:

- `Admit` — the incoming mutation carries the current entity epoch.
- `Reject` — the mutation epoch is stale or mismatched.

A request that no longer carries the supplied current epoch cannot reach the production transition through the fenced API. The decision gates one entity and does not prove runtime writer exclusivity.

## Shard by construction

The machine keeps each admitted node's state separate. There is no shared mutable database.

The fenced decision admits only an event that targets the same entity. It never widens to a shared store or a cross-entity write, but it does not prove external storage isolation.

## Output and proof

The decision contains public, bounded fields:

- the entity identity;
- the current and offered epochs;
- the disposition;
- implementation and contract identities;
- required non-claims.

Production transitions and proof targets use the same decision implementation. Verus specifications and proofs appear beside the production code. Reused Trellis primitives are recorded with their exact claims and licenses.

## Functional core and shell

The pure core computes the admission from supplied values.

The shell owns compiler adapters and lowering. Runtime, storage, and scheduling stay outside Choregraph.

## Reference

The Celld ownership model at revision `553ae73f83c87c3f7c7a5f73c32c2211d9d7341f` (one writer per cell, epoch-fenced, shard by construction) is a bounded Apache-2.0 reference input. It is a comparison source, not a Choregraph requirement, code source, parity claim, or equivalence claim.

## Verification

Positive coverage includes a mutation carrying the current epoch and admission within one entity.

Negative coverage includes a stale or mismatched epoch, a terminal mutation, and a cross-entity write.

Boundary coverage rejects storage or clock reads, shared-store coupling, runtime-authority claims, and arbitrary-distributed-correctness claims.
