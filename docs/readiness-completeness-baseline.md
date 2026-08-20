# Readiness completeness baseline

<!-- r[impl choregraph.proof.single_step.readiness_completeness_baseline] -->
<!-- r[verify choregraph.proof.single_step.readiness_completeness_baseline] -->

This baseline uses Choregraph parent revision `1dc26fe0a90336ee6a6774ae1af458f25bd8b36f`.

The baseline was recorded before production readiness changed.

## Focused results

The maintained `single-step-verus` Nix check passed.

The focused `choregraph-core` readiness tests passed. They covered root readiness, incomplete dependencies, and suspended nodes.

The two `choregraph-seaglass` Lattice diamond tests passed.

The downstream self-test and Lattice substitution test passed. The substitution ran 21 runner tests at Lattice revision `c8d56ad83a57287064edb85f58808f53e4bf4536`.

The downstream result reported `adapter=choregraph-seaglass` and `source_fallback=none`.

## Baseline behavior

The admitted diamond initially reported only `source` as ready.

After `source` completed, the frontier reported `left` and `right` in admitted topological order.

The compatibility adapter delegated readiness to `choregraph_core::ready_nodes`.

The proof manifest named `readiness_is_sound`. It did not name a completeness or exact frontier theorem.

## Baseline commands

```text
nix build .#checks.x86_64-linux.single-step-verus --no-link -L
nix develop -c cargo test -p choregraph-core ready
nix develop -c cargo test -p choregraph-seaglass --test lattice-diamond
./tools/check-lattice-downstream.rs --self-test
./tools/check-lattice-downstream.rs ../lattice .
```

## Boundary

These results establish selected pure and compatibility behavior only.

They do not prove runtime scheduling, capacity, fairness, authorization, persistence, effect success, or release eligibility.
