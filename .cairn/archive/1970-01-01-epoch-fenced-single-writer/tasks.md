## Machine invariant

- [x] [serial] Add the single-writer, epoch-fenced ownership invariant to machine semantics. r[choregraph.epoch_fence.invariant]
- [x] [serial] Implement the pure admit and reject decision from the current and offered entity epochs. r[choregraph.epoch_fence.decision]
- [x] [serial] Use the same decision implementation for production transitions and proof targets. r[choregraph.epoch_fence.proof]

## Verification and proof

- [x] [parallel] Add positive cases for current-epoch admission within one entity. r[choregraph.epoch_fence.verification]
- [x] [parallel] Add negative and boundary cases for stale and mismatched epochs, terminal mutations, cross-entity writes, and shared-store coupling. r[choregraph.epoch_fence.verification]
- [x] [serial] Add a Verus specification and proof for the invariants over a production transition. r[choregraph.epoch_fence.proof]
- [x] [serial] Run the focused foundation rail, then package, workspace, Clippy, Cairn, and Nix checks. r[choregraph.epoch_fence.verification]
