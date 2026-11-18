# Closed, structural contract for the JSON exported by graph-construction-v1.ncl.
# This is an authoring error boundary, never a substitute for native admission.
let MAX_ITEMS = 4096 in
let MAX_DEPTH = 128 in
let MAX_NAME = 1024 in
let MAX_TEXT = 1048576 in
let is_null = fun value => value == null in
let exact = fun names value =>
  std.is_record value
  && std.array.length (std.record.fields value) == std.array.length names
  && std.array.all (fun name => std.array.elem name names) (std.record.fields value)
in
let field = fun name value => std.record.get name value in
let has = fun name value => std.record.has_field name value in
let text = fun maximum value =>
  std.is_string value && std.string.length value > 0 && std.string.length value <= maximum
in
let natural = fun value =>
  std.is_number value && std.number.is_integer value && value >= 0 && value <= 4294967295
in
let array = fun predicate value =>
  std.is_array value && std.array.length value <= MAX_ITEMS && std.array.all predicate value
in
let distinct = fun values =>
  std.array.all (fun item =>
    std.array.length (std.array.filter (fun other => item == other) values) == 1
  ) values
in
let names = fun values => array (text MAX_NAME) values && distinct values in
let no_slash = fun value => text MAX_NAME value && !(std.string.is_match "/" value) in
let rec json = fun depth value =>
  depth <= MAX_DEPTH
  && (is_null value || std.is_bool value || std.is_number value || std.is_string value
    || (std.is_array value && std.array.length value <= MAX_ITEMS
      && std.array.all (json (depth + 1)) value)
    || (std.is_record value && std.array.length (std.record.fields value) <= MAX_ITEMS
      && std.array.all (fun name => json (depth + 1) (field name value)) (std.record.fields value)))
in
let is_location = fun value =>
  exact ["logical_module", "start", "end"] value
  && text MAX_NAME (field "logical_module" value)
  && natural (field "start" value) && natural (field "end" value)
  && field "start" value <= field "end" value
in
let is_origin = fun value =>
  (exact ["template"] value || exact ["template", "generation"] value
    || exact ["template", "location"] value
    || exact ["template", "generation", "location"] value)
  && text MAX_NAME (field "template" value)
  && (!has "generation" value || is_null (field "generation" value)
    || text MAX_TEXT (field "generation" value))
  && (!has "location" value || is_null (field "location" value)
    || is_location (field "location" value))
in
let is_boundary = fun value =>
  exact ["name", "value_type"] value
  && text MAX_NAME (field "name" value)
  && text MAX_NAME (field "value_type" value)
in
let boundaries = fun value =>
  array is_boundary value && distinct (std.array.map (field "name") value)
in
let is_reference = fun value =>
  exact ["direction", "name"] value
  && std.array.elem (field "direction" value) ["input", "output", "config", "invocation"]
  && text MAX_NAME (field "name" value)
in
let rec is_expression = fun depth value =>
  depth <= MAX_DEPTH && std.is_record value && has "tag" value && (
    (exact ["tag", "value"] value && field "tag" value == "literal"
      && json 0 (field "value" value))
    || (exact ["tag", "name"] value && field "tag" value == "parameter"
      && text MAX_NAME (field "name" value))
    || (exact ["tag", "reference"] value && field "tag" value == "reference"
      && is_reference (field "reference" value))
    || (exact ["tag", "operator", "arguments"] value && field "tag" value == "apply"
      && text MAX_NAME (field "operator" value)
      && array (is_expression (depth + 1)) (field "arguments" value))
  )
in
let expressions = fun value =>
  std.is_record value
  && std.array.length (std.record.fields value) <= MAX_ITEMS
  && std.array.all (fun name =>
    text MAX_NAME name && is_expression 0 (field name value)
  ) (std.record.fields value)
in
let is_content_reference = fun value =>
  exact ["profile", "identity"] value
  && field "profile" value == "choregraph.packed-preserves.v1"
  && std.is_string (field "identity" value)
  && std.string.is_match "^blake3:[0-9a-f]{64}$" (field "identity" value)
in
let is_binding = fun value =>
  std.is_record value && has "mode" value && (
    (exact ["mode", "reference"] value && field "mode" value == "exact"
      && is_content_reference (field "reference" value))
    || (exact ["mode", "slot", "source"] value && field "mode" value == "slot"
      && no_slash (field "slot" value) && is_reference (field "source" value))
  )
