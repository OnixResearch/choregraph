## Context

Choregraph has one proof-linked production transition relation and bounded replay over admitted machine traces. Lattice consumes those decisions and retains runtime authority.

Animus has a separate append-only session ledger and a bounded Universal Weave adapter. Its ledger selects an active branch for model-visible replay. The weave supplies mutable navigation and editing, but it is not durable replay authority.

The Weft planning repository proposed a second deterministic interaction runtime. That plan overlaps Choregraph graph, replay, identity, and effect boundaries.

## Success Contract

The accepted result is a generic immutable history mechanism inside the Choregraph workspace. It does not change the execution-machine transition relation.

Completion evidence includes exact source baselines, positive and negative graph fixtures, unchanged-state rejection tests, machine-replay delegation checks, and disposable Animus and Lattice boundary fixtures.

A bounded blocker is acceptable. Implementation must stop if the history mechanism requires product persistence, automatic semantic merge, or another machine transition relation.

## Approach Registry

| Family | Mechanism | State |
| --- | --- | --- |
| Choregraph history crates | Add isolated history, machine-adapter, and weave crates | selected |
| Extend `choregraph-core` directly | Put branch refs and arbitrary application events inside the verified machine kernel | rejected |
| Weft runtime | Create another graph, replay, effect, view, and host runtime | rejected |
| Lattice product feature | Put reusable session history behind Lattice persistence and runtime | rejected |
| Consumer-local duplication | Keep separate Animus, Lattice, and Molten graph implementations | insufficient |

The selected family reuses Choregraph ownership without widening the verified kernel or Lattice authority.

## Decisions

### Decision: isolate generic history from machine semantics

**Choice:** Add `choregraph-history` as a separate `no_std + alloc` crate.

The crate owns generic event envelopes, canonical identities, graph admission, branch references, cursors, lineage plans, snapshots, and merge envelopes. Payloads remain opaque identities plus bounded safe metadata.

`choregraph-core` will not depend on this crate. The history crate will not depend on `choregraph-core`.

**Rationale:** Animus can consume history without importing execution-machine types. The verified machine kernel gains no arbitrary session semantics.

### Decision: keep machine integration in a one-way adapter

**Choice:** Add `choregraph-history-machine`, which depends on `choregraph-history` and `choregraph-core`.

The adapter admits only an explicit linear lineage of supported machine events. It converts that lineage to the existing bounded trace input and calls the current production replay entrypoint.

A non-linear lineage, unresolved merge, stale machine identity, unsupported event class, or incompatible schema returns a typed denial. The adapter contains no transition table.

**Rationale:** History selection and machine evaluation remain separate. Replay continues to use one production implementation.

### Decision: identify immutable events with canonical BLAKE3 projections

**Choice:** Each event identity binds a domain and schema version, payload identity, ordered parent roles, event class, reducer or projection identity when applicable, and bounded safe metadata identity.

Events are immutable after admission. Duplicate identities with unequal canonical content fail closed.

Large or sensitive payload bytes remain outside the event envelope. Consumers retain storage and disclosure policy.

**Rationale:** Exact event identity supports review, branch comparison, and deterministic projection without making metadata an authenticity claim.

### Decision: separate immutable nodes from mutable branch references

**Choice:** Branch names point to immutable heads through generation-fenced compare-and-set plans.

The pure crate computes a reference mutation plan from the current generation, expected head, proposed head, and policy. A host can persist or reject that plan.

A stale generation or head mismatch leaves graph and reference state unchanged.

**Rationale:** Mutable navigation must not rewrite accepted history. Persistence remains consumer-owned.

### Decision: make lineage choice explicit

**Choice:** The history crate supports two bounded projections.

- An ancestor-closure projection returns deterministic graph facts for comparison and validation.
- A linear-lineage projection requires an explicit parent choice at each multi-parent event.

Canonical parent ordering cannot silently choose application history. An unresolved multi-parent event returns a typed non-linear result.

