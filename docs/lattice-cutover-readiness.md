# Lattice cutover readiness

The Lattice cutover is blocked until one immutable Choregraph release source exists and both consumer repositories remove their old decision logic.

## Current state

| Gate | State | Evidence |
|---|---|---|
| production-linked single-step proof | pass | `evidence/single-step-proof.ncl` |
| production-linked finite-trace proof | pass | `evidence/trace-replay-proof.ncl` |
| Kamacite canonical interchange | pass | `fixtures/kamacite/` |
| restricted Nickel boundary | pass | `docs/restricted-nickel-lowering.md` |
| Seaglass behavior parity | pass | conformance parity evidence |
| Lattice diamond integration | pass | conformance diamond evidence |
| exact Lattice disposable compile | pass | conformance downstream evidence |
| local immutable publication candidate | pass | `evidence/release/publication-candidate.ncl` |
| published immutable Choregraph source | pass | `rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF` at the exact candidate revision |
| Cargo and Nix consumer source alignment | blocked | each consumer must select the exact HTTPS adapter and revision |
| execution-graph legacy removal | blocked | the consumer repository still owns its old implementation |
| Lattice duplicate removal | blocked | Lattice still consumes its legacy vendor package |

Passing conformance does not authorize cutover while a blocked row remains.

## Candidate handoff identity

The local handoff candidate binds:

- source revision `fc47c5f4ecbb7b4341af690fa42199f25d57f54c`;
- source archive BLAKE3 `dcb1faf95a7487145496ce986ae1639908600ace5bd77c4d96a37a530a7538a5`;
- release bundle `a2ac1336679542fa9573b356daddc18266fdda8147ddbe28f85aa37d135030d9`;
- rollback revision `2772442d14bfc90579027e4fbff545d7f829c76c`;
- license `AGPL-3.0-or-later`;
- acquisition adapter `radicle-rid-exact-revision-v1`.

The candidate is published as `rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF`. Consumers must use `https://git.onix.computer/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git` at the exact candidate revision and must rerun their own source gates.

## Required cutover order

1. Publish one admitted Choregraph source with an exact repository identity, revision, archive BLAKE3, license, and release bundle.
2. Re-run current proof, interchange, Nickel, parity, negative, and downstream gates against that exact source.
3. Make Cargo and Nix select the same exact Choregraph revision in `execution-graph-core`.
4. Convert `execution-graph-core` to a facade and remove its graph and transition algorithms.
5. Run the execution-graph parity, duplicate, source, no-std, Cargo, Clippy, Cairn, Octet, and Nix gates.
6. Make Cargo and Nix select the same exact facade and Choregraph sources in Lattice.
7. Dual-run Lattice only in tests. Then select the Choregraph path and remove temporary comparison code.
8. Run Lattice persistence, replay, authority, runtime, duplicate, source, Cargo, Cairn, Octet, and Nix gates.

A feature flag, path override, source switch, or runtime fallback does not complete a step.

## Source alignment requirements

Each consumer must record the same Choregraph repository identity and revision in Cargo, Nix, lockfiles, source evidence, and release receipts.

The source gate rejects:

- floating branches or tags;
- GitHub or sibling-path fallback;
- alternate vendored source;
- Cargo and Nix revision mismatch;
- manual `flake.lock` edits;
- stale archive, proof, receipt, or toolchain identity;
- skipped proof or conformance checks.

The local sibling path used by `tools/check-lattice-downstream.rs` is a disposable test input. It is not an admitted release source.

## Consumer handoff records

`execution-graph-core` must record the assigned Choregraph RID, candidate source revision, source archive BLAKE3, release-bundle identity, generated Cargo lock state, generated Nix lock state, parity result, duplicate-removal result, and its pre-cutover rollback revision.

Lattice must record those same Choregraph identities. It must also record the accepted execution-graph facade revision, persisted-state migration identity, disposable dual-run result, duplicate-removal result, authority-gate result, and its pre-cutover rollback revision.

A consumer record fails if any Choregraph identity differs from the local candidate. It also fails if the assigned RID cannot acquire the exact candidate revision. After source admission, each consumer must rerun its own checks. Choregraph evidence cannot replace those checks.

The active owners remain:

- `execution-graph/cairn/changes/delegate-core-to-choregraph/`;
- `lattice/.cairn/changes/adopt-choregraph-verified-machines/`;
- `lattice/.cairn/changes/authorize-choregraph-effect-intents/`.

## Duplicate-removal requirements

After cutover, maintained checks must show:

- one production `step` implementation in `choregraph-core`;
- no independent ordering, cycle, readiness, lifecycle, resume, replay, or transition algorithm in `choregraph-seaglass`;
- no such algorithm in the `execution-graph-core` facade;
- no duplicate machine decision path in Lattice;
- no production dependency on the legacy Lattice vendor implementation;
- no temporary dual-run code in production targets.

`architecture-boundary.rs` checks the current Choregraph adapter. The execution-graph and Lattice Cairn changes must add equivalent repository-owned gates.

## VCS rollback requirements

Rollback changes source, generated locks, adapters, and compatibility fixtures together in one reviewed VCS operation.

Recorded pre-cutover states are:

- Choregraph conformance baseline: `e27b283c`;
- execution-graph rollback revision: `c415ddf2c8252dd9ba6cb267a4e6346158069823`;
- Lattice rollback revision: `c8d56ad83a57287064edb85f58808f53e4bf4536`.

Before cutover acceptance, each consumer must test its rollback in a disposable checkout and rerun the recorded baseline tests. Rollback must not use a runtime source selector or hidden dependency fallback.

## Cross-repository lifecycle owners

- `execution-graph/cairn/changes/delegate-core-to-choregraph/` owns the published facade migration.
- `lattice/.cairn/changes/adopt-choregraph-verified-machines/` owns product machine adoption.
- `lattice/.cairn/changes/authorize-choregraph-effect-intents/` owns authorization and runtime effect handling.

Choregraph conformance does not complete those changes.

## Claim boundary

This document records gate order, source rules, duplicate-removal rules, and rollback identities. Publication is complete. Consumer cutover, rollback testing, runtime authorization, and consumer release remain separate.
