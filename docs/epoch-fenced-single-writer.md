# Epoch-fenced single-writer boundary

Choregraph provides a pure epoch-fence decision and a production transition wrapper. The wrapper admits a mutation only when all these supplied facts hold:

- the entity exists in the admitted machine;
- the event targets that same entity;
- the offered epoch equals the supplied current epoch;
- the entity is not in a terminal state.

The decision reads no storage, clock, process, network, lease service, or mutable global state.

## API

`EntityEpoch` is a typed `u64` value. Choregraph compares values but does not allocate or advance them.

`EpochFence` carries one admitted `NodeId`, the supplied current epoch, and the offered mutation epoch.

`decide_epoch_fence` returns these bounded facts:

- entity identity;
- current and offered epochs;
- admit or an exact rejection class;
- contract and implementation identities;
- required non-claims.

`step_epoch_fenced` calls `decide_epoch_fence` first. A rejected fence returns no replacement state or effect intent. An admitted fence delegates to the existing production `step` function. It does not contain another lifecycle table.

The authority-neutral `step` API remains available for compatibility and proof composition. A consumer that claims epoch fencing must use `step_epoch_fenced` and must supply the current epoch from its own admitted authority source.

The consumer must atomically compare-and-swap the observed epoch **and** state revision when committing the new state. If either changed, discard the decision and its intents; recompute from a fresh snapshot. Equality in the pure gate alone cannot fence a concurrent writer.

## Rejection classes

The gate rejects these cases in order:

1. the fenced entity is not in the admitted machine;
2. the event targets another entity;
3. the offered epoch differs from the current epoch;
4. the entity is terminal.

After admission, the normal machine transition can still reject invalid state, dependencies, lifecycle actions, effects, or invariants.

## Proof linkage

The production decision calls `verified::primitive::ownership::epoch_fence_decision` with one immutable `EpochFenceFacts` value naming entity knowledge, entity equality, both epochs, and terminal state. Its postcondition is equality with `epoch_fence_decision_spec` over that same value. The Verus proof imports the exact production source through the `verified.rs` namespace root. The focused Nix rail recursively checks the moved proof sources for trust escapes, checks the production delegation, and runs negative fixtures for stale epochs, cross-entity mutations, and terminal mutations. The named record does not change rejection precedence or the facts supplied by the ownership wrapper.

The proof establishes only the decision over supplied facts. It does not establish the truth, freshness, or monotonic allocation of those facts.

## Identities

- Contract: `blake3:791e39edd60b0ae0520bc57b08cc13e1b6d36fe0679faf798d8f939995afe63c`
- Implementation: `blake3:208455a5a682f658e7a0c4fa0cdd3224a2cc0e44d11ca5615450224c972717c7`

These are domain-separated version identities for the selected contract and proof-linked decision shape. They are not source-tree digests.

## Celld reference boundary

The bounded reference is Deno's Celld repository at revision `553ae73f83c87c3f7c7a5f73c32c2211d9d7341f`, licensed under Apache-2.0.

The reviewed source records one SQLite database per cell, owner records with an epoch, compare-and-swap ownership updates, and effects that carry the cell epoch. The useful mechanism is one writer per cell with stale writers fenced by an epoch and failures isolated by per-cell state.

Choregraph does not copy Celld runtime code. It does not adopt Celld object storage, SQLite, leases, routing, replication, V8 execution, scheduling, membership, or runtime authority. The reference does not establish parity, equivalence, interoperability, or distributed-system correctness.

## Non-claims

An admitted fence does not prove:

- that the supplied current epoch is authoritative or fresh;
- lease acquisition, renewal, or expiry;
- storage atomicity, durability, or compare-and-swap behavior;
- consensus, membership, routing, or failover;
- that a runtime has exactly one active writer;
- cross-entity transaction safety;
- effect success or release eligibility.
