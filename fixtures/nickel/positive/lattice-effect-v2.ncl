let NODE = 1 in
let MAX_NODES = 4 in
let MAX_EDGES = 4 in
let MAX_OPERATIONS = 4 in
let MAX_EFFECTS = 4 in
let MAX_IDENTIFIER_BYTES = 128 in
let MAX_REFERENCE_BYTES = 71 in
let MAX_EVENT_BYTES = 4096 in
let MAX_INTENT_BYTES = 1024 in
let MAX_INTENTS = 4 in
{
  source_format = "lattice-workflow-v2",
  schema = "choregraph.lattice-workflow-projection.v2",
  lifecycle_profile = "choregraph.lifecycle.linear.v1",
  events = ["cancel", "complete", "fail", "mark-ready", "start"],
  limits = {
    max_nodes = MAX_NODES,
    max_edges = MAX_EDGES,
    max_operations = MAX_OPERATIONS,
    max_effect_declarations = MAX_EFFECTS,
    max_identifier_bytes = MAX_IDENTIFIER_BYTES,
    max_reference_bytes = MAX_REFERENCE_BYTES,
    max_event_bytes = MAX_EVENT_BYTES,
    max_intent_bytes = MAX_INTENT_BYTES,
    max_intents_per_transition = MAX_INTENTS,
  },
  imports = [],
  nodes = [
    { id = NODE, ports = [] },
  ],
  edges = [],
  configs = [
    { node_id = NODE, operation = "http-request", effectful = true },
  ],
  effects = [
    {
      declaration_key = "request/main",
      node_id = NODE,
      from_state = "ready",
      action = "start",
      resource = "service/example",
      ability = "http/request",
      request_binding = {
        mode = "slot",
        slot_id = "request/input",
        source_ref = "node/1/input/request",
      },
      payload_binding = {
        mode = "exact",
        reference = {
          profile = "choregraph.packed-preserves.v1",
          identity = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        },
      },
    },
  ],
}
