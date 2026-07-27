# Choregraph

Choregraph is a planned compiler and verified Rust kernel for bounded execution graphs.

The project will lower reviewed, data-oriented Nickel graphs into canonical machine artifacts. A pure `no_std` Rust core will execute the admitted state-machine transition relation.

## Status

The buildable foundation, executable machine semantics, and Kamacite interchange adapter exist. The adapter emits canonical packed Preserves artifacts, exact BLAKE3 identity chains, a closed local lifecycle profile, and bounded linkage receipts.

Pure lowering from contracted evaluated Nickel data now exists. The capability-relative evaluator shell remains blocked on a published immutable `cap-root` source.

A production-linked Verus rail now proves the named single-step properties of the indexed kernel. Trace proofs, compatibility parity, and release evidence do not exist yet.

## Owned boundary

Choregraph will own:

- bounded canonical execution-graph and machine IR;
- deterministic graph admission and transition semantics;
- pure state transitions and effect intents;
- Verus proofs over production transition code;
- Kamacite artifact and receipt adapters;
- restricted Nickel lowering after evaluation;
- Seaglass and Lattice compatibility adapters.

The production transition relation will have one implementation. Compatibility crates must delegate to that implementation.

## Excluded authority

Choregraph will not execute effects, verify UCAN tokens, resolve keys, read clocks, open ambient files, schedule resources, persist runs, or claim external effect success.

Lattice retains runtime and product authority. Basalt evaluates bounded authorization facts. `cap-root` and `cap-std` confine local file access. Kamacite binds canonical artifacts and receipts. Valence and Cairn link evidence without creating behavioral proofs.

## Proof boundary

The target claim begins at admitted canonical machine IR. Source and evaluator receipts bind Nickel inputs to observed outputs, but they do not prove arbitrary Nickel evaluation semantics.

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

See [`docs/implementation-order.md`](docs/implementation-order.md) for the dependency-ordered Choregraph, execution-graph, and Lattice Cairn changes.

## Workspace

```text
crates/choregraph-core/
crates/choregraph-kamacite/
crates/choregraph-nickel/
crates/choregraph-seaglass/
crates/choregraph-cli/
tools/choregraph-foundation-check/
fixtures/foundation/
evidence/
```

See [`docs/architecture.md`](docs/architecture.md) for the one-way dependency boundary.

See [`docs/machine-semantics.md`](docs/machine-semantics.md) for the executable transition model and current non-claims.

See [`docs/single-step-verification.md`](docs/single-step-verification.md) for the production linkage, theorems, trust boundary, and proof commands.

See [`docs/kamacite-profile.md`](docs/kamacite-profile.md) for the selected codec, schemas, identity rules, and receipt boundary.

See [`docs/restricted-nickel-lowering.md`](docs/restricted-nickel-lowering.md) for the evaluated-data contract, pure lowerer, identity chain, and capability-shell blocker.

## Validation

```text
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- --deny warnings
nix develop -c cargo run -p choregraph-foundation-check -- --root .
nix flake check -L
```

See [`docs/foundation-validation.md`](docs/foundation-validation.md) for the complete foundation rail.

## License

Choregraph preserves the source implementation license: `AGPL-3.0-or-later`. No relicensing is implied.

## References

- [execution-graph](https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git) — existing deterministic DAG implementation and compatibility baseline.
- [Trellis](https://github.com/OnixResearch/trellis) — verified DAG, lifecycle, scheduler, replay, and choreography primitives.
- [Kamacite](https://github.com/OnixResearch/kamacite) — canonical Preserves artifacts, source envelopes, FSM descriptors, and receipts.
- [Lattice](https://github.com/OnixResearch/lattice) — first consumer and Seaglass compatibility target.
- [Basalt](https://github.com/OnixResearch/basalt) — bounded Nickel and UCAN authorization boundary.
- [cap-root baseline](evidence/source-baseline.ncl) — pure lexical path admission before capability-relative I/O.
- [`cap-std`](https://github.com/bytecodealliance/cap-std) — capability-oriented filesystem I/O used by consumer shells.