**Rationale:** Deterministic sorting is not semantic branch selection.

### Decision: require application-supplied merge resolution

**Choice:** A merge envelope binds every parent head, explicit parent roles, the application resolution identity, schema identity, and merge-policy identity.

The generic crate validates shape, bounds, parents, freshness, and identity. It does not compute message, model, workflow, text, or CRDT meaning.

The first version provides no automatic semantic merge.

**Rationale:** Choregraph can validate a supplied resolution without claiming that the resolution is correct.

### Decision: keep Universal Weave derived and optional

**Choice:** Add `choregraph-weave` as an optional adapter over an exact reviewed Universal Weave revision.

The adapter uses generic ordered node keys, named bounds, pure preflight, unchanged-state rejection, and metadata-only topology snapshots.

The adapter can project topology from admitted history. It cannot create, replace, persist, or authorize history events.

**Rationale:** Mutable branch navigation is useful, but replay depends on immutable ancestors.

### Decision: preserve consumer authority

**Choice:** Animus retains session entry meaning, active model context, summaries, compaction, repair, provider behavior, and session persistence.

Lattice retains workflow decoding, machine and run identity, redb records, retries, schedules, triggers, approvals, credentials, effects, diagnostics, UI, and evidence.

Bounded Exec retains child-process mechanics. Basalt and UCAN retain their bounded authorization roles.

**Rationale:** Generic graph reuse does not transfer product or host authority.

### Decision: supersede the Weft plans without importing a runtime

**Choice:** Record the exact Weft planning revision and both superseded change packages as design provenance.

Do not import Weft runtime, view, host-port, or Iroh synchronization source. No Weft sibling path becomes a dependency.

**Rationale:** The planning work explains requirements and rejected scope. Choregraph remains the implementation owner.

## Dependency Direction

```text
choregraph-core        choregraph-history
        \                 /
         choregraph-history-machine

choregraph-history
        |
        v
choregraph-weave
```

No edge returns from a history crate into `choregraph-core`.

## Validation Strategy

Positive fixtures cover linear history, forks, explicit parent selection, ancestor closure, branch movement, snapshot binding, supplied merge resolution, weave projection, and delegated machine replay.

Negative fixtures cover missing parents, cycles, duplicate conflicts, stale generations, stale heads, unsupported versions, oversized values, unresolved merges, ambiguous lineage, weave overflow, unsupported machine events, and attempted live dispatch.

Property tests cover identity stability, accepted graph acyclicity, unchanged-state rejection, deterministic projection, and bounded traversal.

Architecture checks reject filesystem, environment, clock, process, network, persistence, renderer, Lattice runtime, Animus runtime, Tile, and Rats/Subwayrat dependencies from pure history crates.

The consumer fixtures prove only API and authority separation. They do not prove downstream correctness or adoption.

## Adversarial Audit

The audit must try to show that:

- history adds another transition relation.
- machine replay bypasses the production replay entrypoint.
- parent sorting silently selects a semantic branch.
- a merge envelope claims resolution correctness.
- Universal Weave mutation changes authoritative history.
- stale reference state mutates a branch head.
- a consumer fixture imports persistence or runtime authority.
- Weft becomes a source or sibling-path dependency.

Any successful counterexample blocks closeout.

## Risks and Trade-offs

- Generic history types can become too broad. Opaque payload identities and small envelopes limit the first API.
- Multi-parent history is not naturally linear. Explicit parent selection can require consumer work.
- Universal Weave can reject or panic inside its own contract boundary. Choregraph does not claim upstream correctness.
- Separate crates add package and release work. They prevent product semantics from entering the verified machine core.

## Non-Claims

This design does not prove application merge correctness, event truth, payload authenticity, persistence durability, distributed convergence, effect success, exactly-once execution, rendering equivalence, transport availability, authorization, consumer correctness, production readiness, or release eligibility.
