# Restricted Nickel capability-shell blocker — 2026-07-27

## Completed surfaces

The following unprivileged surfaces are implemented:

- closed Nickel contracts for direct Choregraph data and the selected Lattice port-graph projection;
- normalized positive Nickel exports and contract-negative fixtures;
- pure Rust JSON revalidation with unknown-field rejection and explicit ceilings;
- Lattice port-to-node edge resolution and normal Choregraph machine admission;
- exact lifecycle event validation;
- content-addressed import closure validation over supplied in-memory bytes;
- stable bounded diagnostics that discard raw evaluator text;
- domain-separated translation identities;
- source and evaluator receipts with explicit semantic non-claims.

Focused Rust tests passed nine positive and negative cases. Workspace tests and Clippy passed after the Cargo lock and Kamacite corpus were refreshed. `nix flake check path:$PWD -L` passed all current checks, including `restricted-nickel-contracts`.

Cairn policy freshness, repository validation, and the proposal, design, and tasks gates passed. The tasks gate records eight completed tasks and two blocked tasks.

## Exact blocker

The required CLI shell must use `cap-root` before `cap-std` I/O. The local `cap-root` repository has no configured publication remote, and `https://github.com/OnixResearch/cap-root` is not available anonymously.

The local `cap-root` policy states that consumers must not add sibling path dependencies. Adoption requires a published immutable Git revision and aligned Cargo and Nix source identity.

Observed local candidate revision:

```text
6003544ef57651414e779411ac3a6f2c804a9e56
```

The existing Choregraph source baseline records an older reference-only revision. Neither revision is an admitted release source.

## Required resolution

1. Publish a reviewed immutable `cap-root` revision.
2. Record its anonymous acquisition URL, revision, license, and source identity.
3. Align the exact Cargo dependency and Nix source.
4. Add `cap-std` `4.0.2` in the CLI shell only.
5. Test operator-opened roots, relative path admission, no-follow policy, bounded reads and writes, atomic publication, redaction, and rollback.
6. Run the remaining capability and lifecycle gates.

## Prohibited fallback

Choregraph will not use a sibling path dependency, copy the lexical path implementation, accept graph-selected ambient roots, or report lexical admission as filesystem confinement.

## Claim boundary

The current frontend can validate and lower already evaluated in-memory data. It does not evaluate Nickel source or provide capability-confined file access. This change remains active and is not ready to archive.
