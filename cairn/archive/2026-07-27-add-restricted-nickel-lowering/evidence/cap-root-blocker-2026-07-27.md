# Restricted Nickel capability-shell source resolution — 2026-07-27

## Historical blocker

The CLI shell required a published immutable `cap-root` source. The local repository had no remote, and the configured GitHub URL was unavailable.

A sibling path dependency, copied lexical implementation, or ambient path fallback was prohibited.

## Radicle publication

The reviewed source is now published through Radicle:

- RID: `rad:z3cWBCK7VPbEmbL5MAvKJWKjCE745`
- Revision: `8991e2572578c16e590c2f1c0915db6c46c5d9cb`
- Delegate: `did:key:z6MksnXbFoE8zkCkGWhHc8zuxpnEUhrJHv2KECRV4GSv9gkx`
- License: `AGPL-3.0-or-later`
- Selected source BLAKE3: `60fbdb7eae53dff8c2c0d41bc15aa123cd3a362ec1611eb4bc892fd6be58f3cb`

A fresh Radicle clone contained the reviewed source commit and later distribution-metadata commit `545e77e5a49e4e1d34037aa9a82d924944ac0a1b`.

The initial sync reached no independent seed. The RID and local signed reference do not prove indefinite network availability.

## Source alignment

`tools/sync-cap-root-core.rs` imports the exact Rust source and AGPL license into `vendor/cap-root-core-selected`.

Cargo uses that reviewed vendor path. Nix builds the same repository source and checks the recorded source and license hashes.

Bounded evaluator execution uses Bounded Exec revision `29dac88ecded94457572db3fdfaaaab95fa91525` through its public Radicle HTTPS adapter.

## Capability result

The CLI now:

- opens operator-selected source, contract, scratch, output, and import roots;
- admits every relative path with `cap-root`;
- uses `cap-std` for file operations;
- rejects symlink components through `cap-fs-ext`;
- reads all inputs with explicit byte limits;
- runs Nickel through Bounded Exec with clear environment and explicit process limits;
- discards raw evaluator diagnostics;
- publishes a new canonical machine artifact without replacement;
- rolls back temporary output after publication failure.

Positive and negative tests cover source, imports, path escapes, unknown roots, symlinks, size limits, output collisions, forbidden Nickel imports, evaluator failures, and diagnostic redaction.

## Claim boundary

This result does not prove filesystem confinement, sandboxing, Nickel evaluator correctness, source semantic equivalence, directory-entry durability, independent Radicle replication, or release eligibility.
