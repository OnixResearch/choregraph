# Branchable history baseline

The implementation started from Choregraph `main` commit `11c704b0`.

The planning commit was rebased onto that commit before product edits. The working copy was clean before the rebase.

## Baseline command

```text
nix develop -c cargo test --workspace --all-targets
```

Result: pass.

The baseline ran all existing workspace tests. It included the machine, replay, proof-linked, WASM, adapter, Octet-support, and foundation test surfaces.

## Lifecycle baseline

The local policy snapshot comes from Cairn revision `695124d459574ba7aeba6097310d237f393c243c`.

Current Cairn rejected that older policy schema because `task_marker_policy.markers` is no longer valid. The change therefore uses the recorded Cairn revision for lifecycle validation.

This compatibility fact does not weaken the final Rust, Octet, traceability, or Nix gates.

## Authority baseline

Lattice owns redb persistence, run identity, scheduling, retries, effect handoff, and Bounded Exec use.

Animus owns session meaning, model context, compaction, interrupted-call repair, provider behavior, and session persistence.

Choregraph owns deterministic machine semantics and can own a separate product-neutral history mechanism. It does not gain consumer runtime authority.
