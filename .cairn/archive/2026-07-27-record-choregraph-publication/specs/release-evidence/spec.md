# Release Evidence Specification Delta

## ADDED Requirements

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
