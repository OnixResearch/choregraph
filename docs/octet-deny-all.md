# Octet deny-all gate

r[impl choregraph.quality.octet.boundary]

Choregraph runs the Octet consumer suite at revision `374bd16b26cee2af34211a29bfa531c016811f51`.

## Commands

Run the focused source gate:

```text
nix build .#checks.x86_64-linux.octet-deny-all --no-link -L
```

Run the policy and source gates together:

```text
nix build .#checks.x86_64-linux.octet-deny-all-policy --no-link -L
nix build .#checks.x86_64-linux.octet-deny-all --no-link -L
```

`nix flake check -L` runs both gates.

## Scope and severity

`config/octet-deny-all.ncl` owns the reviewed lint list. The generated JSON must match it exactly.

The policy gate compares that list with the pinned Octet `hooks/octet-deny-all.sh`. Each name receives `-D` through `DYLINT_RUSTFLAGS`.

The consumer gate runs `--workspace --all-targets`. Cargo checks the declared Rust 1.96.0 compiler floor. `dylint.toml` cannot contain disabled lints or warning-level overrides.

Two bounded thresholds remain active:

- `excessive_file_length_max = 1100` keeps proof-linked and canonical codec sources together for direct review.
- `module_file_count_max = 9` covers the current responsibility-specific core and adapter layouts.

These thresholds do not change lint severity. A source that exceeds either limit is an error.

## Exception review

Use a source exception only when a proof, canonical codec, stable public interface, or explicit shell boundary requires the source shape.

Each exception must:

1. name only the required lint;
2. use the smallest practical source scope;
3. have a nearby reason comment and a Rust `reason` value;
4. keep `unjustified_allow` enabled;
5. avoid workspace-wide disabled lists and warning levels.

New source files do not inherit another file's exceptions. Review and remove an exception when its source constraint no longer applies.

## Non-claims

A clean Octet result is static source-lint evidence. It does not prove runtime correctness, Verus soundness, effect safety, authorization, or release eligibility.
