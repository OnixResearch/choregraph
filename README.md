# Choregraph

Choregraph is a compiler and verified Rust kernel for bounded execution graphs.

It lowers reviewed data-oriented Nickel graphs into canonical machine artifacts. A pure `no_std` Rust core runs the admitted state-machine transition relation.

## Status

The buildable foundation, machine semantics, and Kamacite adapter exist.

The Kamacite adapter emits these values:

- canonical packed Preserves artifacts
- exact BLAKE3 identity chains
- a closed local lifecycle profile
- bounded linkage receipts

The restricted Nickel frontend reads source and imports beneath an operator-selected root. It uses `cap-root` and `cap-std` for path and file access.

The frontend runs the selected evaluator through pinned Bounded Exec. Then it lowers checked output and publishes canonical machine artifacts without replacement.

Production-linked Verus rails prove named single-step and finite-trace properties of the indexed kernel. Replay uses the production `step` relation.

The export-only WASM component accepts canonical packed Preserves calls. It re-admits artifacts and delegates to the same production entrypoints on every call.

The epoch-fenced transition wrapper admits supplied ownership facts only for the same entity, equal epoch, and non-terminal state. Choregraph does not discover or authorize the current epoch.

If replay finds a denial, it reports the first denial and returns no dispatchable intents.

A pinned Octet gate checks every workspace target. The gate treats every Octet lint as an error.

The conversion-only Seaglass adapter passes the selected behavior matrix. The reviewed Lattice runner passes all 21 tests with `choregraph-seaglass` substituted for the legacy package.

Active `execution-graph-core` compatibility development now lives in this workspace. The original execution-graph repository remains a frozen compatibility and rollback source.

The compiler-compatibility source has this identity:

- RID: `rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF`
- Revision: `81b260da2256147ec82cb5dd44c0a0c1dcb7500d`

Exact-pin consumers can use Radicle or the read-only HTTPS adapter.

## Owned boundary

Choregraph owns these functions:

- bounded canonical execution-graph and machine IR
- deterministic graph admission and transition semantics
- pure state transitions and effect intents
- Verus proofs over production transition code
- Kamacite artifact and receipt adapters
- restricted Nickel evaluation and canonical lowering
- Seaglass and Lattice compatibility adapters
- a stateless WASM component over the production kernel
- a proof-linked epoch-fence decision over caller-supplied entity facts

The production transition relation has one implementation. Compatibility crates must delegate to that implementation.

## Excluded authority

Choregraph does not perform these functions:

- execute effects
- check UCAN tokens or resolve keys
- read clocks or open ambient files
- schedule resources or persist runs
- claim success for an external effect

Lattice retains runtime and product authority. Basalt evaluates bounded authorization facts.

`cap-root` and `cap-std` confine local file access. Kamacite binds canonical artifacts and receipts.

Valence and Cairn link evidence. They do not create behavioral proofs.

## Proof boundary

The proof claim starts at admitted canonical machine IR. Source and evaluator receipts bind Nickel inputs to observed outputs.

Those receipts do not prove arbitrary Nickel evaluation semantics.

The initial theorem is:

```text
WellFormed(machine)
&& Invariant(machine, state)
&& step(machine, state, event) = Accept(next, effects)
=> Invariant(machine, next)
&& EffectsAllowed(machine, state, event, effects)
```

A denied transition must preserve state and emit no effects.

## Implementation lifecycle

Read [`docs/implementation-order.md`](docs/implementation-order.md) for the dependency order across Choregraph, execution-graph, and Lattice changes.

## Workspace

```text
crates/choregraph-core/
crates/choregraph-kamacite/
crates/choregraph-nickel/
crates/choregraph-release/
crates/choregraph-seaglass/
crates/choregraph-wasm/
crates/execution-graph-core/
crates/choregraph-cli/
tools/choregraph-foundation-check/
fixtures/foundation/
evidence/
```

## Documentation

