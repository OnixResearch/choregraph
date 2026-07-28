let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "octet.validation-receipt.v1",
  status = 'passed,
  command = "nix run .#octet-deny-all",
  identity_basis = "current deny-all config and profile identities",
  claim_scope = "selected source-policy checks only",
  },
}
