## Context

`choregraph-core` exposes one canonical machine IR and production transition relation. The current local states are `Pending`, `Ready`, `Running`, `Completed`, `Failed`, and `Cancelled`. Resume decisions execute non-terminal ready work, await dependencies, skip completed work, and reject failed or cancelled mutation.

A durable workflow gate has different semantics. Reaching the gate records a stable wait, the invoking process can exit, and later execution remains blocked until one exact external fact authorizes a resume transition. Choregraph can model that mechanism without interpreting the external fact as trustworthy approval.

## Success Contract

The accepted result is an explicit versioned lifecycle profile that adds suspension to the same production IR, `step`, invariant, replay, adapter, and proof path.

Completion evidence includes current positive and negative core tests, canonical fixture identity, production-linked Verus checks, downstream Lattice conformance, and unchanged v1 fixtures.

## Approach Registry

| Family | Mechanism | State |
| --- | --- | --- |
| Add a suspended local state | Extend an explicit opt-in lifecycle profile and production transition relation | selected |
| Treat suspension as running | Persist product wait data outside the machine while local state remains running | rejected |
| Add a product approval state | Encode approver, authority, timeout, and persistence semantics in Choregraph | rejected |
| Restart from pending | Reset the gate node and dependents after external approval | rejected |

The selected mechanism preserves the distinction between executable work and durable waiting while keeping product meaning outside the kernel.

## Decisions

### Decision: add an opt-in lifecycle profile

**Choice:** Add a new profile, provisionally `DURABLE_SUSPENSION_V1`, with the existing local states plus `Suspended`.

The profile declares supported transitions explicitly. Existing machine schemas and `LINEAR_LIFECYCLE_V1` canonical bytes remain stable. Importers must select the new profile by exact version. No adapter infers it from node names or product configuration.

**Rationale:** Adding suspension to the existing profile would change compatibility artifacts and permit new transitions for consumers that did not request them.

### Decision: keep suspension facts bounded and opaque

**Choice:** A suspension event contains the node ID, one bounded suspension class, and one canonical reference identity. Choregraph treats the class and reference as opaque typed data.

The reference can bind a product-owned approval request, callback token digest, or wait condition. Choregraph does not parse actors, policy, credentials, timestamps, reasons, or payloads.

Named limits constrain class and reference size before state growth.

**Rationale:** The kernel needs correlation, not product semantics.

### Decision: suspend and resume emit no product effects

**Choice:** A valid suspend event transitions an admitted eligible node to `Suspended` and emits no product effect intent.

A valid resume-signal event must name the same node, profile, suspension class, and reference. It transitions the node to the profile-defined resumption state without emitting a product effect intent. The initial target is `Ready` when dependencies remain complete.

A denied suspend or resume returns no replacement state and no intents.

**Rationale:** Persistence, notifications, approval recording, and later runtime dispatch remain shell and product responsibilities.

### Decision: suspended nodes are not runnable

**Choice:** Readiness excludes suspended nodes. `resume_decision` returns a new explicit wait decision for suspended state. It never returns `Execute` until the matching signal transition has updated the machine state.

Dependency completion does not release suspension. Duplicate resume facts after the first accepted transition are denied or classified as idempotent only when the exact trace semantics can preserve one deterministic outcome without weakening terminal immutability.

**Rationale:** A dependency-only resume would bypass the external gate.

### Decision: bind suspension into canonical replay

**Choice:** Canonical machine and trace forms bind the profile, node, suspension class, reference identity, prior state, signal identity, and resulting state.

Replay validates event order and returns the first exact denial. Wrong-node, wrong-reference, stale, duplicate, malformed, premature, and post-terminal signals are explicit negative classes.

A trace with a suspended node can end validly in suspended state. Replay does not require every trace to reach a terminal state.

**Rationale:** Durable waiting is a valid bounded observation, not an incomplete proof artifact by itself.

### Decision: extend production-linked proofs

**Choice:** Extend executable invariants and Verus specifications over the production transition code.

Proof obligations include:

- admitted suspension preserves `Invariant`;
- suspended nodes are not ready and emit no effects;
- matching resume preserves `Invariant` and allowed-effect policy;
- denied or mismatched signals preserve state and emit no effects;
- terminal nodes remain immutable;
- finite replay preserves the invariant for accepted traces that include suspension.

No proof-only transition function is allowed.

**Rationale:** Suspension changes the verified transition state space and must enter the same proof boundary.

### Decision: preserve product authority downstream

**Choice:** Choregraph validates only transition shape and reference equality. The consumer decides who can create a suspension request, who can signal resume, whether authority is current, how timeouts work, what state is persisted, and whether downstream effects are allowed.

Lattice must revalidate its product approval and persistence facts before it applies a Choregraph event.

**Rationale:** A matching opaque reference does not establish human intent, authorization, freshness, or persistence truth.

## Transition Sketch

```text
eligible non-terminal state + Suspend(class, ref)
  -> Suspended(class, ref), no intents

Suspended(class, ref) + ResumeSignal(class, ref)
  -> Ready, no intents

Suspended(class, ref) + mismatched signal
  -> Deny, no replacement state, no intents

terminal state + any suspend or resume signal
  -> Deny, no replacement state, no intents
```

The exact eligible source state and resumed target state remain profile-owned constants and explicit transition tables.

## Validation Strategy

Positive fixtures cover ready-to-suspended, running-to-suspended when admitted by the profile, valid trace ending suspended, matching resume, resumed readiness, and unchanged v1 behavior.

Negative fixtures cover unsupported profile, suspend from terminal, nested suspension, wrong class, wrong reference, wrong node, stale signal, malformed reference, size limit, duplicate signal, premature signal, changed dependencies, and product data leakage.

Downstream evidence uses one pinned Lattice consumer fixture. It proves conversion and expected decisions only, not approval behavior.

## Adversarial Audit

The audit must try to show that:

- a suspended node becomes ready from dependencies alone;
- an adapter infers the new profile for old data;
- a wrong reference can resume another node;
- duplicate resume can execute a node twice;
- replay rejects a valid terminal suspended trace;
- proof code diverges from production `step`;
- Choregraph output claims authority or persistence truth.

Any successful counterexample blocks publication.

## Risks and Trade-offs

- A new local state widens every exhaustive match. Compiler errors and full workspace tests help expose missed paths.
- Supporting suspension from `Running` can represent a post-dispatch wait but can also hide uncertain effects. The profile must admit source states explicitly.
- Idempotent duplicate signals are convenient but can obscure stale replay. The first slice can deny duplicates unless a stronger deterministic contract is proven.
- Downstream consumers need explicit migration and cannot assume v1 canonical identity remains valid under v2.

## Non-Claims

This design does not prove approver identity, authorization, signal truth, timeout correctness, process survival, persistence durability, notification delivery, credential safety, downstream effect safety, exactly-once execution, workflow success, production readiness, or release eligibility.
