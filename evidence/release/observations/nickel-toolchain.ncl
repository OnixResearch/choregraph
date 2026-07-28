let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "nickel.toolchain.v1",
  status = 'observed,
  command = "restricted-nickel-contracts Nix check",
  identity_basis = "selected evaluator closure and restricted profile",
  claim_scope = "evaluator observation, not Nickel equivalence",
  },
}
