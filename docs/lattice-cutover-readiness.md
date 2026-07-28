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
| immutable Choregraph release source | blocked | release evidence is not complete |
| Cargo and Nix consumer source alignment | blocked | no release source is selected |
| execution-graph legacy removal | blocked | the consumer repository still owns its old implementation |
| Lattice duplicate removal | blocked | Lattice still consumes its legacy vendor package |

Passing conformance does not authorize cutover while a blocked row remains.

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
- `lattice/cairn/changes/adopt-choregraph-verified-machines/` owns product machine adoption.
- `lattice/cairn/changes/authorize-choregraph-effect-intents/` owns authorization and runtime effect handling.

Choregraph conformance does not complete those changes.

## Claim boundary

This document records gate order, source rules, duplicate-removal rules, and rollback identities. It does not claim that consumer cutover, publication, rollback testing, runtime authorization, or release is complete.
