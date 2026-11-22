# Single-step verification

Choregraph verifies the indexed decision kernel that the production transition shell calls.

## Claim

The proof starts with admitted indexed graph and state views.

For these views, the maintained Verus rail proves these properties:

- graph admission includes each node once and places each edge forward;
- dependency readiness holds exactly for pending nodes with completed declared dependencies;
- the production frontier contains every eligible node once in admitted topological order;
- for an invariant snapshot and pending target, effect-free `MarkReady` is accepted exactly when its dependencies are complete;
- two results satisfying the executable indexed step's postcondition on equal inputs have equal observable fields;
- the transition guards imply preservation of the indexed invariant and terminal states, with no post-state premise;
- cancellation never completes a node, and only `Complete` from `Running` can produce `Completed`;
- each non-terminal node under linear v1 or durable v2 can accept an effect-free cancellation (literal linear states cannot be suspended);
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

The provisional guard check does not allocate a replacement state. The indexed kernel derives post-state safety from the guards: a complete dependency is terminal and cannot change, and an action that requires newly complete dependencies checks them before execution. The checked adapter reuses the prior state map and updates the one target. It consumes the indexed decision type, whose constructor is sealed inside the verified kernel; the public `Decision` type is not a proof certificate.

Public denial precedence includes the adapter's limits, request shape, and declaration resolution. The verified precedence of duplicate before undeclared effects applies only to indexed requests. An unknown kernel code or failed conversion or post-check yields `InternalInconsistency`.

`ready_nodes` converts admitted state into indexed values. It calls the verified `ready_frontier` filter and maps the result into public node identifiers.

The filter scans the admitted topological order once. Its executable postcondition equals the mathematical frontier specification.

The source audit checks actual proof items and module linkage, not comments: it rejects undisclosed verifier attributes and trust escapes, checks the declared theorem items, and pins the exact count reported by the selected verifier. Production smoke checks and parity tests are separate observations; they do not prove the identifier adapters.

## Proof sources

The production proof sources are:

- `crates/choregraph-core/src/verified.rs`;
- `crates/choregraph-core/src/verified/codes/` for state, action, profile, and denial encodings;
- `crates/choregraph-core/src/verified/primitive.rs` and its `primitive/{ownership,matching,transition,validity,membership,support}.rs` for focused facts, predicates, and executable witnesses;
- `crates/choregraph-core/src/verified/kernel.rs` and its `kernel/` children for transition targets, suspension relations, decisions, and theorems;
- `crates/choregraph-core/src/verified/readiness.rs`;
- `crates/choregraph-core/src/verified/topology.rs` and its `topology/` children for cardinality, precedence, bounds, and admission obligations;
- `crates/choregraph-core/src/analytics/mod.rs`;
- `crates/choregraph-core/src/transition.rs`;
- `crates/choregraph-core/src/admission.rs`;
- `crates/choregraph-core/src/invariant.rs`.

The namespace roots expose focused child modules, not compatibility reexports. Production callers name those owners directly. `binding::concrete_binding_accepts` receives a binding mode and one immutable `ConcreteBindingFacts` record naming slot equality, source equality, exact-reference equality, and slot/source absence. The executable predicate and its named specification consume the same record and preserve the exact-binding and slot-binding conditions. Trust guards recursively cover the split production proof sources; source moves do not expand the trusted surface.

The proof adapts finite lifecycle, readiness, topological-order, filtered-collection, uniqueness, and source-order shapes from Trellis revision `aead4527bfc3c714e71669bce656719dd2a6b2e2`.

The filtered-collection shapes come from `src/bounded_peers.rs`. Choregraph applies them to admitted node indexes and proves its local obligations.

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

The Nix gate also runs 15 expected-failure Verus fixtures. Each fixture must fail with its selected postcondition diagnostic.

Five frontier fixtures cover omitted, extra, duplicate, reversed, and eligible-rejection defects.

The foundation guard has separate copied-readiness and unrestricted-assumption fixtures.

The mutation rail changes executable and specification lifecycle targets together, and separately changes the executable and specification readiness guard together. Each mutant must fail the expected semantic or preservation obligation, not merely fail to compile.

The concrete reference fixture changes one payload token. Verus rejects the changed postcondition.

## Evidence

The reviewed manifest is `evidence/single-step-proof.ncl`.

The generated canonical JSON is `evidence/generated/single-step-proof.json`. Its project-owned BLAKE3 value is in `evidence/generated/single-step-proof.blake3`.

The manifest binds production sources, proof sources, negative fixtures, theorem names, Verus, Z3, rustc, vstd, the Nix closure, and Trellis inputs.

A source change makes the maintained gate fail until the manifest and proof run are current.

The `verified` status applies only to the recorded source bytes. A digest
mismatch is not permission to re-pin a source: first qualify the changed
inputs and retain successful positive proof, expected-failure fixture, and
production-linkage/trust-guard observations. Refresh the reviewed Nickel
manifest only from that evidence, then regenerate its JSON with
`nickel export --format json evidence/single-step-proof.ncl | jq -S .`
and record the generated file's plain BLAKE3 digest. The complete Nix gate
must subsequently pass with those exact inputs.

## Trust boundary

The project proof core contains no trusted declaration or unrestricted assumption.

The evidence trusts the selected Verus binary, Z3, rustc, vstd contracts, allocation semantics, and target platform.

Normal Cargo builds also trust the scoped `RUSTC_BOOTSTRAP=verified_logic` setting in `.cargo/config.toml` for the pinned Trellis dependency. This does not enable bootstrap for Choregraph crates or change the Verus proof toolchain.

Identifier parsing and `BTreeMap`-to-index conversion are executable Rust. Verus does not prove these adapters.

The proof models canonical references as abstract tokens. Kamacite checks canonical bytes and BLAKE3 identities outside the proof core.

The public shell checks accepted converted outputs again. This check limits the adapter boundary, but it does not prove identifier conversion.

The frontier proof starts from admitted indexed views. The source-link guard establishes selected delegation facts, not semantic equivalence by itself.

## Non-claims

<!-- r[impl choregraph.proof.single_step.readiness_completeness_boundary] -->
<!-- r[verify choregraph.proof.single_step.readiness_completeness_boundary] -->

The rail does not prove Nickel evaluation, authorization, effect execution, persistence, scheduling, capacity, fairness, liveness, or trace induction.

The rail does not prove Verus, Z3, rustc, vstd, the allocator, or the operating system.

The Trellis identity does not transfer a Trellis proof claim to Choregraph.

Role projection evidence proves only exact linkage to the selected Trellis result. It does not prove external-role behavior, transport, runtime execution, fault liveness, or release readiness.
