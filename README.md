# Choregraph

Choregraph is a Rust library and compiler for bounded workflow graphs. It validates a graph, tracks each node's lifecycle, and decides whether an event may change the state. Accepted events return a new state and any declared effect intents; the application decides how to authorize and execute those intents.

Use it when you need deterministic workflow decisions that can be checked and replayed independently of a runner. The repository includes a restricted Nickel compiler, canonical binary artifacts, production-linked Verus proofs, and adapters for Lattice, WebAssembly, role protocols, and branchable history.

Choregraph is not a standalone workflow service. There is no worker daemon, job queue, database, or `run` command. The CLI compiles graph data and handles local artifacts; applications embed the kernel to drive workflows.

## What you can build with it

- **Dependency-driven workflows.** Admit a DAG and ask which nodes are ready. A diamond graph can expose two independent branches after their shared predecessor completes, then hold the final node until both branches finish. See the [Lattice diamond test](crates/choregraph-seaglass/tests/lattice-diamond.rs).
- **Workflows that wait for an external signal.** The opt-in durable-suspension profile records a suspension class and a BLAKE3 reference. A matching resume event moves the node back to ready. The host stores the wait and determines whether the signal is authentic or approved. See the [suspension tests](crates/choregraph-core/tests/durable-suspension.rs).
- **Deterministic recovery and inspection.** Replay a bounded event sequence against an admitted machine and initial state. Replay reports the first rejected event and never returns historical effect intents for dispatch. See the [replay tests](crates/choregraph-core/tests/trace-replay.rs).
- **Branchable application history.** Build immutable event DAGs, select an explicit lineage, and prepare generation-checked branch updates. The machine-history adapter can replay supported machine events from that lineage. The host supplies payload storage and application merge decisions.
- **Role protocol compilation.** Validate declared roles, ownership, transfers, and choices, then project a global protocol into one local protocol per role using pinned Trellis functions. This is a separate library workflow, not a machine runner or a `compile-nickel` mode.

These are implemented mechanisms with fixtures and tests. Product scheduling, distributed storage, transport, authorization, and effect execution remain consumer responsibilities.

## Quick start: compile a graph

Build from source. The workspace packages are version `0.1.0` with `publish = false`; these instructions do not assume a crates.io release.

You need Git and Nix with flakes enabled. The development shell supplies Rust 1.97.1, Nickel, and the verification tools. The manifest declares Rust 1.96.0 as its minimum, but the maintained build uses the pinned toolchain. Cargo fetches the pinned Trellis dependency over GitHub SSH, so direct Cargo builds need working GitHub SSH access or an appropriate local Git URL rewrite to HTTPS.

```sh
git clone https://github.com/OnixResearch/choregraph.git
cd choregraph
nix develop
```

Run the following inside that shell, from the repository root:

```sh
cargo run --locked -p choregraph-cli -- boundary

work=$(mktemp -d)
mkdir -p "$work/scratch" "$work/output"
cargo run --locked -p choregraph-cli -- compile-nickel \
  "$(command -v nickel)" \
  "$PWD/fixtures/nickel" positive/minimal.ncl \
  "$PWD/contracts/nickel" "$work/scratch" \
  "$work/output" minimal.preserves

test -s "$work/output/minimal.preserves"
```

The first command prints the component boundaries, including static `unverified` and `unproved` status labels; it does not run the proof rails. The compiler command prints `status=compiled`, source/import information, the machine's BLAKE3 identity, and its artifact size. It writes a canonical packed Preserves machine artifact to `$work/output/minimal.preserves`.

The [minimal fixture](fixtures/nickel/positive/minimal.ncl) declares one node named `root`, an operation named `run`, no edges, and no effects. Compiling it does not execute `run`. Output publication refuses to replace an existing file; use a fresh directory or a new output name for another compilation.

The positional arguments are:

```text
choregraph compile-nickel <evaluator> <source-root> <source-path> \
  <contract-root> <scratch-root> <output-root> <output-path> \
  [<import-id>=<import-root> ...]
```

Use an absolute evaluator path and absolute operator-selected roots. Source and output paths are relative to their respective roots. Declared graph imports are data: each names a configured root, relative path, media class, byte limit, and expected BLAKE3 digest. See the [declared-import fixture](fixtures/nickel/positive/declared-import.ncl) and [frontend documentation](docs/restricted-nickel-lowering.md).

For a standalone CLI build:

```sh
nix build .#choregraph
./result/bin/choregraph boundary
```

## Embedding the kernel

The central API is `choregraph_core::step(machine, state, event)`. Admission produces an immutable machine; callers retain the current state and submit events explicitly.

