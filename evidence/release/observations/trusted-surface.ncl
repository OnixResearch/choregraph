let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "choregraph.trusted-surface.v1",
  status = 'observed,
  command = "single-step and trace proof trust checks",
  identity_basis = "current generated proof trusted-surface fields",
  claim_scope = "declared trust surface, not tool soundness",
  },
}
