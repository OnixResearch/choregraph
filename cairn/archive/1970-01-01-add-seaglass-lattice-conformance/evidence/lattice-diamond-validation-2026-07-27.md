# Lattice diamond validation — 2026-07-27

## Result

The reviewed Lattice diamond Nickel graph reached the expected terminal state through the production restricted-lowering and Choregraph machine paths.

The fixture declares no effects. Each accepted event therefore returned the expected empty intent sequence.

## Source-to-machine chain

The maintained Nickel check evaluates:

`fixtures/nickel/positive/lattice-diamond.ncl`

with:

`contracts/nickel/evaluated-graph-contract.ncl`

It normalizes the evaluator output and compares it byte-for-byte with:

`fixtures/nickel/evaluated/lattice-diamond.json`

The integration test includes those exact reviewed JSON bytes. It calls `choregraph_nickel::lower_evaluated_json` with the reviewed limits. It then advances the returned `AdmittedMachine` through production `step` calls.

## Expected decisions

The fixture checked these ready sets:

1. `lattice-node/1`
2. `lattice-node/2`, `lattice-node/3`
3. `lattice-node/3`
4. `lattice-node/4`
5. no ready nodes

Each node advanced through `Pending -> Ready -> Running -> Completed`. The final aggregate state contained four completed nodes.

Every accepted transition returned zero intents because the reviewed graph has no effect declarations.

## Negative decisions

The same admitted machine rejected:

- premature readiness for `lattice-node/4` with `DependenciesIncomplete`;
- an undeclared effect request with `UndeclaredEffect`;
- cancellation of the completed root with `TerminalState`.

Each denial returned no replacement state and no intent.

## Validation

These focused commands passed:

```text
nix develop -c cargo test -p choregraph-seaglass --test lattice-diamond
nix develop -c cargo clippy -p choregraph-seaglass --all-targets -- -D warnings
nix run .#octet-deny-all
nix develop -c nickel export --format json --apply-contract contracts/nickel/evaluated-graph-contract.ncl fixtures/nickel/positive/lattice-diamond.ncl
```

The normalized Nickel output matched the checked-in evaluated fixture. Octet reported zero findings, warnings, and errors.

A focused Nix derivation retry could not fetch the exact execution-graph Git source from `https://git.onix.computer`. No alternate source was used. The later full validation gate must retry this exact source.

## Claim boundary

This evidence checks one reviewed graph and finite event sequence. It does not prove arbitrary Nickel semantics, event truth, runtime authorization, external effect execution, Lattice product correctness, or release eligibility.
