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
