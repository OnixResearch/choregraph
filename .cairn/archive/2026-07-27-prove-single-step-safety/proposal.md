## Why

Executable tests do not establish invariant preservation for every admitted state and event. Choregraph needs formal specifications and proofs connected to the production transition implementation before consumers can call the machine core verified.

The proof rail must reuse applicable Trellis primitives without transferring stronger claims. It must also expose every trusted assumption and fail when production code diverges from its specification.

## What Changes

- Add a pinned Verus proof rail over production machine types and transition code.
- Reuse or adapt exact Trellis DAG, lifecycle, and scheduler lemmas with source identity and local revalidation.
- Prove graph-admission closure, readiness soundness, deterministic decisions, accepted-step invariant preservation, allowed intents, and denial without effects.
- Bind proof source, executable source, theorem set, trusted assumptions, verifier, and toolchain into deterministic evidence.
- Add negative proof fixtures and mutation checks that demonstrate the rail rejects violated obligations.

## Success Criteria

- Production transition functions refine named mathematical specifications.
- The accepted-step and denial theorems pass under explicit preconditions.
- The proof manifest lists all trusted code, axioms, external lemmas, and unsupported claims.
- A stale or skipped proof cannot satisfy the maintained verification gate.

## Non-Goals

- Proving Nickel evaluation, Rust compiler correctness, Verus soundness, runtime authorization, effect execution, liveness, or trace induction.
- Treating an imported Trellis reference as automatic downstream certification.

## Impact

- **Proofs**: Verus specifications, helper lemmas, theorem modules, and failure fixtures.
- **Core**: proof-compatible decomposition with pure executable functions and explicit postconditions.
- **Evidence**: BLAKE3-bound proof and trust-boundary manifests.