in
let is_effect = fun value =>
  exact ["key", "from_state", "action", "resource", "ability", "request", "payload"] value
  && std.array.all (fun name => no_slash (field name value))
    ["key", "from_state", "action"]
  && text MAX_NAME (field "resource" value) && text MAX_NAME (field "ability" value)
  && is_binding (field "request" value) && is_binding (field "payload" value)
in
let is_handle = fun value =>
  exact ["root", "instance", "direction", "boundary"] value
  && no_slash (field "root" value) && no_slash (field "instance" value)
  && std.array.elem (field "direction" value) ["input", "output"]
  && text MAX_NAME (field "boundary" value)
in
let is_connection = fun value =>
  exact ["source", "target"] value
  && is_handle (field "source" value) && is_handle (field "target" value)
  && field "direction" (field "source" value) == "output"
  && field "direction" (field "target" value) == "input"
in
let is_exposure = fun value =>
  exact ["name", "handle"] value
  && text MAX_NAME (field "name" value) && is_handle (field "handle" value)
in
let rec is_fragment = fun depth value =>
  depth <= MAX_DEPTH
  && std.is_record value
  && (exact ["key", "kind"] value || exact ["key", "origin", "kind"] value)
  && no_slash (field "key" value)
  && (!has "origin" value || is_null (field "origin" value) || is_origin (field "origin" value))
  && (let kind = field "kind" value in
    (exact ["tag", "operation", "configuration", "effects"] kind
      && field "tag" kind == "operation"
      && no_slash (field "operation" kind)
      && expressions (field "configuration" kind)
      && array is_effect (field "effects" kind)
      && distinct (std.array.map (field "key") (field "effects" kind)))
    || (exact ["tag", "template", "arguments"] kind
      && field "tag" kind == "instantiate"
      && no_slash (field "template" kind)
      && expressions (field "arguments" kind))
    || (exact ["tag", "boundaries"] kind
      && field "tag" kind == "identity"
      && boundaries (field "boundaries" kind))
    || (exact ["tag", "parts"] kind
      && std.array.elem (field "tag" kind) ["sequence", "parallel"]
      && array (is_fragment (depth + 1)) (field "parts" kind)
      && distinct (std.array.map (field "key") (field "parts" kind)))
    || (exact ["tag", "parts", "connections", "inputs", "outputs"] kind
      && field "tag" kind == "wire"
      && array (is_fragment (depth + 1)) (field "parts" kind)
      && distinct (std.array.map (field "key") (field "parts" kind))
      && array is_connection (field "connections" kind)
      && array is_exposure (field "inputs" kind)
      && array is_exposure (field "outputs" kind)
      && distinct (std.array.map (field "name") (field "inputs" kind))
      && distinct (std.array.map (field "name") (field "outputs" kind)))
  )
in
let is_template = fun value =>
  exact ["name", "parameters", "body"] value
  && no_slash (field "name" value)
  && names (field "parameters" value)
  && is_fragment 0 (field "body" value)
in
let is_candidate = fun value =>
  exact ["schema", "root", "target", "registry", "parameters", "templates", "body"] value
  && field "schema" value == "choregraph.graph-construction.v1"
  && no_slash (field "root" value)
  && text MAX_NAME (field "target" value)
  && text MAX_NAME (field "registry" value)
  && std.is_record (field "parameters" value)
  && std.array.length (std.record.fields (field "parameters" value)) <= MAX_ITEMS
  && std.array.all (fun name => text MAX_NAME name && json 0 (field name (field "parameters" value)))
    (std.record.fields (field "parameters" value))
  && array is_template (field "templates" value)
  && distinct (std.array.map (field "name") (field "templates" value))
  && is_fragment 0 (field "body" value)
in
{
  ConstructionCandidate = std.contract.from_predicate is_candidate,
  BoundedGeneration = std.contract.from_predicate (fun value =>
    exact ["cap", "items"] value
    && natural (field "cap" value)
    && field "cap" value <= MAX_ITEMS
    && std.is_array (field "items" value)
    && std.array.length (field "items" value) <= field "cap" value
  ),
}
