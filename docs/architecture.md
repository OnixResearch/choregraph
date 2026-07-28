# Foundation architecture

Choregraph uses one pure machine core and one-way adapter dependencies.

## Workspace direction

```text
choregraph-core
  <- choregraph-kamacite
  <- choregraph-nickel
  <- choregraph-seaglass

choregraph-release

choregraph-core + all adapters + choregraph-release
  <- choregraph-cli
```

`choregraph-foundation-check` is a private repository tool. Product crates do not depend on this tool.

## Core boundary

`choregraph-core` uses `no_std` plus `alloc`. It owns immutable machine data, admission, analytics, replay, and deterministic decisions.

The core excludes:

- filesystem and environment access;
- clocks, processes, and networks;
- Nickel evaluation and token verification;
- persistence and receipt issuance;
- effect execution and runtime authorization.

The foundation checker rejects forbidden dependencies, source tokens, path dependencies, reverse edges, license drift, and unpinned package metadata.

## Adapter boundaries

`choregraph-kamacite` owns pure in-memory conversion to canonical artifacts and receipt vocabulary. It uses a hash-checked, selected Kamacite codec snapshot. It does not read files, run processes, use clocks, access networks, authorize effects, or create behavioral proofs.

`choregraph-cli write-kamacite-corpus` is the imperative shell that writes maintained fixtures. The adapter returns only in-memory values.

`choregraph-nickel` accepts only evaluated data. Source discovery, imports, contracts, and Nickel evaluation remain in a shell.

`choregraph-seaglass` is a conversion-only compatibility boundary. It delegates admission, graph queries, analytics, lifecycle, readiness, resume, aggregate steps, and cycle probes to `choregraph-core`.

The exact execution-graph implementation is compiled only as a conformance-test oracle. It is not a production dependency.

`choregraph-release` owns pure release roles, normalized manifests, linkage verification, Valence projections, and local publication-candidate validation. It has no file, process, network, clock, signing, push, or publication authority.

`choregraph-cli` owns process arguments and terminal output. Later shell behavior must remain separate from the pure core.

## Current claim

Foundation validation covers repository structure and declared boundaries. It does not prove machine, compiler, proof, runtime, consumer, or release correctness.
