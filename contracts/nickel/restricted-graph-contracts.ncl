# Closed contracts for data after restricted Nickel evaluation.

let MAX_NODES = 4096 in
let MAX_EDGES = 16384 in
let MAX_OPERATIONS = 16384 in
let MAX_EFFECTS = 16384 in
let MAX_IMPORTS = 128 in
let MAX_PORTS_PER_NODE = 4096 in
let MAX_IDENTIFIER_CHARS = 256 in
let MAX_IMPORT_PATH_CHARS = 512 in
let LIFECYCLE_EVENTS = ["cancel", "complete", "fail", "mark-ready", "start"] in
let exact_fields = fun allowed value =>
  std.is_record value
  && std.array.length (std.record.fields value) == std.array.length allowed
  && std.array.all (fun field => std.array.elem field allowed) (std.record.fields value)
in
let has = fun field value => std.record.has_field field value in
let field = fun name value => std.record.get name value in
let bounded_string = fun maximum value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= maximum
in
let natural = fun maximum value =>
  std.is_number value
  && std.number.is_integer value
  && value >= 0
  && value <= maximum
in
let positive = fun maximum value => natural maximum value && value > 0 in
let string_array = fun maximum value =>
  std.is_array value
  && std.array.length value <= maximum
  && std.array.all (bounded_string MAX_IDENTIFIER_CHARS) value
in
let is_limits = fun value =>
  exact_fields [
    "max_nodes",
    "max_edges",
    "max_operations",
    "max_effect_declarations",
    "max_identifier_bytes",
    "max_reference_bytes",
    "max_event_bytes",
    "max_intent_bytes",
    "max_intents_per_transition",
  ] value
  && positive MAX_NODES (field "max_nodes" value)
  && positive MAX_EDGES (field "max_edges" value)
  && positive MAX_OPERATIONS (field "max_operations" value)
  && positive MAX_EFFECTS (field "max_effect_declarations" value)
  && positive 4096 (field "max_identifier_bytes" value)
  && positive 4096 (field "max_reference_bytes" value)
  && positive 1048576 (field "max_event_bytes" value)
  && positive 1048576 (field "max_intent_bytes" value)
  && positive 4096 (field "max_intents_per_transition" value)
in
let is_events = fun value =>
  std.is_array value
  && std.array.length value == std.array.length LIFECYCLE_EVENTS
  && std.array.all (fun event => std.array.elem event LIFECYCLE_EVENTS) value
  && std.array.all (fun event => std.array.elem event value) LIFECYCLE_EVENTS
in
let is_import = fun value =>
  exact_fields ["logical_id", "root_id", "relative_path", "media_class", "max_bytes", "expected_blake3"] value
  && bounded_string 128 (field "logical_id" value)
  && bounded_string 128 (field "root_id" value)
  && bounded_string MAX_IMPORT_PATH_CHARS (field "relative_path" value)
  && !(std.string.is_match "^/" (field "relative_path" value))
  && !(std.array.elem ".." (std.string.split "/" (field "relative_path" value)))
  && std.array.elem (field "media_class" value) ["nickel-data", "utf8-text", "preserves-packed"]
  && positive 1048576 (field "max_bytes" value)
  && std.is_string (field "expected_blake3" value)
  && std.string.is_match "^[0-9a-f]{64}$" (field "expected_blake3" value)
in
let is_imports = fun value =>
  std.is_array value
  && std.array.length value <= MAX_IMPORTS
  && std.array.all is_import value
in
let is_node = fun value =>
  exact_fields ["id", "operations"] value
  && bounded_string MAX_IDENTIFIER_CHARS (field "id" value)
  && string_array MAX_OPERATIONS (field "operations" value)
in
let is_edge = fun value =>
  exact_fields ["source", "target"] value
  && bounded_string MAX_IDENTIFIER_CHARS (field "source" value)
  && bounded_string MAX_IDENTIFIER_CHARS (field "target" value)
in
let is_effect = fun value =>
  exact_fields ["node_id", "from_state", "action", "operation", "resource", "ability", "request", "payload"] value
  && std.array.all (fun name => bounded_string MAX_IDENTIFIER_CHARS (field name value)) ["node_id", "from_state", "action", "operation", "resource", "ability", "request", "payload"]
in
let is_concrete_reference = fun value =>
  exact_fields ["profile", "identity"] value
  && field "profile" value == "choregraph.packed-preserves.v1"
  && std.is_string (field "identity" value)
  && std.string.is_match "^blake3:[0-9a-f]{64}$" (field "identity" value)
in
let is_reference_binding = fun value =>
  std.is_record value
  && has "mode" value
  && (
    (field "mode" value == "exact"
      && exact_fields ["mode", "reference"] value
      && is_concrete_reference (field "reference" value))
    || (field "mode" value == "slot"
      && exact_fields ["mode", "slot_id", "source_ref"] value
      && bounded_string MAX_IDENTIFIER_CHARS (field "slot_id" value)
      && bounded_string MAX_IDENTIFIER_CHARS (field "source_ref" value))
  )
in
let is_port = fun value =>
  exact_fields ["id", "direction"] value
  && natural 18446744073709551615 (field "id" value)
  && std.array.elem (field "direction" value) ["input", "output"]
in
let is_lattice_node = fun value =>
  exact_fields ["id", "ports"] value
  && natural 18446744073709551615 (field "id" value)
  && std.is_array (field "ports" value)
  && std.array.length (field "ports" value) <= MAX_PORTS_PER_NODE
  && std.array.all is_port (field "ports" value)
