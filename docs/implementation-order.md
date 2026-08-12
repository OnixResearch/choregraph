# Choregraph implementation order

This document indexes the active Cairn changes for the full Choregraph implementation.

## Local Choregraph graph

```text
establish-choregraph-foundation
  -> define-bounded-machine-semantics
       -> prove-single-step-safety
            -> prove-trace-replay-safety
       -> add-kamacite-machine-artifacts
            -> add-restricted-nickel-lowering

prove-trace-replay-safety
  + add-restricted-nickel-lowering
  -> add-seaglass-lattice-conformance
       -> prepare-choregraph-release-evidence
```

The proof and interchange branches can proceed in parallel after machine semantics are accepted.

## Cross-repository order

1. Complete all Choregraph changes through `prepare-choregraph-release-evidence`.
2. Accept an immutable Choregraph source and evidence identity.
3. Implement `execution-graph/cairn/changes/delegate-core-to-choregraph`.
4. Complete Lattice's execution-graph and Nickel evaluator prerequisite changes.
5. Implement `lattice/.cairn/changes/adopt-choregraph-verified-machines`.
6. Implement `lattice/.cairn/changes/authorize-choregraph-effect-intents`.
7. Remove every temporary dual-run path after parity and rollback checks pass.

## External boundaries

No Kamacite change is required for the first implementation. Choregraph owns its aggregate Preserves descriptor and consumes Kamacite's existing envelopes and FSM receipt profiles.

No Trellis change is required until a specific missing lemma is demonstrated. Choregraph must revalidate every reused or adapted proof locally.

No cap-root or cap-std change is required. Lattice uses `cap-root` for lexical admission and its existing `cap-std` directory-capability shell for file effects.

A Basalt change is not yet required. Lattice consumes Basalt's exact authorization boundary. Public release remains blocked while Basalt's UCAN dependency requires unavailable authenticated SSH access.

## Completion evidence

Full completion requires:

- one production transition implementation in Choregraph;
- passing single-step and finite-trace proofs over production code;
- positive and expected-negative proof rails;
- canonical Kamacite artifact and receipt linkage;
- restricted Nickel lowering with explicit source-semantic non-claims;
- execution-graph and Lattice parity;
- removal of duplicate transition logic;
- exact Cargo and Nix source alignment;
- tested VCS rollback;
- current Cairn, Octet, traceability, and Nix evidence.

Passing lifecycle gates alone does not satisfy these implementation requirements.
