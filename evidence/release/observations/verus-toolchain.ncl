let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "octet.verus-toolchain.v1",
  status = 'observed,
  command = "octet-production-verus --identity",
  identity_basis = "pinned verifier binary and closure identities",
  claim_scope = "tool identity, not verifier soundness",
  },
}
