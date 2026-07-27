# Kamacite Machine Interchange

## ADDED Requirements

### Requirement: Reviewed Kamacite source

r[choregraph.interchange.kamacite.source] Choregraph MUST pin the Kamacite source or artifact revision, supported profiles, license, and canonicalization rules used for maintained interchange.

#### Scenario: Kamacite profile is unresolved
- GIVEN a machine artifact without an exact supported Kamacite profile identity
- WHEN interchange validation runs
- THEN the artifact MUST be rejected.

### Requirement: Choregraph aggregate descriptor

r[choregraph.interchange.kamacite.aggregate] Choregraph MUST own a versioned canonical Preserves descriptor for bounded aggregate graph and machine data.

#### Scenario: Aggregate descriptor is encoded
- GIVEN an admitted Choregraph machine
- WHEN the descriptor is encoded
- THEN canonical packed Preserves bytes MUST be the authoritative identity material.

### Requirement: Local lifecycle profile

r[choregraph.interchange.kamacite.lifecycle] The selected per-node lifecycle MUST use a supported closed Kamacite FSM profile, while aggregate state MUST remain parameterized.

#### Scenario: Aggregate product table is requested
- GIVEN a graph with multiple nodes
- WHEN artifact generation runs
- THEN generation MUST NOT require enumeration of every global local-state combination.

### Requirement: Pure artifact adapter

r[choregraph.interchange.kamacite.adapter] `choregraph-kamacite` MUST convert and validate in-memory values without filesystem, process, network, clock, or diagnostic-rendering authority.

#### Scenario: Adapter requires file access
- GIVEN an adapter path that reads a source file directly
- WHEN architecture checks run
- THEN the adapter boundary MUST fail.

### Requirement: Canonical identity chain

r[choregraph.interchange.kamacite.identity] Project-owned BLAKE3 identities MUST bind canonical machine, lifecycle, admitted IR, proof set, generator, adapter, profile, and toolchain artifacts.

#### Scenario: Bound artifact changes
- GIVEN one changed semantic artifact
- WHEN identity derivation runs
- THEN the affected binding identity MUST change.

### Requirement: Receipt role separation

r[choregraph.interchange.kamacite.receipts] Choregraph MUST validate exact admission, lifecycle, exhaustiveness, trace, source, and proof-reference receipt roles without treating a receipt as behavioral proof.

#### Scenario: Proof reference is present
- GIVEN a structurally valid proof-reference receipt
- WHEN receipt linkage passes
- THEN the result MUST NOT claim that the referenced proof is sound or current without its proof gate.

### Requirement: Preserves authority

r[choregraph.interchange.kamacite.preserves] Canonical packed Preserves MUST be authoritative for interchange identity, and JSON MUST remain a diagnostic or compatibility projection.

#### Scenario: JSON bytes replace Preserves bytes
- GIVEN a JSON projection without its matching canonical Preserves artifact
- WHEN authority is checked
- THEN the projection MUST NOT satisfy canonical machine identity.

### Requirement: Interchange fixture coverage

r[choregraph.interchange.kamacite.fixtures] Tests MUST include canonical success cases and malformed, unsupported, non-canonical, stale, mismatched, substituted, enumerated, and overclaim failure cases.

#### Scenario: Fixture corpus runs
- GIVEN the reviewed interchange corpus
- WHEN validation runs
- THEN supported artifacts MUST pass and invalid artifacts MUST fail deterministically.

### Requirement: Interchange validation rail

r[choregraph.interchange.kamacite.validation] Maintained checks MUST run adapter tests, canonical corpus checks, and current profile linkage checks.

#### Scenario: Canonical fixture becomes stale
- GIVEN a changed schema or adapter identity
- WHEN maintained checks run
- THEN stale canonical fixtures MUST fail closed.
