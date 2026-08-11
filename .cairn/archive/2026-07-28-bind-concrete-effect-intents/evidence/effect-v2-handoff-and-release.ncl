let Identity = {
  role | String,
  path | String,
  blake3 | String,
} in
let Evidence = {
  schema | String,
  recorded_on | String,
  handoff_schema | String,
  release_bundle_identity | String,
  publication_status | String,
  publication_blocker | String,
  identities | Array Identity,
  checked_commands | Array String,
  non_claims | Array String,
} in
{
  schema = "choregraph.effect-v2.handoff-release-evidence.v1",
  recorded_on = "2026-07-28",
  handoff_schema = "choregraph.lattice-concrete-intent-handoff.v1",
  release_bundle_identity = "e9318450dfbf245d8f05d5910ae00fc661c2694f870708891ac5eedb5a4b3027",
  publication_status = "blocked-not-authorized",
  publication_blocker = "No exact v2 source revision, source archive, publication candidate, or operator publication authority exists.",
  identities = [
    {
      role = "canonical-v2-machine",
      path = "fixtures/kamacite/positive/machine-v2.preserves",
      blake3 = "dc4b5ea207f8a0828784170d661c7fbd8aeb9d96661da6db9f301afca91ffb8d",
    },
    {
      role = "canonical-concrete-intent",
      path = "fixtures/kamacite/positive/concrete-intent-v2.preserves",
      blake3 = "f618ebe21809686b00d11b7b9d8f5f9e6aa65da5428e74646792d988a9c9b35d",
    },
    {
      role = "canonical-concrete-intent-receipt",
      path = "fixtures/kamacite/positive/concrete-intent.receipt.preserves",
      blake3 = "f6fc1aca137e639717f387e8b7f749b53c6615089226272b98f296c70d2cbfac",
    },
    {
      role = "disposable-lattice-handoff",
      path = "fixtures/lattice-handoff/consumer.rs",
      blake3 = "bf41a63f168fb32d7aa507fb1aa37eaf17220bc8de862221e2d42564d6f659f7",
    },
    {
      role = "lattice-handoff-positive-negative-test",
      path = "crates/choregraph-seaglass/tests/concrete-handoff.rs",
      blake3 = "9b14bf5ec3fcc0b4fdbe3b331737646af551e4ce161bbe5059161fc935559f71",
    },
    {
      role = "single-step-proof-manifest",
      path = "evidence/generated/single-step-proof.json",
      blake3 = "40dfb2bc3d6272b64cfa836c6570507c6c23a23e5498917e59c6c3ed16e5da2e",
    },
    {
      role = "trace-proof-manifest",
      path = "evidence/generated/trace-replay-proof.json",
      blake3 = "4a16885b969b2a62d857885cca9d049636a6fe734143b9e415c17463a6b57c99",
    },
    {
      role = "release-bundle-file",
      path = "evidence/release/generated/choregraph.release.bundle",
      blake3 = "07241a6af971b9951b330e3ebc214a1e9d714e10b96f4a1137f8d2d0e6e71cb5",
    },
  ],
  checked_commands = [
    "nix develop -c cargo test -p choregraph-core --all-targets",
    "nix develop -c cargo test -p choregraph-kamacite",
    "nix develop -c cargo test -p choregraph-nickel",
    "nix develop -c cargo test -p choregraph-seaglass",
    "nix develop -c octet-production-verus --crate-type=lib verus/proofs/single_step.rs",
    "nix develop -c octet-production-verus --crate-type=lib verus/proofs/trace_replay.rs",
    "choregraph verify-release-bundle",
  ],
  non_claims = [
    "The handoff does not prove Lattice authorization or persistence.",
    "The content identity is not a run, attempt, replay, or deduplication identity.",
    "The bundle does not grant publication authority.",
  ],
} | Evidence
