# Enforce Octet deny-all

## Why

Choregraph pins Octet for Verus, but it does not run Octet source lints. Clippy warnings are errors, but Octet warnings can remain unseen.

## What Changes

- Add a consumer-owned Octet configuration.
- Pin the complete deny-all lint set to the reviewed Octet revision.
- Run Octet against the full workspace and all targets.
- Make the strict Octet check part of `nix flake check`.
- Reject warning-level, disabled, incomplete, duplicate, or stale lint policy.
- Require bounded thresholds and documented source-local exceptions for reviewed compatibility constraints.

## Impact

The change adds build-time checks only. It does not add runtime authority or change machine semantics.
