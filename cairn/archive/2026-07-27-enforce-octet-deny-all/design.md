# Design

## Strict policy source

A typed Nickel file owns the reviewed Octet revision and complete lint-name list. Checked-in JSON gives Nix a deterministic evaluation input.

The native `dylint.toml` contains no disabled lints or warning-level overrides. It sets bounded file-count and file-length thresholds for co-located proof and canonical-codec sources. Findings beyond those thresholds remain errors.

## Nix gate

Nix builds `DYLINT_RUSTFLAGS` from the generated list. Each registered Octet lint and `unknown_lints` receives `-D`.

The Octet consumer check runs the full workspace with all targets. The check uses the pinned Octet flake input and Cargo lock file.

A separate policy check compares the local lint list with `hooks/octet-deny-all.sh` from the same pinned Octet source. It fails on missing, extra, duplicate, reordered, warning-level, or disabled entries.

## Remediation and exceptions

The rollout refactors the foundation checker, renames searchable source layouts, and fixes direct naming findings. Proof-linked, canonical-codec, stable-interface, and shell-boundary findings can use source-local exceptions.

Each exception names only required lints and includes a nearby reason. Workspace disabled lists, warning-level overrides, and unexplained suppressions remain forbidden.

## Tests

Positive checks validate the current complete list and run the full source lint gate.

Negative Nickel fixtures reject warning mode, an empty lint list, disabled lints, and duplicate names. Source lint failures must fail the Nix derivation.

## Boundaries

Octet findings are static source evidence. They do not prove runtime correctness, formal proof soundness, or release eligibility by themselves.
