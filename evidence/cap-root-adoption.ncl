let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let RadicleRid = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^rad:z[1-9A-HJ-NP-Za-km-z]+$" value
) in
{
  adoption = {
    schema = "choregraph.cap-root-adoption.v1",
    observed_on = "2026-07-27",
    dependency = "cap-root-core",
    repository_rid | RadicleRid = "rad:z3cWBCK7VPbEmbL5MAvKJWKjCE745",
    repository = "rad://z3cWBCK7VPbEmbL5MAvKJWKjCE745",
    revision | GitRevision = "8991e2572578c16e590c2f1c0915db6c46c5d9cb",
    source_blake3 | Blake3 = "60fbdb7eae53dff8c2c0d41bc15aa123cd3a362ec1611eb4bc892fd6be58f3cb",
    license = "AGPL-3.0-or-later",
    public_repository_available = true,
    exact_revision_clone_verified = true,
    independent_replica_count = 0,
    cargo_source_admitted = true,
    nix_source_aligned = true,
    capability_shell_ready = true,
    vendor_path = "vendor/cap-root-core-selected",
    import_tool = "tools/sync-cap-root-core.rs",
    process_mechanism = {
      repository = "https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git",
      revision | GitRevision = "29dac88ecded94457572db3fdfaaaab95fa91525",
      role = "bounded evaluator process observation without sandbox authority",
    },
    non_claims = [
      "lexical admission is not filesystem confinement",
      "Radicle publication does not prove indefinite replication",
      "bounded process execution does not prove sandboxing",
      "Nickel evaluation does not prove source semantic equivalence",
    ],
  },
}
