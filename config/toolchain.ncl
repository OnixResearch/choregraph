let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
# r[impl choregraph.proof.single_step.toolchain]
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3Digest = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let Toolchain = {
  rust_channel | NonEmpty,
  rust_targets | Array NonEmpty,
  nixpkgs_revision | GitRevision,
  flake_utils_revision | GitRevision,
  rust_overlay_revision | GitRevision,
  crane_revision | GitRevision,
  octet_revision | GitRevision,
  verus_package | NonEmpty,
  verus_release | NonEmpty,
  verus_source_revision | GitRevision,
  verus_binary_blake3 | Blake3Digest,
  verus_rust_toolchain | NonEmpty,
  verus_rustc_blake3 | Blake3Digest,
  verus_solver | NonEmpty,
  verus_solver_blake3 | Blake3Digest,
  verus_nix_closure_graph_blake3 | Blake3Digest,
  vstd_crate | NonEmpty,
  trellis_revision | GitRevision,
  trellis_topo_sort_blake3 | Blake3Digest,
  trellis_job_state_blake3 | Blake3Digest,
  trellis_job_dag_blake3 | Blake3Digest,
  nickel_provider | NonEmpty,
  nix_provider | NonEmpty,
  claim_scope | NonEmpty,
} in
{
  toolchain | Toolchain = {
    rust_channel = "1.97.1",
    rust_targets = ["wasm32-unknown-unknown", "thumbv7em-none-eabihf"],
    nixpkgs_revision = "20535e48e12c86043b577b8518234ff5dbb26957",
    flake_utils_revision = "11707dc2f618dd54ca8739b309ec4fc024de578b",
    rust_overlay_revision = "3c38e1e1ba9c8d7030f7b5a801398ea7d8a6fdc0",
    crane_revision = "b556d7bbae5ff86e378451511873dfd07e4504cd",
    octet_revision = "374bd16b26cee2af34211a29bfa531c016811f51",
    verus_package = "octet-production-verus",
    verus_release = "0.2026.05.17.e479cce",
    verus_source_revision = "e479cce36490b8fa4b0fd7755aa742aec354372c",
    verus_binary_blake3 = "2aafa795a63160e2b262e0d8df00ad237766ed87fcd7fe9a981034207b1367c7",
    verus_rust_toolchain = "1.95.0-x86_64-unknown-linux-gnu",
    verus_rustc_blake3 = "4b001492d51b266eef82aeb98469d135d60f63d2720d24d9712c1e83fa10c9e3",
    verus_solver = "z3@verus-0.2026.05.17.e479cce",
    verus_solver_blake3 = "42e48a26bf3b38ca33d84749f4317cc179791de0d578d4d98a67f51782b54331",
    verus_nix_closure_graph_blake3 = "5f20a8243ed93278d1574e4269838858e99489abd1158f23ab2ffa8471a8d43b",
    vstd_crate = "vstd@0.0.0-2026-01-25-0057-no-default-features-alloc",
    trellis_revision = "aead4527bfc3c714e71669bce656719dd2a6b2e2",
    trellis_topo_sort_blake3 = "1e4665e37556c5549dccc23cea8fe62ea8619573e8827ea5a66e7c6e2362e8a8",
    trellis_job_state_blake3 = "3236af6e78b333e8c9dbe61f3e898647d24c747c9186a79a0fbc5506f71495a1",
    trellis_job_dag_blake3 = "33d61c877ea238302c843a16647667d8703ede28c1f2888a4d5b8fc4164fcc70",
    nickel_provider = "nixpkgs@20535e48e12c86043b577b8518234ff5dbb26957",
    nix_provider = "nixpkgs@20535e48e12c86043b577b8518234ff5dbb26957",
    claim_scope = "pinned-tool-identity-only-not-tool-soundness",
  },
}
