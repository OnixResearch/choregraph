# Choregraph

Choregraph is a planned compiler and verified Rust kernel for bounded execution graphs.

The project will lower reviewed, data-oriented Nickel graphs into canonical machine artifacts. A pure `no_std` Rust core will execute the admitted state-machine transition relation.

## Status

Only lifecycle and architecture plans exist. No implementation, proof, compiler, or release claim exists yet.

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

## Planned workspace

```text
crates/choregraph-core/
crates/choregraph-kamacite/
crates/choregraph-nickel/
crates/choregraph-seaglass/
crates/choregraph-cli/
contracts/
fixtures/positive/
fixtures/negative/
proofs/
```

## License

Choregraph preserves the source implementation license: `AGPL-3.0-or-later`. No relicensing is implied.

## References

- [`../execution-graph/`](../execution-graph/) — existing deterministic DAG implementation and compatibility baseline.
- [`../trellis/`](../trellis/) — verified DAG, lifecycle, scheduler, replay, and choreography primitives.
- [`../kamacite/`](../kamacite/) — canonical Preserves artifacts, source envelopes, FSM descriptors, and receipts.
- [`../lattice/`](../lattice/) — first consumer and Seaglass compatibility target.
- [`../basalt/`](../basalt/) — bounded Nickel and UCAN authorization boundary.
- [`../cap-root/`](../cap-root/) — pure lexical path admission before capability-relative I/O.
- [`cap-std`](https://github.com/bytecodealliance/cap-std) — capability-oriented filesystem I/O used by consumer shells.
