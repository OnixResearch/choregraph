let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
{
  source = {
    repository_rid = "rad:z3cWBCK7VPbEmbL5MAvKJWKjCE745",
    repository = "rad://z3cWBCK7VPbEmbL5MAvKJWKjCE745",
    revision | GitRevision = "8991e2572578c16e590c2f1c0915db6c46c5d9cb",
    source_path = "crates/cap-root-core/src/lib.rs",
    source_blake3 | Blake3 = "60fbdb7eae53dff8c2c0d41bc15aa123cd3a362ec1611eb4bc892fd6be58f3cb",
    license_blake3 | Blake3 = "ca0292789c0a0ecba706f4b3f3493007a9951731c1d432c52c4aac1f1abba8ef",
    license = "AGPL-3.0-or-later",
    adaptation = "package manifest only; Rust source is byte-identical",
    import_tool = "tools/sync-cap-root-core.rs",
    selected_scope = "cap-root-core lexical admission crate",
    non_claims = [
      "not filesystem confinement",
      "not authority to select an ambient root",
      "not authority to edit the vendored Rust source directly",
      "Radicle publication does not prove indefinite replication",
    ],
  },
}
