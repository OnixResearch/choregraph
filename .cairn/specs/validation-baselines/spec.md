## ADDED Requirements

### Requirement: Trace failure fixtures follow the production proof signature

r[choregraph.validation_baselines.trace_fixtures]

Negative trace fixtures MUST supply the production replay suspension and lifecycle-profile inputs while preserving their intended false claims.

#### Scenario: A negative trace fixture is checked

- **GIVEN** the current production replay proof signature
- **WHEN** Verus checks a negative fixture
- **THEN** the fixture compiles to the intended proof obligation and fails on its named postcondition

### Requirement: The repository policy parses with the selected Cairn source

r[choregraph.validation_baselines.cairn_policy]

The repository-owned Cairn policy MUST export with the selected Cairn workflow fields and pass the selected Cairn policy parser.

#### Scenario: The local policy is selected

- **GIVEN** the pinned upstream policy snapshot and Choregraph overrides
- **WHEN** the policy is exported and parsed
- **THEN** all required workflow and policy fields are present

### Requirement: Project policy overrides remain explicit

r[choregraph.validation_baselines.policy_overrides]

Choregraph-specific traceability, evidence, receipt, mutation, and non-claim fields MUST remain explicit forced overrides of the pinned upstream default.

#### Scenario: Upstream adds a policy field

- **GIVEN** a new field in the pinned upstream policy
- **WHEN** the Choregraph policy is exported
- **THEN** the new field is inherited without replacing project-owned override fields

### Requirement: Invalid policy inputs remain rejected

r[choregraph.validation_baselines.negative_policy]

The policy checks MUST include an invalid workflow-path fixture and MUST reject it during Nickel export.

#### Scenario: An unsafe workflow path is supplied

- **GIVEN** a workflow artifact path that escapes the change directory
- **WHEN** Nickel applies the policy contract
- **THEN** export fails and no runtime policy is published

### Requirement: Strict source-quality checks accept the repaired baseline

r[choregraph.validation_baselines.source_quality]

The workspace MUST pass formatting, strict Clippy, and the configured Octet deny-all check after validation baselines are repaired.

#### Scenario: The repaired workspace is checked

- **GIVEN** the current durable-suspension, compatibility, proof, and adapter sources
- **WHEN** formatting, strict Clippy, and Octet run
- **THEN** all checks pass without hidden lint findings
