# r[impl choregraph.foundation.compiler_floor]
# r[verify choregraph.foundation.compiler_floor]
# r[impl choregraph.foundation.consumer_compatibility]
# r[verify choregraph.foundation.consumer_compatibility]
{
  schema = "choregraph.foundation-requirement-links.v1",
  compiler_floor = {
    declaration = "Cargo.toml workspace.package.rust-version",
    maintained_gate = "checks.x86_64-linux.octet-deny-all",
    bypass = false,
  },
  consumer_compatibility = {
    declarations = ["blake3", "serde_json"],
    lockfile = "Cargo.lock",
    lattice_patch_resolution = "checked",
  },
  non_claim = "These links record maintained source and gate locations. They do not prove compiler or consumer correctness.",
}
