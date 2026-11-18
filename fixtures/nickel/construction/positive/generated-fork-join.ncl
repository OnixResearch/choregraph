let g = import "./choregraph/graph-construction-v1.ncl" in
let stage = g.template "stage" ["label"]
  (g.operation "body" "pass" {
    label = { tag = "parameter", name = "label" },
  } []) in
let branches = g.generate 2 ["a", "b"] (fun branch_label =>
  g.instantiate branch_label "stage" {
    label = { tag = "literal", value = branch_label },
  }) in
g.finish "generated" "lattice-test" "fixture-registry" {} [stage]
  (g.wire "root"
    ([g.operation "fork" "fork" {} []] @ branches @ [g.operation "join" "join" {} []])
    [
      g.connect (g.output "generated" "fork" "left") (g.input "generated" "a" "in"),
      g.connect (g.output "generated" "fork" "right") (g.input "generated" "b" "in"),
      g.connect (g.output "generated" "a" "out") (g.input "generated" "join" "left"),
      g.connect (g.output "generated" "b" "out") (g.input "generated" "join" "right"),
    ] [] [])
