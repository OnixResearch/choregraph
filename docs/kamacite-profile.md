# Kamacite interchange profile

Choregraph selects Kamacite revision `6245a52498f7de511df660eabec3f9b350338cb6`.

The selected Kamacite license is `MIT OR Apache-2.0`. This dependency is compatible with the Choregraph `AGPL-3.0-or-later` distribution.

Choregraph vendors Kamacite's selected canonical codec from the exact revision.
`tools/sync-kamacite-codec.rs` checks the original codec and license blobs, applies
the reviewed local correction with pinned GNU patch and zero fuzz, then checks
the correction and resulting source BLAKE3 values recorded in
`vendor/kamacite-core-selected/SOURCE.ncl`. The correction qualifies Rust policy
and error quantities by unit, places codec constants under `preserves::codec::profile`,
and borrows static malformed-input diagnostics. It does not publish a Kamacite
revision or change the selected Git revision or original license files. Refresh
the selected source through the importer, not by editing its vendored output.
The correction also separates codec regressions into an exactly hash-bound child
member. The importer installs that child as `preserves/tests.rs`, the path Rust
resolves from the selected codec's `#[path]` module declaration, and removes the
obsolete nested layout. Both corrected member hashes are checked before
installation and by the foundation evidence gate. Consumer packaging restores
both members; ordinary workspace tests compile the actual codec regressions.


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

Role choreography uses distinct versioned source, global, local, ordered-local-set, diagnostics, and projection-receipt Preserves profiles. Packed Preserves bytes own each identity. The complete local-set identity binds normalized role order and every local artifact identity.

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

The Rust API has no crate-root reexports or compatibility aliases. Callers select
the semantic owner:

- `aggregate::record` owns machine descriptor rows; `aggregate::assembly` builds
  and validates linear and durable artifacts; `aggregate::policy` binds the
  selected interchange profile and compatibility revisions.
- `codec::{graph, effect, state}` owns schema/version and root-label constants.
  `codec::{encode, decode}` is the shared canonical conversion boundary.
- `concrete::{record, conversion}` owns v2 machine, concrete-intent, and exact
  intent-linkage receipt data and conversion.
- `lifecycle::{record, conversion}` owns closed FSM rows, coverage evidence,
  lifecycle construction, validation, and transition identities.
- `receipt::{record, validation}` owns linkage and FSM receipt data and checks.
- `profile::{source, protocol, observation}` owns role-choreography source,
  global/local/local-set, and diagnostics/projection-receipt encoders. Names in
  these modules omit the former `ROLE_CHOREOGRAPHY_` and
  `encode_role_choreography_` prefixes; source uses `SCHEMA`, `ROOT`, and `encode`.
- `identity` owns identity domains, framing, validation, and hashing; `boundary`
  owns the adapter's explicit claims and non-claims.

This source-level cutover preserves schema strings, Serde fields, field order,
canonical machine/intent encodings, identity framing and denial semantics.
Source-bound adapter/generator identities and dependent receipts must still be
regenerated when their exact source bytes change; a namespace-only change does
not promise an unchanged source-backed artifact identity. The admitted-IR
projection borrows the already-normalized aggregate descriptor records rather than copying
the machine a second time. Owned wire rows allocate their required strings at
their typed conversion boundary. Error diagnostics use bounded category tables;
unclassified future categories retain their variant and payload in diagnostics.

`choregraph-cli write-kamacite-corpus fixtures/kamacite` writes the maintained corpus. Tests rebuild each corpus entry and compare its exact bytes with the checked-in artifact.

The corpus manifest is [`../fixtures/kamacite/corpus.ncl`](../fixtures/kamacite/corpus.ncl). It includes positive canonical artifacts and negative malformed, non-canonical, unsupported, stale, mismatched, JSON, global-enumeration, trace, and overclaim cases.

## Receipt roles

Choregraph validates admission, lifecycle, source, and proof-reference link receipts. It also validates Kamacite-shaped exhaustiveness and trace receipts.

All receipt bodies bind exact subject and dependency identities. Exhaustiveness receipts bind the state, event, transition, and denial coverage identities.

A choreography projection receipt also binds the source envelope, import closure, evaluated DTO, global protocol, each local protocol, complete local set, Trellis sources and proof references, lowerer, schemas, limits, diagnostics, toolchains, and non-claims.

Trace receipts bind each presented transition and positive guard evidence. Concrete-intent receipts bind one v2 machine identity, intent identity, and declaration key.

## Receipt non-claims

A receipt binds exact artifacts and observed trace data. It does not prove transition correctness, source equivalence, verifier soundness, event truth, authorization, effect success, or release eligibility.

A concrete-intent identity names content. Lattice must add run and attempt facts to create an occurrence identity.
