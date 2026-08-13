# Branchable Event History Specification

## Purpose

Defines the `branchable-event-history` capability.

## Requirements

### Requirement: History has a separate pure boundary

r[choregraph.history.boundary] Choregraph MUST provide branchable event history through separate `no_std + alloc` crates that do not add application meaning, persistence, runtime effects, authorization, transport, clocks, rendering, or a second machine transition relation.

#### Scenario: Pure history receives explicit data

r[choregraph.history.boundary.pure]
- GIVEN admitted event envelopes, graph state, reference state, policy, and an explicit operation
- WHEN a history core computes admission, lineage, merge validation, or a reference plan
- THEN it MUST use only supplied values
- AND it MUST NOT access ambient host services.

#### Scenario: History gains product or host authority

r[choregraph.history.boundary.rejects_authority]
- GIVEN a pure history crate imports filesystem, process, socket, clock, renderer, persistence, Lattice runtime, Animus runtime, Tile, or Rats/Subwayrat authority
- WHEN architecture validation runs
- THEN validation MUST fail
- AND the candidate MUST NOT satisfy closeout.

### Requirement: Events form an immutable canonical DAG

r[choregraph.history.event_dag] Choregraph MUST identify each immutable history event with a domain-separated BLAKE3 projection that binds its version, event class, payload identity, ordered parent roles, schema identity, and bounded safe metadata identity.

#### Scenario: Valid event extends admitted history

r[choregraph.history.event_dag.append]
- GIVEN every parent exists, the projection is canonical, and every selected bound passes
- WHEN event admission runs
- THEN it MUST compute the expected identity and return an extended immutable graph
- AND every previously admitted event MUST remain unchanged.

#### Scenario: Event graph is malformed

r[choregraph.history.event_dag.rejects]
- GIVEN an event has a missing parent, cycle, conflicting duplicate identity, unsupported version, invalid canonical projection, excessive fan-in, or excessive payload metadata
- WHEN event admission runs
- THEN it MUST return a typed denial
- AND the accepted graph MUST remain unchanged.

### Requirement: Branch references are generation-fenced

r[choregraph.history.branch_refs] Choregraph MUST keep mutable branch references separate from immutable events and MUST plan reference changes against an exact generation and expected head.

#### Scenario: Current branch moves to an admitted head

r[choregraph.history.branch_refs.advance]
- GIVEN the branch generation and expected head match current state and the proposed head exists
- WHEN reference planning runs
- THEN it MUST return one deterministic mutation plan with the successor generation
- AND it MUST NOT perform persistence.

#### Scenario: Branch reference is stale

r[choregraph.history.branch_refs.stale]
- GIVEN the supplied generation or expected head differs from current state
- WHEN reference planning runs
- THEN it MUST deny the change before mutation
- AND the existing branch reference MUST remain current.

### Requirement: Lineage selection preserves branch ambiguity

r[choregraph.history.lineage] Choregraph MUST provide bounded ancestor-closure and linear-lineage projections without treating canonical parent ordering as semantic branch selection.

#### Scenario: Explicit parent choices select one lineage

r[choregraph.history.lineage.linear]
- GIVEN a selected head and one valid explicit parent choice for each multi-parent event
- WHEN linear-lineage projection runs
- THEN it MUST return one deterministic root-to-head lineage within the selected bound
- AND every returned event MUST belong to that explicit lineage.

#### Scenario: Multi-parent lineage lacks a choice

r[choregraph.history.lineage.ambiguous]
- GIVEN a selected path reaches a multi-parent event without an explicit parent choice
- WHEN linear-lineage projection runs
- THEN it MUST return a typed non-linear outcome
- AND it MUST NOT select a parent by sort order.

### Requirement: Machine replay delegates to the production kernel

r[choregraph.history.machine_replay] Choregraph MUST adapt eligible linear machine-event lineages to existing bounded replay entrypoints without adding another transition implementation or dispatch authority.

#### Scenario: Eligible lineage replays through production code

r[choregraph.history.machine_replay.valid]
- GIVEN an admitted linear lineage has matching machine, schema, binding, and event identities
- WHEN the machine-history adapter evaluates it
- THEN it MUST construct the existing bounded replay input and call the production replay entrypoint
- AND the replay result MUST contain no dispatch authority.

#### Scenario: Lineage cannot become a machine trace

r[choregraph.history.machine_replay.rejects]
- GIVEN a lineage is non-linear, has an unresolved merge, uses unsupported event classes, or has stale machine, schema, or binding identity
- WHEN machine-trace admission runs
- THEN it MUST return a typed denial before production replay
- AND it MUST NOT reinterpret the lineage through another transition table.

### Requirement: Merge envelopes require supplied resolution

r[choregraph.history.merge] Choregraph MUST validate merge envelopes that bind every parent head, explicit parent roles, one application-supplied resolution identity, schema identity, and merge-policy identity.

#### Scenario: Supplied merge resolution is structurally valid

