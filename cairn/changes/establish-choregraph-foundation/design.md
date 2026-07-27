## Context

Choregraph must become the sole implementation of verified machine transitions. The existing execution-graph repository remains a published compatibility surface and source baseline. A new repository that merely copies its code would create an unverified duplicate.

## Success Contract

Completion requires a buildable workspace skeleton, explicit ownership boundaries, immutable source provenance, preserved AGPL licensing, and checks that reject forbidden dependency direction. Placeholder crates or two independent transition cores are false completion.

## Decisions

### Create one standalone workspace

The workspace contains `choregraph-core`, `choregraph-kamacite`, `choregraph-nickel`, `choregraph-seaglass`, and `choregraph-cli`. The core has no dependency on the adapter or shell crates.

### Keep the functional core strict

`choregraph-core` uses `no_std` plus `alloc`. It owns immutable machine data and deterministic decisions. It does not read files, inspect environment state, use clocks, spawn processes, access networks, evaluate Nickel, verify tokens, persist runs, or execute effects.

### Migrate to one implementation

The existing `execution-graph-core` implementation is the behavior baseline. Choregraph will absorb or replace its transition semantics with preserved provenance. After parity and proof gates pass, the execution-graph crate will become a compatibility facade over Choregraph.

No permanent dual implementation is permitted. Temporary dual-run code is test-only and must have a removal gate.

### Preserve license and provenance

Copied or adapted execution-graph and Seaglass material remains `AGPL-3.0-or-later`. The repository records exact source revisions and BLAKE3 identities. No plan implies relicensing.

### Keep durable dependency inputs immutable

Release dependencies use exact reviewed revisions and declared acquisition adapters. Workspace-relative paths are allowed only for local development or disposable conformance checks. They are not release authority.

### Make validation layered

The foundation runs workspace metadata checks, architecture tests, positive and negative dependency fixtures, Cargo checks, Nix checks, and Cairn gates. Formal proof claims begin in later changes.

## Risks and Mitigations

- **Duplicate semantics**: architecture checks and the later facade change block release while two production implementations remain.
- **License drift**: manifest and source checks require AGPL declarations and provenance.
- **Shell leakage**: dependency and source scans reject I/O dependencies in the core.
- **Premature proof claims**: README and receipts label this stage as unverified foundation work.

## Claim Boundary

Passing this change proves repository structure and declared boundaries only. It does not prove graph correctness, transition safety, compiler correctness, or consumer behavior.
