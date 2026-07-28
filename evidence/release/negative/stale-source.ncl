let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "rust.source.v1",
  status = 'observed,
  command = "proof source-linkage checks",
  identity_basis = "deliberately stale source identity",
  claim_scope = "source identity only; release linkage must reject stale content",
  },
}
