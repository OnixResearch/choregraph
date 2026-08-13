# Foundation architecture

Choregraph uses one pure machine core and one-way adapter dependencies.

## Workspace direction

```text
choregraph-core
  <- choregraph-kamacite
  <- choregraph-nickel
  <- choregraph-seaglass

choregraph-kamacite
  <- choregraph-choreography

choregraph-history
  <- choregraph-history-machine -> choregraph-core
  <- choregraph-weave

choregraph-release

choregraph-core + all adapters + choregraph-release
  <- choregraph-cli
```

`choregraph-foundation-check` is a private repository tool. Product crates do not depend on this tool.

## Core boundary

`choregraph-core` uses `no_std` plus `alloc`. It owns immutable machine data, admission, analytics, replay, and deterministic decisions.

V1 and v2 machines use one admitted machine type. Both transition wrappers call the same production indexed kernel.

The core excludes:

- filesystem and environment access;
- clocks, processes, and networks;
- Nickel evaluation and token verification;
- persistence and receipt issuance;
- effect execution and runtime authorization.

The foundation checker rejects forbidden dependencies, source tokens, path dependencies, reverse edges, license drift, and unpinned package metadata.

`choregraph-history` uses `no_std` plus `alloc`. It owns immutable event DAGs, explicit lineages, branch plans, snapshots, cursors, and merge-envelope shape.

`choregraph-history-machine` depends on history and the machine core. It delegates only to production replay entrypoints.

`choregraph-weave` depends on history and the exact Universal Weave cohort. Its mutable topology remains derived and metadata-only.

These crates do not own persistence, application merge meaning, scheduling, effects, rendering, transport, authorization, or consumer evidence.

## Adapter boundaries

`choregraph-choreography` owns pure DTO admission, Trellis lowering, complete endpoint projection, and projection-receipt construction. It has no dependency on the machine core. It keeps dynamic role choices separate from execution-graph machine IR.

The crate calls pinned Trellis production functions for projectability and endpoint projection. It does not own a second projection algorithm.

`choregraph-kamacite` owns pure in-memory conversion to canonical artifacts and receipt vocabulary. It uses a hash-checked, selected Kamacite codec snapshot.

The adapter encodes distinct v1 and v2 machine constructors. It also encodes v2 concrete intents and exact linkage receipts.

It does not read files, run processes, use clocks, access networks, authorize effects, or create behavioral proofs.

`choregraph-cli write-kamacite-corpus` is the imperative shell that writes maintained fixtures. The adapter returns only in-memory values.

`choregraph-nickel` accepts only evaluated data. Source discovery, imports, contracts, and Nickel evaluation remain in a shell.

`choregraph-seaglass` is a conversion-only compatibility boundary. It delegates admission, graph queries, analytics, lifecycle, readiness, resume, aggregate steps, and cycle probes to `choregraph-core`.

The v2 path converts concrete intents only. It does not add run, policy, authority, persistence, attempt, or dispatch facts.

The exact execution-graph implementation is compiled only as a conformance-test oracle. It is not a production dependency.

`choregraph-release` owns pure release roles, normalized manifests, linkage verification, Valence projections, and local publication-candidate validation. It has no file, process, network, clock, signing, push, or publication authority.

`choregraph-cli` owns process arguments and terminal output. Later shell behavior must remain separate from the pure core.

## Current claim

Foundation validation covers repository structure and declared boundaries. It does not prove machine, compiler, proof, runtime, consumer, or release correctness.
