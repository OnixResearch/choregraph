let WorkspaceModel = import "model-contract.ncl" in
let package = fun package_name package_dependencies => {
  name = package_name,
  license = "AGPL-3.0-or-later",
  rust_version = "1.96.0",
  publish_disabled = true,
  dependencies = package_dependencies,
} in
let workspace = fun package_name => {
  name = package_name,
  requirement = "=0.1.0",
  source = "workspace",
  path = "",
} in
let reviewed_universal_weave = {
  name = "universal-weave",
  requirement = "=0.4.1",
  source = "registry",
  path = "",
} in
fun model_dependencies source_text =>
  {
    packages = [
      package "choregraph-core" model_dependencies,
      package "choregraph-history" [],
      package "choregraph-history-machine" [
        workspace "choregraph-core",
        workspace "choregraph-history",
      ],
      package "choregraph-weave" [
        workspace "choregraph-history",
        reviewed_universal_weave,
      ],
    ],
    core_sources = [
      {
        path = "crates/choregraph-core/src/lib.rs",
        text = source_text,
      },
    ],
    history_sources = [
      {
        path = "crates/choregraph-history/src/lib.rs",
        text = "#![no_std]\nextern crate alloc;\n",
      },
    ],
    history_machine_sources = [
      {
        path = "crates/choregraph-history-machine/src/lib.rs",
        text = "#![no_std]\nextern crate alloc;\nchoregraph_core::replay(replay_input);\nchoregraph_core::replay_concrete(replay_input);\n",
      },
    ],
    weave_sources = [
      {
        path = "crates/choregraph-weave/src/lib.rs",
        text = "#![no_std]\nextern crate alloc;\n",
      },
    ],
    wasm_adapter_sources = [],
    wasm_wit_sources = [],
  } | WorkspaceModel
