let g = import "./choregraph/graph-construction-v1.ncl" in
let stage = g.template "invocation-stage" ["label"]
  (g.operation "body" "effect-tick" {
    label = g.parameter "label",
    request = g.literal "configuration is not the dispatch request",
  } [{
    key = "invoke", from_state = "ready", action = "start",
    resource = "machine", ability = "run",
    request = g.slot_binding "dispatch-request" "invocation" "dispatch-request",
    payload = g.slot_binding "resolved-payload" "invocation" "resolved-payload",
  }]) in
g.finish "invocation" "lattice-test" "fixture-registry" {} [stage]
  (g.sequence "root" [
    g.instantiate "first" "invocation-stage" { label = g.literal "one" },
    g.instantiate "second" "invocation-stage" { label = g.literal "two" },
  ])
