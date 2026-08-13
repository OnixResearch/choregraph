# Single-step verification

Choregraph verifies the indexed decision kernel that the production transition shell calls.

## Claim

The proof starts with admitted indexed graph and state views.

For these views, the maintained Verus rail proves these properties:

- graph admission includes each node once and places each edge forward;
- readiness requires each declared dependency to be complete;
- equal explicit inputs produce equal indexed decisions;
- accepted steps preserve the indexed state invariant;
- accepted intent indexes belong to the admitted transition route;
- denied steps contain no replacement state or intent indexes;
- exact and slot binding modes fail closed;
- concrete reference witnesses preserve request and payload reference tokens.

The positive proof root includes the production files by path. It does not contain a copied transition implementation.

## Production path

`AdmittedMachine::admit` builds normalized identifier maps and indexed graph views. `well_formed` calls the verified indexed graph gate.

`step` and `step_concrete` convert admitted state and event data into indexed inputs. Both wrappers call the same verified kernel.

The v2 resolver calls the verified binding predicate for each request and payload binding.

The shell converts an accepted indexed decision into public types. It then runs `invariant`, `terminal_states_preserved`, and `effects_allowed` again.

The source-link gate rejects these changes:

- a missing production call to the verified kernel;
- a separate lifecycle table in the transition shell;
- a missing production call to the verified graph gate;
- an unrecorded `external_body`, `assume_specification`, `assume`, or axiom in the proof core.

## Proof sources

The production proof sources are:

- `crates/choregraph-core/src/verified.rs`;
- `crates/choregraph-core/src/verified/topology.rs`;
- `crates/choregraph-core/src/transition.rs`;
- `crates/choregraph-core/src/admission.rs`;
- `crates/choregraph-core/src/invariant.rs`.

The proof adapts finite lifecycle, readiness, and topological-order shapes from Trellis revision `aead4527bfc3c714e71669bce656719dd2a6b2e2`.

Role choreography uses a separate proof-reference boundary. The adapter pins Trellis revision `d65974b14978e1a44b0f5067beb9ef6d83317dc8` and calls its production ownership, projectability, and endpoint-projection functions. Choregraph records exact theorem and source-set identities but does not restate those proofs.

The manifest records the exact Trellis source values. Choregraph proves the adapted functions again with its pinned verifier.

## Run the rail

Run the positive proof:

```text
nix develop -c octet-production-verus --crate-type=lib verus/proofs/single_step.rs -o /tmp/choregraph-single-step.rlib
```

Run the maintained Nix gate:

```text
nix build .#checks.x86_64-linux.single-step-verus --no-link -L
```

The Nix gate also runs seven expected-failure fixtures. Each fixture must fail with its selected postcondition diagnostic.

The concrete reference fixture changes one payload token. Verus rejects the changed postcondition.

## Evidence

The reviewed manifest is `evidence/single-step-proof.ncl`.

The generated canonical JSON is `evidence/generated/single-step-proof.json`. Its project-owned BLAKE3 value is in `evidence/generated/single-step-proof.blake3`.

The manifest binds production sources, proof sources, negative fixtures, theorem names, Verus, Z3, rustc, vstd, the Nix closure, and Trellis inputs.

A source change makes the maintained gate fail until the manifest and proof run are current.

## Trust boundary

The project proof core contains no trusted declaration or unrestricted assumption.

The evidence trusts the selected Verus binary, Z3, rustc, vstd contracts, allocation semantics, and target platform.

Identifier parsing and `BTreeMap`-to-index conversion are executable Rust. Verus does not prove these adapters.

The proof models canonical references as abstract tokens. Kamacite checks canonical bytes and BLAKE3 identities outside the proof core.

The public shell checks accepted converted outputs again. This check limits the adapter boundary, but it does not prove the adapter.

## Non-claims

The rail does not prove Nickel evaluation, authorization, effect execution, persistence, scheduling, liveness, or trace induction.

The rail does not prove Verus, Z3, rustc, vstd, the allocator, or the operating system.

The Trellis identity does not transfer a Trellis proof claim to Choregraph.

Role projection evidence proves only exact linkage to the selected Trellis result. It does not prove external-role behavior, transport, runtime execution, fault liveness, or release readiness.
