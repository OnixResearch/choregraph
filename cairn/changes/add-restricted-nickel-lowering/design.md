## Context

Nickel contracts can validate evaluated shape. They do not prove that arbitrary Nickel evaluation preserves intended semantics. Choregraph therefore separates source evaluation, evaluated-value lowering, canonical admission, and formal machine proof.

## Success Contract

Completion requires typed contracts, a bounded evaluation profile, pure Rust revalidation, canonical lowering, capability-based I/O, exact identity bindings, and positive and negative fixtures. Passing Nickel export alone or hashing unvalidated output is false completion.

## Restricted Authoring Profile

The source profile permits data-oriented records, arrays, enums, bounded scalar values, contract application, and declared imports from an explicit capability root.

Each import has a safe relative path, media class, byte limit, and expected BLAKE3 identity. The complete import closure is recorded and bounded.

Network access, process execution, clocks, hidden environment reads, undeclared imports, ambient absolute paths, and secret discovery are excluded.

## Evaluation and Lowering

The CLI shell evaluates source through one pinned evaluator adapter. It records evaluator package, version, features, source identity, import-closure identity, and observed output identity.

The pure Rust lowerer receives an in-memory evaluated value. It revalidates schema version, all bounds, identifiers, edges, lifecycle profile, event declarations, and effect declarations. It then constructs the Choregraph canonical descriptor and calls normal machine admission.

No evaluator output can bypass Rust validation.

## Identity Chain

The translation bundle binds source bytes, declared import bytes, evaluator identity, evaluated output, lowerer source, canonical descriptor, admitted IR, selected proof set, and toolchain through BLAKE3 identities.

The bundle proves exact presented linkage only. It does not prove the evaluator produced the mathematically intended result.

## Capability Shell

`choregraph-cli` opens operator-declared source and output roots once at startup. `cap-root` admits graph-provided relative paths. `cap-std` performs descriptor-relative file access.

The graph can name a configured root identifier and a relative logical path. It cannot select a new ambient host root.

Symlink, no-follow, atomic-publication, permission, and race policy remain explicit shell responsibilities. Lexical admission alone is not reported as filesystem confinement.

## Diagnostics

The lowerer emits stable typed issue codes and bounded summaries. Safe output excludes raw source, imported values, credentials, environment values, and host paths.

## Validation

Positive fixtures include the Lattice diamond graph, explicit imports, minimal valid graphs, and source-order variation. Negative fixtures include malformed contracts, unknown fields, duplicate IDs, cycles, missing endpoints, excessive bounds, undeclared imports, hash mismatch, path escape, stale evaluator identity, and redaction failures.

## Claim Boundary

Passing frontend checks proves bounded contract, lowering, canonical admission, identity, and selected observation facts. It does not prove arbitrary Nickel semantics, evaluator soundness, source intent, runtime effects, or release eligibility.
