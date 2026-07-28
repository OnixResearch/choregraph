## Why

Choregraph can emit only effect values that already exist exactly in an immutable machine declaration. The current Lattice workflow profile lowers no effect declarations. Lattice therefore cannot submit a resolved runtime request and payload without inventing an intent or rebuilding the machine around run data.

Rebuilding the machine for each request makes machine identity unstable. Inventing the intent in Lattice bypasses Choregraph's declared-effect check. Both paths break the intended authority boundary.

## What Changes

- Add a versioned concrete-effect schema that keeps one immutable machine identity.
- Give each effect declaration a stable key and explicit exact-reference or runtime-slot bindings with declared source references.
- Let runtime events supply canonical request and payload references for declared slots.
- Emit concrete intents that retain the declared operation, resource, ability, source, and exact event references.
- Make canonical packed Preserves the authoritative request and payload encoding and use domain-separated BLAKE3 content identities.
- Normalize old and new schemas into one production transition implementation while preserving existing v1 behavior and artifact identities.
- Extend the Nickel frontend, Kamacite artifacts, Seaglass conversion layer, production-linked proofs, replay checks, fixtures, and release evidence.
- Produce an exact Lattice handoff contract without adding Basalt, UCAN, persistence, filesystem, process, network, or dispatch authority to Choregraph.

## Impact

- **Core**: `crates/choregraph-core/` gains versioned declarations, binding modes, concrete request references, and concrete intent output through the existing pure transition boundary.
- **Frontend**: `crates/choregraph-nickel/`, Nickel contracts, and fixtures gain an explicit `lattice-workflow-v2` effect profile.
- **Interchange**: `crates/choregraph-kamacite/` and Preserves schemas gain canonical v2 artifacts and BLAKE3 identity chains.
- **Compatibility**: `crates/choregraph-seaglass/` remains conversion-only and exposes the v2 intent handoff without authorization logic.
- **Proofs**: single-step and trace rails prove declared binding, exact reference preservation, denial safety, determinism, and replay suppression over production code.
- **Release**: evidence and downstream checks bind the exact schema, source, proof, adapter, and candidate identities. Publication remains a separate explicitly authorized action.
- **Testing**: positive and negative tests cover exact and slot bindings, malformed references, undeclared requests, duplicate intents, stale identities, v1 compatibility, replay, and downstream conversion.