r[choregraph.history.merge.valid]
- GIVEN every parent exists and the supplied resolution, schema, policy, roles, and bounds are valid
- WHEN merge admission runs
- THEN it MUST admit one immutable merge event
- AND the event identity MUST bind every parent and the supplied resolution.

#### Scenario: Automatic semantic merge is requested

r[choregraph.history.merge.rejects_implicit]
- GIVEN concurrent histories have no supplied application resolution
- WHEN merge admission runs
- THEN it MUST preserve separate heads or request a resolution
- AND it MUST NOT claim that sorting, concatenation, or topology convergence resolves application meaning.

### Requirement: Universal Weave remains derived topology

r[choregraph.history.weave] Choregraph MUST keep Universal Weave behind an optional bounded adapter whose mutable topology cannot create, replace, persist, or authorize immutable history.

#### Scenario: Bounded topology operation succeeds

r[choregraph.history.weave.valid]
- GIVEN an admitted history projection, current topology generation, valid node keys, and an operation within named bounds
- WHEN the weave adapter applies the operation
- THEN it MUST advance derived topology deterministically
- AND it MUST return a metadata-only snapshot without changing history events.

#### Scenario: Topology operation is invalid

r[choregraph.history.weave.rejects]
- GIVEN preflight detects stale generation, invalid parentage, cycle risk, missing node, overflow, or policy excess
- WHEN operation admission runs
- THEN it MUST reject before upstream mutation
- AND the prior topology identity MUST remain unchanged.

### Requirement: Consumer authority remains outside history

r[choregraph.history.consumers] Choregraph MUST provide consumer adapters and fixtures without owning Animus session meaning or Lattice runtime, persistence, retry, scheduling, approval, effect, diagnostic, UI, or evidence authority.

#### Scenario: Animus maps a session lineage

r[choregraph.history.consumers.animus]
- GIVEN an Animus adapter supplies admitted opaque session-event identities and explicit branch choices
- WHEN Choregraph history projects the selected lineage
- THEN it MUST return structural history facts only
- AND Animus MUST retain model-context, summary, compaction, repair, provider, and persistence decisions.

#### Scenario: Lattice references branchable history

r[choregraph.history.consumers.lattice]
- GIVEN a Lattice adapter binds workflow, machine, run, and history identities
- WHEN it requests history projection or machine replay
- THEN Choregraph MUST return pure graph or replay decisions
- AND Lattice MUST retain redb records, authorization, attempts, retries, schedules, effects, diagnostics, UI, and evidence.

### Requirement: Weft plans are superseded by Choregraph ownership

r[choregraph.history.supersession] Choregraph MUST record the exact Weft planning revision and superseded change packages without importing a Weft runtime, view framework, host shell, synchronization host, or sibling-path dependency.

#### Scenario: Weft planning informs the change

r[choregraph.history.supersession.provenance]
- GIVEN the Weft proposal, design, specifications, and tasks are reviewed
- WHEN source provenance is recorded
- THEN Choregraph MUST identify the exact revision and adopted or rejected requirements
- AND the records MUST describe Weft as planning provenance only.

#### Scenario: Weft implementation enters the dependency graph

r[choregraph.history.supersession.rejects_dependency]
- GIVEN a candidate adds a Weft source, runtime, view, host-port, Iroh, or sibling-path dependency
- WHEN dependency validation runs
- THEN validation MUST fail
- AND Choregraph MUST remain the implementation owner.

### Requirement: Source adoption preserves behavior and provenance

r[choregraph.history.source_adoption] Choregraph MUST record exact source revisions, paths, licenses, BLAKE3 domains, public behavior, and positive and negative fixtures for generalized Animus ledger and weave mechanisms.

#### Scenario: Generalized behavior has parity

r[choregraph.history.source_adoption.parity]
- GIVEN recorded source fixtures and the new Choregraph implementation
- WHEN parity validation runs
- THEN equivalent supported operations MUST return equivalent structural decisions and compatible identities where required
- AND intentional differences MUST use explicit versioned migration.

#### Scenario: Product semantics leak into the generic crate

r[choregraph.history.source_adoption.rejects_product]
- GIVEN generalized code contains model roles, prompts, tool-call meaning, workflow retry policy, redb behavior, or renderer policy
- WHEN source-boundary validation runs
- THEN validation MUST fail
- AND those semantics MUST remain with the consumer.

### Requirement: Verification covers positive and negative paths

r[choregraph.history.verification] Choregraph MUST include positive and negative tests for event DAGs, references, cursors, lineage, machine delegation, merge envelopes, weave bounds, source parity, consumer boundaries, no-std, Wasm, determinism, and claim hygiene.

#### Scenario: Closeout evidence is complete

r[choregraph.history.verification.complete]
- GIVEN maintainers intend to archive the history change
- WHEN closeout validation runs
- THEN focused tests, property tests, repeated-process tests, dependency checks, no-std and Wasm checks, Cairn gates, Octet, and relevant Nix checks MUST pass
- AND every blocked check MUST record an exact blocker and next validation.
