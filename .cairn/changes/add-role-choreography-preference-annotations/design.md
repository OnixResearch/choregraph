# Design: default-off preference annotations

## Profile gate

`ProfileSelection` defaults to `CooperativeBaseline`. The baseline `compile_evaluated` keeps strict v1 parsing. When parsing fails and the input's top-level `schema` is the annotated schema, the baseline returns `preference-profile-not-selected` instead of a generic malformed-data diagnostic. `preference::compile_evaluated` and `preference::admission::admit_source` refuse every input unless the selection is `PreferenceAnnotations`. Selector strings are exact (`ProfileSelection::from_selector`).

## Source and admission

`AnnotatedRoleChoreographySource` has exactly four fields:

- `schema`;
- `base`, an unchanged v1 DTO;
- `annotations`, which holds `factors`, `worlds` and `choices`; and
- `non_claims`, a closed set of nine values.

A factor declares an `id`, an owning `role` and a lowercase kebab `name`; it has no weight. A world variable declares an `id` and a `name`, with no distribution. A choice annotation has these fields:

- `path`, a list of `next` or `branch{label}` steps from the protocol root;
- `decider`;
- `factors`, a strictly ordered list of factor IDs owned by the decider;
- `world`, an optional world ID; and
- `span`.

Admission first calls the unchanged `crate::admission::admit_source` on `base`. Annotation checks then run against the admitted tree, under fixed limits that callers cannot widen. Every declared factor and world variable must be used. Invalid spans are reported at the origin span so diagnostics stay safe.

## Address correspondence, not projection

`build_bundle` calls `endpoint::project_all` and `linkage::build_projection_bundle` on the admitted base. The v1 local protocols, local artifacts, complete local set and v1 receipt are therefore the baseline values. Byte-identical baseline receipts were compared before and after this change.

For each annotation, `projection::local_address` walks the global path and the decider's Trellis local protocol in lockstep:

- A message involving the decider advances its local cursor by one `next` step, after the direction, peer and label are checked.
- An uninvolved message leaves the cursor unchanged.
- The decider's own choice advances through `InternalChoice` with identical labels.
- A foreign choice advances only through `Offer{from = foreign decider}` with identical labels.

The target must be an `InternalChoice` with exactly the global labels. Trellis merges a foreign choice when the branches are identical or have a common send or receive prefix. In that case the choice has no unique local address, so the walk fails closed. This check only reads Trellis output; it never constructs a local protocol.

## Artifacts and identities

The profile defines four new Preserves schemas under `schemas/preserves/`:

- source;
- annotation set;
- annotated local set; and
- receipt.

The schema versions and root labels are private `ArtifactProfile` constants in `preference::linkage`, which calls `choregraph_kamacite::codec::encode` directly. Kamacite adapter sources are not edited, so the Kamacite adapter identity and the checked-in Kamacite corpus stay byte-stable. A first attempt added a Kamacite `profile::preference` module; that changed the adapter identity and the corpus test caught it, so the module was removed. New identity domains (`preference-*`) use the existing derive-key framing, and the v1 domain labels are unchanged. The annotation lowerer identity hashes the following:

- a preimage;
- the v1 lowerer identity, which covers the v1 `admission.rs` and `endpoint.rs`; and
- the annotation `model.rs`, `admission.rs` and `projection.rs`.

The annotation schema-set identity hashes the four schema files. `AnnotatedLocalSetBody` names the v1 complete-local-set identity and, for each role, the v1 local artifact identity plus its decider annotations. `PreferenceProjectionReceipt` binds these values:

- profile and schema;
- the v1 receipt, global artifact, ordered local artifacts and complete local set;
- the annotated strict DTO, annotation set and annotated local set;
- the schema set and lowerer;
- the non-claims;
- `game_mapping = not-formalized`; and
- the linkage-only claim.

The receipt verifier requires exact body and bytes. `validate_set` rejects missing, duplicated, reordered and stale entries, and factors owned by another role.

## Reports

`Report` keeps three outcomes separate:

- `status()` returns `projected` or `refused`;
- `refusal()` returns the diagnostics; and
- `game_mapping()` always returns `NotFormalized`.

`Summary` is the serializable `choregraph.role-choreography.preference-report.v1` review projection.

## Nickel and rail

`contracts/nickel/role-choreography-preference-contracts.ncl` applies the unchanged v1 contract to `base` and closed predicates to the other fields. The positive fixture, `annotated-two-branch-choice.ncl`, encodes the issue's two-branch case; its exported JSON is checked in. The `role-choreography-contracts` Nix check typechecks both contracts and requires that JSON to match a fresh export byte for byte. It also requires every negative fixture, including the four preference negatives and v1 annotation smuggling, to fail export. The Rust tests compile that exact JSON.

## Ownership

Choregraph maintains the profile, schemas, contract and fixtures. Lattice owns consumer adoption, factor recording and its default-off session profile.
