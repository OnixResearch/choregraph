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
    adaptation = "reviewed local producer-owned units/profile/Cow correction from the exact base",
    correction_path = "tools/kamacite-codec-correction.patch",
    correction_blake3 | Blake3 = "af29bedfead5c029c5c930675c9f41cf9856230f9429cc941f6700ae613f2b4d",
    corrected_source_blake3 | Blake3 = "6001e5509f44b1659d94d91a2a9c829df0492c0b734da79a485d7585b4d7ed20",
    corrected_tests_source_path = "crates/kamacite-core/src/preserves_codec/tests.rs",
    corrected_tests_destination = "preserves/tests.rs",
    corrected_tests_blake3 | Blake3 = "06ceb627b178d9029f7c1270d2a5696d3f0079aac7077283089192fbffd2d86a",
    correction_application = "pinned GNU patch --batch --forward --fuzz=0; exact base, patch and corrected-source BLAKE3 checks",
    publication_boundary = "local authorized correction only; no producer commit, publication, or canonical consumer pin change",
    import_tool = "tools/sync-kamacite-codec.rs",
    selected_scope = "canonical Preserves codec only",
    non_claims = [
      "not the complete Kamacite core",
      "not a transfer of Kamacite behavioral claims",
      "not authority to edit the vendored source directly",
      "corrected compiled source is not byte-identical to the original Git blob",
      "original MIT/Apache license files are retained source-identically; no new license grant is implied",
      "not a published producer revision or release qualification",
    ],
  },
}
