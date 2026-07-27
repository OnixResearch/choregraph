let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
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
    nickel_provider = "nixpkgs@20535e48e12c86043b577b8518234ff5dbb26957",
    nix_provider = "nixpkgs@20535e48e12c86043b577b8518234ff5dbb26957",
    claim_scope = "pinned-tool-identity-only-not-tool-soundness",
  },
}
