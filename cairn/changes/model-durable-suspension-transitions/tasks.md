## Phase 1: Baseline and dependency

- [x] [depends:bind-concrete-effect-intents] S1 Begin transition work only after concrete effect intents use the one production `step` relation and its current proof links. r[choregraph.machine.suspension_profile]
- [x] [serial] S2 Run and record current core, lifecycle, resume, replay, canonical corpus, adapter, no-std, Verus, downstream, Octet, Cairn, and Nix baselines. r[choregraph.machine.suspension_proof]
- [x] [serial] S3 Inventory every exhaustive local-state, lifecycle-profile, transition, readiness, resume, replay, serialization, adapter, fixture, proof, and report match. r[choregraph.machine.suspension_profile] r[choregraph.machine.suspension_replay]
- [x] [parallel] S4 Record the Swamp durable suspension reference behavior, AGPL clean-room boundary, adopt/reject decisions, and no runtime dependency. r[choregraph.machine.suspension_boundary]

## Phase 2: Profile and canonical model

- [ ] [serial] M1 Add an explicit versioned durable-suspension lifecycle profile while preserving current v1 canonical bytes and decisions. r[choregraph.machine.suspension_profile]
- [ ] [serial] M2 Add bounded suspension class and canonical reference data with named limits and no product actor, policy, credential, clock, or payload fields. r[choregraph.machine.suspension_profile] r[choregraph.machine.suspension_boundary]
- [ ] [serial] M3 Extend canonical machine, state, event, trace, diagnostic, and identity forms for exact suspension facts without display-order dependence. r[choregraph.machine.suspension_replay]
- [ ] [parallel] M4 Add positive and negative admission fixtures for explicit profile selection, unsupported profiles, malformed references, duplicate fields, missing limits, and v1 compatibility. r[choregraph.machine.suspension_profile]

## Phase 3: Production transitions and resume decisions

- [ ] [serial] T1 Extend the one production transition relation with admitted suspend and exact matching resume-signal cases that emit no product effects. r[choregraph.machine.suspension_transition]
- [ ] [serial] T2 Deny terminal, nested, premature, wrong-node, wrong-class, wrong-reference, stale, duplicate, malformed, and oversized suspend or resume cases without replacement state or intents. r[choregraph.machine.suspension_transition] r[choregraph.machine.resume_signal]
- [ ] [serial] T3 Exclude suspended nodes from readiness and add an explicit wait decision to `resume_decision` until a matching signal transition changes state. r[choregraph.machine.resume_signal]
- [ ] [parallel] T4 Add exhaustive positive and negative transition, readiness, dependency, terminal-immutability, and no-effect tests through production entrypoints. r[choregraph.machine.suspension_transition] r[choregraph.machine.resume_signal]

## Phase 4: Replay and proof

- [ ] [serial] R1 Extend bounded replay and first-denial reporting for valid suspended traces, matching resume, stale signals, duplicate signals, and traces that end suspended. r[choregraph.machine.suspension_replay]
- [ ] [parallel] R2 Add reordered-input determinism, trace-binding, canonical-corpus, malformed-event, limit, and v1 identity regression fixtures. r[choregraph.machine.suspension_replay]
- [ ] [serial] P1 Extend executable invariants over the production state and transition types for suspended state, non-readiness, matching resume, terminal immutability, and no-effect denial. r[choregraph.machine.suspension_proof]
- [ ] [serial] P2 Add production-linked Verus single-step lemmas and finite-trace proof coverage for accepted and denied suspension cases. r[choregraph.machine.suspension_proof]
- [ ] [parallel] P3 Add negative proof and linkage fixtures that fail for duplicate transition logic, missing suspended cases, unrestricted assumptions, stale proof identity, or unexecuted proof receipts. r[choregraph.machine.suspension_proof]

## Phase 5: Adapters and downstream boundary

- [ ] [serial] A1 Update Nickel and compatibility adapters to select the new profile explicitly and reject inference, unsupported versions, lossy conversion, or product approval fields. r[choregraph.machine.suspension_profile] r[choregraph.machine.suspension_boundary]
- [ ] [serial] A2 Add a pinned Lattice downstream fixture that converts a generic wait node and observes suspended, wait, matching resume, and mismatched denial decisions. r[choregraph.machine.suspension_replay]
- [ ] [parallel] A3 Add boundary tests that reject approver, policy, credential, clock, persistence, notification, and execution authority in Choregraph input or claims. r[choregraph.machine.suspension_boundary]
- [ ] [serial] A4 Document the profile, transition table, resume decision, replay behavior, migration, proof boundary, and explicit product non-claims. r[choregraph.machine.suspension_boundary]

## Phase 6: Closeout

- [ ] [serial] C1 Run workspace tests, Clippy, foundation rail, no-std targets, canonical corpus, Verus, downstream checks, Octet deny-all, Cairn validation and gates, and `nix flake check -L`. r[choregraph.machine.suspension_proof]
- [ ] [serial] C2 Review dry-run sync and archive plans, make sure that every requirement reaches machine-semantics, and archive only after current positive and negative evidence passes. r[choregraph.machine.suspension_replay] r[choregraph.machine.suspension_boundary]
