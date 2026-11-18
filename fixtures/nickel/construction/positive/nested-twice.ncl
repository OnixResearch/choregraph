let g = import "./choregraph/graph-construction-v1.ncl" in
let pass = g.template "pass-template" ["label"]
  (g.operation "pass-body" "pass" {
    label = { tag = "parameter", name = "label" },
  } []) in
let wrapper = g.template "wrapper" ["label"]
  (g.instantiate "nested-pass" "pass-template" {
    label = { tag = "parameter", name = "label" },
  }) in
let src = g.operation "fork" "fork" {} [] in
let left = g.instantiate "left" "wrapper" {
  label = { tag = "literal", value = "left" },
} in
let right = g.instantiate "right" "wrapper" {
  label = { tag = "literal", value = "right" },
} in
let sink = g.operation "join" "join" {} [] in
g.finish "workflow" "lattice-test" "fixture-registry" {} [pass, wrapper]
  (g.wire "root" [src, left, right, sink] [
    g.connect (g.output "workflow" "fork" "left") (g.input "workflow" "left" "in"),
    g.connect (g.output "workflow" "fork" "right") (g.input "workflow" "right" "in"),
    g.connect (g.output "workflow" "left" "out") (g.input "workflow" "join" "left"),
    g.connect (g.output "workflow" "right" "out") (g.input "workflow" "join" "right"),
  ] [] [])
