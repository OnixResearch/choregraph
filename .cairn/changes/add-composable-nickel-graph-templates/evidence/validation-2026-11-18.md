# Construction validation, 2026-11-18

This record covers the landing of the construction slice onto `main`
`0dd723f1a53bdc7d0529dde8f2813f7d3c2d7922`. It is execution evidence for the
named revisions only. It is not a release, a publication candidate or a
Lattice adoption record.

## Revisions

| Commit | Tree | Content |
| --- | --- | --- |
| `f766b369eb67bb63fe1a1072747cc54cea7d210e` | `8b96adfad57a3f676375d478b2f47be1e9d6c06a` | Octet `5b1e119` deny-all lints and owner-qualified namespaces |
| `6a1ab382c4030d6ab2e28cfdbe2d516de51c7033` | `bf0c7305f3a85f2d92d1c7853fc966d2e46d9974` | This change: `choregraph.graph-construction.v1` library, contracts, native elaboration, fixtures and rail |

Host: x86_64-linux. The locked development shell supplied Rust 1.97.1 and the
pinned Nickel; Nix builds ran on the configured remote builder.

## Maintained rail

`nix flake check -L` against each exact revision
(`git+file://…?rev=<commit>`) ran all 21 `x86_64-linux` checks:
`cairn-policy`, `core-thumb`, `core-wasm`, `epoch-fence-verus`,
`foundation-boundaries`, `foundation-evidence`, `graph-construction`,
`history-wasm`, `octet-deny-all`, `octet-deny-all-policy`,
`restricted-nickel-contracts`, `role-choreography`,
`role-choreography-contracts`, `single-step-verus`, `trace-replay-verus`,
`wasm-kernel-component`, `wasm-kernel-conformance`, `wasm-kernel-evidence`,
`workspace-clippy`, `workspace-format` and `workspace-tests`.

Both runs exited 0 with `all checks passed!`. In the construction revision's
`workspace-tests`, `sequence_parallel_interchange_changes_barriers_and_is_not_equivalent`
and `actual_nickel_exports_compile_and_native_controls_reject` reported `ok`.

The `graph-construction` check typechecked the shipped library and contracts,
exported every positive and native-negative fixture with the pinned Nickel,
required both contract-negative fixtures to fail export, and ran
`actual_nickel_exports_compile_and_native_controls_reject` against those exact
export bytes (`ok`). The same test is reported `ignored` in ordinary Cargo
runs, as designed.

Development-shell runs on the construction tree: `cargo test --locked -p
choregraph-nickel --lib` passed 56 tests; workspace `cargo test`, `cargo
clippy -- --deny warnings` and `choregraph-foundation-check --root .` passed;
`nix run path:../cairn#cairn -- validate --root .` with Cairn `1cca6fe`
reported `valid: true` with no findings.

## Task 7: captured evaluation

Observed: exact exported fixture JSON in the maintained rail, as above.

Not observed: the nickel-eval `nickel-eval.captured-modules.v1` worker and
closure controls. nickel-eval's captured-module implementation
(`src/captured/`, `src/closed/worker.rs`, `tests/captured_modules.rs`) exists
only as uncommitted work in a local checkout; its published revision
`89846a6` has no captured-module profile. The construction rail's export
uses `nickel export`, not a captured worker, and the native unit fixtures
use explicitly synthetic evaluator observations. Task 7 stays open until
nickel-eval publishes the profile and a captured run of these fixtures is
recorded.

## Task 8: native admission controls

The 56 `choregraph-nickel` library tests and the export integration test
exercise native admission, limits, semantic negatives and stale bindings:

- `construction::tests::capture`: exact materials and observations,
  source/import/parameter drift, stale compilation materials, bounds,
  node and identifier ceilings, origin offsets and UTF-8 boundaries.
- `construction::tests::denial`: wrong direction, unknown port, type mismatch,
  wrong-scope handle, duplicate key, unknown operation/template/parameter,
  open root, cycle, registry-selected copy/drop and effect rules, and author
  labels that cannot widen registry eligibility.
