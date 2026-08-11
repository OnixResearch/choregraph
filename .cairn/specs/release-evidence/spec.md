# Release Evidence Specification

## Purpose

Defines the `release-evidence` capability.

## Requirements

### Requirement: Typed release bundle

r[choregraph.release.bundle] Choregraph MUST define stable release roles, required artifacts, supported schemas, BLAKE3 identity rules, and explicit non-claims.

#### Scenario: Required role is absent
- GIVEN a release bundle without one required role
- WHEN bundle validation runs
- THEN release readiness MUST fail.

### Requirement: Deterministic manifest

r[choregraph.release.manifest] The release manifest MUST bind source, license, schemas, production code, proofs, assumptions, toolchains, Kamacite carriers, fixtures, conformance, downstream, and lifecycle evidence.

#### Scenario: Equal inputs are bundled twice
- GIVEN equal canonical inputs and policy
- WHEN manifest generation runs twice
- THEN project-owned manifest identities MUST be equal.

### Requirement: Independent linkage verification

r[choregraph.release.verification] Bundle verification MUST rehash declared artifacts and validate roles, schemas, receipts, proof identities, source identities, policy identities, and toolchain identities.

#### Scenario: Proof source changes
- GIVEN a bundle with an earlier proof-source identity
- WHEN current linkage is checked
- THEN verification MUST fail as stale.

### Requirement: Positive and negative bundle fixtures

r[choregraph.release.fixtures] Release checks MUST include complete bundles and missing, stale, skipped, malformed, mismatched, unsupported, undeclared, and overclaim bundles.

#### Scenario: Passing proof is marked skipped
- GIVEN a bundle that omits current proof execution
- WHEN release validation runs
- THEN the bundle MUST fail even if unit tests pass.

### Requirement: Valence role projection

r[choregraph.release.valence] Valence-friendly projections MUST preserve canonicalization, proof, lifecycle, source-observation, conformance, and runtime roles without broadening claims.

#### Scenario: Linkage projection passes
- GIVEN valid linked artifacts
- WHEN Valence projection is checked
- THEN no new behavioral or release claim may be inferred from linkage alone.

### Requirement: Immutable publication candidate

r[choregraph.release.publication_candidate] A publication candidate MUST bind exact source revision, source archive BLAKE3, license, package set, acquisition adapter, release bundle, and rollback baseline.

#### Scenario: Candidate uses a floating source
- GIVEN a publication candidate without an exact source revision
- WHEN candidate validation runs
- THEN the candidate MUST be rejected.

### Requirement: Downstream handoff

r[choregraph.release.downstream] Execution-graph and Lattice cutovers MUST cite exact accepted Choregraph source and release-bundle identities and MUST run their own integration checks.

#### Scenario: Downstream cites only a project name
- GIVEN a consumer change without exact source and bundle identities
- WHEN handoff validation runs
- THEN the dependency MUST remain unsatisfied.

### Requirement: Comprehensive release gates

r[choregraph.release.gates] Release readiness MUST require current formatting, tests, properties, expected failures, Clippy, no-std, Verus, Nickel, canonical corpus, adapters, parity, downstream, Octet, Cairn, traceability, and Nix evidence.

#### Scenario: Maintained gate fails
- GIVEN one required gate with a non-passing result
- WHEN release readiness runs
- THEN readiness MUST fail closed.

### Requirement: Publication authority remains explicit

r[choregraph.release.authority] Choregraph release preparation MUST NOT publish, push, sign, or change remote authority without a separate explicit operator action.

#### Scenario: Candidate generation runs
- GIVEN local source and evidence
- WHEN a publication candidate is generated
- THEN no remote repository or authority state MUST change.

### Requirement: Release claim boundary

r[choregraph.release.claim_boundary] Passing release evidence MUST state that it does not prove verifier soundness, compiler correctness, Nickel equivalence, event truth, authorization freshness, runtime effects, remote availability, or whole-product correctness.

#### Scenario: Release bundle passes
- GIVEN a valid complete bundle
- WHEN its result is reported
- THEN reporting MUST remain within the selected evidence and theorem claims.

### Requirement: Authorized publication receipt

r[choregraph.release.publication] Choregraph MUST bind an operator-authorized publication to the unchanged candidate, exact Radicle identity, exact source revision, and parent-feature signed refs.

#### Scenario: Publication identity matches the candidate

- GIVEN the accepted local candidate and an assigned RID
- WHEN publication evidence is validated
- THEN the source revision, archive, bundle, license, and candidate identity MUST match exactly.

#### Scenario: Publication authority or signed refs are weak

- GIVEN missing operator authority or signed refs below the parent feature
- WHEN publication evidence is validated
- THEN validation MUST fail.

### Requirement: Independent acquisition evidence

r[choregraph.release.acquisition] Publication evidence MUST include exact native acquisition from both governed seeds, one independent replica, and the read-only HTTPS adapter.

#### Scenario: Every selected acquisition path agrees

- GIVEN the published RID and exact revision
- WHEN fresh clients acquire the source
- THEN every admitted path MUST resolve the exact revision and source archive identity.

#### Scenario: A write or alternate source is attempted

- GIVEN a receive-pack request, unknown RID, floating revision, or alternate source
- WHEN acquisition validation runs
- THEN the request MUST fail closed.

### Requirement: Publication validation gate

r[choregraph.release.publication_validation] Maintained gates MUST validate the typed publication receipt and reject stale, mismatched, incomplete, writable, or overclaimed fixtures.

#### Scenario: Current receipt passes

- GIVEN current publication observations and required non-claims
- WHEN release checks run
- THEN the receipt MUST pass deterministically.

#### Scenario: Receipt linkage is stale

- GIVEN a changed candidate, source, RID, signed refs, replica set, or endpoint policy
- WHEN release checks run
- THEN validation MUST fail.

### Requirement: Compatibility publication receipt

r[choregraph.release.compatibility_publication] Choregraph MUST bind the unchanged compiler-compatibility candidate to the existing Radicle RID, exact source revision, release bundle, operator authority, and current parent-feature signed refs without changing the earlier publication receipt.

#### Scenario: Compatibility candidate is published
- GIVEN explicit operator authorization and the validated immutable candidate
- WHEN main and candidate branches are published
- THEN both branches and the new receipt MUST identify the exact candidate source revision.

### Requirement: Compatibility acquisition verification

r[choregraph.release.compatibility_acquisition] Choregraph MUST verify the compatibility source through three observed native replicas, isolated Aspen and desktop clients, and the read-only HTTPS adapter.

#### Scenario: One acquisition path is stale
- GIVEN a native or HTTPS acquisition path that resolves another revision or archive identity
- WHEN publication evidence is validated
- THEN compatibility publication verification MUST fail closed.
