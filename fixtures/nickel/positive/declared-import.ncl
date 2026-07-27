let Contracts = import "../../../contracts/nickel/restricted-graph-contracts.ncl" in
({
  source_format = "choregraph-v1",
  schema = "choregraph.evaluated-graph.v1",
  lifecycle_profile = "choregraph.lifecycle.linear.v1",
  events = ["cancel", "complete", "fail", "mark-ready", "start"],
  limits = {
    max_nodes = 4,
    max_edges = 4,
    max_operations = 4,
    max_effect_declarations = 4,
    max_identifier_bytes = 128,
    max_reference_bytes = 71,
    max_event_bytes = 4096,
    max_intent_bytes = 1024,
    max_intents_per_transition = 4,
  },
  imports = [{
    logical_id = "defaults",
    root_id = "source",
    relative_path = "data/defaults.ncl",
    media_class = "nickel-data",
    max_bytes = 16,
    expected_blake3 = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
  }],
  nodes = [{ id = "root", operations = ["run"] }],
  edges = [],
  effects = [],
}) | Contracts.EvaluatedGraph
