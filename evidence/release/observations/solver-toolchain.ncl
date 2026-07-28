let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "octet.solver-toolchain.v1",
  status = 'observed,
  command = "octet-production-verus --identity",
  identity_basis = "pinned solver identity in current proof manifests",
  claim_scope = "tool identity, not solver soundness",
  },
}
