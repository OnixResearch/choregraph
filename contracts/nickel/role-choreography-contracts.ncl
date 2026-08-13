# Closed contracts for evaluated bounded role choreography data.

let MAX_ROLES = 64 in
let MAX_VALUES = 256 in
let MAX_STEPS = 4096 in
let MAX_BRANCHES = 256 in
let MAX_NESTING_DEPTH = 128 in
let MAX_IDENTIFIER_BYTES = 256 in
let MAX_IMPORTS = 128 in
let MAX_DIAGNOSTICS = 128 in
let MAX_DIAGNOSTIC_TEXT_BYTES = 512 in
let MAX_CANONICAL_OUTPUT_BYTES = 1048576 in
let MAX_U32 = 4294967295 in
let REQUIRED_NON_CLAIMS = [
  "no-runtime-execution",
  "no-host-placement",
  "no-transport-delivery",
  "no-persistence",
  "no-authorization",
  "no-external-role-correctness",
  "no-exactly-once-transfer",
  "no-deadlock-freedom-under-faults",
  "no-production-readiness",
  "no-release-eligibility",
] in
let exact_fields = fun allowed value =>
  std.is_record value
  && std.array.length (std.record.fields value) == std.array.length allowed
  && std.array.all (fun field => std.array.elem field allowed) (std.record.fields value)
in
let field = fun name value => std.record.get name value in
let natural = fun maximum value =>
  std.is_number value
  && std.number.is_integer value
  && value >= 0
  && value <= maximum
in
let bounded_string = fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= MAX_IDENTIFIER_BYTES
in
let blake3_identity = fun value =>
  bounded_string value
  && std.string.is_match "^blake3:[0-9a-f]{64}$" value
in
let raw_blake3 = fun value =>
  std.is_string value
  && std.string.is_match "^[0-9a-f]{64}$" value
in
let is_span = fun value =>
  exact_fields ["start", "end"] value
  && natural MAX_U32 (field "start" value)
  && natural MAX_U32 (field "end" value)
  && field "start" value <= field "end" value
in
let is_role_kind = fun value =>
  std.is_record value
  && (
    (exact_fields ["kind"] value && field "kind" value == "native")
    || (
      exact_fields ["kind", "contract_identity", "input_schema_identity", "output_schema_identity"] value
      && field "kind" value == "external"
      && blake3_identity (field "contract_identity" value)
      && blake3_identity (field "input_schema_identity" value)
      && blake3_identity (field "output_schema_identity" value)
    )
  )
in
let is_role = fun value =>
  exact_fields ["id", "kind", "span"] value
  && natural MAX_U32 (field "id" value)
  && is_role_kind (field "kind" value)
  && is_span (field "span" value)
in
let is_value = fun value =>
  exact_fields ["id", "payload_tag", "initial_owner", "span"] value
  && natural MAX_U32 (field "id" value)
  && natural MAX_U32 (field "payload_tag" value)
  && natural MAX_U32 (field "initial_owner" value)
  && is_span (field "span" value)
in
let is_import = fun value =>
  exact_fields ["logical_id", "blake3"] value
  && bounded_string (field "logical_id" value)
  && raw_blake3 (field "blake3" value)
in
let is_limits = fun value =>
  exact_fields [
    "max_roles",
    "max_values",
    "max_steps",
    "max_branches",
    "max_nesting_depth",
    "max_identifier_bytes",
    "max_imports",
    "max_diagnostics",
    "max_diagnostic_text_bytes",
    "max_canonical_output_bytes",
  ] value
  && field "max_roles" value == MAX_ROLES
  && field "max_values" value == MAX_VALUES
  && field "max_steps" value == MAX_STEPS
  && field "max_branches" value == MAX_BRANCHES
  && field "max_nesting_depth" value == MAX_NESTING_DEPTH
  && field "max_identifier_bytes" value == MAX_IDENTIFIER_BYTES
  && field "max_imports" value == MAX_IMPORTS
  && field "max_diagnostics" value == MAX_DIAGNOSTICS
  && field "max_diagnostic_text_bytes" value == MAX_DIAGNOSTIC_TEXT_BYTES
  && field "max_canonical_output_bytes" value == MAX_CANONICAL_OUTPUT_BYTES
