let LowerHex40 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let LowerHex64 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
{
  Candidate = {
    schema | String,
    candidate_blake3 | LowerHex64,
    source_revision | LowerHex40,
    source_archive_blake3 | LowerHex64,
    source_archive_bytes | Number,
    license | String,
    packages | Array String,
    acquisition_adapter | String,
    radicle_rid | [| 'unassigned |],
    release_bundle_blake3 | LowerHex64,
    rollback_revision | LowerHex40,
    publication_authority | [| 'none |],
    publication_status | [| 'not_published |],
  },
}
