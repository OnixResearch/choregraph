let WorkspaceModel = import "model-contract.ncl" in
fun model_dependencies source_text =>
  {
    packages = [
      {
        name = "choregraph-core",
        license = "AGPL-3.0-or-later",
        rust_version = "1.97.1",
        publish_disabled = true,
        dependencies = model_dependencies,
      },
    ],
    core_sources = [
      {
        path = "crates/choregraph-core/src/lib.rs",
        text = source_text,
      },
    ],
  } | WorkspaceModel
