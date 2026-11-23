# Add default-off preference annotations to role choreographies

## Problem

[Choregraph #4](https://github.com/OnixResearch/choregraph/issues/4) notes that the role-choreography model records who decides a `Choice` but not why. Pact (arXiv 2605.03143) proposes declared preference factors ("values") and exogenous world variables for self-interested agents. Lattice ([Lattice #16](https://github.com/OnixResearch/lattice/issues/16)) wants to record such factors as metadata on admitted projection cohorts. Its `docs/role-protocol-execution.md` blocks that work until Choregraph publishes the following:

- an exact annotation schema and profile selector, distinct from `choregraph.role-choreography.v1`;
- identity-linked factor and world metadata on `Choice` deciders;
- an annotated complete local set; and
- a projection receipt binding the factors to the projection cohort.

The related Noble filing is https://github.com/OnixResearch/noble-lang/issues/67#issuecomment-6796434640.

## Proposed capability

Add the default-off profile `choregraph.role-choreography.profile.preference-annotations.v1` with these parts:

- **Source schema.** The annotated source schema is `choregraph.role-choreography.preference-annotations.v1`. It wraps an unchanged v1 base and adds declared factors, world variables and path-addressed choice annotations.
- **Pure admission.** Admission reuses v1 admission for the base.
- **Projection.** Annotations attach to the decider `InternalChoice` of the unchanged Trellis projection.
- **Artifacts.** The profile emits an annotated complete local set and a receipt that binds the annotations to the v1 cohort identities.
- **Report.** The report keeps `projected`, `refused` and `game mapping not-formalized` as separate fields.

The unannotated cooperative baseline stays the default and refuses annotated sources unless the caller selects the new profile.

## Scope and exclusions

The change includes the following:

- Rust DTOs, admission, address correspondence, artifacts and receipt;
- four Preserves schemas;
- a Nickel contract with positive and negative fixtures;
- the Nix contract rail; and
- documentation.

The change excludes the following:

- any change to v1 source, projection, identity or receipt bytes;
- a second projection algorithm;
- a Pact solver, utility function, world distribution, equilibrium, mechanism-design or incentive-compatibility check;
- game semantics;
- any claim that Trellis projection is sound for self-interested agents;
- runtime execution, authority and transport; and
- Lattice adoption.

## Evidence posture

The implementation slice records local observations in `evidence/`. Lattice adoption, publication and any release claim require separate records.
