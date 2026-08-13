# Branchable event history

Choregraph provides branchable event history through three separate crates. These crates do not change the production machine transition relation.

## Crates

`choregraph-history` owns immutable event DAGs, branch plans, projections, snapshots, cursors, and merge envelopes. It uses `no_std` plus `alloc`.

`choregraph-history-machine` converts an explicit linear lineage into existing production replay inputs. It calls `replay` or `replay_concrete` directly.

`choregraph-weave` wraps `universal-weave = 0.4.1` with default features disabled. It provides bounded, metadata-only, mutable topology.

The dependency direction is:

```text
choregraph-core        choregraph-history
        \                 /
         choregraph-history-machine

choregraph-history
        |
        v
choregraph-weave
```

No dependency returns from a history crate into `choregraph-core`.

## Event admission

An event identity binds these values:

- the history event version
- the event class
- the payload identity
- the payload schema identity
- the optional projection identity
- every ordered parent identity and role
- every safe metadata key and value identity

The graph stores identities, not payload bytes. A consumer owns payload storage, disclosure, truth, and authenticity.

Admission returns a new graph value. A denial does not change the input graph.

## Branch references

A `BranchRef` contains a bounded name, an immutable head, and a generation. `plan_branch_advance` returns a compare-and-set plan.

The function rejects a stale generation, stale head, missing head, unchanged head, or generation overflow. It does not persist the plan.

## Projections

`ancestor_closure` returns bounded parent-before-child graph facts. It does not select application history.

`linear_lineage` requires an explicit choice at each multi-parent event. It returns `AmbiguousLineage` when a choice is absent.

A snapshot binds the graph, branch, generation, policy, projection, head, and event count. A cursor also binds its position.

## Merge envelopes

A merge envelope requires at least two distinct parents. It also requires an application-supplied resolution identity and merge-policy identity.

Choregraph validates the envelope shape and identity. It does not compute or approve application merge meaning.

## Machine replay

The machine adapter validates all history bindings before replay. It rejects non-linear projections, merges, unsupported classes, stale identities, and payload mismatches.

Accepted aggregate events call `choregraph_core::replay`. Accepted concrete events call `choregraph_core::replay_concrete`.

Replay output contains no effect intents or dispatch authority.

## Universal Weave boundary

The weave adapter supports generic canonical keys. Operations use explicit generations and return a new adapter value plus a receipt.

Each operation first changes a cloned weave. A denied operation leaves the original adapter unchanged.

Snapshots contain keys, edge counts, flags, and content identities. They contain no payload or node text.

Immutable history remains authoritative. Weave topology cannot create, persist, authorize, or replace history events.

## Consumer duties

Animus retains session meaning, model context, summaries, compaction, repair, provider behavior, and persistence.

Lattice retains workflow decoding, run records, redb storage, retries, schedules, approvals, credentials, effects, diagnostics, UI, and evidence.

Hosts retain branch-plan persistence and application merge decisions. Bounded Exec retains child-process mechanics.

## Source provenance and migration

The generalized source baseline is Animus revision `48baf2fc92cad29d3f929ecf0c2e5a91b5be51bd`. The source license is `MIT OR Apache-2.0`.

The weave cohort is crates.io `universal-weave` version `0.4.1`. Its registry checksum is `73d3e16ddf00e9b597111a05c2c7e0738646fac49f98784da64f761fe8879eeb`.

Choregraph uses new binary-framed BLAKE3 domains. It does not preserve Animus text-framed receipt identities.

Consumers must map payloads to opaque identities. Consumers must also choose each multi-parent lineage explicitly.

## Weft supersession

The reviewed Weft planning revision is `8a89ffa9253f3d2a6d34ce635a0df95526ceab30`.

Choregraph adopts the immutable event DAG, branch, cursor, snapshot, and explicit merge requirements. It rejects the proposed second transition and host runtime.

The Weft `add-iroh-workspace-sync` plan remains planning provenance. Choregraph does not import its Iroh host or synchronization runtime.

No Weft source or sibling path is a product dependency.

## Non-claims

This work does not prove application merge correctness, event truth, payload authenticity, persistence durability, or distributed convergence.

It does not prove effect success, authorization, consumer correctness, upstream weave correctness, production readiness, or release eligibility.
