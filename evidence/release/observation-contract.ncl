let Status = [| 'passed, 'observed, 'canonical, 'verified |] in
{
  Observation = {
    schema | String,
    status | Status,
    command | String,
    identity_basis | String,
    claim_scope | String,
  },
}
