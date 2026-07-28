# Execution-graph migration

Choregraph must become the only production transition implementation.

## Migration sequence

1. Preserve the immutable execution-graph behavior baseline.
2. Implement machine semantics only in `choregraph-core`.
3. Prove the production Choregraph transition path.
4. Dual-run old and new behavior in tests only. This step now passes for the selected matrix.
5. Change `execution-graph-core` into a conversion-only Choregraph facade.
6. Remove the old transition algorithm and temporary dual-run code.

`choregraph-seaglass` now provides the conversion-only adapter and test-only comparator. Its legacy-shaped entry points preserve the selected package signatures.

The v2 entry points accept only an admitted v2 machine. They delegate concrete events and convert returned intents without authority logic.

The adapter rejects identifiers outside the Choregraph profile, unknown resume nodes, and invalid sparse state. Legacy-shaped collection calls return no result for these failures. Legacy-shaped resume calls return `RejectTerminalMutation`. Checked calls retain the exact failure class.

`tools/check-lattice-downstream.rs` archives exact Lattice revision `c8d56ad83a57287064edb85f58808f53e4bf4536`. It substitutes only the Seaglass package dependency and runs the 21 focused runner tests in a disposable directory.

## Release gate

Release readiness fails while two production transition implementations remain. A feature flag or runtime fallback does not satisfy this gate.

Temporary dual-run logic must remain test-only. The migration must include a removal validation before consumer cutover.

V1 source is not rewritten to v2. Lattice must adopt the exact v2 schema and immutable Choregraph source through a separate downstream change.

## Rollback

Rollback uses a reviewed VCS revision. Choregraph does not retain a hidden runtime fallback after cutover.

## Claim boundary

Parity validates selected behavior under reviewed fixtures. It does not prove Lattice, Seaglass, external effects, persistence, or release correctness.
