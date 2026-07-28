# Lattice authority boundary

Choregraph returns pure machine decisions and declared effect intents. Lattice retains authorization, runtime, persistence, and product authority.

## Verified Choregraph boundary

The production `choregraph-seaglass` package has one production dependency: `choregraph-core`.

The maintained architecture test checks that the adapter:

- calls the required Choregraph admission, analytics, lifecycle, readiness, resume, transition, and cycle paths;
- contains no independent `while` or `loop` algorithm;
- contains no filesystem, network, process, clock, capability, Basalt, UCAN, Tokio, or command token;
- gains no additional production dependency.

Positive and negative tests prove that the gate accepts the reviewed adapter and rejects representative duplicate-loop, ambient-filesystem, extra-dependency, and malformed-manifest cases.

This is syntactic architecture evidence. It is not a semantic proof.

## Authority ownership

| Concern | Owner |
|---|---|
| immutable graph admission and machine decisions | Choregraph |
| declared pure effect intent | Choregraph |
| evaluator selection and product contract | Lattice |
| workflow, run, credential, and runtime identity | Lattice |
| Basalt policy selection and authorization request | Lattice |
| UCAN facts, caveats, revocation, and replay observations | Lattice shell and selected integrations |
| configured capability root handles | Lattice |
| `cap-root` lexical admission | Lattice effect adapter |
| `cap-std` and no-follow filesystem operations | Lattice effect adapter |
| bounded process and network execution | Lattice runtime adapters |
| attempt persistence and crash recovery | Lattice |
| retries, schedules, triggers, budgets, and approvals | Lattice |
| diagnostics, UI, and product evidence | Lattice |

## Intent handoff

A Choregraph intent is a request for consumer handling. It is not an authorization result or proof of an external effect.

The required Lattice sequence is:

1. Bind the exact machine, workflow, run, node, transition, operation, resource, ability, request, and payload identities.
2. Persist the intent and attempt identity before a product effect.
3. Obtain current external authorization facts.
4. Ask Basalt for a bounded decision over those supplied facts.
5. If Basalt denies the request, persist a denial and execute no product effect.
6. If Basalt allows the request, route it through the selected bounded Lattice adapter.
7. Record the observed outcome.
8. Submit one canonical result event to Choregraph.

The conformance authority fixture checks a bounded denial path. `Start` returns one declared intent. A later consumer-owned `Fail` result moves the node to `Failed` and does not return the prior intent again.

## Replay and crash boundary

Replay calls production `step`, discards any historical intent, and has no dispatch field. A prior intent or authorization receipt cannot become fresh authority.

Lattice must persist attempt identity before dispatch. A crash after an external effect but before outcome persistence remains an unknown outcome. Neither repository can claim external exactly-once behavior from this sequence.

## Source blocker

Basalt currently pins UCAN through authenticated SSH. Anonymous or public release adoption remains blocked until Lattice admits a public source or an equivalently reviewed vendored boundary.

## Claim boundary

This audit verifies the selected dependency and source boundaries plus bounded intent and denial fixtures. It does not prove Basalt policy correctness, UCAN cryptography, key truth, revocation freshness, operating-system confinement, event truth, external effect success, or Lattice release readiness.
