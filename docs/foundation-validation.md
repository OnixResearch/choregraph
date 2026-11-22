# Foundation validation

Run these commands from the repository root.

The first four commands below and `nix flake check -L` also run in the
**Foundation qualification** GitHub workflow on pull requests and pushes to
`main`. CI uses locked Cargo resolution, read-only repository permissions, and
does not need a deploy key. The Trellis dependency retains its pinned revision
and is acquired over anonymous HTTPS by both Cargo and Nix. A runner-local SSH
rewrite is not sufficient for Nix's sandboxed fetcher.

```text
nix develop -c cargo fmt --all -- --check
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- --deny warnings
nix develop -c cargo check -p choregraph-core --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-history -p choregraph-history-machine -p choregraph-weave --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-core --target thumbv7em-none-eabihf
nix develop -c cargo run -p choregraph-foundation-check -- --root .
nix run path:../cairn#cairn -- validate --root .
nix flake check -L
```

Positive Nickel fixtures describe the accepted workspace direction. Negative fixtures cover I/O, evaluator, runtime, receipt, policy, path, source, and reverse-edge failures.

The workspace declares Rust 1.96.0 as its tested compiler floor. The reviewed release toolchain remains Rust 1.97.1. The Octet gate checks the compiler floor without `--ignore-rust-version`.

The foundation validator rejects exact BLAKE3 and Serde JSON patch requirements in consumer library packages. `Cargo.lock` still selects exact reviewed versions for standalone builds.

The source baseline check compares a fresh Nickel export with the checked-in JSON. It also validates the checked-in BLAKE3 value.

## Verus source and verifier evidence

The three Nix Verus checks run the pinned verifier and their negative controls,
then audit the positive proof closure. The audit reads Rust/Verus token trees:
comments and literals cannot stand in for a declaration, attribute, or module
edge. Local proof modules must use reviewed literal paths under
`crates/choregraph-core/src/verified*`. The allowed proof syntax has no
`verifier::*` attributes; the trigger annotations in use control SMT
instantiation, not trust. Local macro definitions, unsupported macro
invocations, verifier assumption/declaration escapes, and conditional proof
items are rejected. Executable functions must be in `verus!` and declare an
`ensures` clause. The only non-Verus function scope in the checked subtree is
the explicit `#[cfg(test)] mod tests` in `verified.rs` and `verified/topology.rs`;
those helpers are test infrastructure and are not credited as proof items.
`extern crate alloc`, standard data derives, and the reviewed proof imports
support data and specification syntax rather than introduce trusted contracts.

`tools/choregraph-foundation-check/proof-policy.json` separately pins the local
module edges and function inventory and maps each advertised theorem label to
the fully qualified executable or proof item(s) in its rail. The positive rail's
source paths each need exactly one manifest digest. The manifests keep
`theorems` (unique labels matching this map) and `verified_count` (the exact
positive integer from the pinned verifier's unique zero-error summary). Both
fields are under `.proof` for single-step and trace-replay and at the root for
epoch-fence. A hash re-pin alone cannot change the source policy or compensate
for omitted checked items. Any intentional module, item, or claim-set change
requires reviewing the independent policy diff as well as regenerating evidence.

To inspect a Verus check manually, use the manifest exported for that rail and
the stdout from its positive verifier run:

```sh
python3 -B tools/choregraph-foundation-check/qualification.py --root . \
  proof single-step --manifest evidence/generated/single-step-proof.json \
  --stdout /path/to/positive.stdout
```

This reconciles source item identity and the aggregate verifier result. It is
not a per-item solver attestation, an independent proof-soundness check, or a
proof about the unverified Rust adapters. Any remaining production text probes
are smoke checks, not semantic guarantees.

## Octet exemption ratchet

```sh
python3 -B tools/choregraph-foundation-check/qualification.py --root . octet
```

`octet-deny-all-policy` scans first-party Rust under `crates`, `tools`, and
`verus/proofs`, including tests and build scripts. It checks source
`allow`, `expect`, and `warn` for the pinned Octet lints and `unknown_lints`,
including lint attributes nested in `cfg_attr`. Its baseline records exact
counts by file, scope, lint level, and lint name; widening an item suppression
to the file or changing a level is not grandfathered. Blanket allowances such
as `allow(warnings)` are rejected. If a suppression is removed, lower the
corresponding baseline count in the same change. Any increase needs a separate
review of the policy diff and the source justification, not a regenerated
baseline accepted on trust.

The reviewed inventory scanned 212 Rust files: 90 had exemptions. It recorded
536 file-scope lint occurrences across 82 files and 90 item/block occurrences
across 20 files. Outside the Verus source files alone, 58 files accounted for
330 file-scope occurrences and 19 accounted for 88 item/block occurrences.
These are attribute occurrences per lint, not a count of disabled diagnostics.
Start cleanup with broad `non_trait_imports` and `assertion_density`
exceptions outside proof code: qualify imports and narrow justified
assertion exceptions to the relevant tests. Then replace repeated arithmetic
or collection-growth suppressions with checked arithmetic and explicit bounds.
Remove the companion `unknown_lints` exemption when a file no longer needs
Octet-specific suppressions.

The workspace checks do not prove graph semantics, lowering correctness,
proof soundness, runtime behavior, or release eligibility.
