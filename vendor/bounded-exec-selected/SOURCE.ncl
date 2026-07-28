let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
{
  source = {
    repository_rid = "rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo",
    repository = "https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git",
    revision | GitRevision = "29dac88ecded94457572db3fdfaaaab95fa91525",
    root_manifest_blake3 | Blake3 = "20ff51e3f2c5f9b230e05b797b5dbb1029e738d5f84d25cecce02d49c2d3427e",
    core_manifest_blake3 | Blake3 = "a44db7a57873047d80a3c9c4ad33f24cc234f80e8654c36c162547314fd0b27e",
    core_source_blake3 | Blake3 = "6129cfe3f293f4b51ef2427c944a7e0ccd60b328bd3045fd13927fdb6b35c406",
    shell_manifest_blake3 | Blake3 = "40daaca455d83ea74c42bab2de7b3922b5bdc80524051caa5d74ef36d7b5e2a5",
    shell_source_blake3 | Blake3 = "f32e9808d24d7de774e96e4bd38115641be41483d8cac250c8b15b18cd06b078",
    license_blake3 | Blake3 = "ca0292789c0a0ecba706f4b3f3493007a9951731c1d432c52c4aac1f1abba8ef",
    license = "AGPL-3.0-or-later",
    adaptation = "none; selected workspace manifests, Rust sources, and license are byte-identical",
    import_tool = "tools/sync-bounded-exec.rs",
    selected_scope = "bounded evaluator process mechanics used by the CLI shell",
    non_claims = [
      "not filesystem or network confinement",
      "not evaluator correctness",
      "not runtime effect authority",
      "not authority to edit the selected files directly",
    ],
  },
}
