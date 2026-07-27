let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Project = {
  name | NonEmpty,
  license | NonEmpty,
  status | NonEmpty,
  core | NonEmpty,
  first_consumer | NonEmpty,
  workspace_members | Array NonEmpty,
  excluded_authority | Array NonEmpty,
  claim_scope | NonEmpty,
} in
{
  project | Project = {
    name = "choregraph",
    license = "AGPL-3.0-or-later",
    status = "finite-trace-verified-indexed-kernel",
    core = "pure-no-std-plus-alloc-machine-boundary",
    first_consumer = "lattice-seaglass",
    workspace_members = [
      "choregraph-core",
      "choregraph-kamacite",
      "choregraph-nickel",
      "choregraph-seaglass",
      "choregraph-cli",
      "choregraph-foundation-check",
    ],
    excluded_authority = [
      "filesystem",
      "environment",
      "clock",
      "process",
      "network",
      "nickel-evaluation",
      "token-verification",
      "persistence",
      "receipt-issuance",
      "effect-execution",
    ],
    claim_scope = "named-single-step-and-finite-trace-properties-over-production-called-indexed-kernel",
  },
}
