let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
{
  source = {
    repository = "git@github.com:OnixResearch/kamacite.git",
    revision | GitRevision = "6245a52498f7de511df660eabec3f9b350338cb6",
    source_path = "crates/kamacite-core/src/preserves_codec.rs",
    source_blake3 | Blake3 = "bbb09dd44cae9aaac5226d7298633600480ff1e1776717b05138c422167056a0",
    apache_license_blake3 | Blake3 = "83cb3a2fcf829b6138e095b083016c34ddcdfa07b68d38782722c14fcf85ace6",
    mit_license_blake3 | Blake3 = "4b370111beabfcbb64bc16b8e84411a741280be5df1324f95f9014268f36c6af",
    license = "MIT OR Apache-2.0",
    adaptation = "none",
    import_tool = "tools/sync-kamacite-codec.rs",
    selected_scope = "canonical Preserves codec only",
    non_claims = [
      "not the complete Kamacite core",
      "not a transfer of Kamacite behavioral claims",
      "not authority to edit the vendored source directly",
    ],
  },
}
