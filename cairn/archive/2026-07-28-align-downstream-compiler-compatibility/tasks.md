## Phase 1: Compatibility contract

- [x] [serial] Record the passing workspace baseline and isolated downstream failures. r[choregraph.foundation.consumer_compatibility]
- [x] [serial] Declare the tested Rust 1.96.0 compiler floor and remove the Octet version bypass. r[choregraph.foundation.compiler_floor]
- [x] [serial] Relax public BLAKE3 and Serde JSON patch requirements while retaining a generated exact lock. r[choregraph.foundation.consumer_compatibility]

## Phase 2: Regression coverage

- [x] [serial] Add pure positive and negative foundation tests for the compiler floor and public dependency requirements. r[choregraph.foundation.consumer_compatibility]
- [x] [serial] Validate Choregraph with workspace tests, Clippy, foundation checks, Octet, and Nix. r[choregraph.foundation.compiler_floor]
- [x] [serial] Validate Lattice resolution with `choregraph-nickel`, BLAKE3 1.8.5, and Serde JSON 1.0.151. r[choregraph.foundation.consumer_compatibility]

## Phase 3: Evidence and lifecycle

- [x] [serial] Regenerate identity-sensitive release evidence for the accepted source revision. r[choregraph.foundation.consumer_compatibility]
- [x] [serial] Validate Cairn gates, sync accepted requirements, and archive the completed change. r[choregraph.foundation.consumer_compatibility]
