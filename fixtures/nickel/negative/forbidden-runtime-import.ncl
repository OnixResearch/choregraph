let value = import "missing-declared-import.ncl" in
{
  source_format = "choregraph-v1",
  schema = "choregraph.evaluated-graph.v1",
  lifecycle_profile = "choregraph.lifecycle.linear.v1",
  events = ["cancel", "complete", "fail", "mark-ready", "start"],
  limits = {},
  imports = [],
  nodes = [],
  edges = [],
  effects = [],
}
