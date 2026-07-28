let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "rust.library-artifact.v1",
  status = 'passed,
  command = "nix flake check -L",
  identity_basis = "current Nix workspace build output",
  claim_scope = "built artifact observation, not semantic correctness",
  },
}
