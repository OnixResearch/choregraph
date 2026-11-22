# Execution-graph and Lattice handoff — 2026-07-27

## State

The handoff contract is defined. Consumer cutover remains blocked because the local candidate has no Radicle RID.

## Exact upstream values

- Candidate: `e4b5cd7bd77d78c20cf8b96cd0c038144028d2287000757c8ab68f5f5dfdff1a`
- Source revision: `fc47c5f4ecbb7b4341af690fa42199f25d57f54c`
- Source archive: `dcb1faf95a7487145496ce986ae1639908600ace5bd77c4d96a37a530a7538a5`
- Release bundle: `a2ac1336679542fa9573b356daddc18266fdda8147ddbe28f85aa37d135030d9`
- Rollback revision: `2772442d14bfc90579027e4fbff545d7f829c76c`
- License: `AGPL-3.0-or-later`
- Acquisition adapter: `radicle-rid-exact-revision-v1`
- Radicle RID: unassigned

## Execution-graph acceptance

`execution-graph-core` must use one assigned RID and the exact source revision in Cargo and Nix. It must generate lockfiles, rerun full parity, remove the legacy implementation, reject fallback sources, and record its own rollback test.

Its evidence must cite the candidate, source archive, and release bundle above. A source or evidence mismatch blocks acceptance.

## Lattice acceptance

Lattice must use the same Choregraph RID and revision. It must also select the accepted execution-graph facade revision.

Lattice must rerun persisted-state migration, replay suppression, dual-run parity, duplicate-removal, effect-intent binding, Basalt authorization, capability routing, runtime, and rollback checks.

The Basalt authenticated-SSH UCAN dependency remains a separate blocker for anonymous effect-authority adoption.

## Preserved worktrees

No execution-graph or Lattice source, lockfile, or VCS state changed during this release-evidence step. Their active Cairn changes remain the owners of cutover and authorization work.

## Claim boundary

This handoff defines exact admission data and consumer-owned gates. It does not prove publication, source acquisition, consumer migration, runtime authority, or release eligibility.
