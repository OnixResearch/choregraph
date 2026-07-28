let WorkspaceModel = import "../model-contract.ncl" in
let package = fun package_name package_dependencies => {
  name = package_name,
  license = "AGPL-3.0-or-later",
  rust_version = "1.97.1",
  publish_disabled = true,
  dependencies = package_dependencies,
} in
let workspace = fun package_name => {
  name = package_name,
  source = "workspace",
  path = "",
} in
let reviewed_bounded_exec = {
  name = "bounded-exec",
  source = "local-path",
  path = "vendor/bounded-exec-selected/crates/bounded-exec",
} in
let reviewed_cap_root = {
  name = "cap-root-core",
  source = "local-path",
  path = "vendor/cap-root-core-selected",
} in
let reviewed_execution_graph_oracle = {
  name = "execution-graph-core",
  source = "local-path",
  path = "vendor/execution-graph-core-selected",
} in
{
  packages = [
    package "choregraph-core" [],
    package "choregraph-kamacite" [workspace "choregraph-core"],
    package "choregraph-nickel" [workspace "choregraph-core"],
    package "choregraph-release" [],
    package "choregraph-seaglass" [
      workspace "choregraph-core",
      workspace "choregraph-nickel",
      reviewed_execution_graph_oracle,
    ],
    package "choregraph-cli" [
      workspace "choregraph-core",
      workspace "choregraph-kamacite",
      workspace "choregraph-nickel",
      workspace "choregraph-seaglass",
      reviewed_bounded_exec,
      reviewed_cap_root,
    ],
    package "choregraph-foundation-check" [],
  ],
  core_sources = [
    {
      path = "crates/choregraph-core/src/lib.rs",
      text = "#![no_std]\nextern crate alloc;\n",
    },
  ],
} | WorkspaceModel
