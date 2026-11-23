# Role choreography

Choregraph accepts a bounded role protocol after restricted Nickel evaluation. The pure `choregraph-choreography` crate revalidates the evaluated value.

The crate then calls the pinned Trellis production functions. It does not contain a second endpoint-projection algorithm.

## Data path

The data path has these stages:

1. Nickel checks the closed source shape.
2. Rust rejects unknown fields and values outside fixed limits.
3. Rust admits declared roles, external contracts, values, and initial owners.
4. Trellis admits each ownership state and each transfer.
5. Choregraph lowers the admitted tree to `verified_logic::GlobalChoreo`.
6. Trellis checks projectability and runs `project_endpoint` for each declared role.
7. Choregraph converts each Trellis local value into a canonical artifact body.
8. Kamacite encodes the global value, each local value, the complete local set, and the receipt as packed Preserves.
9. Domain-separated BLAKE3 identities bind all stages.

Projection is atomic. If one stage fails, the compiler emits no canonical local set.

## Rust API ownership

The crate root exposes `compile_evaluated` and `CLAIM_BOUNDARY`, not DTO or
linkage re-exports. Source DTOs live in `model::schema::{RoleSource,
ROLE_CHOREOGRAPHY_SCHEMA, REQUIRED_NON_CLAIMS}`; role IDs, declarations, kinds,
imports, and spans in `model::participants`; protocol `Node` and `ChoiceBranch` in
`model::tree`; fixed `Limits` and `MAX_*` values in `model::bounds`.
Admission uses `admission::{admit_source, parse_evaluated_json, Admitted,
Error}`, endpoint projection uses `endpoint::{project_all,
validate_complete_set, Projected, GlobalProtocol, LocalProtocol, ...}`,
diagnostics use `issue`, and canonical bundles use `linkage`.
`linkage::digest` owns identity domains, framing, and validation;
`linkage::output` owns canonical artifact records; `linkage::evidence` owns
projection evidence and receipt records; `linkage::provenance` owns pinned
producer metadata. These are Rust path changes only: Serde containers keep
their original record labels (for example, `RoleSource` still serializes as
`RoleChoreographySource` and `Node` as `ChoreographyNode`), and serialized
field names, packed source/global/local encoding and identity domain labels
are unchanged.
The lowerer identity intentionally hashes `admission.rs` and `endpoint.rs`, so
editing that implementation changes its digest and the derived receipt bytes;
no historical receipt is silently treated as a new build's receipt.

`admission::Error::Diagnostics` retains the ordered, span-bearing
diagnostics for existing malformed inputs. `UnsupportedNode` instead reports
an unrecognized Rust protocol-node shape without inventing a source span; it
is a typed, unlocated Rust failure and is not serialized as a diagnostic.
Nested unknown nodes propagate through validation and lowering, before any
canonical bundle is emitted. New role kinds fail with `InvalidRoleKind` at
the declaration's real span.

## Fixed limits

The source must use the reviewed limits. A caller cannot widen them.

| Limit | Value |
| --- | ---: |
| roles | 64 |
| values | 256 |
| steps | 4,096 |
| branches | 256 |
| nesting depth | 128 |
| identifier bytes | 256 |
| imports | 128 |
| diagnostics | 128 |
| diagnostic text bytes | 512 |
| canonical output bytes | 1,048,576 |

## Role kinds

A native role has no external contract fields.

An external role has one immutable contract identity. It also has one input-schema identity and one output-schema identity. Each identity uses `blake3:<64 lowercase hexadecimal characters>`.

These fields are interchange data. They do not grant host, adapter, credential, transport, capability, or effect authority.

## Trellis dependency

The adapter pins Trellis revision `d65974b14978e1a44b0f5067beb9ef6d83317dc8`.

The selected source-set manifest has BLAKE3 identity `28bdef948c196d9ff84268a2028e90d7148e7950bcc4046489f5483b0421c99a`.

Trellis uses the unstable `register_tool` feature to retain verifier attributes in normal Cargo builds. Choregraph stays on Rust 1.97.1. `.cargo/config.toml` sets `RUSTC_BOOTSTRAP=verified_logic`, which limits the exception to the pinned dependency crate.

This exception does not apply to Choregraph crates. The foundation, Clippy, Octet, and Nix rails check the complete workspace.

## Diagnostics

A diagnostic contains a stable code, one primary source-relative span, ordered secondary spans, bounded notes, bounded hints, and an optional bounded replacement.

