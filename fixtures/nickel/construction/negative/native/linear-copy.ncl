let g = import "./choregraph/graph-construction-v1.ncl" in
g.finish "linear-copy" "lattice-test" "fixture-registry" {} []
  (g.wire "root" [
    g.operation "source" "linear-source" {} [],
    g.operation "first" "linear-sink" {} [],
    g.operation "second" "linear-sink" {} [],
  ] [
    g.connect (g.output "linear-copy" "source" "out") (g.input "linear-copy" "first" "in"),
    g.connect (g.output "linear-copy" "source" "out") (g.input "linear-copy" "second" "in"),
  ] [] [])
