# Project Foundation Delta

## ADDED Requirements

### Requirement: Maintained compiler floor

r[choregraph.foundation.compiler_floor] Choregraph packages MUST declare the oldest compiler exercised by the maintained Octet gate, and the gate MUST NOT bypass Cargo's Rust-version check.

#### Scenario: Octet checks the declared compiler floor
- GIVEN the accepted Choregraph source and pinned Octet compiler
- WHEN the maintained Octet gate compiles the workspace
- THEN Cargo MUST accept the declared Rust version without an ignore-version argument.

### Requirement: Consumer-compatible public dependencies

r[choregraph.foundation.consumer_compatibility] Public Choregraph crates MUST permit reviewed semver-compatible BLAKE3 and Serde JSON patch releases while the workspace lock selects exact reviewed versions.

#### Scenario: Lattice selects newer compatible patches
- GIVEN Lattice selects BLAKE3 1.8.5 and Serde JSON 1.0.151
- WHEN Cargo resolves `choregraph-nickel` in the Lattice graph
- THEN resolution MUST succeed without a duplicate lower Choregraph patch requirement.

#### Scenario: An exact public patch requirement returns
- GIVEN a public Choregraph package requires one exact BLAKE3 or Serde JSON patch
- WHEN foundation validation runs
- THEN validation MUST reject the requirement before release.