```rust
use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, Decision, Event, MachineSpec, NodeSpec,
};

let spec = MachineSpec::new(vec![NodeSpec::new("build", vec![])], vec![]);
let machine = AdmittedMachine::admit(spec, AdmissionLimits::default())
    .expect("valid graph");
let mut state = machine.initial_state();

for action in [Action::MarkReady, Action::Start, Action::Complete] {
    match choregraph_core::step(&machine, &state, &Event::new("build", action)) {
        Decision::Accept { next, intents } => {
            assert!(intents.is_empty()); // This graph declares no effects.
            state = next;
        }
        Decision::Deny { reason } => panic!("event rejected: {reason:?}"),
    }
}
```

This example records lifecycle decisions only. A real host would perform the work between `Start` and `Complete`, submit failure when appropriate, and store accepted state changes. Use the [core crate](crates/choregraph-core) as a path dependency from a source checkout; replace the example's panic handling with application error handling.

The default linear lifecycle is:

```text
Pending -> Ready -> Running -> Completed
                       |
                       +----> Failed

Pending, Ready, or Running -> Cancelled
```

Readiness and start require completed dependencies. Completed, failed, and cancelled nodes reject further changes. A denied event returns a reason, no replacement state, and no intents.

The explicit `choregraph.lifecycle.durable-suspension.v1` profile also permits `Ready` or `Running` to become `Suspended`, then return to `Ready` on a matching `ResumeSignal`. Suspension and resumption cannot request effects. There is no automatic timeout, retry, or approval decision.

Other core entry points include:

| API | Purpose |
| --- | --- |
| `AdmittedMachine::admit` / `admit_v2` | Validate and normalize graph specifications under explicit bounds. |
| `ready_nodes`, `parallel_groups`, `critical_path`, `bottlenecks` | Inspect readiness and graph structure; they do not launch concurrent tasks. |
| `step_concrete` | Resolve v2 effect declarations with exact references or declared runtime slots. |
| `replay` / `replay_concrete` | Fold a bounded trace through production transitions without returning dispatchable effects. |
| `step_epoch_fenced` | Gate a transition on caller-supplied matching entity and epoch facts and a non-terminal state. |

## Architecture and supported inputs

The core is functional: inputs determine outputs, and the caller owns side effects. `choregraph-core`, the history crates, and the release-linkage core use `no_std` plus `alloc`. The CLI is the imperative shell that opens files and launches the evaluator.

```text
Restricted Nickel source
    -> CLI: capability-relative reads and bounded evaluator process
    -> choregraph-nickel: checked evaluated data and import closure
    -> choregraph-core: admitted machine and pure transitions
    -> choregraph-kamacite: canonical artifacts and identity receipts

Host events + admitted machine + state
    -> step / step_concrete -> accepted state + effect intents, or denial

Explicit history lineage -> choregraph-history-machine -> production replay
Evaluated role protocol  -> choregraph-choreography -> Trellis -> local protocols
```

| Crate | Implemented responsibility |
| --- | --- |
| [`choregraph-core`](crates/choregraph-core) | DAG admission, lifecycle transitions, effect declarations, graph analysis, replay, and epoch fencing. |
| [`choregraph-nickel`](crates/choregraph-nickel) | Pure validation and lowering of evaluated graph data; it does not launch Nickel itself. |
| [`choregraph-cli`](crates/choregraph-cli) | Restricted evaluation, local artifact writing, corpus generation, and release-evidence commands. The executable is named `choregraph`. |
| [`choregraph-kamacite`](crates/choregraph-kamacite) | Packed Preserves codecs, versioned machine/lifecycle artifacts, BLAKE3 identities, and linkage receipts. |
| [`choregraph-choreography`](crates/choregraph-choreography) | Bounded role/ownership admission and Trellis-backed endpoint projection through `compile_evaluated`. |
| [`choregraph-seaglass`](crates/choregraph-seaglass), [`execution-graph-core`](crates/execution-graph-core) | Compatibility APIs that delegate machine behavior to the core. |
| [`choregraph-wasm`](crates/choregraph-wasm) | Stateless, export-only component for admission, step, concrete step, and both replay variants; each call re-admits supplied artifacts. |
| [`choregraph-history`](crates/choregraph-history) | Immutable event identities and DAGs, branch update plans, snapshots, cursors, and explicit lineage selection. |
| [`choregraph-history-machine`](crates/choregraph-history-machine) | Validate history bindings and delegate supported linear machine histories to production replay. |
| [`choregraph-weave`](crates/choregraph-weave) | Optional consumer adapter for bounded Universal Weave metadata topology derived from history. |
| [`choregraph-release`](crates/choregraph-release) | Validate local evidence bundles and candidate identity linkage. |

The machine frontend accepts `choregraph.evaluated-graph.v1` and selected Lattice workflow projections (`choregraph.lattice-workflow-projection.v1` and `.v2`). V2 supports concrete effect declarations. The separate `choregraph.role-choreography.v1` profile produces protocol artifacts, not machine IR.

