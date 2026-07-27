let Contracts = import "../../../contracts/nickel/restricted-graph-contracts.ncl" in
({
  source_format = "lattice-workflow-v1",
  schema = "choregraph.lattice-workflow-projection.v1",
  lifecycle_profile = "choregraph.lifecycle.linear.v1",
  events = ["cancel", "complete", "fail", "mark-ready", "start"],
  limits = {
    max_nodes = 16,
    max_edges = 32,
    max_operations = 32,
    max_effect_declarations = 32,
    max_identifier_bytes = 128,
    max_reference_bytes = 71,
    max_event_bytes = 4096,
    max_intent_bytes = 1024,
    max_intents_per_transition = 16,
  },
  imports = [],
  nodes = [
    { id = 4, ports = [{ id = 40, direction = "input" }] },
    { id = 2, ports = [{ id = 20, direction = "input" }, { id = 21, direction = "output" }] },
    { id = 1, ports = [{ id = 11, direction = "output" }, { id = 12, direction = "output" }] },
    { id = 3, ports = [{ id = 30, direction = "input" }, { id = 31, direction = "output" }] },
  ],
  edges = [
    { source = 31, target = 40 },
    { source = 12, target = 30 },
    { source = 11, target = 20 },
    { source = 21, target = 40 },
  ],
  configs = [
    { node_id = 3, operation = "branch-right" },
    { node_id = 1, operation = "root" },
    { node_id = 4, operation = "join" },
    { node_id = 2, operation = "branch-left" },
  ],
}) | Contracts.EvaluatedGraph
