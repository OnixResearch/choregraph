# Restricted Nickel lowering

Choregraph begins its maintained frontend claim at evaluated data that satisfies a closed contract.

## Contract boundary

[`../contracts/nickel/restricted-graph-contracts.ncl`](../contracts/nickel/restricted-graph-contracts.ncl) defines three evaluated formats:

- `choregraph.evaluated-graph.v1` for direct graph data;
- `choregraph.lattice-workflow-projection.v1` for selected Lattice port graphs;
- `choregraph.lattice-workflow-projection.v2` for Lattice graphs with explicit concrete-effect declarations.

The separate [`graph-construction`](graph-construction.md) profile `choregraph.graph-construction.v1` uses exact captured Nickel module bytes and native elaboration into Lattice projection v3 with control edges. It does **not** reinterpret these three existing formats or add a pre-evaluation loader to this restricted CLI.

The contract checks exact fields, required schemas, fixed lifecycle events, array bounds, scalar types, import declarations, and selected graph shapes.

Direct graph data opts in with the exact `lifecycle_profile` string
`choregraph.lifecycle.durable-suspension.v1` or `choregraph.lifecycle.durable-suspension.v2`.
Both use the existing `choregraph.evaluated-graph.v1` schema, require the same
seven events (including `suspend` and `resume-signal`), and allow a bounded
`suspensions` array of closed `{node_id, class, reference}` records. The linear
contract retains its five events and closed field set; the two Lattice
projections still select the linear profile. Rust admission checks the graph
and exact suspension bindings. Only the V2 lifecycle accepts `cancel` from
`suspended`, without a suspension fact or effects. The evaluated fixture
`fixtures/nickel/evaluated/durable-cancellation.json` exercises this profile
selection; the locked CLI publication path still uses the linear artifact set.

[`../contracts/nickel/role-choreography-contracts.ncl`](../contracts/nickel/role-choreography-contracts.ncl) defines the separate `choregraph.role-choreography.v1` profile. It checks native and external roles, values, ownership, communication, owned transfer, choice, termination, fixed limits, and required non-claims. This profile does not lower to machine IR.

The v2 contract requires each effectful node to have a declaration. It rejects effects on nodes that are not effectful.

Rust parses with `deny_unknown_fields`, checks numeric ceilings, and validates the import closure. V1 calls `AdmittedMachine::admit`. V2 calls `AdmittedMachine::admit_v2`.

A v2 slot source must name the declared node input, output, or configuration namespace. The frontend preserves this source reference.

The CLI reads bounded UTF-8 through an operator-opened capability root. Its conservative Nickel 1.17 lexical subset accepts `#` line comments and ordinary single-line double-quoted strings, including the simple `\\`, `\"`, `\'`, `\%`, `\n`, `\r`, and `\t` escapes. It rejects imports, interpolation, physical newlines inside strings, the multiline and symbolic delimiters, numeric escapes, and all unquoted `%` (including raw primops and modulo). `//` and `/*` are not Nickel comments and are not admitted. The gate conservatively rejects an `import` suffix outside strings and comments even if it might belong to an identifier. It is not a parser; remaining syntax errors are rejected by Nickel. Declared graph imports remain data and are loaded after evaluation.

The shell checks both operator contract files byte-for-byte against its embedded materials. It passes only the embedded copies to the launcher, which supplies the exact bytes through file descriptors for read-only mounts. Nickel applies [`../contracts/nickel/evaluated-graph-contract.ncl`](../contracts/nickel/evaluated-graph-contract.ncl) and resolves its relative library import within that private mount. The evaluator never reopens the supplied contract root. Passing Nickel export is not sufficient for admission.

Construction `finish` applies its own closed contract to exported JSON. Its supported evaluator is the distinct `nickel-eval.captured-modules.v1` profile with an actually observed module-resolution closure; declared graph imports here are still post-evaluation data, not a source module map. Native construction admission and full compilation-binding validation are mandatory after evaluation.

## Import closure

Each import declares:

- a bounded logical identifier;
- a configured root identifier;
- an ASCII relative path;
- a selected media class;
- a positive byte limit;
- a lowercase BLAKE3 identity.

Rust requires exact declaration and observation coverage. It rejects duplicates, missing or undeclared observations, path escapes, oversized bytes, total-byte overflow, and hash mismatch. Raw imported bytes are not retained in the validated closure result.