in
let is_lattice_edge = fun value =>
  exact_fields ["source", "target"] value
  && natural 18446744073709551615 (field "source" value)
  && natural 18446744073709551615 (field "target" value)
in
let is_lattice_config = fun value =>
  exact_fields ["node_id", "operation"] value
  && natural 18446744073709551615 (field "node_id" value)
  && bounded_string MAX_IDENTIFIER_CHARS (field "operation" value)
in
let is_lattice_config_v2 = fun value =>
  exact_fields ["node_id", "operation", "effectful"] value
  && natural 18446744073709551615 (field "node_id" value)
  && bounded_string MAX_IDENTIFIER_CHARS (field "operation" value)
  && std.is_bool (field "effectful" value)
in
let is_lattice_effect_v2 = fun value =>
  exact_fields ["declaration_key", "node_id", "from_state", "action", "resource", "ability", "request_binding", "payload_binding"] value
  && bounded_string MAX_IDENTIFIER_CHARS (field "declaration_key" value)
  && natural 18446744073709551615 (field "node_id" value)
  && std.array.all (fun name => bounded_string MAX_IDENTIFIER_CHARS (field name value)) ["from_state", "action", "resource", "ability"]
  && is_reference_binding (field "request_binding" value)
  && is_reference_binding (field "payload_binding" value)
in
let is_choregraph = fun value =>
  exact_fields ["source_format", "schema", "lifecycle_profile", "events", "limits", "imports", "nodes", "edges", "effects"] value
  && field "source_format" value == "choregraph-v1"
  && field "schema" value == "choregraph.evaluated-graph.v1"
  && field "lifecycle_profile" value == "choregraph.lifecycle.linear.v1"
  && is_events (field "events" value)
  && is_limits (field "limits" value)
  && is_imports (field "imports" value)
  && std.is_array (field "nodes" value)
  && std.array.length (field "nodes" value) <= MAX_NODES
  && std.array.all is_node (field "nodes" value)
  && std.is_array (field "edges" value)
  && std.array.length (field "edges" value) <= MAX_EDGES
  && std.array.all is_edge (field "edges" value)
  && std.is_array (field "effects" value)
  && std.array.length (field "effects" value) <= MAX_EFFECTS
  && std.array.all is_effect (field "effects" value)
in
let is_lattice = fun value =>
  exact_fields ["source_format", "schema", "lifecycle_profile", "events", "limits", "imports", "nodes", "edges", "configs"] value
  && field "source_format" value == "lattice-workflow-v1"
  && field "schema" value == "choregraph.lattice-workflow-projection.v1"
  && field "lifecycle_profile" value == "choregraph.lifecycle.linear.v1"
  && is_events (field "events" value)
  && is_limits (field "limits" value)
  && is_imports (field "imports" value)
  && std.is_array (field "nodes" value)
  && std.array.length (field "nodes" value) <= MAX_NODES
  && std.array.all is_lattice_node (field "nodes" value)
  && std.is_array (field "edges" value)
  && std.array.length (field "edges" value) <= MAX_EDGES
  && std.array.all is_lattice_edge (field "edges" value)
  && std.is_array (field "configs" value)
  && std.array.length (field "configs" value) <= MAX_NODES
  && std.array.all is_lattice_config (field "configs" value)
in
let is_lattice_v2 = fun value =>
  exact_fields ["source_format", "schema", "lifecycle_profile", "events", "limits", "imports", "nodes", "edges", "configs", "effects"] value
  && field "source_format" value == "lattice-workflow-v2"
  && field "schema" value == "choregraph.lattice-workflow-projection.v2"
  && field "lifecycle_profile" value == "choregraph.lifecycle.linear.v1"
  && is_events (field "events" value)
  && is_limits (field "limits" value)
  && is_imports (field "imports" value)
  && std.is_array (field "nodes" value)
  && std.array.length (field "nodes" value) <= MAX_NODES
  && std.array.all is_lattice_node (field "nodes" value)
  && std.is_array (field "edges" value)
  && std.array.length (field "edges" value) <= MAX_EDGES
  && std.array.all is_lattice_edge (field "edges" value)
  && std.is_array (field "configs" value)
  && std.array.length (field "configs" value) <= MAX_NODES
  && std.array.all is_lattice_config_v2 (field "configs" value)
  && std.is_array (field "effects" value)
  && std.array.length (field "effects" value) <= MAX_EFFECTS
  && std.array.all is_lattice_effect_v2 (field "effects" value)
  && std.array.all
    (fun config =>
      !(field "effectful" config)
      || std.array.any (fun effect => field "node_id" effect == field "node_id" config) (field "effects" value))
    (field "configs" value)
  && std.array.all
    (fun effect =>
      std.array.any
        (fun config => field "node_id" config == field "node_id" effect && field "effectful" config)
        (field "configs" value))
    (field "effects" value)
in
{
  EvaluatedGraph = std.contract.from_predicate (fun value => is_choregraph value || is_lattice value || is_lattice_v2 value),
  RestrictedProfile = {
    id = "choregraph.nickel-restricted.v1",
    allowed_values = ["records", "arrays", "strings", "numbers", "booleans", "enums"],
    declared_imports_only = true,
    content_addressed_imports = true,
    forbidden_authority = ["network", "process", "clock", "environment", "undeclared-import", "ambient-path", "secret-discovery"],
    non_claims = [
      "contract success does not prove arbitrary Nickel semantics",
      "evaluator success does not prove source intent",
      "path shape validation is not filesystem confinement",
    ],
  },
}
