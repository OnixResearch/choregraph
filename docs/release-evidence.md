# Release evidence

Choregraph release evidence uses a pure deterministic linkage core and a separate local artifact shell.

## Pure core

`choregraph-release` is `no_std + alloc`. It reads no files, starts no process, uses no clock, accesses no network, and has no publication authority.

The core defines 30 closed artifact roles for:

- source, license, schemas, production source, and built library;
- single-step and trace proof sources and results;
- trusted surface and Rust, Verus, solver, Nickel, Nix, and Cargo identities;
- Kamacite machine, lifecycle, trace, exhaustiveness, source, and proof-reference artifacts;
- positive and negative fixtures;
- execution-graph parity and Lattice downstream evidence;
- Cairn, Octet, and Nix gate receipts.

Each role has one required schema and status. A complete bundle has each role exactly once.

## Identity

Project-owned bundle and candidate identities use BLAKE3 with separate derive-key contexts.

The bundle identity uses length-framed bytes for:

- bundle schema and profile;
- sorted role, relative path, schema, status, and artifact digest fields;
- the fixed non-claim list.

Input order does not affect the identity. Paths must be relative lexical paths without empty, dot, parent, backslash, NUL, or drive-prefix segments.

## Verification

The pure verifier rebuilds the expected bundle from current observations. It compares schema, profile, non-claims, normalized artifacts, and bundle identity.

Validation fails for missing, duplicate, malformed, stale, skipped, unsupported, mismatched, or overclaimed evidence.

`choregraph-cli write-release-bundle` reads 30 fixed relative paths through capability roots. It enforces input bounds and publishes one local manifest without replacement.

`choregraph-cli verify-release-bundle` reopens the artifact and manifest roots. It rebuilds the expected bundle and requires exact manifest-byte equality. It does not trust a recorded identity without current artifact bytes.

The checked-in manifest is `evidence/release/generated/choregraph.release.bundle`. The adjacent BLAKE3 file identifies the manifest bytes, not the internal bundle framing.

## Valence projection

The Valence-friendly projection separates these roles:

- identity observation;
- canonical artifact;
- presented proof result;
- conformance observation;
- gate observation.

Every projection entry states `linkage-only-no-claim-promotion`. Projection validation rebuilds the exact projection from the bundle.

Valence linkage does not create a behavioral proof or release decision.

## Publication candidate

The pure candidate model requires:

- exact 40-character lowercase source and rollback revisions;
- a nonzero source archive BLAKE3;
- `AGPL-3.0-or-later`;
- the exact six-package set;
- `radicle-rid-exact-revision-v1` acquisition;
- the exact release-bundle identity.

Candidate construction performs no publication, signing, push, remote update, rollback, or source acquisition.

## Local commands

```text
choregraph write-release-bundle <artifact-root> <output-root> <output-path>
choregraph verify-release-bundle <artifact-root> <manifest-root> <manifest-path>
```

All roots are absolute operator inputs. All artifact and output names under those roots are fixed or capability-admitted relative paths.

## Architecture selection

Three designs were reviewed:

1. A pure release crate plus a thin capability-relative CLI shell. This is selected.
2. Kamacite-owned release semantics. This was rejected because Kamacite owns canonical interchange, not Choregraph release policy.
3. A tool-only script. This was rejected because consumers need a reusable pure verifier.

A secondary review correctly emphasized exact role completeness, stale-link rejection, license, package, acquisition, and rollback coverage. Suggestions that the pure core apply rollback or infer user capabilities were rejected because they would import authority.

## Claim boundary

A valid bundle proves exact linkage for the presented artifacts and selected gate observations. It does not prove verifier soundness, compiler correctness, Nickel equivalence, event truth, authorization freshness, runtime effects, remote availability, consumer cutover, or whole-product correctness.
