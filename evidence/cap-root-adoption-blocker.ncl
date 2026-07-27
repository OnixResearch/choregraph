let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
{
  blocker = {
    schema = "choregraph.cap-root-adoption-blocker.v1",
    observed_on = "2026-07-27",
    dependency = "cap-root-core",
    baseline_reference_revision | GitRevision = "8991e2572578c16e590c2f1c0915db6c46c5d9cb",
    local_candidate_revision | GitRevision = "6003544ef57651414e779411ac3a6f2c804a9e56",
    public_repository = "https://github.com/OnixResearch/cap-root",
    public_repository_available = false,
    cargo_source_admitted = false,
    nix_source_aligned = false,
    capability_shell_ready = false,
    required_resolution = "publish an immutable reviewed cap-root revision and align Cargo and Nix source identity",
    prohibited_fallbacks = [
      "sibling path dependency",
      "unreviewed source copy",
      "ambient absolute path access",
      "lexical admission reported as filesystem confinement",
    ],
    unaffected_surfaces = [
      "pure evaluated-value contracts",
      "pure Rust lowering",
      "content-addressed import validation over supplied bytes",
      "translation identities and non-claim receipts",
    ],
  },
}