- `construction::tests::traversal`: recursive-template and depth budgets,
  expression depth, identity-only cycles and denial ordering.
- `construction::tests::readmission`: current-registry readmission of
  persisted v3 projections and node/port/expression ceilings.
- `construction::tests::references` and `composition`: scoped effect,
  configuration and slot rewriting with exact references unchanged,
  invocation roles, nested double instantiation, shared handles and ordered
  boundaries.
- `construction::tests::execution`: Unit sequences and generated fork/join
  through production readiness.

## Task 10: observer-law review

The bounded observer compares operations and the union of data and control
dependencies modulo scoped renaming
(`construction::tests::algebra::unit_observer`). Reviewed controls:

| Claim or control | Test |
| --- | --- |
| Independent pure parallel parts commute; authored order stays bound | `pure_laws_hold_only_when_dependency_and_effect_premises_hold` |
| Sequence association with equal barriers | `bounded_associativity_and_dependency_mutation_controls` |
| Changed operation, omitted data or Unit dependency, wrong boundary are detected | both tests above |
| Identity neutrality keeps routing and dependencies | `structural_identity_keeps_exact_routing_and_required_dependencies` |
| Boundary permutation and inverse; swapped outputs are observable | `inverse_public_boundary_permutation_is_not_identity`, `ordered_parallel_boundaries_change_positional_sequence_routing` |
| Effectful sequence keeps its control edge and distinct effect bindings | `actual_nickel_exports_compile_and_native_controls_reject` (`effectful-sequence`) |
| Sequence/parallel interchange is not an equivalence | `sequence_parallel_interchange_changes_barriers_and_is_not_equivalent` |

The interchange control was missing from the authored slice and was added
before landing. It admits `(a;b) ∥ (c;d)` and `(a ∥ c);(b ∥ d)`, shows equal
operations, the lane relation `{a→b, c→d}`, the all-to-all stage relation
`{a→b, a→d, c→b, c→d}`, and different admitted machines.

Conclusion: the documented conditional laws have bounded positive and
mutation controls, and general interchange is shown nonequivalent. No
semantic-equivalence theorem or Nickel-level law is claimed.

## Task 11: identities and release authority

Validation identities are recorded above. The landing pushes source to
`main` at the maintainer's request; it is not a release and records no
release authority. Task 11 stays open for a release decision.

## Published-cohort qualification

This follow-up is separate from the landing observation above. It concerns an
exact published evaluator/consumer cohort, not general Nickel correctness or
release authority.

| Role | Published revision |
| --- | --- |
| Captured evaluator, `nickel-eval` | `f0ea0d2a31d3482ddf4e575482f6ddf7a7e7d4e9` |
| Choregraph construction producer pinned by Lattice | `c3577352ee1e39391f5204ee234d991c7dbeb151` |
| Lattice consumer | `b858d85b89af8d1dda4ed0047afafd99ab233d0b` |

On `x86_64-linux`, `nix flake check --no-write-lock-file -L` on the exact
evaluator revision from its canonical anonymous Radicle seed passed all four
checks: all-target tests, native-source provenance, vendored-package listing,
and formatting. The fetched NAR hash matched the Lattice lock
(`sha256-faGsaWICfFaA1Jzl4eRW8NxQMpDscikTZSR2vd6XlV4=`). The test rail ran
12 captured-module and 4 captured-worker tests, all passing. These cover
observed logical resolution, no ambient fallback, parent receipt validation,
and fail-closed wire and resource controls.

The canonical seed also served Lattice's Choregraph pin with the locked NAR
hash `sha256-3ct+0RK3oCsSqNZlxXpbgdw6EJYJf+e+YmvPS7HgAzw=`. The shipped
construction library and contracts, fixture directory, Rust construction
subtree, and native export test were byte-identical at that pin and at
Choregraph `cca40da47ef3e64ba61b87fdf255e7772a5a85bd`. This comparison does
not qualify Lattice against the later Choregraph kernel or change its pin.