- [`docs/architecture.md`](docs/architecture.md): one-way dependency boundaries.
- [`docs/machine-semantics.md`](docs/machine-semantics.md): executable transitions and current non-claims.
- [`docs/single-step-verification.md`](docs/single-step-verification.md): production linkage and the single-step proof boundary.
- [`docs/trace-replay-verification.md`](docs/trace-replay-verification.md): bounded replay, finite-trace theorems, and failure behavior.
- [`docs/kamacite-profile.md`](docs/kamacite-profile.md): codecs, schemas, identities, and receipts.
- [`docs/restricted-nickel-lowering.md`](docs/restricted-nickel-lowering.md): source admission, evaluation, lowering, identities, and non-claims.
- [`docs/octet-deny-all.md`](docs/octet-deny-all.md): lint policy, strict commands, and exceptions.
- [`docs/lattice-cutover-readiness.md`](docs/lattice-cutover-readiness.md): source order, duplicate removal, and rollback gates.
- [`docs/lattice-authority-boundary.md`](docs/lattice-authority-boundary.md): intent handoff and retained Lattice authority.
- [`docs/release-evidence.md`](docs/release-evidence.md): bundle roles, Valence projection, and publication checks.
- [`docs/wasm-kernel-adapter.md`](docs/wasm-kernel-adapter.md): WIT, Preserves envelopes, limits, identities, host duties, and non-claims.
- [`docs/epoch-fenced-single-writer.md`](docs/epoch-fenced-single-writer.md): supplied epoch facts, the proof-linked gate, Celld reference scope, and non-claims.

## Checks

Run all commands from the repository root:

```text
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- --deny warnings
nix develop -c cargo run -p choregraph-foundation-check -- --root .
nix build .#choregraph-wasm-component --no-link -L
nix build .#checks.x86_64-linux.wasm-kernel-conformance --no-link -L
nix build .#checks.x86_64-linux.wasm-kernel-evidence --no-link -L
nix build .#checks.x86_64-linux.epoch-fence-verus --no-link -L
nix build .#checks.x86_64-linux.octet-deny-all --no-link -L
./tools/check-lattice-downstream.rs --self-test
./tools/check-lattice-downstream.rs ../lattice .
nix flake check -L
```

Read [`docs/foundation-validation.md`](docs/foundation-validation.md) for the complete foundation rail.

## License

Choregraph preserves the source implementation license, `AGPL-3.0-or-later`. This repository does not imply relicensing.

## References

- [Choregraph publication](https://git.onix.computer/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git) — read-only adapter for the published candidate.
- [execution-graph](https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git) — deterministic DAG implementation and compatibility baseline.
- [Trellis](https://github.com/OnixResearch/trellis) — verified graph, lifecycle, scheduler, replay, and choreography primitives.
- [Kamacite](https://github.com/OnixResearch/kamacite) — canonical Preserves artifacts, source envelopes, FSM descriptors, and receipts.
- [Lattice](https://github.com/OnixResearch/lattice) — first consumer and Seaglass compatibility target.
- [Basalt](https://github.com/OnixResearch/basalt) — bounded Nickel and UCAN authorization boundary.
- [Bounded Exec](https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git) — bounded evaluator execution without sandbox or evaluator-correctness claims.
- [cap-root](rad://z3cWBCK7VPbEmbL5MAvKJWKjCE745) — lexical path admission before capability-relative I/O.
- [`cap-std`](https://github.com/bytecodealliance/cap-std) — capability-oriented filesystem I/O for consumer shells.
- [Swamp](https://github.com/swamp-club/swamp) — product reference for durable workflow suspension.
- [Celld at `553ae73f83c8`](https://github.com/denoland/celld/tree/553ae73f83c87c3f7c7a5f73c32c2211d9d7341f) — Apache-2.0 reference for per-cell ownership epochs and shard-by-construction isolation.

Choregraph adapts only Swamp's clean-room lifecycle mechanism. It does not adopt Swamp runtime code, approval authority, YAML/CEL authoring, or TypeScript loading.

Choregraph uses Celld only as a bounded mechanism reference. It does not adopt Celld code, storage, leases, routing, replication, runtime authority, or distributed-correctness claims.
