## Why

Lattice authors execution graphs in Nickel, but Choregraph proofs begin at admitted canonical machine IR. Choregraph needs a restricted, deterministic lowering boundary that binds source and evaluator observations without claiming arbitrary Nickel semantic equivalence.

The frontend must keep evaluator and filesystem effects outside the core. Rust must revalidate all evaluated data before it can become an admitted machine.

## What Changes

- Define typed Nickel contracts for bounded Choregraph and Lattice execution-graph source data.
- Define a restricted evaluation profile with declared content-addressed imports and no network, process, clock, or environment authority.
- Bind source, import closure, evaluator, evaluated output, lowerer, canonical IR, proof set, and toolchain identities.
- Implement a pure Rust lowerer from evaluated values into canonical machine IR.
- Implement a capability-based CLI shell using `cap-root` and `cap-std` for source and output paths.
- Add Lattice diamond and malformed Nickel fixtures with stable safe diagnostics.

## Success Criteria

- The pure lowerer rejects every value outside the selected contract or machine semantics.
- Canonical IR does not depend on source field order or host paths.
- Source and evaluator receipts identify the observed translation boundary.
- Evidence states that arbitrary Nickel source semantics are not proved.

## Non-Goals

- Verifying the Nickel evaluator, import implementation, or source-to-output semantic equivalence.
- Running Nickel in `choregraph-core`.
- Reading ambient files or exposing raw source, imported secrets, or host paths in safe diagnostics.

## Impact

- **Contracts**: typed Nickel graph and import-closure profiles.
- **Lowerer**: pure evaluated-value validation and canonical IR construction.
- **CLI**: capability-relative source loading, export, inspection, and receipt generation.
- **Fixtures**: positive Lattice diamond and negative contract, import, graph, bound, and diagnostic cases.