Safe output rejects absolute home paths and common secret-bearing fragments. It does not persist raw evaluator output, imported source, environment values, or credentials.

## Canonical artifacts

Packed Preserves bytes own artifact identity. JSON and text are review projections only.

The projection receipt binds these identities:

- source envelope;
- import closure;
- evaluator output;
- strict Rust DTO;
- global protocol;
- every local protocol;
- complete local set;
- Trellis revision, source set, and theorem set;
- lowerer;
- schema set;
- limits;
- diagnostics;
- toolchains; and
- required non-claims.

Valence-friendly evidence roles describe linkage only. They add no Valence runtime dependency or authority.

## Preference annotation profile (default off)

The cooperative baseline is the default. `compile_evaluated` and
`admission::admit_source` admit only unannotated
`choregraph.role-choreography.v1` sources. Their behavior, serialized shape and
identity bytes are unchanged. If the baseline receives a source whose `schema`
is the annotated schema, it refuses it with `preference-profile-not-selected`.

A separately versioned profile attaches declared preference factors to
`Choice` deciders. This follows Pact's `values`, which are factors only and
not a utility function. Each annotated choice can also name at most one
exogenous world variable. Callers must select this profile explicitly.

| Item | Exact value |
| --- | --- |
| Source `schema` | `choregraph.role-choreography.preference-annotations.v1` |
| Profile selector (selected) | `choregraph.role-choreography.profile.preference-annotations.v1` |
| Profile selector (default) | `choregraph.role-choreography.profile.cooperative-baseline.v1` |
| Report schema | `choregraph.role-choreography.preference-report.v1` |
| Nickel contract | `contracts/nickel/role-choreography-preference-contracts.ncl` (`AnnotatedRoleChoreography`) |
| Preserves source | `choregraph.role-choreography-preference-source-preserves.v1`, root `AnnotatedRoleChoreographySource` |
| Preserves annotations | `choregraph.role-choreography-preference-annotations-preserves.v1`, root `PreferenceAnnotationSet` |
| Preserves annotated local set | `choregraph.role-choreography-preference-local-set-preserves.v1`, root `AnnotatedLocalSetBody` |
| Preserves receipt | `choregraph.role-choreography-preference-receipt-preserves.v1`, root `PreferenceProjectionReceipt` |

### Source shape

```text
{ schema, base: <unchanged v1 RoleChoreographySource>,
  annotations: { factors: [{ id, role, name, span }],
                 worlds: [{ id, name, span }],
                 choices: [{ path: [{kind = "next"} | {kind = "branch", label}],
                             decider, factors: [factor id], world: id | null, span }] },
  non_claims: [the nine annotation non-claims] }
```

A `path` runs from the protocol root to the annotated `Choice`. A
communication or transfer node advances with `next`; a choice node advances
through the branch with the given `label`. Admission first admits `base` with
the unchanged v1 function. It then rejects any of the following:

- unknown fields or a wrong schema;
- a missing, extra, or changed annotation non-claim;
- a factor, world variable, or choice annotation count above its fixed limit
  (256, 64, and 256), a choice with more than 64 factors, or a path longer than
  128 steps;
- an empty annotation set;
- a name that is not lowercase kebab text of at most 64 bytes;
- a duplicate or unsorted factor, world variable, or choice annotation;
- a factor that names an undeclared role;
- a path that does not match the tree or does not end at a `Choice`;
- an annotation whose decider differs from the `Choice` decider;
- a choice with no factors, or with unsorted or unknown factors;
- a factor owned by a role other than the decider;
- an unknown world variable; and
- a declared factor or world variable that no choice uses.

### Projection

No second projection algorithm exists. The annotated path calls
`endpoint::project_all` and `linkage::build_projection_bundle` on the admitted
base, so the global artifact, every local artifact and the complete local set
match the unannotated projection byte for byte. Projection then follows each
annotation path through the decider's Trellis local protocol:

- a message involving the decider advances its local cursor by `next`;
- an uninvolved message leaves the cursor unchanged;
- an outer choice owned by the decider advances through `InternalChoice`;
- another role's choice advances only through an `Offer` from that role with
  identical labels.

The target must be the decider's `InternalChoice`, with exactly the global
labels. A decider whose local protocol merges an outer foreign choice has no
single local address for that choice. The compiler refuses it with
`preference-path-not-locally-addressable`. It reports any other disagreement
with the Trellis output as `preference-local-mismatch`.

