# Cutover and authority audit — 2026-07-27

## Result

The cross-repository cutover order, source-alignment rules, duplicate-removal gates, rollback revisions, and retained Lattice authority are recorded.

Consumer cutover remains blocked. Choregraph does not yet have an admitted immutable release source, and both consumers still contain the pre-cutover implementation.

## Cutover evidence

`docs/lattice-cutover-readiness.md` records:

- the dependency-ordered gate sequence;
- exact Cargo and Nix source-alignment requirements;
- rejected floating, fallback, stale, skipped, and mismatched source cases;
- post-cutover duplicate-removal criteria;
- VCS-only rollback requirements;
- pre-cutover Choregraph, execution-graph, and Lattice revisions;
- the owning execution-graph and Lattice Cairn changes.

The disposable Lattice check proved that the reviewed rollback revision can compile with the current adapter. It did not change the Lattice working tree.

## Authority evidence

`docs/lattice-authority-boundary.md` records the authority matrix and intent handoff.

`crates/choregraph-seaglass/tests/architecture-boundary.rs` passed its positive and negative cases. It checks that the production adapter depends only on `choregraph-core`, calls each required delegate, and contains no selected runtime-authority or independent-loop tokens.

The effect-authority fixture also passed. One declared start request emitted one pure intent. A consumer-owned denial result moved the node to `Failed` and emitted no repeated intent.

Lattice retains evaluator, Basalt, UCAN-fact, capability-root, filesystem, process, network, persistence, retry, schedule, trigger, diagnostic, UI, and evidence authority.

## Open blockers

- Choregraph release source and bundle are not selected.
- execution-graph has not removed its legacy implementation.
- Lattice has not selected Choregraph as its only machine path.
- Consumer rollback checks remain owned by their active Cairn changes.
- Basalt still uses an authenticated UCAN source.

## Validation

These commands passed:

```text
nix develop -c cargo test -p choregraph-seaglass --test architecture-boundary
nix develop -c cargo test -p choregraph-seaglass
nix develop -c cargo clippy -p choregraph-seaglass --all-targets -- -D warnings
nix run .#octet-deny-all
nix run path:../cairn#cairn -- validate --root .
```

Octet reported zero findings, warnings, and errors.

## Claim boundary

This audit proves selected Choregraph source structure and records consumer obligations. It does not prove consumer cutover, runtime authorization, external effects, VCS publication, or release eligibility.
