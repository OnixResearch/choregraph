# Closed contracts for the default-off preference-annotation profile.
# The `base` field is checked by the unchanged v1 role-choreography contract.
# Rust admission revalidates every rule here and adds path and projection checks.

let Base = import "role-choreography-contracts.ncl" in
let ANNOTATED_SCHEMA = "choregraph.role-choreography.preference-annotations.v1" in
let MAX_FACTORS = 256 in
let MAX_WORLD_VARIABLES = 64 in
let MAX_CHOICE_ANNOTATIONS = 256 in
let MAX_FACTORS_PER_CHOICE = 64 in
let MAX_PATH_STEPS = 128 in
let MAX_NAME_BYTES = 64 in
let MAX_U32 = 4294967295 in
let REQUIRED_NON_CLAIMS = [
  "no-branch-selection",
  "no-equilibrium-analysis",
  "no-game-semantics",
  "no-label-boolean-conversion",
  "no-preference-authority",
  "no-self-interested-projection-soundness",
  "no-solver",
  "no-utility-function",
  "no-world-distribution",
] in
let exact_fields = fun allowed value =>
  std.is_record value
  && std.array.length (std.record.fields value) == std.array.length allowed
  && std.array.all (fun field => std.array.elem field allowed) (std.record.fields value)
in
let field = fun name value => std.record.get name value in
let natural = fun value =>
  std.is_number value
  && std.number.is_integer value
  && value >= 0
  && value <= MAX_U32
in
let is_span = fun value =>
  exact_fields ["start", "end"] value
  && natural (field "start" value)
  && natural (field "end" value)
  && field "start" value <= field "end" value
in
let is_name = fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= MAX_NAME_BYTES
  && std.string.is_match "^[a-z][a-z0-9-]*$" value
in
let bounded_array = fun maximum predicate value =>
  std.is_array value
  && std.array.length value <= maximum
  && std.array.all predicate value
in
let is_factor = fun value =>
  exact_fields ["id", "role", "name", "span"] value
  && natural (field "id" value)
  && natural (field "role" value)
  && is_name (field "name" value)
  && is_span (field "span" value)
in
let is_world = fun value =>
  exact_fields ["id", "name", "span"] value
  && natural (field "id" value)
  && is_name (field "name" value)
  && is_span (field "span" value)
in
let is_step = fun value =>
  (exact_fields ["kind"] value && field "kind" value == "next")
  || (
    exact_fields ["kind", "label"] value
    && field "kind" value == "branch"
    && natural (field "label" value)
  )
in
let is_choice = fun value =>
  exact_fields ["path", "decider", "factors", "world", "span"] value
  && bounded_array MAX_PATH_STEPS is_step (field "path" value)
  && natural (field "decider" value)
  && bounded_array MAX_FACTORS_PER_CHOICE natural (field "factors" value)
  && std.array.length (field "factors" value) > 0
  && (field "world" value == null || natural (field "world" value))
  && is_span (field "span" value)
in
let is_annotations = fun value =>
  exact_fields ["factors", "worlds", "choices"] value
  && bounded_array MAX_FACTORS is_factor (field "factors" value)
  && bounded_array MAX_WORLD_VARIABLES is_world (field "worlds" value)
  && bounded_array MAX_CHOICE_ANNOTATIONS is_choice (field "choices" value)
  && std.array.length (field "choices" value) > 0
in
let is_non_claims = fun value =>
  std.is_array value
  && std.array.length value == std.array.length REQUIRED_NON_CLAIMS
  && std.array.all (fun item => std.array.elem item REQUIRED_NON_CLAIMS) value
  && std.array.all (fun item => std.array.elem item value) REQUIRED_NON_CLAIMS
in
{
  AnnotatedRoleChoreography = {
    schema | std.contract.from_predicate (fun value => value == ANNOTATED_SCHEMA),
    base | Base.RoleChoreography,
    annotations | std.contract.from_predicate is_annotations,
    non_claims | std.contract.from_predicate is_non_claims,
  },
  RequiredNonClaims = REQUIRED_NON_CLAIMS,
}