in
let rec is_node = fun depth value =>
  depth <= MAX_NESTING_DEPTH
  && std.is_record value
  && (
    (
      exact_fields ["kind", "span"] value
      && field "kind" value == "end"
      && is_span (field "span" value)
    )
    || (
      exact_fields ["kind", "from", "to", "label", "payload_tag", "next", "span"] value
      && field "kind" value == "communication"
      && natural MAX_U32 (field "from" value)
      && natural MAX_U32 (field "to" value)
      && natural MAX_U32 (field "label" value)
      && natural MAX_U32 (field "payload_tag" value)
      && is_span (field "span" value)
      && is_node (depth + 1) (field "next" value)
    )
    || (
      exact_fields ["kind", "from", "to", "label", "payload_tag", "value", "next", "span"] value
      && field "kind" value == "transfer"
      && natural MAX_U32 (field "from" value)
      && natural MAX_U32 (field "to" value)
      && natural MAX_U32 (field "label" value)
      && natural MAX_U32 (field "payload_tag" value)
      && natural MAX_U32 (field "value" value)
      && is_span (field "span" value)
      && is_node (depth + 1) (field "next" value)
    )
    || (
      exact_fields ["kind", "decider", "branches", "span"] value
      && field "kind" value == "choice"
      && natural MAX_U32 (field "decider" value)
      && is_span (field "span" value)
      && std.is_array (field "branches" value)
      && std.array.length (field "branches" value) > 0
      && std.array.length (field "branches" value) <= MAX_BRANCHES
      && std.array.all (fun branch =>
        exact_fields ["label", "body", "span"] branch
        && natural MAX_U32 (field "label" branch)
        && is_span (field "span" branch)
        && is_node (depth + 1) (field "body" branch)
      ) (field "branches" value)
    )
  )
in
let is_non_claims = fun value =>
  std.is_array value
  && std.array.length value == std.array.length REQUIRED_NON_CLAIMS
  && std.array.all (fun item => std.array.elem item REQUIRED_NON_CLAIMS) value
  && std.array.all (fun item => std.array.elem item value) REQUIRED_NON_CLAIMS
in
let is_role_choreography = fun value =>
  exact_fields ["schema", "limits", "imports", "roles", "values", "protocol", "non_claims"] value
  && field "schema" value == "choregraph.role-choreography.v1"
  && is_limits (field "limits" value)
  && std.is_array (field "imports" value)
  && std.array.length (field "imports" value) <= MAX_IMPORTS
  && std.array.all is_import (field "imports" value)
  && std.is_array (field "roles" value)
  && std.array.length (field "roles" value) > 0
  && std.array.length (field "roles" value) <= MAX_ROLES
  && std.array.all is_role (field "roles" value)
  && std.is_array (field "values" value)
  && std.array.length (field "values" value) <= MAX_VALUES
  && std.array.all is_value (field "values" value)
  && is_node 0 (field "protocol" value)
  && is_non_claims (field "non_claims" value)
in
{
  RoleChoreography = std.contract.from_predicate is_role_choreography,
  ReviewedLimits = {
    max_roles = MAX_ROLES,
    max_values = MAX_VALUES,
    max_steps = MAX_STEPS,
    max_branches = MAX_BRANCHES,
    max_nesting_depth = MAX_NESTING_DEPTH,
    max_identifier_bytes = MAX_IDENTIFIER_BYTES,
    max_imports = MAX_IMPORTS,
    max_diagnostics = MAX_DIAGNOSTICS,
    max_diagnostic_text_bytes = MAX_DIAGNOSTIC_TEXT_BYTES,
    max_canonical_output_bytes = MAX_CANONICAL_OUTPUT_BYTES,
  },
  RequiredNonClaims = REQUIRED_NON_CLAIMS,
}
