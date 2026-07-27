## Context

The single-step theorem provides the induction step. Trace safety still needs an explicit base case, a bounded fold, and replay semantics that do not expose historical intents as fresh work.

## Success Contract

Completion requires a production replay function, inductive trace theorem, deterministic indexed failures, suppression of dispatchable intents, and positive and negative checks. Reusing ordinary `step` output as a runtime dispatch queue during replay is false completion.

## Trace Model

A trace is a bounded ordered sequence of canonical events. The caller supplies an admitted machine, an invariant initial state, and explicit replay limits.

`ReplayDecision` returns either a final state and consumed-event count or a failure containing the first invalid event index, stable reason, and last valid state. It does not return effect intents.

The replay core may use the same transition relation internally. It discards intent output after checking transition validity. This preserves one implementation while preventing replay from granting dispatch authority.

## Proof Shape

The base theorem proves that an invariant initial state is safe for an empty trace.

The induction step applies single-step preservation to the next accepted event. A denied event terminates replay at that index and leaves the prior state unchanged.

The main theorem states that successful replay of a bounded trace from a well-formed machine and invariant initial state produces an invariant final state.

Supporting lemmas cover prefix length, first-failure position, deterministic fold order, and state coverage.

## Runtime Boundary

Choregraph does not read event logs or persist replay output. Consumer shells load bounded event data, validate canonical identities, call replay, and decide product recovery policy.

A historical effect intent is evidence of a prior decision only. It is not a new Basalt authorization, capability grant, or execution request.

## Validation

Positive tests cover empty, linear, diamond, valid terminal, and denied-ending traces. Negative tests cover malformed events, unknown nodes, invalid transitions, terminal mutation, oversized traces, stale machine identity, and reordered events.

Property tests compare replay with repeated `step` state results while asserting that replay returns no dispatchable intents.

## Claim Boundary

Passing trace proofs establishes finite trace safety under stated inputs and assumptions. It does not prove event truth, history completeness, storage durability, runtime effects, liveness, or external exactly-once behavior.
