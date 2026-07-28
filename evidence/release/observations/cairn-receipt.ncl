let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "cairn.validation-receipt.v1",
  status = 'passed,
  command = "cairn validate and release-change gates",
  identity_basis = "current Cairn policy, validation, and gate outputs",
  claim_scope = "lifecycle policy only",
  },
}
