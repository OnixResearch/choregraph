let Contracts = import "../observation-contract.ncl" in
{
  observation | Contracts.Observation = {
  schema = "choregraph.negative-fixture-corpus.v1",
  status = 'observed,
  command = "workspace and Nix expected-failure checks",
  identity_basis = "maintained malformed, stale, skipped, mismatch, denial, and proof-failure fixtures",
  claim_scope = "selected negative cases only",
  },
}
