# Execution-graph migration

Choregraph must become the only production transition implementation.

## Migration sequence

1. Preserve the immutable execution-graph behavior baseline.
2. Implement machine semantics only in `choregraph-core`.
3. Prove the production Choregraph transition path.
4. Dual-run old and new behavior in tests only. This step now passes for the selected matrix.
5. Change `execution-graph-core` into a conversion-only Choregraph facade.
6. Remove the old transition algorithm and temporary dual-run code.

`choregraph-seaglass` now provides the conversion-only adapter and test-only comparator. It preserves selected types, errors, ordering, readiness, lifecycle, resume, and analytics behavior.

The adapter rejects identifiers outside the Choregraph profile, unknown resume nodes, and invalid sparse state. These fail-closed differences are explicit migration inputs.

## Release gate

Release readiness fails while two production transition implementations remain. A feature flag or runtime fallback does not satisfy this gate.

Temporary dual-run logic must remain test-only. The migration must include a removal validation before consumer cutover.

## Rollback

Rollback uses a reviewed VCS revision. Choregraph does not retain a hidden runtime fallback after cutover.

## Claim boundary

Parity validates selected behavior under reviewed fixtures. It does not prove Lattice, Seaglass, external effects, persistence, or release correctness.
