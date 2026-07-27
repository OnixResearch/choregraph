## Why

The published `execution-graph-core` crate already supplies deterministic DAG behavior to Seaglass and Lattice. Choregraph needs a separate compiler, canonical machine model, formal proof rail, and adapter workspace without creating a second production transition implementation.

A reviewed foundation must establish ownership, licensing, dependency direction, compatibility, and verification boundaries before machine semantics move.

## What Changes

- Establish Choregraph as a standalone Rust, Nickel, Nix, and Cairn repository under `AGPL-3.0-or-later`.
- Define workspace boundaries for the pure core, Kamacite adapter, Nickel frontend, Seaglass adapter, and CLI shell.
- Keep the core `no_std` plus `alloc` and keep every effectful operation in a thin shell.
- Record the exact `execution-graph-core` source and behavior baseline before migration.
- Require one production transition implementation and a later compatibility facade for the published `execution-graph-core` API.
- Pin toolchain and dependency identities without using ambient sibling paths as release inputs.

## Success Criteria

- Repository policy, licensing, provenance, workspace layout, and verification commands are explicit.
- Dependency direction prevents filesystem, evaluator, runtime, or receipt authority from entering the core.
- The migration plan cannot leave independent Choregraph and execution-graph transition implementations.
- Positive and negative foundation checks cover supported and forbidden dependency edges.

## Non-Goals

- Implementing machine semantics or formal proofs.
- Changing current Seaglass or Lattice behavior.
- Publishing, pushing, or changing Radicle authority.
- Relicensing existing AGPL source.

## Impact

- **Repository**: root documentation, licensing, workspace manifests, Nix development shell, and Cairn policy.
- **Compatibility**: immutable baseline evidence for `execution-graph-core` and its consumers.
- **Testing**: workspace boundary, license, provenance, and forbidden-dependency checks.
