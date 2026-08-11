# Exact v1 identity baseline before concrete-effect v2 implementation.
let Artifact = {
  role | String,
  path | String,
  blake3 | String,
} in
let Baseline = {
  schema | String,
  recorded_on | String,
  source_revision | String,
  algorithm | String,
  artifacts | Array Artifact,
} in
{
  schema = "choregraph.effect-v2.v1-baseline.v1",
  recorded_on = "2026-07-28",
  source_revision = "4e67de1c1c2479b397db10061d166005909405eb",
  algorithm = "blake3",
  artifacts = [
    {
      role = "machine-and-effect-fixture-source",
      path = "crates/choregraph-core/tests/machine-semantics.rs",
      blake3 = "078fc7eb5305ae75f0882be2009cf02c2a00e0d71a2a180ff3052524522fe15c",
    },
    {
      role = "replay-fixture-source",
      path = "crates/choregraph-core/tests/trace-replay.rs",
      blake3 = "cb1687418a1b758700b003026c3e066d069ef53d13c7f27100cbcddd35666a32",
    },
    {
      role = "preserves-machine-schema",
      path = "schemas/preserves/choregraph-machine-v1.preserves",
      blake3 = "4cb7079deb17e77fe2806bd24a80f47e5df1ca7c405725429a3ea47236ecb0f6",
    },
    {
      role = "canonical-preserves-machine",
      path = "fixtures/kamacite/positive/machine.preserves",
      blake3 = "c530e84f04d538ecc2149e9b147cc9ca1a4279811efc1ff5ffa54ad05be1e8b4",
    },
    {
      role = "single-step-proof-result",
      path = "evidence/generated/single-step-proof.json",
      blake3 = "ef2820597bf41f3d741c0baa1dfce64ba4211f4e51c34227926ef2c895e5dd24",
    },
    {
      role = "trace-proof-result",
      path = "evidence/generated/trace-replay-proof.json",
      blake3 = "84051015036cc037670d0e32570d7215e2773d0da47460f5d4ef2fa55216e6fd",
    },
    {
      role = "seaglass-parity-fixture-source",
      path = "crates/choregraph-seaglass/tests/parity.rs",
      blake3 = "50b4a82a676426a6327882399ec5e83d5e98d9064838f89b454f3fe09e721aa7",
    },
    {
      role = "lattice-diamond-evaluated-fixture",
      path = "fixtures/nickel/evaluated/lattice-diamond.json",
      blake3 = "7908dda09a596e6c48f1f76814eb29e4e559eb920af7b10789fb3c11b0b8f97c",
    },
    {
      role = "effect-authority-evaluated-fixture",
      path = "fixtures/nickel/evaluated/effect-authority.json",
      blake3 = "78e0e3c954d2b28b81674889eabde3df95e431feb4185b448e395aa4fb11527e",
    },
  ],
} | Baseline
