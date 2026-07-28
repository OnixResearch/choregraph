## Context

The current machine schema stores exact effect declarations. Each declaration contains a node, transition, operation, resource, ability, request identity, and payload reference. An event must repeat those exact values. An accepted transition copies the declaration into an `EffectIntent`.

This model works for static references. It cannot represent a stable workflow machine whose request and payload become known only when one run resolves node inputs.

The current `lattice-workflow-v1` frontend branch omits effects. The Seaglass adapter therefore creates machines without effects and correctly rejects a non-empty request set. Lattice cannot authorize a Choregraph-emitted runtime intent until this upstream model changes.

## Success Contract

Completion requires one stable machine across runs, explicit declared effect slots, concrete request and payload references supplied as bounded event facts, production-linked proof, replay without dispatch output, preserved v1 behavior, canonical interchange, and an exact downstream handoff.

The following results are false completion:

- building a new machine from each runtime request;
- letting Lattice create the authoritative intent after Choregraph returns;
- treating a graph declaration as an authorization grant;
- hashing non-canonical JSON or text projections as the payload identity;
- adding a second transition algorithm for v2;
- returning historical intents from replay;
- reporting passing executable tests as proof; or
- publishing without explicit repository authority.

## Decisions

### Decision: Keep one immutable machine and derive concrete intent identities

**Choice:** One admitted machine contains static declarations. A run supplies concrete request and payload references through an event. The accepted output is a concrete intent, not a new machine.

**Rationale:** Workflow topology and declared effect classes remain stable. Run data changes only intent and attempt identities. Persisted machine state, migration, replay, and release evidence keep one meaning for machine identity.

A request-scoped second machine is rejected. It would mix runtime data with immutable workflow identity and would make replay and migration depend on transient machine artifacts.

### Decision: Add a versioned schema and preserve v1

**Choice:** Add `choregraph-machine-v2` and `lattice-workflow-v2`. Keep current v1 parsing, artifacts, tests, and public behavior.

Both versions normalize into one admitted internal representation. Existing v1 declarations normalize to exact bindings. V2 declarations can use exact bindings or named runtime slots. Public wrappers delegate to one production transition kernel.

**Rationale:** Existing publication identities and exact consumers must not change silently. A new schema makes the semantic change explicit while preserving one implementation.

### Decision: Separate declaration keys from content identities

**Choice:** Each v2 declaration has one bounded unique `DeclarationKey`. The key is stable authored data inside the immutable machine.

Canonical adapters derive these identities:

- machine artifact identity from canonical packed machine bytes;
- concrete Choregraph intent content identity from canonical packed intent bytes; and
- downstream bound-intent identity from the machine, workflow revision, run, intent, policy, and authority facts.

Choregraph owns the first two identities. Lattice owns the downstream bound-intent and attempt identities.

A Choregraph intent identity is a content identity, not an occurrence identity. Equal concrete intent data in two runs can have the same Choregraph identity. Lattice must add run and attempt facts to produce distinct occurrence identities and must not use the Choregraph content identity as a replay or deduplication key by itself.

**Rationale:** The pure core does not need filesystem access or ambient hashing inputs. A declaration key selects immutable machine data. BLAKE3 identities bind canonical artifacts at the interchange boundary.

### Decision: Use exact and slot reference bindings

**Choice:** A v2 declaration contains static operation, resource, and ability values plus one request binding and one payload binding.

Each binding is one of:

- `Exact(reference)`: the event must carry that exact canonical reference and no slot; or
- `Slot(slot_id, source_ref)`: the event names that exact slot and supplies one concrete canonical reference.

The bounded `source_ref` is an opaque declaration of the intended input source. The generic core preserves it but does not resolve product data. The Lattice v2 profile maps it to one reviewed node input, output, or configuration source and retains that mapping in the canonical machine artifact.

A v2 effect request contains the declaration key, selected request and payload slots when required, and the concrete references. One declaration key can occur at most once in an event. Multiple effects require distinct declaration keys. The transition core validates the declaration route, binding mode, selected slots, declared source references, reference shape, event bounds, and cardinality rules.

The accepted intent contains the declaration key, node, transition, static operation, static resource, static ability, selected slots, declared source references, and exact concrete references.

**Rationale:** Static authority requirements and intended value sources stay in reviewed graph data. Runtime values can vary without changing machine identity. Choregraph can prove that accepted output preserves both the declaration and explicit event inputs.

A slot permits any well-shaped concrete reference for its declared source. The core does not prove that the consumer resolved that source honestly. It does not prove the referenced bytes, request truth, or authorization.

### Decision: Make packed Preserves authoritative

**Choice:** Request envelopes and payloads use a named canonical packed Preserves profile. Their project-owned content references use BLAKE3.

Each identity hashes the complete canonical Preserves constructor, including its schema and profile tags. Raw digest display text, JSON, and text projections are not hash material. Distinct machine, declaration, request, payload, and intent constructors provide domain separation.

The core receives typed profile and digest values as explicit facts. It validates their bounded shape but does not read or hash payload bytes. `choregraph-kamacite` verifies bytes and derives identities when bytes are present. JSON and text remain projections.

**Rationale:** This keeps the core `no_std`, pure, and deterministic. It also prevents competing JSON encodings from becoming authority-bearing payload identities.

### Decision: Keep authorization and execution downstream

**Choice:** Choregraph emits declared concrete intent data only. It does not select Basalt policy, verify UCAN, resolve keys, read clocks, inspect revocation or replay stores, open capability roots, persist attempts, dispatch adapters, or report external success.

