# Negative consumer validation — 2026-07-27

## Result

The maintained conformance tests fail closed for the selected graph, lifecycle, replay, effect, authority-result, source, receipt, and fallback failures.

## Failure matrix

| Failure class | Maintained case | Expected result |
|---|---|---|
| malformed graph | empty evaluated graph and parity admission corpus | malformed value or stable admission diagnostic |
| invalid lifecycle | completed diamond root cancellation | `TerminalState` with no state or intent |
| replay suffix | premature sink readiness after the accepted root prefix | first failure at event index 3, three events consumed |
| undeclared effect | effect request on the no-effect diamond root | `UndeclaredEffect` with no state or intent |
| authorization denial result | `Fail` after an accepted start intent | node becomes `Failed`; the prior intent is not returned again |
| stale source | translation receipts checked against changed source bytes | `StaleTranslationReceipt` |
| receipt mismatch | source and evaluator receipts from different identity chains | `StaleTranslationReceipt` |
| forbidden fallback | source-selected Nickel runtime import | `ForbiddenNickelAuthority` |

The replay failure preserves the last valid state. The root is completed, and the sink remains pending. `ReplayDecision` has no intent or dispatch field.

## Authority-result fixture

`fixtures/nickel/positive/effect-authority.ncl` declares one start effect. The checked evaluator output is stored at `fixtures/nickel/evaluated/effect-authority.json`.

The test confirms this sequence:

1. `MarkReady` emits no intent.
2. `Start` with the exact declared request emits one pure intent.
3. The consumer-owned authorization denial is represented by a `Fail` result event.
4. `Fail` moves the node to `Failed` and emits no intent.

This sequence does not claim that Choregraph authorized or executed the effect.

## Validation

These commands passed:

```text
nix develop -c cargo test -p choregraph-seaglass --test consumer-failures
nix develop -c cargo test -p choregraph-nickel
nix develop -c cargo clippy -p choregraph-seaglass --all-targets -- -D warnings
nix run .#octet-deny-all
```

All positive Nickel fixtures also passed contract evaluation and byte-for-byte normalized evaluated-fixture comparison. Octet reported zero findings, warnings, and errors.

## Claim boundary

These are bounded failure examples. They do not prove arbitrary malformed-input completeness, evaluator soundness, event truth, authorization policy correctness, external exactly-once behavior, or release eligibility.
