# Source provenance

## Animus ledger and weave

The reviewed Animus source revision is `48baf2fc92cad29d3f929ecf0c2e5a91b5be51bd`.

The reviewed paths are:

- `crates/animus-adapters/src/ledger/`
- `crates/animus-adapter-weave/`
- `docs/pi-session-tree-ledger.md`
- `docs/universal-weave-adapter.md`

The source license is `MIT OR Apache-2.0`. OnixResearch owns this source and authorized reuse.

Choregraph did not copy Animus product types or text content. It generalized these structural behaviors:

- append-only parent-linked history
- explicit active lineage selection
- missing-parent and cycle rejection
- bounded mutable topology
- unchanged-state rejection
- metadata-only topology snapshots

The Animus ledger does not define a project-owned BLAKE3 event domain. Choregraph adds canonical immutable event identities.

The reviewed Animus weave domains are:

- `animus_adapter_weave_policy_v1`
- `animus_adapter_weave_topology_v1`
- `animus_adapter_weave_receipt_v1`

Choregraph uses these new domains:

- `choregraph.history.event.v1`
- `choregraph.history.graph.v1`
- `choregraph.history.projection.v1`
- `choregraph.history.branch.v1`
- `choregraph.history.snapshot.v1`
- `choregraph.history.cursor.v1`
- `choregraph.weave.policy.v1`
- `choregraph.weave.topology.v1`
- `choregraph.weave.receipt.v1`

These identities are an intentional versioned migration. Choregraph uses binary length framing and opaque content identities instead of Animus text framing.

Positive parity fixtures cover add, move, activate, bookmark, linear replay, forks, snapshots, and bounded topology projection.

Negative fixtures cover missing parents, cycles, stale generations, duplicate events, missing lineage choices, invalid moves, bounds, and unchanged state.

## Universal Weave

The selected upstream cohort is crates.io `universal-weave = 0.4.1` with default features disabled.

The exact crates.io checksum is `73d3e16ddf00e9b597111a05c2c7e0738646fac49f98784da64f761fe8879eeb`.

Choregraph uses `IndependentWeave` only. It does not enable the serialization, CRDT, legacy, or action-queue lanes.

The dependency license is `Unlicense`, as published in the selected crate. This record does not change the upstream license.

## Weft planning

The reviewed Weft planning revision is `8a89ffa9253f3d2a6d34ce635a0df95526ceab30`.

The superseded change packages are:

- `.cairn/changes/establish-deterministic-interaction-runtime/`
- `.cairn/changes/add-iroh-workspace-sync/`

Choregraph adopts immutable event DAGs, explicit branches, snapshots, cursors, and supplied merge events.

Choregraph rejects the second transition runtime, semantic view runtime, host ports, Iroh host, and synchronization runtime.

Weft remains planning provenance only. No Weft source, runtime, or sibling path is a dependency.

## Consumer boundaries

The reviewed Lattice source revision is `923ab0863905c0db420ba981083446705784b458`.

Lattice retains redb records, run identities, occurrence admission, retries, schedules, credentials, effects, diagnostics, UI, and evidence.

The reviewed Bounded Exec revision remains `29dac88ecded94457572db3fdfaaaab95fa91525` in Lattice.

These revisions identify reviewed source facts. They do not prove downstream adoption or correctness.
