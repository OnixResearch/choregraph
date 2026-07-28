let WorkspaceModel = import "../model-contract.ncl" in
{
  packages = [
    {
      name = "",
      license = "AGPL-3.0-or-later",
      rust_version = "1.96.0",
      publish_disabled = true,
      dependencies = [],
    },
  ],
  core_sources = [
    {
      path = "crates/choregraph-core/src/lib.rs",
      text = "#![no_std]\nextern crate alloc;\n",
    },
  ],
} | WorkspaceModel
