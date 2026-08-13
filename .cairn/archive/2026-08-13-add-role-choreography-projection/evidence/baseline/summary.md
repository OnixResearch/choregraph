# Pre-change baseline

The isolated workspace used `main@rad` revision `0809c9f10ba4` as its baseline.

## Results

- Workspace tests passed before product changes. The log covers the core, Nickel, Kamacite, release, CLI, WASM, compatibility, and foundation packages.
- Workspace Clippy passed with `--workspace --all-targets -- --deny warnings` after disabling incremental compilation and using one isolated target.
- `choregraph-foundation-check --root .` passed.
- The direct Trellis dependency spike compiled and called the selected production projectability and projection functions.
- Cairn proposal, design, tasks, and repository validation gates passed with `/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json`.
- The initial `nix flake check path:$PWD -L` reached the Octet check evaluation. It failed because the mutable path changed during evaluation. This failure is retained as baseline evidence and does not count as final acceptance.
- The initial workspace had no role-choreography Nickel profile, Kamacite artifact profile, proof linkage, or release linkage. The change adds those surfaces without modifying the existing machine proof claim or release authority.

The files in this directory contain exact command output. Final acceptance uses fresh isolated targets and a stable source path.
