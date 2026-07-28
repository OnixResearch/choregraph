let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "nix.flake-check-receipt.v1",
  status = 'passed,
  command = "nix flake check -L",
  identity_basis = "current exact flake inputs and maintained checks",
  claim_scope = "selected build and check observations only",
  },
}
