# Release Evidence Delta

## ADDED Requirements

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
