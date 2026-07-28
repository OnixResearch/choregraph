let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "choregraph.verus-toolchain-observation.v1",
  status = 'passed,
  command = "nix build .#checks.x86_64-linux.choregraph-single-step-verus --no-link -L",
  identity_basis = 1,
  claim_scope = "an incomplete toolchain identity cannot enter the release bundle",
  },
}
