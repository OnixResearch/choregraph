let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
{
  source = {
    repository_rid = "rad:z2oYsb9jGTyp68BKYhzpivY1eK58a",
    repository = "https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git",
    revision | GitRevision = "03736f1ec46c377ff86b451260ad68aa70ff3b0b",
    manifest_path = "Cargo.toml",
    manifest_blake3 | Blake3 = "11e2b2a660ab68108c382ba354ec985685615d6dc0e24847887cd3f275307654",
    source_path = "src/lib.rs",
    source_blake3 | Blake3 = "91450b94f6256c71057507b6ae97f63e505dc57377198be38087530bedefccb4",
    license_path = "LICENSE",
    license_blake3 | Blake3 = "ca0292789c0a0ecba706f4b3f3493007a9951731c1d432c52c4aac1f1abba8ef",
    license = "AGPL-3.0-or-later",
    adaptation = "none; manifest, Rust source, and license are byte-identical",
    import_tool = "tools/sync-execution-graph-oracle.rs",
    selected_scope = "test-only immutable execution-graph parity oracle",
    non_claims = [
      "not a production Choregraph dependency",
      "not an execution-graph cutover source",
      "not a runtime or dependency fallback",
      "not authority to edit the selected files directly",
    ],
  },
}
