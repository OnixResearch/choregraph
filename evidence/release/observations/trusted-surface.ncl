let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "choregraph.trusted-surface.v1",
  status = 'observed,
  command = "single-step and trace proof trust checks",
  identity_basis = "current generated proof trusted-surface fields and .cargo/config.toml",
  claim_scope = "declared trust surface, including RUSTC_BOOTSTRAP restricted to verified_logic for normal Cargo compilation; not tool soundness",
  },
}
