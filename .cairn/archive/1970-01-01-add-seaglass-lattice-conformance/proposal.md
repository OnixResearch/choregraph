## Why

Lattice and Seaglass already depend on the published execution-graph behavior. Choregraph must prove compatibility against that real consumer surface before becoming the sole transition implementation.

The first integration must exercise Lattice-authored Nickel, canonical lowering, machine admission, readiness, lifecycle, replay, and effect-intent handoff without moving runtime or product authority into Choregraph.

## What Changes

- Add a Seaglass compatibility adapter over Choregraph production types and decisions.
- Dual-run every inventoried execution-graph behavior against the immutable baseline.
- Use the Lattice diamond graph as the first end-to-end positive fixture.
- Add malformed graph, invalid lifecycle, replay, source, receipt, and effect-intent negative fixtures.
- Add a disposable downstream Lattice integration check against an exact reviewed revision.
- Define cutover and rollback evidence for Choregraph, execution-graph, Seaglass, and Lattice.

## Success Criteria

- Every selected baseline behavior is equal or has an explicit reviewed semantic migration.
- The Lattice graph reaches the expected pure states and effect intents through Choregraph.
- Lattice retains runtime, scheduler, values, policies, persistence, and evidence authority.
- Temporary dual-run and compatibility code cannot become a second production transition implementation.

## Non-Goals

- Executing Lattice effects, changing node types, or claiming whole-workflow correctness.
- Publishing or pushing consumer repositories.
- Treating parity as formal proof or release eligibility.

## Impact

- **Adapter**: `choregraph-seaglass` compatibility types and mappings.
- **Tests**: execution-graph parity matrix and Lattice downstream integration.
- **Migration**: evidence requirements for the execution-graph facade and Lattice cutover Cairns.
