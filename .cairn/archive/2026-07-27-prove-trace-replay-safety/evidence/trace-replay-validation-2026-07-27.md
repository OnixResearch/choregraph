# Trace replay validation — 2026-07-27

## Baseline

The pre-change workspace tests passed:

```text
nix develop -c cargo test --workspace
```

The pre-change single-step proof gate also passed:

```text
nix build .#checks.x86_64-linux.single-step-verus --no-link -L
```

## Production result

The public replay fold calls production `step` for each ordered event. It rejects over-limit and mismatched-binding inputs before it processes an event.

A denied event returns its exact index, the accepted count, the denial reason, and the last valid state. A complete replay returns only final state and count.

`ReplayDecision` has no effect-intent field. Accepted historical intents are validated by `step` and then discarded.

## Formal result

The production-linked trace proof reported:

```text
verification results:: 65 verified, 0 errors
```

The rail proves:

- explicit trace admission respects the selected limit and binding matches;
- empty replay preserves the initial state;
- accepted prefix length stays bounded;
- finite replay preserves the indexed invariant;
- equal indexed inputs, limits, and binding-match facts produce equal decisions;
- failure identifies the first denied event and last valid prefix state;
- replay returns no dispatch-intent sequence.

Four expected-failure proof fixtures failed with their named postconditions:

- `empty_trace_mutation.rs`;
- `failure_index_skips_denial.rs`;
- `preservation_without_initial_invariant.rs`;
- `replay_with_dispatch_intent.rs`.

The proof source contains no project `external_body`, `assume_specification`, unrestricted `assume`, or axiom.

## Executable coverage

The replay tests cover these positive cases:

- empty trace;
- linear completion;
- diamond completion;
- terminal cancellation;
- a trace exactly at its selected limit;
- deterministic equal inputs;
- an effectful transition with suppressed replay output.

They also cover these negative cases:

- malformed node identity;
- unknown node;
- invalid transition;
- terminal mutation;
- dependency reorder;
- oversized trace;
- stale machine binding;
- stale initial-state binding;
- invalid initial state;
- denied middle event with an ignored suffix.

Generated-prefix properties compare replay output with repeated production `step` calls.

## Search and audit record

The public-step-fold approach survived because it uses the exported production relation and cannot return intents.

The indexed-proof-fold approach survived because it recursively applies the production-called indexed kernel. It does not copy the lifecycle table.

The state-sequence-certificate approach was rejected. It adds caller-supplied intermediate states and a second mismatch surface.

The adversarial review found one determinism gap in the first draft. The formal input model now includes explicit limits and binding-match facts. A boundary test now covers a trace exactly at its limit.

VibeThinker supplied advisory audit checks. Repository tests and proof tools remained authoritative.

## Final commands

These commands passed:

```text
nix develop -c cargo fmt --all -- --check
nix develop -c cargo test --workspace --all-targets
nix develop -c cargo clippy --workspace --all-targets -- --deny warnings
nix develop -c cargo check -p choregraph-core --target wasm32-unknown-unknown
nix develop -c cargo check -p choregraph-core --target thumbv7em-none-eabihf
nix develop -c cargo run -p choregraph-foundation-check -- --root .
nix build .#checks.x86_64-linux.octet-deny-all --no-link -L
nix build .#checks.x86_64-linux.single-step-verus --no-link -L
nix build .#checks.x86_64-linux.trace-replay-verus --no-link -L
nix run path:../cairn#cairn -- validate --root .
nix run path:../cairn#cairn -- gate proposal prove-trace-replay-safety --root .
nix run path:../cairn#cairn -- gate design prove-trace-replay-safety --root .
nix run path:../cairn#cairn -- gate tasks prove-trace-replay-safety --root .
nix flake check -L
```

The Octet consumer reported zero findings, warnings, and errors. The full flake ended with `all checks passed!`.

The unavailable Spectrum cache and remote Nix builder produced transient warnings. Local derivations completed successfully.

## Claim boundary

This result proves finite indexed replay safety under the recorded assumptions. It does not prove event truth, history completeness, storage durability, liveness, authorization, effect success, external exactly-once behavior, or consumer binding derivation.
