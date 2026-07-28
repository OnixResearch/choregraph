# Choregraph release validation — 2026-07-27

## Result

All maintained release gates pass for the local candidate and evidence overlay.

Publication remains blocked pending explicit operator action and an assigned Radicle RID.

## Rust gates

```text
nix develop -c cargo fmt --all -- --check
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo check -p choregraph-core -p choregraph-kamacite -p choregraph-nickel -p choregraph-release -p choregraph-seaglass --target wasm32-unknown-unknown
```

All commands passed. Workspace tests include bounded transition matrices, deterministic bundle order, positive and negative lifecycle behavior, canonical corpus freshness, Seaglass parity, Lattice diamond integration, stale-link rejection, and capability-shell failures.

## Verus gates

```text
nix build .#checks.x86_64-linux.single-step-verus .#checks.x86_64-linux.trace-replay-verus --no-link -L
```

Results:

```text
single-step: 79 verified, 0 errors
finite trace and replay: 65 verified, 0 errors
```

Both Nix checks also ran every classified negative Verus fixture and required its intended theorem failure.

## Nickel and release linkage

All positive release observation files and the publication candidate exported through their contracts.

The incomplete-toolchain and failed-gate observations failed their contracts as intended. The structurally valid stale-source case is rejected by Rust linkage checks.

Current release values:

```text
bundle-blake3=a2ac1336679542fa9573b356daddc18266fdda8147ddbe28f85aa37d135030d9
manifest-file-blake3=7eeb1983c3dc0e8be1f8e6450b28477f3f9de999537a019e9c22c498925e8f1f
candidate-blake3=e4b5cd7bd77d78c20cf8b96cd0c038144028d2287000757c8ab68f5f5dfdff1a
source-revision=fc47c5f4ecbb7b4341af690fa42199f25d57f54c
source-archive-blake3=dcb1faf95a7487145496ce986ae1639908600ace5bd77c4d96a37a530a7538a5
```

The bundle verifier reopened and rehashed 30 required artifacts. Candidate validation reopened and rehashed the source archive and rebuilt the release bundle.

## Adapter and downstream gates

```text
./tools/check-lattice-downstream.rs --self-test
./tools/check-lattice-downstream.rs ../lattice .
```

The script self-test passed. The exact disposable Lattice runner passed 21 tests with `source_fallback=none` at Lattice revision `c8d56ad83a57287064edb85f58808f53e4bf4536`.

## Octet, Cairn, and traceability

`nix run .#octet-deny-all` passed with zero findings.

Cairn repository validation and all three change gates passed. Traceability improved from 65 of 78 to 78 of 78 before synchronization. After release-spec synchronization, 88 of 88 accepted requirements have references, with no missing or dangling IDs.

## Nix

```text
nix flake check -L
```

The full flake check passed. It included formatting, workspace tests, policy checks, foundation boundaries, restricted Nickel, Kamacite, proof rails, source alignment, Octet, and conformance checks.

The configured remote builder and one optional cache were unavailable during some commands. Nix completed the required builds locally. This warning did not skip a maintained check.

## Claim boundary

These results prove the selected local checks and exact presented linkage. They do not prove tool soundness, arbitrary Nickel equivalence, event truth, authorization freshness, runtime effects, remote availability, consumer cutover, or whole-product correctness.