### Captured exports and native admission

The fixture run submitted each unchanged entry with the exact shipped library
and contracts in a three-member logical map to Lattice's packaged companion
worker (`lattice-closed-eval-worker`, BLAKE3
`d85d145a0d20cd318b080df2feb93c0f2fad614fa2451f8a4d2a44e1d63de86d`).
The direct wire probe used an empty environment, `/` as CWD, a 2 GiB
address-space limit and a 5 s deadline. On each success it checked the receipt
against the exact input bytes, the observed entry → library → contracts edges,
the pinned cohort and limits, and the output byte count and digest. Both
contract-negative fixtures and `unit-sequence` with the captured contract
omitted failed with no result envelope.

| Fixture export | Bytes | Exact JSON BLAKE3 |
| --- | ---: | --- |
| `effectful-sequence` | 2483 | `1ac052c5373ed44e55160e73d6895708ad1fc585b186fd903cfbb537de6225c2` |
| `generated-fork-join` | 3101 | `44a507a59eb3f50de48497a76f3a74980ed120e7f4245321fcc39d4f7b793a1a` |
| `invocation-sequence` | 2154 | `bae165f6f5344a1aae5fcb5d79c2e92413cde6993b900b72fc815e71eadf6b2d` |
| `nested-twice` | 3513 | `9f30148a85fda22b3b2c65ea3230e75803f0af38e010801eb59579ac9afed756` |
| `unit-sequence` | 1023 | `cfbdc41ede70bfce36e83334b966337abc679043df19426034d3c8b9ed6af231` |
| `negative/cycle` | 1643 | `7fb7b2b0d49d7fc732dda85fefb6ab210e829af1d54e1693f87fe0c3148191a6` |
| `negative/linear-copy` | 1694 | `b9c28e8f6aef59049bdba4a9c3e2d2eb50c57dce63894d9f1ddd26485d4169c9` |
| `negative/linear-drop` | 556 | `69d8f88282c1c3f35b7d438cdcc734f808d00f902dadeee4bd6187e2cd63ba07` |
| `negative/wrong-type` | 1114 | `98887a53b75f98133dc564dc1633a4776e528adf6a1ff50b7944f074eb8a6a86` |

`actual_nickel_exports_compile_and_native_controls_reject`, built by the
maintained `graph-construction` Nix rail, passed with
`CHOREGRAPH_CONSTRUCTION_EXPORTED_DIR` pointing to those exact export bytes,
without JSON re-encoding. It admitted all five positive candidates and
rejected the four native-negative controls. Its host binding remains a
synthetic fixture; the separate worker receipts and the consumer's real
binding path are the evidence of captured evaluation.

### Lattice host run

The published `b858d85b89af8d1dda4ed0047afafd99ab233d0b` flake built from the
canonical seed. `nix run <exact-revision>#construction-runtime-qualification
-- <new-output-directory>` used the same derivations from the local immutable
Git object and the same companion worker as the fixture run. The installed
Lattice executable BLAKE3 was
`83cf9fcd3500452d0747871719fbbea969889debc9e9f591ac2e43ac0b32d377`.

The host app passed all 28 steps with zero unexpected outcomes, including
captured import, origin checks, author and expanded exports, and inspection
of exact effect authority. Execution with that exact grant completed six
nodes with zero failures. Execution with no grant or a foreign grant failed;
wrong-scope and generation-overflow imports left the admitted revision
unchanged, and an unlabeled source export was rejected.

All 67 selected tests passed: 11 construction authoring, 14 runtime binding,
8 construction qualification, 32 source authoring, 1 workflow-format and 1
TUI origin test. In particular, the consumer rejected a foreign registry and
a runtime expression reference on materialized readmission, rejected mutated
retained bindings, and kept the old authorized execution snapshot while
readmitting changed source. These observations qualify the named local-shell
cohort only. They are not a pin update, external-service qualification,
semantic-equivalence claim or release decision.
