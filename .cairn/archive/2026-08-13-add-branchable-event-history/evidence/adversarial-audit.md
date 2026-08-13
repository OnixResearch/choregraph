# Adversarial audit

## Goal

The goal was to falsify the branchable-history completion claim before closeout.

Completion requires one immutable history mechanism, explicit branch choices, production replay delegation, derived weave topology, and no consumer authority.

False completion includes a second transition relation, implicit merge, authoritative weave state, stale branch mutation, product authority, Weft dependencies, or unsupported claims.

## Search budget

The audit used three serial lenses. These lenses are correlated because one agent performed all reviews.

The audit used repository source, tests, foundation validation, source provenance, Cargo metadata, and lifecycle artifacts. It used no external runtime authority.

## Lens A: graph and identity invariants

The audit reviewed event admission, parent order, duplicate handling, branch plans, projections, snapshots, cursors, and merge envelopes.

The audit found one direct-merge bypass. A caller could construct `EventDraft` with `EventClass::Merge` without a merge-policy identity.

The implementation now rejects direct merge drafts without at least two distinct parents and a merge-policy identity. A negative test covers this path.

The audit also found that public `Projection` fields permitted forged projection values. The fields are now private and available through read-only accessors.

## Lens B: replay and authority boundaries

The audit reviewed machine-event payload bindings and every call from history into the machine core.

The adapter calls only `choregraph_core::replay` and `choregraph_core::replay_concrete`. It contains no transition table or dispatch field.

The foundation checker rejects calls to production step entrypoints from the history adapter. It also rejects runtime and host dependencies.

Machine, schema, trace, event-class, lineage, and payload mismatches fail before production replay.

## Lens C: weave mutation and consumer leakage

The audit reviewed clone-before-mutation behavior, generations, topology snapshots, source-history binding, and consumer fixtures.

The audit found that a no-op move could advance the generation because the comparison included the successor generation.

The adapter now compares topology before it advances the generation. A same-parent move returns `NoChange` and preserves the original identity.

The weave stores content identities only. Snapshots expose no content bytes or consumer product fields.

The dependency gate rejects Animus runtime, Lattice runtime, Weft, Bounded Exec, redb, network, process, clock, Tile, and Rats/Subwayrat authority.

## Result

No surviving counterexample adds a second transition relation, implicit merge, authoritative weave history, stale branch mutation, or product authority.

The audit establishes bounded source and test evidence only. It does not prove compiler correctness, upstream weave correctness, consumer correctness, or release eligibility.
