# Octet Deny All Specification

## Purpose

Defines the `octet-deny-all` capability.

## Requirements

### Requirement: Complete strict lint policy

r[choregraph.quality.octet.complete] Choregraph MUST promote every lint in the pinned Octet deny-all catalog to an error.

#### Scenario: Octet adds or removes a lint
- GIVEN the reviewed local lint list
- WHEN it differs from the pinned Octet deny-all hook
- THEN the policy gate MUST fail.

### Requirement: No warning or disabled Octet lint

r[choregraph.quality.octet.severity] The maintained Octet gate MUST reject warning-level overrides and disabled lints.

#### Scenario: A lint becomes warning-only
- GIVEN an Octet lint configured below deny level
- WHEN the strict policy is validated
- THEN validation MUST fail.

#### Scenario: A source exception has no reason
- GIVEN a maintained Octet source exception
- WHEN it has no nearby justification
- THEN the deny-all consumer gate MUST fail it through `unjustified_allow`.

### Requirement: Full workspace coverage

r[choregraph.quality.octet.scope] The Octet gate MUST check every workspace package and all Cargo targets.

#### Scenario: A target contains a finding
- GIVEN an Octet finding in any maintained package or target
- WHEN `nix flake check` runs
- THEN the Octet check MUST fail.

### Requirement: Pinned reproducible execution

r[choregraph.quality.octet.pin] The Octet gate MUST use the exact reviewed flake source, Cargo lock file, and generated deny-all policy.

#### Scenario: Policy evidence is stale
- GIVEN generated policy that differs from the Nickel source
- WHEN the policy gate runs
- THEN the gate MUST fail.

### Requirement: Explicit non-claims

r[choregraph.quality.octet.boundary] Octet evidence MUST state that static lint success does not prove runtime correctness, proof soundness, or release eligibility.

#### Scenario: Lint output is promoted
- GIVEN a passing Octet lint result
- WHEN evidence is reviewed
- THEN it MUST remain bounded to static source lint observations.
