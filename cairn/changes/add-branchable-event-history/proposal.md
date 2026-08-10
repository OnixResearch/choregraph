## Why

Choregraph already owns deterministic graph admission, pure transitions, bounded replay, effect intents, and canonical machine identities.

Animus also has a branchable session ledger and a Universal Weave adapter. The abandoned Weft plan proposed another product-neutral graph and replay runtime around those mechanisms.

A second runtime would duplicate Choregraph semantics and create unclear ownership with Lattice. Choregraph can own the generic history mechanism while consumers retain session meaning, persistence, effects, and operator behavior.

## What Changes

- Add `choregraph-history` as a pure `no_std + alloc` crate for immutable event DAGs, branch references, cursors, bounded lineage selection, snapshots, and explicit merge envelopes. r[choregraph.history.boundary] r[choregraph.history.event_dag] r[choregraph.history.branch_refs] r[choregraph.history.lineage] r[choregraph.history.merge]
- Add `choregraph-history-machine` as a pure adapter from admitted linear machine-event lineages to the existing production `replay` entrypoints. r[choregraph.history.machine_replay]
- Add `choregraph-weave` as an optional bounded Universal Weave topology adapter. The immutable history graph remains authoritative. r[choregraph.history.weave]
- Generalize the reviewed Animus session-ledger and weave fixtures behind opaque application payload identities and Choregraph-owned bounds. r[choregraph.history.source_adoption]
- Add disposable Animus and Lattice consumer fixtures without moving their product authority into Choregraph. r[choregraph.history.consumers]
- Record that the Weft runtime and Iroh synchronization plans are superseded. No Weft runtime implementation becomes a dependency. r[choregraph.history.supersession]
- Keep persistence, retries, scheduling, authorization, rendering, transport, child processes, and application merge meaning outside Choregraph. r[choregraph.history.boundary] r[choregraph.history.consumers]

## Success Criteria

- One generic history crate admits immutable BLAKE3 event DAGs without importing machine, runtime, storage, UI, or network authority.
- One separate machine adapter delegates eligible linear traces to existing Choregraph replay entrypoints.
- Universal Weave remains optional mutable topology and cannot replace immutable history.
- Animus can map session entries without importing Lattice runtime behavior.
- Lattice can reference branchable history without replacing Choregraph machine state or redb run records.
- Positive and negative tests cover graph admission, refs, cursors, lineage, merge envelopes, weave bounds, delegation, and consumer boundaries.

## False Completion

A second transition relation, scheduler, effect runtime, persistence layer, session format, or workflow runner is not completion.

Calling canonical parent sorting a semantic merge is not completion. Treating Universal Weave state as authoritative replay history is not completion.

A graph library without Animus and Lattice boundary fixtures is not sufficient extraction evidence.

## Impact

- **Workspace**: add `choregraph-history`, `choregraph-history-machine`, and `choregraph-weave`.
- **Core**: keep the production transition relation and proof-linked replay implementation unchanged.
- **Consumers**: later Animus and Lattice changes can adopt an exact published Choregraph revision.
- **Weft**: retain planning provenance only. Do not implement its proposed runtime or synchronization host.
- **Testing**: add positive, negative, property, no-std, Wasm, source-parity, dependency, Cairn, Octet, and Nix checks.

## Out of Scope

- Animus message types, model context, compaction, interrupted-tool repair, or persistence.
- Lattice run persistence, retries, schedules, triggers, approvals, credentials, or effect execution.
- Bounded Exec process mechanics.
- Semantic view frameworks, rendering, Tile workspaces, or Rats/Subwayrat widgets.
- Iroh synchronization, distributed branch convergence, CRDT correctness, or transport authority.
- Automatic application merge decisions.

## Affected Specs

- `branchable-event-history`: immutable history, branch references, lineage, machine replay delegation, merge envelopes, weave topology, consumer boundaries, and supersession.
