let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "rust.source.v1",
  status = 'observed,
  command = "proof source-linkage checks",
  identity_basis = "all production-linked core source paths in current proof manifests",
  claim_scope = "source identity only",
  },
}
