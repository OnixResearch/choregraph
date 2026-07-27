## Context

The production core must remain normal Rust while Verus checks its semantic obligations. A separate proof-only transition implementation would not verify consumer behavior.

## Success Contract

Completion requires proof-to-production linkage, successful named theorems, explicit assumptions, negative proof evidence, and deterministic manifests. A proof of a second model, copied theorem text, or tests labeled as proofs is false completion.

## Proof Architecture

Each non-trivial operation is split into a pure specification, a pure executable implementation, and small helper lemmas. Executable functions state postconditions that connect results to their specifications.

The proof set includes:

- admitted topology satisfies `WellFormed`;
- topological order contains each admitted node once and respects every edge;
- reported readiness implies every dependency is complete;
- `step` is deterministic for equal explicit inputs;
- accepted `step` preserves `Invariant`;
- accepted intents satisfy `EffectsAllowed`;
- denied `step` returns no next state and no intents.

The primary accepted-step theorem is:

```text
WellFormed(machine)
&& Invariant(machine, state)
&& step(machine, state, event) = Accept(next, intents)
=> Invariant(machine, next)
&& EffectsAllowed(machine, state, event, intents)
```

## Trellis Reuse

Candidate Trellis modules include DAG admission, topological order, cycle detection, job state, dependency scheduling, and durable replay. Every reused artifact is pinned by source revision and BLAKE3 identity.

Direct code reuse is preferred when toolchain, license, and API boundaries fit. Otherwise Choregraph carries a small adapted lemma with provenance and proves it locally. Imported references never become proof by citation alone.

## Trusted Surface

The proof manifest names Verus, Rust, standard library assumptions, trusted declarations, admitted external facts, proof-tool flags, and theorem source identities. It distinguishes proved theorems from tests and runtime assumptions.

No unrestricted axiom, `assume`, external body, or unverifiable escape can enter the strict core without an explicit reviewed trust record and blocking policy.

## Negative Evidence

Compile-fail or proof-fail fixtures violate one postcondition at a time. They cover premature readiness, terminal mutation, undeclared intents, state-map mismatch, and denial that emits an intent.

The maintained rail verifies that each negative fixture fails for its expected obligation. Process failure without the expected diagnostic is not accepted evidence.

## Claim Boundary

Passing the proof rail establishes the named mathematical properties for the selected source and assumptions. It does not prove the verifier, compiler, evaluator, operating system, authorization backend, or external effects.
