let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in

let Project = {
  name | NonEmpty,
  license | NonEmpty,
  status | NonEmpty,
  core | NonEmpty,
  first_consumer | NonEmpty,
} in

{
  project | Project = {
    name = "choregraph",
    license = "AGPL-3.0-or-later",
    status = "lifecycle-design-only",
    core = "pure-no-std-verified-state-machine",
    first_consumer = "lattice-seaglass",
  },
}
