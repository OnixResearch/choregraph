let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let DependencySource = std.contract.from_predicate (fun value =>
  value == "workspace"
  || value == "registry"
  || value == "local-path"
  || value == "external-path"
) in
let Dependency = {
  name | NonEmpty,
  requirement | NonEmpty,
  source | DependencySource,
  path | String,
} in
let Package = {
  name | NonEmpty,
  license | NonEmpty,
  rust_version | NonEmpty,
  publish_disabled | Bool,
  dependencies | Array Dependency,
} in
let SourceUnit = {
  path | NonEmpty,
  text | NonEmpty,
} in
{
  packages | Array Package,
  core_sources | Array SourceUnit,
}
