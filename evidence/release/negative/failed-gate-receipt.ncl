let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "choregraph.nix-receipt-observation.v1",
  status = 'failed,
  command = "nix flake check -L",
  identity_basis = "deliberately failed gate result",
  claim_scope = "a failed gate cannot satisfy a required passed release role",
  },
}
