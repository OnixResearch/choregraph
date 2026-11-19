# Role Choreography Delta

## ADDED Requirements

### Requirement: Preference annotations are a default-off profile

r[choregraph.role_choreography.preference.profile] The cooperative baseline MUST remain the default admission path. `compile_evaluated` and v1 admission MUST admit only unannotated `choregraph.role-choreography.v1` sources with unchanged behavior, serialized shape and identity bytes. Annotated sources MUST be refused with `preference-profile-not-selected` unless the caller explicitly selects `choregraph.role-choreography.profile.preference-annotations.v1`.

#### Scenario: Annotated source reaches the baseline

r[choregraph.role_choreography.preference.profile.baseline_default]
- GIVEN an evaluated source whose schema is `choregraph.role-choreography.preference-annotations.v1`
- WHEN the baseline compiler, or the annotated compiler with the default selection, runs
- THEN admission MUST fail with `preference-profile-not-selected`
- AND no canonical artifact MUST be produced.

#### Scenario: Unannotated source is compiled after this change

r[choregraph.role_choreography.preference.profile.unchanged]
- GIVEN an unannotated v1 source
- WHEN the baseline compiler runs
- THEN unknown fields MUST still be rejected
- AND the projection receipt bytes MUST equal those produced before this change.

### Requirement: Annotated source is strict and bounded

r[choregraph.role_choreography.preference.source] The annotated source MUST wrap an unchanged v1 `base` and add a closed annotation set of declared factors (name and owning role only), named exogenous world variables, and choice annotations addressed by a structural path. Each choice annotation MUST name its decider, a non-empty strictly ordered set of that decider's factors and at most one world variable. Admission MUST admit the base with the unchanged v1 function, then reject unknown fields, wrong schema, non-claim drift, limit excess, invalid names, duplicate or unsorted declarations or annotations, undeclared roles, factors or worlds, paths that do not end at a `Choice`, stale deciders, factors owned by another role, and unused declarations. A Nickel contract MUST describe the same closed shape.

#### Scenario: Annotation is stale or malformed

r[choregraph.role_choreography.preference.source.rejected]
- GIVEN an annotation with a wrong decider, an unknown factor, an extra field or a path that leaves the protocol tree
- WHEN admission runs
- THEN it MUST fail with one bounded, safe diagnostic before projection.

### Requirement: Annotations attach to the unchanged Trellis projection

r[choregraph.role_choreography.preference.projection] The annotated path MUST project the admitted base with the existing `project_all` and v1 bundle builder; it MUST NOT add a projection algorithm. Every annotated choice MUST correspond to exactly one decider `InternalChoice` with identical labels. The local address MUST advance through the decider's own internal choices and through an `Offer` from a foreign decider with identical labels. Any other correspondence MUST fail closed with `preference-path-not-locally-addressable` or `preference-local-mismatch`. The annotated complete local set MUST contain one entry per declared role, in normalized order, naming the exact v1 local artifact identity.

#### Scenario: Two-branch choice with an uninvolved role

r[choregraph.role_choreography.preference.projection.skeleton]
- GIVEN the two-branch `projectable_choice_includes_uninvolved_role` protocol annotated with decider factors and one world variable
- WHEN the selected profile compiles it
- THEN the global artifact, every local artifact and the complete local set MUST equal the unannotated projection
- AND only the decider entry MUST carry the factor and world metadata.

#### Scenario: Decider merges an outer foreign choice

r[choregraph.role_choreography.preference.projection.unaddressable]
- GIVEN an annotated choice under another role's choice whose branches the decider's local projection merges
- WHEN annotated projection runs
- THEN it MUST fail with `preference-path-not-locally-addressable` and emit no annotated artifact.

### Requirement: Receipt binds annotations to the projection cohort

r[choregraph.role_choreography.preference.identity] A canonical packed-Preserves `PreferenceProjectionReceipt` MUST bind the following under domain-separated BLAKE3 identities:

- the profile and the schema;
- the v1 projection receipt;
- the global artifact, the ordered local artifacts and the complete local set;
- the annotated strict DTO, the annotation set and the annotated local set;
- the annotation schema set, the annotation lowerer and the annotation non-claims.

The receipt MUST record `game_mapping = not-formalized`. Verification MUST reject any field or byte difference.

#### Scenario: Receipt or local set is stale

r[choregraph.role_choreography.preference.identity.stale]
- GIVEN a receipt or annotated local set with a changed cohort identity, reordered locals, a missing or duplicated role, or a factor moved to another role
- WHEN verification runs
- THEN it MUST fail.

### Requirement: Reports keep outcomes distinct

r[choregraph.role_choreography.preference.report] The annotated compiler MUST report projection status (`projected` or `refused`), diagnostics, and game-mapping status as separate fields. Game-mapping status MUST be `not-formalized` for every outcome.

#### Scenario: Success and refusal summaries

r[choregraph.role_choreography.preference.report.distinct]
- GIVEN one projected and one refused annotated compilation
- WHEN their summaries are produced
- THEN the statuses MUST differ, only the projected summary MUST carry a receipt identity, and both MUST state `not-formalized`.

### Requirement: Preference annotations claim no strategic semantics

r[choregraph.role_choreography.preference.boundary] Annotations MUST be metadata only. They MUST NOT select a branch, weigh factors, grant authority, or convert a label set into a Boolean. Choregraph MUST NOT claim a solver, a utility function, a world distribution, equilibrium or incentive-compatibility analysis, a game semantics, or the soundness of Trellis projection for self-interested participants. The closed annotation non-claim set MUST be declared by every annotated source. Existing v1 non-claims MUST remain in force.

#### Scenario: Non-claim is removed or a claim is added

r[choregraph.role_choreography.preference.boundary.non_claims]
- GIVEN an annotated source missing one annotation non-claim, or a receipt whose claim promotes solver or equilibrium evidence
- WHEN admission or receipt verification runs
- THEN it MUST fail.
