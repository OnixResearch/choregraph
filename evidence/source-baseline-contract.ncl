let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let Source = {
  id | NonEmpty,
  role | NonEmpty,
  repository | NonEmpty,
  revision | GitRevision,
  source_scopes | Array NonEmpty,
  content_blake3 | Blake3,
  hash_protocol | NonEmpty,
  hash_scope | NonEmpty,
  license | NonEmpty,
  acquisition_status | NonEmpty,
  imported | Bool,
  working_tree_used | Bool,
} in
{
  schema | NonEmpty,
  observed_on | NonEmpty,
  sources | Array Source,
  non_claims | Array NonEmpty,
}
