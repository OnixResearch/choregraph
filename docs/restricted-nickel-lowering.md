# Restricted Nickel lowering

Choregraph begins its maintained frontend claim at evaluated data that satisfies a closed contract.

## Contract boundary

[`../contracts/nickel/restricted-graph-contracts.ncl`](../contracts/nickel/restricted-graph-contracts.ncl) defines two evaluated formats:

- `choregraph.evaluated-graph.v1` for direct graph data;
- `choregraph.lattice-workflow-projection.v1` for selected Lattice port graphs.

The contract checks exact fields, required schemas, fixed lifecycle events, array bounds, scalar types, import declarations, and selected graph shapes. Rust then parses with `deny_unknown_fields`, rechecks every numeric ceiling, validates the import closure, constructs `MachineSpec`, and calls normal `AdmittedMachine::admit`.

Passing Nickel export is not sufficient for admission.

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

## Capability-shell blocker

The required filesystem shell is not complete. `cap-root` has no published immutable repository source at the reviewed candidate revision. Its local policy requires a published revision and aligned Cargo and Nix identity before consumer adoption.

Choregraph will not use a sibling path, copy the path algorithm, or claim capability confinement without that gate. `cap-std` `4.0.2` remains the selected I/O mechanism after `cap-root` admission becomes available.

See [`../evidence/cap-root-adoption-blocker.ncl`](../evidence/cap-root-adoption-blocker.ncl).
