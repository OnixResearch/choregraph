# Restricted Nickel lowering

Choregraph begins its maintained frontend claim at evaluated data that satisfies a closed contract.

## Contract boundary

[`../contracts/nickel/restricted-graph-contracts.ncl`](../contracts/nickel/restricted-graph-contracts.ncl) defines three evaluated formats:

- `choregraph.evaluated-graph.v1` for direct graph data;
- `choregraph.lattice-workflow-projection.v1` for selected Lattice port graphs;
- `choregraph.lattice-workflow-projection.v2` for Lattice graphs with explicit concrete-effect declarations.

The contract checks exact fields, required schemas, fixed lifecycle events, array bounds, scalar types, import declarations, and selected graph shapes.

[`../contracts/nickel/role-choreography-contracts.ncl`](../contracts/nickel/role-choreography-contracts.ncl) defines the separate `choregraph.role-choreography.v1` profile. It checks native and external roles, values, ownership, communication, owned transfer, choice, termination, fixed limits, and required non-claims. This profile does not lower to machine IR.

The v2 contract requires each effectful node to have a declaration. It rejects effects on nodes that are not effectful.

Rust parses with `deny_unknown_fields`, checks numeric ceilings, and validates the import closure. V1 calls `AdmittedMachine::admit`. V2 calls `AdmittedMachine::admit_v2`.

A v2 slot source must name the declared node input, output, or configuration namespace. The frontend preserves this source reference.

The CLI reads source through an operator-opened capability root. A conservative source gate rejects Nickel `import` syntax and string interpolation before evaluator execution. Declared graph imports remain data and are loaded after evaluation.

The shell applies [`../contracts/nickel/evaluated-graph-contract.ncl`](../contracts/nickel/evaluated-graph-contract.ncl) through the pinned Nickel executable. Passing Nickel export is not sufficient for admission.

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

## Diagnostics

The pure frontend emits stable issue codes and constant bounded summaries. Evaluator redaction discards raw text. Safe diagnostics do not include source content, imported values, credentials, environment values, or host paths.

The choreography profile also reports bounded primary and secondary spans, notes, hints, and optional replacements. Rust revalidates each diagnostic before rendering or artifact encoding.

## Capability shell

`cap-root-core` comes from Radicle project `rad:z3cWBCK7VPbEmbL5MAvKJWKjCE745` at exact revision `8991e2572578c16e590c2f1c0915db6c46c5d9cb`. The reviewed Rust source is vendored without changes. [`../tools/sync-cap-root-core.rs`](../tools/sync-cap-root-core.rs) reproduces the import.

The shell opens operator-selected source, contract, scratch, output, and import roots once. `cap-root` admits every relative file path. `cap-std` performs the I/O. `cap-fs-ext` rejects symlinks in each traversed directory and final file component.

Bounded Exec runs Nickel with an absolute executable, an empty inherited environment, bounded input and output, a deadline, and owned process-group teardown. This mechanism does not provide a filesystem or network sandbox. The conservative source gate excludes source-selected file imports before process execution.

The output path is operator-selected. Publication writes and synchronizes one new temporary regular file. It then creates the final name without replacement and removes the temporary name. This is atomic name publication, not directory-entry durability.

The shell returns stable issue codes and discards raw evaluator diagnostics. Graph data cannot select an ambient root.

See [`../evidence/cap-root-adoption.ncl`](../evidence/cap-root-adoption.ncl) for exact source and process-mechanism identities.
