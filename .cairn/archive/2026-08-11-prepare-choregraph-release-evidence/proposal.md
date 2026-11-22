## Why

Proof, compiler, canonical artifact, and consumer compatibility results need one deterministic release-evidence boundary. Without exact source and toolchain bindings, a consumer could attach passing evidence to different machine code, schemas, or proofs.

Choregraph also needs an immutable publication candidate before execution-graph and Lattice can perform their separate cutovers. Preparing evidence must not publish or push without explicit operator authority.

## What Changes

- Build a deterministic release bundle for source, binaries, schemas, canonical fixtures, proofs, assumptions, toolchains, Kamacite receipts, and conformance results.
- Add Valence-friendly role and non-claim projections without transferring proof or release authority.
- Add Cairn lifecycle, traceability, Octet, Cargo, Clippy, no-std, Verus, Nickel, canonical corpus, and Nix release gates.
- Define an immutable Radicle publication candidate and acquisition evidence without automatic publication.
- Require downstream execution-graph and Lattice cutovers to cite exact accepted Choregraph evidence.

## Success Criteria

- The bundle fails closed on stale, skipped, mismatched, malformed, or overclaimed evidence.
- Repeated generation over equal inputs produces equal project-owned identities.
- The release candidate names exact source, proof, schema, dependency, and toolchain identities.
- No command publishes, pushes, signs, or changes remote authority without explicit operator action.

## Non-Goals

- Proving verifier, compiler, evaluator, operating-system, remote transport, or consumer correctness.
- Publishing a Radicle repository or changing GitHub configuration during this change.
- Claiming general Nickel source equivalence or external effect correctness.

## Impact

- **Evidence**: release manifest, proof bundle, Kamacite carriers, Valence projection, and verification receipts.
- **Release**: deterministic checks, immutable publication candidate, and downstream handoff contract.
- **Documentation**: exact claims, assumptions, unsupported claims, acquisition, and rollback guidance.
