## Phase 1: Dependencies, provenance, and baseline

- [ ] [depends:trellis-prove-choreography-value-transfer] D1 Pin the accepted Trellis value-transfer revision, APIs, theorem set, toolchain, license, and source identities before implementation. r[choregraph.role_choreography.trellis]
- [ ] [serial] P1 Record the exact Wyzer revision, direct author-confirmed MIT disposition, selected authoring and diagnostic concepts, adopted source fragments, and rejected components. r[choregraph.role_choreography.reference]
- [ ] [serial] B1 Record current Nickel, Kamacite, core, proof, release, foundation, Octet, Cairn, and Nix baselines. r[choregraph.role_choreography.validation]
- [ ] [serial] B2 Prove a minimal direct-dependency or selected-source Trellis integration can compile under the Choregraph no-std, Verus, license, and provenance boundaries. r[choregraph.role_choreography.trellis]

## Phase 2: Source profile and pure admission

- [ ] [serial] S1 Define typed Nickel contracts for schema version, roles, role kinds, values, owners, transfers, choices, termination, limits, and non-claims. r[choregraph.role_choreography.source]
- [ ] [serial] S2 Add named limits for roles, values, steps, branches, nesting, identifiers, imports, diagnostics, and canonical output size. r[choregraph.role_choreography.bounds]
- [ ] [depends:role-choreography-contracts] S3 Implement strict pure Rust DTO admission with unknown-field rejection, distinct identity domains, deterministic ordering, and no ambient effects. r[choregraph.role_choreography.admission]
- [ ] [depends:role-choreography-admission] S4 Add native and external role admission with exactly one immutable contract identity for each external role and no authority-bearing fields. r[choregraph.role_choreography.roles]

## Phase 3: Trellis lowering and endpoint projection

- [ ] [depends:trellis-integration-spike] T1 Add `choregraph-choreography` as a pure adapter crate with one path from admitted DTOs into the selected Trellis global model. r[choregraph.role_choreography.lowering] r[choregraph.role_choreography.trellis]
- [ ] [depends:choreography-global-lowering] T2 Use the selected Trellis projectability and endpoint projection semantics without an unproved second projection implementation. r[choregraph.role_choreography.projection]
- [ ] [depends:choreography-projection] T3 Reject partial projection and require exactly one local artifact for every declared role in normalized role order. r[choregraph.role_choreography.complete_set]
- [ ] [depends:choreography-projection] T4 Add local conformance and proof evidence for exact peers, labels, payload tags, values, branches, continuations, and ownership transitions. r[choregraph.role_choreography.trellis] r[choregraph.role_choreography.projection]

## Phase 4: Canonical artifacts and diagnostics

- [ ] [serial] K1 Add versioned canonical Preserves schemas for global, local, ordered-local-set, and projection-receipt artifacts. r[choregraph.role_choreography.artifacts]
- [ ] [depends:choreography-complete-set] K2 Extend the pure Kamacite adapter with source-to-global-to-local identity linkage and domain-separated BLAKE3 identities. r[choregraph.role_choreography.artifacts] r[choregraph.role_choreography.identity]
- [ ] [depends:choreography-artifacts] K3 Add Valence-friendly projection roles without adding Valence runtime authority or dependency. r[choregraph.role_choreography.identity] r[choregraph.role_choreography.boundary]
- [ ] [serial] G1 Add stable structured diagnostic DTOs with bounded primary and secondary spans, notes, hints, safe fixes, and deterministic ordering. r[choregraph.role_choreography.diagnostics]
- [ ] [depends:structured-choreography-diagnostics] G2 Add shell renderers and redaction checks that exclude host paths, credentials, environment values, imported secrets, and unbounded source text. r[choregraph.role_choreography.diagnostics]

## Phase 5: Positive and negative evidence

- [ ] [parallel] V1 Add positive two-role transfer, three-role choice, shared-prefix offer, uninvolved-role, chained-transfer, native-role, and external-role fixtures. r[choregraph.role_choreography.fixtures]
- [ ] [parallel] V2 Add negative duplicate-role, undeclared-role, self-transfer, stale-owner, duplicate-label, unsorted-label, non-projectable, incomplete-set, and wrong-role-kind fixtures. r[choregraph.role_choreography.fixtures]
- [ ] [parallel] V3 Add stale Trellis, stale schema, stale lowerer, stale proof-set, wrong local artifact, wrong local order, and mismatched complete-set identity fixtures. r[choregraph.role_choreography.identity] r[choregraph.role_choreography.fixtures]
- [ ] [parallel] V4 Add oversized input, diagnostic leakage, external-authority, partial-output, proof-transfer, deadlock, delivery, and release-overclaim failures. r[choregraph.role_choreography.bounds] r[choregraph.role_choreography.boundary] r[choregraph.role_choreography.fixtures]

## Phase 6: Documentation and closeout

- [ ] [serial] E1 Update architecture, restricted Nickel, Kamacite, proof-reference, release-evidence, and runtime-authority documentation. r[choregraph.role_choreography.boundary]
- [ ] [serial] E2 Add Wyzer to README references and record that MIT permission removes the license blocker without promoting prototype claims. r[choregraph.role_choreography.reference]
- [ ] [serial] X1 Run workspace tests, Clippy, no-std targets, conformance, foundation checks, Verus, Octet, Cairn validation and gates, Tracey, and `nix flake check -L`. r[choregraph.role_choreography.validation]
- [ ] [serial] X2 Run the adversarial audit and block archive for semantic duplication, partial projection, stale identity, diagnostic leakage, authority transfer, or overclaim. r[choregraph.role_choreography.validation] r[choregraph.role_choreography.boundary]
- [ ] [serial] X3 Review sync and archive plans and archive only with fresh exact projection and identity evidence. r[choregraph.role_choreography.validation]
