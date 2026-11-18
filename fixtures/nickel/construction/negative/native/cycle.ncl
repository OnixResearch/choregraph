let g = import "./choregraph/graph-construction-v1.ncl" in
g.finish "cycle" "lattice-test" "fixture-registry" {} []
  (g.wire "root" [
    g.operation "a" "pass" {label = {tag = "literal", value = "a"}} [],
    g.operation "b" "pass" {label = {tag = "literal", value = "b"}} [],
  ] [
    g.connect (g.output "cycle" "a" "out") (g.input "cycle" "b" "in"),
    g.connect (g.output "cycle" "b" "out") (g.input "cycle" "a" "in"),
  ] [] [])
