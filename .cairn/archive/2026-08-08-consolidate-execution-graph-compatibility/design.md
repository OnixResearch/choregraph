## Context

`execution-graph-core` already converts its public data types and calls `choregraph-core`. The old repository has a separate RID and immutable rollback evidence.

## Design

Add `crates/execution-graph-core/` to the Choregraph workspace. The package keeps its public name and moves to version `0.2.0` because its canonical source repository changes.

The crate depends only on `choregraph-core` and `serde`. Its adapter converts inputs, diagnostics, states, actions, and results. It does not contain graph admission, ordering, readiness, lifecycle, resume, or analytics algorithms.

Keep `vendor/execution-graph-core-selected/` as the immutable test oracle for the pre-Choregraph implementation. The active facade and the legacy oracle use different package versions, so Cargo can audit both sources.

The old repository remains unchanged except for a frozen-status notice. Consumers can retain its exact revision while they prepare a separate pin migration.

## Validation

Run the compatibility crate tests, the complete workspace tests, Clippy, the foundation rail, no-std compilation, Cairn gates, and Nix validation.

Negative cases must cover malformed graphs, cycles, invalid transitions, terminal mutation, missing nodes, and selected bounds.

## Non-claims

This change does not publish source, update downstream pins, prove graph correctness, or grant execution authority.
