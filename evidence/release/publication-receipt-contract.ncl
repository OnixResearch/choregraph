let LowerHex40 =
  std.contract.from_predicate (fun value =>
    std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
  )
in
let LowerHex64 =
  std.contract.from_predicate (fun value =>
    std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
  )
in
let RadicleRid =
  std.contract.from_predicate (fun value =>
    std.is_string value && std.string.is_match "^rad:z[1-9A-HJ-NP-Za-km-z]+$" value
  )
in
{
  Receipt = {
    schema | String,
    status | [| 'published |],
    authority | [| 'operator_authorized |],
    candidate_blake3 | LowerHex64,
    source_revision | LowerHex40,
    source_archive_blake3 | LowerHex64,
    source_archive_bytes | Number,
    release_bundle_blake3 | LowerHex64,
    license | String,
    radicle_rid | RadicleRid,
    identity_revision | LowerHex40,
    delegates | Array String,
    threshold | Number,
    author_sigrefs | LowerHex40,
    signed_refs_feature | [| 'parent |],
    candidate_branch | String,
    native_replicas | Array String,
    https_url | String,
    https_revision | LowerHex40,
    https_archive_blake3 | LowerHex64,
    git_fsck | [| 'verified |],
    receive_pack | [| 'rejected |],
    unknown_rid | [| 'rejected |],
    non_claims | Array String,
  },
}
