# Closed contract for the strict Octet consumer policy.

let MAX_LINTS = 256 in
let REVISION_CHARS = 40 in
let BLAKE3_CHARS = 64 in
let exact_fields = fun allowed value =>
  std.is_record value
  && std.array.length (std.record.fields value) == std.array.length allowed
  && std.array.all (fun field => std.array.elem field allowed) (std.record.fields value)
in
let field = fun name value => std.record.get name value in
let valid_lint = fun value =>
  std.is_string value
  && std.string.is_match "^[a-z][a-z0-9_]*$" value
in
let valid_config = fun value =>
  exact_fields ["schema", "mode", "octet_revision", "hook_blake3", "lints"] value
  && field "schema" value == "choregraph.octet-deny-all.v1"
  && field "mode" value == "deny-all"
  && std.is_string (field "octet_revision" value)
  && std.string.length (field "octet_revision" value) == REVISION_CHARS
  && std.string.is_match "^[0-9a-f]+$" (field "octet_revision" value)
  && std.is_string (field "hook_blake3" value)
  && std.string.length (field "hook_blake3" value) == BLAKE3_CHARS
  && std.string.is_match "^[0-9a-f]+$" (field "hook_blake3" value)
  && std.is_array (field "lints" value)
  && std.array.length (field "lints" value) > 0
  && std.array.length (field "lints" value) <= MAX_LINTS
  && std.array.all valid_lint (field "lints" value)
in
{
  DenyAllConfig = std.contract.from_predicate valid_config,
}
