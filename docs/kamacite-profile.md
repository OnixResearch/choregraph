# Kamacite interchange profile

Choregraph selects Kamacite revision `6245a52498f7de511df660eabec3f9b350338cb6`.

The selected Kamacite license is `MIT OR Apache-2.0`. This dependency is compatible with the Choregraph `AGPL-3.0-or-later` distribution.

Choregraph vendors Kamacite's selected canonical codec from the exact revision. `tools/sync-kamacite-codec.rs` performs the reproducible import.

The adapter uses the exact Preserves crate version from the profile. It does not use an ambient sibling path.

The reviewed Kamacite Git source requires authenticated access. Therefore, it is not a Cargo dependency. The hash-checked selected snapshot keeps builds anonymous and deterministic without copying unrelated Kamacite policy.

## Supported profiles

- Canonical codec: `kamacite-preserves-packed-canonical-v2-preserves-rs-5.0.0-rc.7`
- Preserves package: `5.0.0-rc.7`
- Preserves specification: `0.996.3`
- FSM specification: `kamacite.fsm-spec.v2`
- FSM specification root: `kamacite-fsm-spec-v2`
- FSM receipt: `kamacite.fsm-receipt.v3`
- FSM receipt root: `kamacite-fsm-receipt-v3`
- FSM coverage: `kamacite.fsm-coverage.v1`
- Determinism policy: `deterministic`
- Receipt kinds: `exhaustiveness` and `trace`
- Denial side-effect class: `no-side-effect`

Canonical packed Preserves bytes are authoritative. Text and JSON values are projections only.

## Choregraph schemas

The v1 aggregate descriptor uses `choregraph.machine-preserves.v1` and the `choregraph-machine-v1` root. Its review schema is [`../schemas/preserves/choregraph-machine-v1.preserves`](../schemas/preserves/choregraph-machine-v1.preserves).

The v2 descriptor uses `choregraph.machine-preserves.v2` and the `choregraph-machine-v2` root. Its review schema is [`../schemas/preserves/choregraph-machine-v2.preserves`](../schemas/preserves/choregraph-machine-v2.preserves).

Concrete intents use `choregraph.concrete-intent-preserves.v2`. The intent schema is [`../schemas/preserves/choregraph-concrete-intent-v2.preserves`](../schemas/preserves/choregraph-concrete-intent-v2.preserves).

The descriptor contains normalized nodes, edges, dependencies, effect declarations, limits, compatibility revisions, and exact profile identities. It identifies aggregate state as a parameterized node map. It does not contain a global product-state table.

The local lifecycle is one closed `kamacite.fsm-spec.v2` descriptor. It contains six local states, five events, accepted transitions, and explicit denial rows. Every local state and event pair has one row.

## Identity chain

Project-owned, domain-separated BLAKE3 identities bind these materials:

- canonical v1 or v2 machine bytes;
- canonical concrete-intent bytes;
- canonical lifecycle bytes;
- canonical admitted IR bytes;
- the proof set and trusted-surface material;
- generator source material;
- adapter source material;
- the selected Kamacite profile;
- the selected toolchain.

Kamacite FSM identities use the selected Kamacite domains and raw canonical packed bytes. Display names, host paths, and diagnostics are not identity material.

## Adapter and shell

`choregraph-kamacite` converts and validates in-memory values only. It has no filesystem, process, network, clock, policy, or runtime-effect authority.

`choregraph-cli write-kamacite-corpus fixtures/kamacite` writes the maintained corpus. Tests rebuild each corpus entry and compare its exact bytes with the checked-in artifact.

The corpus manifest is [`../fixtures/kamacite/corpus.ncl`](../fixtures/kamacite/corpus.ncl). It includes positive canonical artifacts and negative malformed, non-canonical, unsupported, stale, mismatched, JSON, global-enumeration, trace, and overclaim cases.

## Receipt roles

Choregraph validates admission, lifecycle, source, and proof-reference link receipts. It also validates Kamacite-shaped exhaustiveness and trace receipts.

All receipt bodies bind exact subject and dependency identities. Exhaustiveness receipts bind the state, event, transition, and denial coverage identities.

Trace receipts bind each presented transition and positive guard evidence. Concrete-intent receipts bind one v2 machine identity, intent identity, and declaration key.

## Receipt non-claims

A receipt binds exact artifacts and observed trace data. It does not prove transition correctness, source equivalence, verifier soundness, event truth, authorization, effect success, or release eligibility.

A concrete-intent identity names content. Lattice must add run and attempt facts to create an occurrence identity.