The annotated complete local set has one `AnnotatedLocal` per declared role, in
normalized role order. Each entry names the exact v1 local artifact identity
and lists its decider annotations (`global_path`, `local_path`, `labels`, full
factor declarations and optional world declaration). Non-deciders have empty
lists. `preference::projection::validate_set` rejects a missing, duplicated,
reordered or stale entry, and a factor owned by another role.

### Receipt and report

`PreferenceProjectionReceipt` binds the following identities:

- profile and schema;
- the v1 projection receipt identity;
- the global artifact;
- the ordered local artifacts;
- the complete local set;
- the annotated strict DTO;
- the annotation set;
- the annotated local set;
- the annotation schema set;
- the annotation lowerer, which hashes the v1 lowerer identity (itself a hash of the v1 admission and endpoint sources) and the annotation model, admission and projection sources; and
- the annotation non-claims.

It records `game_mapping = not-formalized` and claim
`identity-linkage-only-no-claim-promotion`. `preference::linkage::verify_receipt`
rejects any field or byte difference. New BLAKE3 identity domains are
`preference-strict-dto`, `preference-annotations`, `preference-local-set`,
`preference-schema-set`, `preference-lowerer`, `preference-non-claims` and
`preference-receipt`.

`preference::compile_evaluated(evaluated, envelope, selection)` returns a
`Report`. Its fields are separate:

- `status()` is `projected` or `refused`;
- `refusal()` returns span-bearing diagnostics; and
- `game_mapping()` is always `not-formalized`.

`Report::summary()` is the serializable `PreferenceReport` review projection.

### Rust API

These items are Rust paths under `choregraph_choreography::preference`:

- `ProfileSelection`, which defaults to `CooperativeBaseline`. Use
  `selector()` and `from_selector()` to convert to and from the exact selector
  string. The module also exports `ANNOTATED_PROFILE`, `BASELINE_PROFILE`,
  `REPORT_SCHEMA`, `ProjectionStatus`, `Report`, `Summary` and
  `compile_evaluated`.
- `model`: `AnnotatedSource`, `AnnotationSet`, `FactorDeclaration`,
  `WorldDeclaration`, `ChoiceAnnotation`, `PathStep`, `FactorId`, `WorldId`,
  `ANNOTATED_SCHEMA` and `REQUIRED_NON_CLAIMS`.
- `admission`: `parse_evaluated_json`, `admit_source(source, selection)`,
  `Admitted` (whose `base()` returns the admitted v1 cohort),
  `refusal_without_selection`, `profile_not_selected` and the `MAX_*` limits.
- `projection`: `AnnotatedLocalSet`, `AnnotatedLocal`, `DeciderAnnotation`,
  `annotate` and `validate_set`.
- `linkage`: `build_bundle`, `AnnotatedBundle`, `ReceiptBody`, `GameMapping`,
  `verify_receipt` and `verify_local_set`.

`AnnotatedBundle::base()` returns the unchanged v1 `ProjectionBundle` and
`projected()` returns its Trellis `Projected` set, so an existing v1 consumer
can admit the same cohort unchanged.

The annotation encoders call `choregraph_kamacite::codec::encode` directly.
The Kamacite adapter sources and their corpus identity are unchanged.

### Non-claims of the profile

Annotation non-claims: `no-branch-selection`, `no-equilibrium-analysis`,
`no-game-semantics`, `no-label-boolean-conversion`, `no-preference-authority`,
`no-self-interested-projection-soundness`, `no-solver`, `no-utility-function`,
`no-world-distribution`.

Factors are metadata. They do not select a label, weigh branches or grant
authority, and a label set never becomes a Boolean. This profile implements no
Pact solver, equilibrium, mechanism-design or incentive-compatibility check.
The Pact paper's solver and game semantics are unpublished author claims and
are not evidence here. The Trellis projection theorems cover the cooperative
protocol only. They do not establish that endpoint projection is sound for
self-interested participants. All v1 non-claims remain in force.

## Non-claims

A passing result does not prove or perform runtime execution, host placement, transport delivery, persistence, authorization, external-role correctness, exactly-once transfer, deadlock freedom under faults, production readiness, or release eligibility.

The Wyzer reference supplies authoring and diagnostic design ideas. Choregraph does not import Wyzer projection, networking, runtime, Perceus, compile-time execution, or C ABI code.