The path check is a deterministic source-profile check. It is not a replacement for `cap-root`, no-follow I/O, or capability ownership.

## Identity and receipts

The translation chain uses domain-separated BLAKE3 identities for source, import closure, evaluator, evaluated output, lowerer, canonical descriptor, admitted IR, proof set, and toolchain materials.

Source and evaluator receipts contain exact identities and explicit non-claims. Receipt validation recomputes the complete expected body. It rejects stale lowerer or artifact material.

These receipts do not prove arbitrary Nickel semantic equivalence, evaluator soundness, source intent, transition correctness, runtime effects, or release eligibility.

The restricted CLI publishes the canonical machine artifact and reports its machine identity; it does not emit a separate evaluator receipt. Its evaluator and launcher/runtime identity is the selected Nix build: compile-time absolute store references for Nickel 1.17.0, Bash, Bubblewrap, and the pinned Nickel ELF file list, plus the launcher policy embedded in the CLI and its embedded contract bytes. A version string or a caller-selected executable does not establish that identity. These references trust the local Nix store and build inputs rather than rehashing/read-verifying the executables on each run.

## Diagnostics

The pure frontend emits stable issue codes and constant bounded summaries. Evaluator redaction discards raw text. Safe diagnostics do not include source content, imported values, credentials, environment values, or host paths.

The choreography profile also reports bounded primary and secondary spans, notes, hints, and optional replacements. Rust revalidates each diagnostic before rendering or artifact encoding.

## Capability shell

`cap-root-core` comes from Radicle project `rad:z3cWBCK7VPbEmbL5MAvKJWKjCE745` at exact revision `8991e2572578c16e590c2f1c0915db6c46c5d9cb`. The reviewed Rust source is vendored without changes. [`../tools/sync-cap-root-core.rs`](../tools/sync-cap-root-core.rs) reproduces the import.

The shell opens operator-selected source, contract, output, and import roots. `cap-root` admits every relative file path. `cap-std` performs the I/O. `cap-fs-ext` rejects symlinks in each traversed directory and final file component. The existing scratch-root argument must still name an accessible absolute directory, but is not passed to the evaluator; it uses an empty `/work` inside the sandbox.

Bounded Exec supplies bounded stdin and captured stdout/stderr, a deadline, and process-group teardown. It starts the pinned Bash launcher with no inherited environment. Bubblewrap requires Linux user, mount, PID, network, IPC, and UTS isolation, drops capabilities, and does not mount `/proc`, `/dev`, any operator root, or the whole Nix store. The visible filesystem is a read-only private root with the staged contracts and only the individual Nickel executable, ELF interpreter, and shared library files at their original store paths. The network namespace is not connected to the host network. The trusted Nickel build and source profile do not expose process-creation or clock primitives; this is not a syscall filter or protection against a malicious launcher, runtime, or kernel.

Use `choregraph compile-nickel --pinned …` with `nix build .#choregraph`, or `cargo run --locked -p choregraph-cli -- compile-nickel --pinned …` within `nix develop`. An absolute evaluator argument remains accepted only if it resolves to the exact pinned executable. Builds without the compile-time sandbox configuration, non-Linux hosts, and Linux hosts that disallow the required namespaces reject evaluation. There is no unisolated fallback; other CLI commands are unaffected.

The output path is operator-selected. Publication writes and synchronizes one new temporary regular file. It then creates the final name without replacement and removes the temporary name. This is atomic name publication, not directory-entry durability.

The shell returns stable issue codes and discards raw evaluator diagnostics. Graph data cannot select an ambient root.

See [`../evidence/cap-root-adoption.ncl`](../evidence/cap-root-adoption.ncl) for exact source and process-mechanism identities.

## Validation

`nix build .#checks.<system>.restricted-nickel-contracts` validates typed contracts and reviewed fixture exports inside the Nix build environment and checks source rejection before launch. It does not attest to nested namespaces or host confinement. On a supported Linux host, run the real acceptance test separately:

```sh
nix develop -c cargo test --locked -p choregraph-cli linux_host_sandbox_acceptance -- --ignored
```

The host test checks minimal and declared-import fixtures against canonical machine identities and bytes from the reviewed exports, and verifies that a disposable host scratch fixture is not exposed. A failed or skipped host test is not evidence of host sandbox acceptance.
