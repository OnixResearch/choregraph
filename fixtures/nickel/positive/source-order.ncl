let Contracts = import "../../../contracts/nickel/restricted-graph-contracts.ncl" in
({
  effects = [],
  edges = [],
  nodes = [{ operations = ["run"], id = "root" }],
  imports = [],
  limits = {
    max_intents_per_transition = 4,
    max_intent_bytes = 1024,
    max_event_bytes = 4096,
    max_reference_bytes = 71,
    max_identifier_bytes = 128,
    max_effect_declarations = 4,
    max_operations = 4,
    max_edges = 4,
    max_nodes = 4,
  },
  events = ["start", "mark-ready", "fail", "complete", "cancel"],
  lifecycle_profile = "choregraph.lifecycle.linear.v1",
  schema = "choregraph.evaluated-graph.v1",
  source_format = "choregraph-v1",
}) | Contracts.EvaluatedGraph
