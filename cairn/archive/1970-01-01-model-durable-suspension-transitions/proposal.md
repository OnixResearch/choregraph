## Why

Choregraph models deterministic graph admission, local lifecycle transitions, resume decisions, replay, and effect intents. Its current linear lifecycle has pending, ready, running, completed, failed, and cancelled states. It cannot represent a durable wait whose process exits and later resumes only after one exact external signal.

Lattice needs this state for generic manual approval gates and other bounded wait-for-signal workflows. Swamp's durable manual-gate behavior is a product reference: the process stops, persisted state remains suspended, and a later invocation resumes from recorded facts.

Choregraph will adapt only the pure lifecycle mechanism. It will not own approvers, identities, credentials, clocks, persistence, notifications, UI, or product authorization.

## What Changes

- Add an opt-in lifecycle profile that includes a non-terminal suspended state with bounded suspension class and reference identity. r[choregraph.machine.suspension_profile]
- Add deterministic suspend and resume-signal transitions that preserve denied state and emit no product effect intents. r[choregraph.machine.suspension_transition]
- Add resume decisions that keep suspended nodes non-runnable until one matching admitted signal is applied. r[choregraph.machine.resume_signal]
- Extend replay and canonical identity so suspension facts, signals, stale references, duplicates, and ordering remain deterministic. r[choregraph.machine.suspension_replay]
- Link executable invariants and Verus proof obligations to the production transition relation. r[choregraph.machine.suspension_proof]
- Keep product approval, authorization, timeout, persistence, and outcome interpretation outside Choregraph. r[choregraph.machine.suspension_boundary]

## Success Criteria

- Existing `LINEAR_LIFECYCLE_V1` behavior and canonical artifacts remain unchanged unless the new profile is selected explicitly.
- A valid suspend event moves only an admitted non-terminal node into suspended state and emits no product effect intent.
- A suspended node is never ready or executable before one matching resume signal.
- Missing, malformed, duplicate, stale, wrong-node, wrong-reference, and post-terminal signals fail closed.
- Replay of the same admitted trace produces the same state, decisions, diagnostics, and trace identity.
- The production `step` relation remains the single implementation used by tests, replay, adapters, and proofs.
- Documentation states that suspension conformance does not establish approval authority, persistence durability, signal truth, or downstream effect safety.

## False Completion

A Lattice-only flag that bypasses Choregraph lifecycle is not completion. A second suspension transition implementation used only by proofs or tests is not completion.

Treating suspended as running, ready, failed, cancelled, or completed loses the required durable state and is not completion.

## Impact

- **choregraph-core**: additive lifecycle profile, state and event data, transition cases, resume decisions, replay bindings, invariants, and tests.
- **Proofs**: production-linked single-step and finite-trace properties for suspend, matching resume, stale signal denial, terminal immutability, and no-effect denial.
- **Adapters**: explicit profile/version conversion with compatibility tests. Adapters do not infer suspension for v1 inputs.
- **Lattice**: later consumes an exact published revision and retains all product authority.
- **Testing**: positive and negative core, replay, no-std, adapter, proof, downstream, Cairn, Octet, and Nix rails.

## Out of Scope

- Human approval records, actor identity, UCAN or Basalt decisions, credential handling, notifications, expiration clocks, persistence engines, CLI commands, UI, and runtime execution.
- Automatic retry, process parking, external callbacks, remote workers, distributed consensus, or exactly-once signals.
- Claims for authorization correctness, signal truth, persistence durability, workflow correctness, effect safety, production readiness, or release eligibility.

## Dependencies

- Local Cairn change: `bind-concrete-effect-intents` must finish first so suspension extends the one production transition relation.

## Affected Specs

- `machine-semantics`: durable suspension profile, transitions, resume decisions, replay, proofs, and authority boundary.
