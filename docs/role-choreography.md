# Role choreography

Choregraph accepts a bounded role protocol after restricted Nickel evaluation. The pure `choregraph-choreography` crate revalidates the evaluated value.

The crate then calls the pinned Trellis production functions. It does not contain a second endpoint-projection algorithm.

## Data path

The data path has these stages:

1. Nickel checks the closed source shape.
2. Rust rejects unknown fields and values outside fixed limits.
3. Rust admits declared roles, external contracts, values, and initial owners.
4. Trellis admits each ownership state and each transfer.
5. Choregraph lowers the admitted tree to `verified_logic::GlobalChoreo`.
6. Trellis checks projectability and runs `project_endpoint` for each declared role.
7. Choregraph converts each Trellis local value into a canonical artifact body.
8. Kamacite encodes the global value, each local value, the complete local set, and the receipt as packed Preserves.
9. Domain-separated BLAKE3 identities bind all stages.

Projection is atomic. If one stage fails, the compiler emits no canonical local set.

## Fixed limits

The source must use the reviewed limits. A caller cannot widen them.

| Limit | Value |
| --- | ---: |
| roles | 64 |
| values | 256 |
| steps | 4,096 |
| branches | 256 |
| nesting depth | 128 |
| identifier bytes | 256 |
| imports | 128 |
| diagnostics | 128 |
| diagnostic text bytes | 512 |
| canonical output bytes | 1,048,576 |

## Role kinds

A native role has no external contract fields.

An external role has one immutable contract identity. It also has one input-schema identity and one output-schema identity. Each identity uses `blake3:<64 lowercase hexadecimal characters>`.

These fields are interchange data. They do not grant host, adapter, credential, transport, capability, or effect authority.

## Trellis dependency

The adapter pins Trellis revision `d65974b14978e1a44b0f5067beb9ef6d83317dc8`.

The selected source-set manifest has BLAKE3 identity `28bdef948c196d9ff84268a2028e90d7148e7950bcc4046489f5483b0421c99a`.

Trellis uses the unstable `register_tool` feature to retain verifier attributes in normal Cargo builds. Choregraph stays on Rust 1.97.1. `.cargo/config.toml` sets `RUSTC_BOOTSTRAP=verified_logic`, which limits the exception to the pinned dependency crate.

This exception does not apply to Choregraph crates. The foundation, Clippy, Octet, and Nix rails check the complete workspace.

## Diagnostics

A diagnostic contains a stable code, one primary source-relative span, ordered secondary spans, bounded notes, bounded hints, and an optional bounded replacement.

Safe output rejects absolute home paths and common secret-bearing fragments. It does not persist raw evaluator output, imported source, environment values, or credentials.

## Canonical artifacts

Packed Preserves bytes own artifact identity. JSON and text are review projections only.

The projection receipt binds these identities:

- source envelope;
- import closure;
- evaluator output;
- strict Rust DTO;
- global protocol;
- every local protocol;
- complete local set;
- Trellis revision, source set, and theorem set;
- lowerer;
- schema set;
- limits;
- diagnostics;
- toolchains; and
- required non-claims.

Valence-friendly evidence roles describe linkage only. They add no Valence runtime dependency or authority.

## Non-claims

A passing result does not prove or perform runtime execution, host placement, transport delivery, persistence, authorization, external-role correctness, exactly-once transfer, deadlock freedom under faults, production readiness, or release eligibility.

The Wyzer reference supplies authoring and diagnostic design ideas. Choregraph does not import Wyzer projection, networking, runtime, Perceus, compile-time execution, or C ABI code.
