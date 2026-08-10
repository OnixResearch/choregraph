## Phase 1: Baselines and ownership

- [ ] [serial] B1 Record the current Choregraph machine, replay, proof, adapter, Octet, Cairn, and Nix baselines before core work. r[choregraph.history.boundary] r[choregraph.history.machine_replay] r[choregraph.history.verification]
- [ ] [serial] B2 Record exact Animus ledger and Universal Weave adapter revisions, paths, licenses, BLAKE3 domains, APIs, and positive and negative fixtures. r[choregraph.history.source_adoption]
- [ ] [serial] B3 Record the exact Weft planning revision, both superseded changes, adopted requirements, rejected runtime scope, and no-dependency decision. r[choregraph.history.supersession]
- [ ] [serial] B4 Record the current Lattice authority boundary, redb ownership, run identity, effect handoff, and Bounded Exec boundary. r[choregraph.history.consumers]
- [ ] [serial] B5 Add workspace manifests for `choregraph-history`, `choregraph-history-machine`, and `choregraph-weave` with one-way dependency checks. r[choregraph.history.boundary]

## Phase 2: Immutable history core

- [ ] [serial] H1 Add canonical history versions, event classes, payload identities, parent roles, schema identities, safe metadata identities, named bounds, and typed errors. r[choregraph.history.event_dag]
- [ ] [depends:H1] H2 Implement immutable event admission with domain-separated BLAKE3 identities, parent validation, cycle rejection, duplicate conflict rejection, and unchanged-state denial. r[choregraph.history.event_dag]
- [ ] [depends:H2] H3 Add generation-fenced branch references and pure compare-and-set mutation plans over admitted heads. r[choregraph.history.branch_refs]
- [ ] [depends:H2] H4 Add bounded ancestor-closure projection and explicit linear-lineage projection with typed ambiguity. r[choregraph.history.lineage]
- [ ] [depends:H2] H5 Add explicit merge envelopes that bind all parents, roles, supplied resolution, schema, and merge-policy identity. r[choregraph.history.merge]
- [ ] [depends:H2] H6 Add derived snapshots and cursors bound to graph, branch, generation, policy, projection, and selected head identities. r[choregraph.history.branch_refs] r[choregraph.history.lineage]

## Phase 3: Machine and weave adapters

- [ ] [depends:H4] M1 Add `choregraph-history-machine` admission for supported linear machine-event lineages. r[choregraph.history.machine_replay]
- [ ] [depends:M1] M2 Delegate admitted lineages to existing `replay` and `replay_concrete` entrypoints without another transition table or dispatch field. r[choregraph.history.machine_replay]
- [ ] [depends:H2] W1 Add `choregraph-weave` over an exact reviewed Universal Weave revision with generic keys and named bounds. r[choregraph.history.weave] r[choregraph.history.source_adoption]
- [ ] [depends:W1] W2 Add pure preflight, unchanged-state rejection, deterministic metadata-only topology snapshots, and authoritative-history separation. r[choregraph.history.weave]
- [ ] [parallel] W3 Add source parity fixtures for supported Animus ledger and weave operations, with explicit migrations for intentional identity changes. r[choregraph.history.source_adoption]

## Phase 4: Consumer boundaries

- [ ] [depends:H4] C1 Add a disposable Animus fixture that maps opaque session events and explicit branch choices without model, provider, persistence, compaction, or repair authority. r[choregraph.history.consumers.animus]
- [ ] [depends:M2] C2 Add a disposable Lattice fixture that binds workflow, machine, run, and history identities without redb, retry, schedule, approval, credential, effect, diagnostic, UI, or evidence authority. r[choregraph.history.consumers.lattice]
- [ ] [parallel] C3 Add dependency failures for Weft runtime, sibling paths, Animus runtime, Lattice runtime, Bounded Exec, files, processes, networks, clocks, persistence, rendering, Tile, and Rats/Subwayrat in pure history crates. r[choregraph.history.boundary] r[choregraph.history.supersession]

## Phase 5: Positive and negative validation

- [ ] [parallel] T1 Add positive linear, fork, ancestor-closure, explicit parent-choice, branch-advance, snapshot, cursor, merge-envelope, weave, and delegated-machine-replay tests. r[choregraph.history.verification]
- [ ] [parallel] T2 Add negative missing-parent, cycle, conflicting-duplicate, stale-generation, stale-head, ambiguous-lineage, unresolved-merge, bound-overflow, weave-rejection, stale-machine, and unsupported-event tests. r[choregraph.history.verification]
- [ ] [parallel] T3 Add property tests for identity stability, accepted acyclicity, unchanged-state rejection, deterministic projection, and traversal bounds. r[choregraph.history.event_dag] r[choregraph.history.lineage] r[choregraph.history.verification]
- [ ] [parallel] T4 Add no-std, `wasm32-unknown-unknown`, repeated-process determinism, redaction, dependency, and claim-hygiene checks. r[choregraph.history.boundary] r[choregraph.history.verification]
- [ ] [serial] T5 Add a source-link check that requires machine delegation to production replay and rejects another transition implementation. r[choregraph.history.machine_replay]

## Phase 6: Documentation and closeout

- [ ] [serial] D1 Document public APIs, dependency direction, source provenance, branch and merge semantics, consumer duties, migration, and all non-claims. r[choregraph.history.boundary] r[choregraph.history.source_adoption] r[choregraph.history.consumers]
- [ ] [serial] D2 Record that Choregraph supersedes Weft runtime ownership while Weft remains planning provenance only. r[choregraph.history.supersession]
- [ ] [serial] X1 Run formatting, workspace tests, Clippy, foundation checks, production replay tests, property tests, no-std and Wasm checks, source parity, consumer fixtures, Octet, Cairn validation and gates, Tracey, and `nix flake check -L`. r[choregraph.history.verification]
- [ ] [serial] X2 Review adversarial cases and block closeout for duplicate transitions, implicit merge, authoritative weave state, stale branch mutation, product-authority leakage, Weft dependencies, or unsupported claims. r[choregraph.history.verification]
- [ ] [serial] X3 Review dry-run sync and archive plans, make sure that every requirement reaches `branchable-event-history`, and archive only after current evidence passes. r[choregraph.history.verification]
