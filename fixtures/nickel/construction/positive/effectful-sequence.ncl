let g = import "./choregraph/graph-construction-v1.ncl" in
let stage = g.template "effect-stage" ["label"]
  (g.operation "body" "effect-tick" {
    label = g.parameter "label",
    request = g.apply "concat" [
      g.literal "request:",
      g.reference "config" "label",
    ],
  } [{
    key = "invoke", from_state = "ready", action = "start",
    resource = "machine", ability = "run",
    request = g.exact_binding "choregraph.packed-preserves.v1"
      "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    payload = g.slot_binding "payload" "config" "request",
  }]) in
g.finish "effects" "lattice-test" "fixture-registry" {} [stage]
  (g.sequence "root" [
    g.instantiate "first" "effect-stage" { label = g.literal "one" },
    g.instantiate "second" "effect-stage" { label = g.literal "two" },
  ])
