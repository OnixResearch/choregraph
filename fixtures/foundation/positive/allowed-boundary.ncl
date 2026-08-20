let WorkspaceModel = import "../model-contract.ncl" in
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
let reviewed_bounded_exec = {
  name = "bounded-exec",
  requirement = "=0.1.0",
  source = "local-path",
  path = "vendor/bounded-exec-selected/crates/bounded-exec",
} in
let reviewed_cap_root = {
  name = "cap-root-core",
  requirement = "=0.1.0",
  source = "local-path",
  path = "vendor/cap-root-core-selected",
} in
let reviewed_execution_graph_oracle = {
  name = "execution-graph-core",
  requirement = "=0.1.0",
  source = "local-path",
  path = "vendor/execution-graph-core-selected",
} in
let reviewed_universal_weave = {
  name = "universal-weave",
  requirement = "=0.4.1",
  source = "registry",
  path = "",
} in
{
  packages = [
    package "choregraph-choreography" [workspace "choregraph-kamacite"],
    package "choregraph-core" [],
    package "choregraph-history" [],
    package "choregraph-history-machine" [
      workspace "choregraph-core",
      workspace "choregraph-history",
    ],
    package "choregraph-weave" [
      workspace "choregraph-history",
      reviewed_universal_weave,
    ],
    package "execution-graph-core" [workspace "choregraph-core"],
    package "choregraph-kamacite" [workspace "choregraph-core"],
    package "choregraph-nickel" [workspace "choregraph-core"],
    package "choregraph-release" [],
    package "choregraph-seaglass" [
      workspace "choregraph-core",
      workspace "choregraph-nickel",
      reviewed_execution_graph_oracle,
    ],
    package "choregraph-wasm" [
      workspace "choregraph-core",
      workspace "choregraph-kamacite",
    ],
    package "choregraph-cli" [
      workspace "choregraph-core",
      workspace "choregraph-kamacite",
      workspace "choregraph-nickel",
      workspace "choregraph-release",
      workspace "choregraph-seaglass",
      reviewed_bounded_exec,
      reviewed_cap_root,
    ],
    package "choregraph-foundation-check" [],
    package "choregraph-wasm-conformance" [
      workspace "choregraph-core",
      workspace "choregraph-kamacite",
      workspace "choregraph-wasm",
    ],
  ],
  core_sources = [
    {
      path = "crates/choregraph-core/src/lib.rs",
      text = "#![no_std]\nextern crate alloc;\n",
    },
    {
      path = "crates/choregraph-core/src/analytics/mod.rs",
      text = "pub fn ready_nodes() { crate::verified::ready_frontier(); }\n/// Returns one deterministic resume decision\n",
    },
    {
      path = "crates/choregraph-core/src/verified.rs",
      text = "pub proof fn readiness_is_sound() {}\n",
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
  wasm_adapter_sources = [
    {
      path = "dispatch.rs",
      text = "validate_machine_artifact(\nvalidate_durable_machine_artifact(\nvalidate_concrete_machine_artifact(\nstep(&context.machine\nstep_concrete(&context.machine\nreplay(input)\nreplay_concrete(input)\n",
    },
  ],
  wasm_wit_sources = [
    {
      path = "kernel.wit",
      text = "package onix:choregraph@1.0.0;\nworld kernel {\n  export call: func(request: list<u8>) -> list<u8>;\n}\n",
    },
  ],
} | WorkspaceModel
