let SourceBaseline = import "../source-baseline-contract.ncl" in
{
  schema = "choregraph.source-baseline.v1",
  observed_on = "2026-07-27",
  sources = [
    {
      id = "missing-revision",
      role = "negative fixture",
      repository = "invalid",
      revision = "",
      source_scopes = ["src"],
      content_blake3 = "2573fd529650e74e7966988cf9b2cedeedc363485dfb45b4d18c6ff4086dabcf",
      hash_protocol = "blake3-file-bytes-v1",
      hash_scope = "negative fixture",
      license = "AGPL-3.0-or-later",
      acquisition_status = "invalid",
      imported = false,
      working_tree_used = false,
    },
  ],
  non_claims = ["negative fixture"],
} | SourceBaseline
