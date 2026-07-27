# Project Foundation Specification

## Purpose

Defines the `project-foundation` capability.

## Requirements

### Requirement: Standalone workspace identity

r[choregraph.foundation.workspace] Choregraph MUST be a standalone Rust, Nickel, Nix, and Cairn repository with separate pure-core and imperative-shell crates.

#### Scenario: Workspace metadata is inspected
- GIVEN the reviewed workspace manifests
- WHEN the foundation checks run
- THEN the core and adapter crates MUST have explicit one-way dependency edges.

### Requirement: Functional core boundary

r[choregraph.foundation.boundaries] The core MUST NOT read files, inspect environment state, use clocks, spawn processes, access networks, evaluate Nickel, verify tokens, persist runs, issue receipts, or execute effects.

#### Scenario: Forbidden dependency is introduced
- GIVEN a core manifest or source file that imports a forbidden shell capability
- WHEN the architecture checks run
- THEN the repository MUST reject the change before release.

### Requirement: Single production implementation

r[choregraph.foundation.single_implementation] Choregraph MUST become the sole production implementation of its transition relation, and compatibility crates MUST delegate without retaining independent transition logic.

#### Scenario: Duplicate transition logic remains
- GIVEN Choregraph and a compatibility crate each contain independent production transition code
- WHEN migration readiness is checked
- THEN readiness MUST fail until one implementation is removed.

### Requirement: License preservation

r[choregraph.foundation.license] Choregraph MUST preserve `AGPL-3.0-or-later` for adapted execution-graph and Seaglass implementation material unless the copyright holder authorizes another license.

#### Scenario: Manifest declares a different license
- GIVEN adapted AGPL implementation material
- WHEN package metadata declares an incompatible license
- THEN the license check MUST fail.

### Requirement: Source provenance

r[choregraph.foundation.provenance] Choregraph MUST record exact source revisions and BLAKE3 identities for imported implementation and proof references.

#### Scenario: Source identity is missing
- GIVEN imported execution-graph or Trellis material
- WHEN provenance validation runs
- THEN missing or ambiguous source identity MUST fail closed.

### Requirement: Reproducible toolchain boundary

r[choregraph.foundation.toolchain] The workspace MUST pin the Rust, Verus, Nickel, and Nix inputs used by maintained verification rails.

#### Scenario: Floating tool input is selected
- GIVEN a maintained verification rail
- WHEN its tool input is floating or unresolved
- THEN the rail MUST reject the input as non-reproducible.

### Requirement: Foundation validation

r[choregraph.foundation.validation] Foundation checks MUST include positive accepted-boundary fixtures and negative forbidden-boundary fixtures.

#### Scenario: Foundation checks pass
- GIVEN all foundation checks pass
- WHEN results are reported
- THEN the result MUST NOT claim graph, compiler, proof, runtime, or release correctness.
