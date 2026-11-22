## Why

Choregraph owns a pure verified machine kernel with deterministic transition semantics over bounded execution graphs. Its machine-semantics contract does not yet record an entity-ownership invariant: one writer per entity, fenced by a monotonic epoch, so a writer that lost its epoch cannot mutate the entity, and blast radius stays bounded to one entity.

Celld gives each Durable Object one writer and an ownership epoch that fences the cell. A node that lost its lease cannot damage the cell state. The application shards by construction, and the blast radius of any failure is one cell. That is a clean reference for a machine invariant: record the single-writer, epoch-fenced guarantee as an explicit transition rule.

Choregraph should record this as an invariant of its machine semantics and formalize it. It does not own runtime authority, storage, or consensus.

## What Changes

- Add the single-writer, epoch-fenced entity-ownership invariant to machine semantics.
- Require every mutation that claims epoch fencing to use the typed wrapper with a supplied current entity epoch.
- Reject any transition from a stale epoch.
- Keep the invariant in the `no_std` core, with one implementation for production and proof.
- Add positive, negative, and boundary fixtures for stale-epoch, terminal-mutation, and cross-entity cases.
- Reference the reviewed Celld ownership model as a bounded, non-parity input.

## Impact

The change makes the supplied-fact ownership gate explicit, testable, and provable. It does not add storage, consensus, or runtime authority, and it does not claim that the authority-neutral compatibility `step` API is fenced.

Epoch and receipt identities remain BLAKE3.

## Dependencies

This change has no downstream prerequisite.

## Non-goals

- Do not add storage, a scheduler, or a consensus service.
- Do not claim runtime or deployment authority.
- Do not claim parity with, or equivalence to, the Celld distributed runtime.
- Do not convert the invariant into a proof of arbitrary distributed correctness.
