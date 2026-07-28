## Context

Choregraph's lock file must remain exact and reproducible. Its public dependency requirements need not force that exact patch version into every consumer graph. Cargo currently cannot select one BLAKE3 or Serde JSON version that satisfies Choregraph and Lattice. Choregraph also declares a higher package compiler floor than its pinned Octet integration compiler, even though an isolated check shows that the source compiles and reaches lint execution under Rust 1.96.

## Decisions

### Decision: Separate the package compiler floor from the release toolchain

**Choice:** Set workspace `rust-version` to 1.96.0, keep the release toolchain at 1.97.1, and remove `--ignore-rust-version` from Octet arguments.

**Rationale:** The compiler floor states source compatibility. The release toolchain states the reviewed release environment. Octet must compile the declared package without bypassing Cargo's version check.

### Decision: Keep reproducibility in the lock, not exact public patch requirements

**Choice:** Use compatible BLAKE3 and Serde JSON requirements while generating `Cargo.lock` with BLAKE3 1.8.5 and Serde JSON 1.0.151.

**Rationale:** Choregraph retains exact standalone builds through its lock. Consumers can select compatible patch releases without duplicate or unsatisfiable graphs.

### Decision: Guard the compatibility boundary in the pure foundation validator

**Choice:** Normalize dependency requirements from Cargo metadata. Reject exact requirements for BLAKE3 and Serde JSON in public Choregraph packages. Test accepted compatible requirements and rejected exact requirements without filesystem or process mocks.

**Rationale:** A pure validator gives deterministic positive and negative regression coverage. The shell remains responsible only for Cargo metadata and source reads.

## Risks / Trade-offs

- A future semver-compatible dependency release can change behavior. Choregraph's own lock and release gates still select reviewed exact versions.
- Lowering the compiler floor expands the supported build surface only to Rust 1.96. It does not change the reviewed Rust 1.97.1 release toolchain.
- Manifest and lock changes invalidate earlier release identities. A new immutable candidate requires separate publication authority.
