## Tasks

- [x] [serial] Record the exact execution-graph, Seaglass, Lattice, Trellis, Kamacite, Basalt, and cap-root source baselines before implementation. r[choregraph.foundation.provenance]
- [x] [serial] Create the Rust workspace, crate manifests, root license, README, local agent rules, and typed Nickel project profile. r[choregraph.foundation.workspace] r[choregraph.foundation.license]
- [x] [depends:workspace-layout] Configure the pinned Rust, Verus, Nickel, and Nix development surfaces without manually editing `flake.lock`. r[choregraph.foundation.toolchain]
- [x] [parallel] Add positive workspace-boundary checks for allowed core and adapter dependency edges. r[choregraph.foundation.boundaries]
- [x] [parallel] Add negative checks that reject I/O, evaluator, runtime, receipt, policy, and ambient sibling release dependencies from the core. r[choregraph.foundation.boundaries]
- [x] [serial] Document the single-implementation migration and the future `execution-graph-core` compatibility facade. r[choregraph.foundation.single_implementation]
- [x] [serial] Run repository metadata, license, provenance, Cargo, Nix, and Cairn checks and record the unverified foundation boundary. r[choregraph.foundation.validation]
