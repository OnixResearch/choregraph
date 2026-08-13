## Context

Choregraph has a restricted Nickel frontend, a pure verified machine core, a canonical Kamacite adapter, stable diagnostics, and release evidence. Its machine IR is a bounded execution graph. Endpoint-projected choreographies are a separate protocol artifact and must not be forced into that DAG before a runtime use case proves the mapping.

Trellis supplies the semantic source for finite global and local choreography. Wyzer supplies an MIT design reference for the authoring experience and negative examples.

## Success Contract

The accepted result is one bounded path from reviewed Nickel source to one admitted global choreography and one exact local artifact per declared role. Every projection result is tied to the selected Trellis semantics and exact source identities.

No runtime action occurs. A proven compiler blocker is acceptable if the selected Trellis API cannot be reused or locally revalidated without semantic duplication.

## Approach Registry

| Family | Mechanism | State |
| --- | --- | --- |
| Trellis-backed protocol adapter | Lower bounded Nickel into Trellis global syntax and use exact projection semantics | selected |
| Extend current DAG machine | Encode protocol branches as ordinary execution-graph nodes | deferred |
| Port Wyzer projection | Import the OCaml projection pass | rejected |
| Independent Rust projection | Reimplement projection from prose without equivalence proof | rejected |
| Generate native binaries | Compile one executable per role | deferred |

## Decisions

### Decision: keep protocol artifacts separate from machine IR

**Choice:** Add `choregraph-choreography` as a pure adapter crate. Its first output is a global protocol and local protocol set, not an `AdmittedMachine`.

**Rationale:** Current machine semantics own DAG lifecycle and effect intents. Dynamic role choices and message synchronization have different invariants.

A later Cairn can define a machine integration after Lattice provides an exact runtime need.

### Decision: use bounded typed Nickel source

**Choice:** Add a closed source profile with these semantic fields:

- schema version;
- declared role identities and kinds;
- declared value identities, payload tags, and initial owners;
- transfer steps with exact source, destination, label, value, and continuation;
- choices with exact decider and strictly ordered unique labels;
- termination;
- named limits; and
- required non-claims.

Display names and documentation remain metadata. Role, value, label, payload, contract, and artifact identities remain separate domains.

**Rationale:** Nickel gives reviewable contracts and blame. Rust revalidation remains the admission authority for evaluated values.

### Decision: support native and external role kinds as declarations

**Choice:** Every role has one closed kind.

A native role has no external-contract identity. An external role has exactly one immutable contract identity and declared input and output schema identities.

**Rationale:** The compiler must know which local artifact needs an external conformance boundary. The role kind does not select a host or grant authority.

Credentials, capabilities, adapter readiness, endpoint discovery, and current authorization stay outside the artifact.

### Decision: use exact Trellis semantics

**Choice:** The adapter lowers strict DTOs into the selected Trellis global choreography and ownership model. Projectability and endpoint projection use one pinned Trellis revision.

Direct dependency is preferred when the published API, toolchain, and build boundary fit. Otherwise Choregraph can import or adapt the smallest selected source set with exact provenance and Choregraph-local Verus revalidation.

A local projection algorithm that lacks equivalence evidence is forbidden.

**Rationale:** Proof by citation does not transfer. Semantic duplication without proof is worse.

### Decision: reject partial projection

**Choice:** Projection is all-or-nothing. Success contains one local artifact for every declared role in normalized role order.

Any undeclared role, malformed branch, ownership denial, merge conflict, missing endpoint, or projection failure returns ordered diagnostics and no canonical local set.

**Rationale:** Partial output can be deployed as if the protocol were complete.

### Decision: make canonical artifacts explicit

**Choice:** `choregraph-kamacite` adds versioned Preserves constructors for:

- role-choreography global artifact;
- role-choreography local artifact;
- ordered local artifact set; and
- projection receipt.

The receipt binds source envelope, import closure, evaluator output, Rust DTO, global artifact, every local artifact, Trellis source and proof set, lowerer, schemas, bounds, toolchain, diagnostics, and non-claims.

JSON remains a review projection. Canonical packed Preserves bytes own artifact identity.

**Rationale:** One-to-many compiler output needs exact complete-set identity.

### Decision: use structured safe diagnostics

**Choice:** The core returns diagnostics with stable code, severity, primary source-relative span, ordered secondary spans, bounded notes, bounded hints, and optional bounded safe replacement.

The diagnostic identity excludes rendering. The shell can render text or structured output.

Raw source excerpts, absolute host paths, credentials, environment values, imported secrets, and evaluator internals do not enter safe diagnostics.

**Rationale:** Wyzer demonstrates useful multi-span and hint structure. Choregraph retains its existing redaction boundary.

### Decision: keep external authority outside the compiler

**Choice:** A valid external-role contract is exact interchange data only.

Lattice retains adapter, transport, persistence, retry, observation, and product authority. Basalt and UCAN retain bounded authorization decisions. OnixOS retains role-to-target binding.

**Rationale:** Projectability does not prove external behavior or grant execution rights.

### Decision: record Wyzer as an MIT reference, not semantic authority

**Choice:** Pin the reviewed Wyzer revision and record direct author-confirmed MIT disposition in provenance evidence.

Source fragments can be adopted when they are smaller and clearer than replacement code. Every adopted fragment must pass Choregraph formatting, tests, Octet, architecture, provenance, and claim-boundary checks.

Wyzer projection, typechecker, runtime, Perceus, compile-time execution, and ABI are not imported by this change.

**Rationale:** The license is not a blocker. The existing implementation maturity and proof boundary still require selective use.

## Functional Core and Shell

The pure core owns DTO admission, normalization, lowering, projectability decisions, projection, canonical artifact construction, identity preimages, and diagnostics.

The shell owns source roots, file reads, Nickel evaluation, bounded process execution, output publication, terminal rendering, and evidence file writes.

## Validation Strategy

Positive fixtures cover a two-role transfer, a three-role choice, shared-prefix offer, uninvolved role, chained ownership transfer, native role, and external role.

Negative fixtures cover duplicate or undeclared roles, self-transfer, stale owner, duplicate labels, unsorted labels, non-projectable branches, missing local artifact, wrong contract kind, stale Trellis identity, stale schema, oversized source, unsafe diagnostic, and claim promotion.

Conformance compares exact Choregraph local artifacts with the selected Trellis projection for the same admitted global value. A separate negative fixture must detect any changed peer, label, payload tag, value, branch, continuation, or role ordering.

## Adversarial Audit

The audit must try to show that:

- projection drops a role, branch, step, or value;
- argument-like collections change arity after filtering;
- role wrappers disappear during type compatibility;
- an external role gains authority from its declaration;
- partial local output receives a passing receipt;
- diagnostics leak host or source secrets;
- a stale Trellis proof identity remains accepted; or
- a Wyzer runtime component enters the dependency graph.

Any successful counterexample blocks archive.

## Risks and Trade-offs

- A direct Trellis dependency can widen the build surface.
- A selected-source import adds provenance and update work.
- One-to-many canonical output increases stale-fixture checks.
- External-role contracts can become policy containers unless kept identity-only.
- Current Choregraph machine consumers cannot execute local protocols without a later Lattice change.

## Non-Claims

This design does not prove Nickel semantics, evaluator correctness, Trellis verifier soundness, transport delivery, runtime authorization, external-role correctness, persistence, retries, exactly-once effects, deadlock freedom under faults, whole-system correctness, production readiness, or release eligibility.
