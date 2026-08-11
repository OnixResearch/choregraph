## Why

Choregraph needs portable canonical machine artifacts and receipts without duplicating Kamacite's Preserves encoding, source envelopes, FSM descriptors, or evidence vocabulary.

Kamacite's explicit closed FSM model fits the local node lifecycle. It does not fit an enumerated aggregate DAG machine because the global product state grows exponentially. Choregraph must own the parameterized aggregate descriptor while using Kamacite for canonical interchange and receipt bindings.

## What Changes

- Define a versioned Choregraph Preserves descriptor for bounded aggregate graph and machine data.
- Represent the selected per-node lifecycle through a supported Kamacite FSM profile.
- Wrap Choregraph artifacts in Kamacite canonical envelopes with project-owned BLAKE3 identities.
- Validate Kamacite FSM, exhaustiveness, trace, source, and proof-reference receipts without treating them as behavioral proofs.
- Bind machine, lifecycle, proof set, trace, generator, and toolchain identities into one portable artifact set.
- Add canonical positive and negative fixtures, round trips, and stale-binding checks.

## Success Criteria

- Canonical packed Preserves bytes are authoritative for Choregraph artifact identity.
- JSON remains a diagnostic or compatibility projection only.
- The aggregate graph is parameterized and does not enumerate every global state.
- Kamacite receipts link exact artifacts while retaining explicit proof and runtime non-claims.

## Non-Goals

- Moving Choregraph semantics into Kamacite.
- Claiming a receipt proves transition behavior or source equivalence.
- Executing FSMs, authorizing effects, or defining Lattice runtime policy.

## Impact

- **Schemas**: Choregraph-owned Preserves aggregate descriptor and supported Kamacite profile declarations.
- **Adapter**: pure conversions and validators in `choregraph-kamacite`.
- **Fixtures**: canonical positive, malformed, stale, mismatched, and overclaim artifacts.