The Lattice handoff requires Lattice to add machine, workflow revision, run, policy, authority, and attempt identities before authorization and dispatch.

**Rationale:** Choregraph proves local transition and declaration properties. Lattice owns product facts and runtime authority.

### Decision: Replay discards concrete intents

**Choice:** Replay folds the same production transition kernel. It may validate historical concrete references while reconstructing state, but replay output contains no intent and no dispatch authority.

**Rationale:** Historical declarations, intents, and authorization receipts are evidence. They are not fresh permission.

### Decision: Extend production-linked proofs

**Choice:** The single-step proof must establish that every accepted v2 intent:

- matches one admitted declaration and route;
- preserves the declaration's operation, resource, and ability;
- preserves exact or selected-slot request and payload references from the event;
- is deterministically ordered and bounded; and
- accompanies an invariant-preserving state transition.

Denial must produce no replacement state and no intents. Trace proof must preserve the existing invariant and replay non-dispatch claims. Negative proof fixtures must fail if an intent changes a reference, bypasses a declaration, appears on denial, or escapes replay.

**Rationale:** The new claim must remain linked to production behavior rather than a parallel model.

### Decision: Treat Lattice adoption as a downstream gate

**Choice:** Choregraph will provide a conversion-only Seaglass API, fixtures, source identities, and a disposable downstream check. Lattice will complete authorization, persistence, Basalt, UCAN, capability, and adapter work in its own Cairn change after it admits an exact Choregraph v2 source.

**Rationale:** An upstream pass does not prove downstream authorization or release readiness.

## Functional Core and Shell Boundary

Pure functions own:

- v1 and v2 normalization;
- declaration and reference admission;
- binding resolution;
- duplicate detection;
- deterministic intent construction and ordering;
- state transition decisions;
- replay folding without dispatch output; and
- canonical in-memory projection data.

Thin shells own:

- Nickel process execution;
- source and import reads;
- packed Preserves byte I/O;
- BLAKE3 calculation over observed bytes;
- receipt and bundle publication;
- downstream checkout acquisition; and
- command diagnostics.

Every non-trivial shell operation must delegate its deterministic decisions to a pure core.

## Data and Identity Flow

```text
reviewed Nickel lattice-workflow-v2
  -> restricted evaluation
  -> pure Rust lowering
  -> admitted immutable machine
  -> canonical packed machine artifact + BLAKE3 machine identity

resolved runtime request and payload bytes
  -> canonical packed Preserves envelopes
  -> BLAKE3 concrete references
  -> bounded Choregraph event
  -> production step
  -> concrete Choregraph intent
  -> canonical packed intent artifact + BLAKE3 intent identity
  -> Lattice-owned run, policy, authority, persistence, and dispatch binding
```

The byte-to-reference arrows are shell or Kamacite adapter observations. The core proves only relations over the references it receives.

## Compatibility and Migration

- Existing v1 fixtures and canonical artifact bytes must remain unchanged.
- Existing v1 public constructors and transition calls must delegate to the normalized production kernel.
- V2 uses distinct schema tags and Preserves constructors.
- `lattice-workflow-v1` continues to lower without effects.
- `lattice-workflow-v2` requires explicit effect declarations for effectful nodes.
- Unknown binding modes, profiles, slots, source references, declaration keys, and schema versions fail closed.
- Safe diagnostics use stable bounded codes and do not include raw request or payload bytes.
- No implicit v1-to-v2 source rewrite is allowed.

## Validation

Positive checks cover exact bindings, request slots, payload slots, mixed bindings, multiple declarations, deterministic ordering, v1 parity, canonical round trips, proof success, replay, and conversion-only downstream use.

Negative checks cover malformed references, duplicate keys, repeated declaration use, unknown declarations, wrong routes, wrong exact references, wrong slots, wrong source references, duplicate requests, oversized inputs, unsafe diagnostics, stale identities, non-canonical bytes, content-versus-occurrence confusion, intent mutation, denial with intent, replay with dispatch output, adapter authority imports, and v1 identity drift.

The maintained rails include workspace tests, no-std checks, Clippy, foundation checks, Verus positive and expected-failure fixtures, Nickel contracts, Kamacite corpus checks, Seaglass architecture checks, Octet deny-all, Cairn validation, Tracey coverage, downstream checks, and Nix release checks.

## Risks / Trade-offs

- **Slot breadth:** A slot accepts any valid concrete reference. Basalt and Lattice must decide whether that exact value is authorized.
- **Digest truth:** The core cannot prove that a supplied digest identifies the claimed bytes. The canonical adapter records that observation.
- **Schema cost:** V2 adds compatibility work. It avoids silently changing existing published semantics.
- **Proof growth:** Binding modes expand the proof state. Named bounded enums and one normalized kernel limit that growth.
- **Crash behavior:** Choregraph does not solve persistence or exactly-once execution. Lattice must retain unknown outcomes after ambiguous external effects.
- **Source readiness:** This change does not fix Basalt's authenticated UCAN source. That remains a downstream release blocker.

## Claim Boundary

Passing this change proves bounded local relations between admitted declarations, explicit event references, accepted concrete intents, state transitions, and replay output for the selected production code and toolchain.

It does not prove request truth, payload-byte availability, BLAKE3 implementation correctness, arbitrary Nickel equivalence, UCAN cryptography, key truth, policy correctness, authorization freshness, persistence durability, operating-system confinement, external effect success, exactly-once behavior, Lattice correctness, or release eligibility outside the checked candidate.
