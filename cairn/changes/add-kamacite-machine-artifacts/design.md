## Context

Kamacite owns lossless interchange, canonical Preserves encoding, source envelopes, generic FSM descriptors, trace and exhaustiveness receipts, and evidence profiles. Choregraph owns aggregate execution-graph semantics and formal proofs.

## Success Contract

Completion requires authoritative canonical Preserves artifacts, deterministic identities, exact profile bindings, pure conversion, and positive and negative fixtures. A new custom envelope, JSON authority, or an enumerated global FSM is false completion.

## Artifact Split

Choregraph owns a versioned `choregraph.machine` Preserves descriptor. It contains bounded nodes, edges, lifecycle-profile identity, effect declarations, admission limits, schema identity, and required compatibility metadata.

Kamacite owns the envelope, canonical packed encoding, generic local FSM descriptor, trace receipt, exhaustiveness receipt, source binding, and proof-reference vocabulary.

The node lifecycle is represented once as a closed Kamacite FSM profile. The aggregate machine references that profile for each applicable node. It does not materialize the product of every local state combination.

## Identity Chain

Project-owned BLAKE3 identities bind:

- canonical machine descriptor bytes;
- local lifecycle FSM descriptor;
- admitted normalized IR;
- proof theorem set and trusted-surface manifest;
- generator and adapter source;
- selected Kamacite schema profile;
- selected toolchain.

Display names, host paths, and diagnostics do not enter semantic identity unless their selected schema declares them semantic.

## Adapter Boundary

`choregraph-kamacite` is a pure adapter over in-memory values. Filesystem access and diagnostic rendering remain in `choregraph-cli`.

The initial integration pins a reviewed Kamacite source or artifact revision. It does not use an ambient sibling path as release input. A direct crate dependency is accepted only after supported profiles and license compatibility are recorded.

## Receipts and Claims

An admission receipt records that Choregraph accepted an exact canonical machine. A proof-reference receipt records an exact proof artifact and theorem role. A trace receipt records exact presented history.

These receipts do not prove proof soundness, source equivalence, transition correctness, event truth, runtime authorization, effect success, or release eligibility.

## Validation

Positive fixtures cover canonical round trips, stable identities, local FSM coverage, aggregate descriptor admission, and exact receipt linkage.

Negative fixtures cover malformed Preserves, unsupported profiles, duplicate fields, non-canonical bytes, stale identities, lifecycle mismatch, proof mismatch, trace mismatch, JSON substitution, global-state enumeration, and overclaim text.

## Claim Boundary

Passing adapter checks proves selected encoding, identity, schema, and linkage facts only. Choregraph proofs and consumer runtime evidence retain their separate roles.