The CLI additionally exposes `write-kamacite-corpus`, `write-release-bundle`, `verify-release-bundle`, and `validate-publication-candidate`. The release commands work with a fixed evidence layout, not arbitrary graph directories. Candidate validation does not publish anything. See [release evidence](docs/release-evidence.md) for the exact arguments and historical publication records.

## Limits and verification scope

- **Bounded DAGs.** Admission rejects cycles, duplicate facts, missing endpoints, malformed identifiers, unsupported profiles, and invalid effect declarations. Default core limits include 4,096 nodes, 16,384 edges, and 64 intents per transition. Default replay is bounded to 65,536 events. Callers supply explicit limits; frontend and WASM profiles impose their own ceilings.
- **Restricted source language.** The CLI rejects Nickel file-import syntax and string interpolation before evaluation. It applies a closed contract and Rust validates the result again. This is not an unrestricted Nickel program runner. Bounded Exec limits the child process but does not provide a filesystem or network sandbox.
- **Host-owned authority.** Effect intents are requests, not evidence of execution or success. The kernel does not check UCAN tokens, resolve keys, read clocks, persist runs, allocate workers, or discover the current ownership epoch. An epoch match only checks supplied facts.
- **Explicit history selection.** Multi-parent history needs explicit lineage choices. The machine-history adapter rejects merges and unsupported event classes; it does not invent application merge semantics. History stores payload identities, not payload bytes. Branch plans require host persistence.
- **Scoped proofs.** Verus rails cover named properties of the production-called indexed kernel, including transition invariants, readiness completeness, finite-trace behavior, and epoch fencing. They do not prove the whole application, identifier-map adapters, arbitrary Nickel semantics, caller-derived identities, external-role correctness, or distributed execution.
- **Identity is not correctness.** Canonical bytes and receipts bind exact artifacts and evidence. They do not establish authorization, exactly-once effects, persistence durability, delivery, or release eligibility. Checked-in evidence describes its recorded source and toolchain; changes need fresh validation.

The [single-step proof boundary](docs/single-step-verification.md), [trace proof boundary](docs/trace-replay-verification.md), and [WASM contract](docs/wasm-kernel-adapter.md) describe the precise claims and trusted surfaces. Reference designs and superseded plans are provenance, not additional shipped features.

## Development and checks

Run the focused foundation checks before the broader Nix checks:

```sh
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- --deny warnings
nix develop -c cargo run -p choregraph-foundation-check -- --root .
nix flake check -L
```

Additional targeted checks and examples:

```sh
nix develop -c cargo run -p choregraph-history --example deterministic-identity
nix develop -c cargo check -p choregraph-history -p choregraph-history-machine -p choregraph-weave --target wasm32-unknown-unknown
nix build .#choregraph-wasm-component --no-link -L
nix build .#checks.x86_64-linux.wasm-kernel-conformance --no-link -L
nix build .#checks.x86_64-linux.single-step-verus --no-link -L
nix build .#checks.x86_64-linux.trace-replay-verus --no-link -L
nix build .#checks.x86_64-linux.epoch-fence-verus --no-link -L
nix build .#checks.x86_64-linux.octet-deny-all --no-link -L
```

The pinned Octet gate treats all its lints as errors. The full [foundation validation guide](docs/foundation-validation.md) includes fixture checks and evidence requirements. Downstream Lattice validation requires a separate Lattice checkout; see [cutover readiness](docs/lattice-cutover-readiness.md).

## Further reading

- [Architecture](docs/architecture.md) and [machine semantics](docs/machine-semantics.md)
- [Restricted Nickel lowering](docs/restricted-nickel-lowering.md) and [Kamacite artifacts](docs/kamacite-profile.md)
- [Role choreography](docs/role-choreography.md)
- [Branchable history](docs/branchable-event-history.md) and [epoch fencing](docs/epoch-fenced-single-writer.md)
- [Lattice authority boundary](docs/lattice-authority-boundary.md)
- [Release evidence and historical exact-pin publications](docs/release-evidence.md)
- [Implementation order and superseded planning provenance](docs/implementation-order.md)

<!-- r[impl choregraph.history.supersession] -->
<!-- r[impl choregraph.history.supersession.provenance] -->
<!-- r[verify choregraph.history.verification.complete] -->

Active `execution-graph-core` compatibility development lives in this workspace. The original execution-graph repository remains a compatibility and rollback reference. Trellis supplies the pinned role projection implementation; Lattice remains the workflow runtime consumer. Earlier Radicle publication revisions are historical candidates, not a substitute for the current GitHub source revision.

## License

Choregraph preserves the source implementation license, [`AGPL-3.0-or-later`](LICENSE). This repository does not imply relicensing. Vendored source provenance and license files remain under [`vendor/`](vendor).
