let g = import "./choregraph/graph-construction-v1.ncl" in
g.finish "wrong-type" "lattice-test" "fixture-registry" {} []
  (g.wire "root" [
    g.operation "source" "fork" {} [],
    g.operation "sink" "linear-sink" {} [],
  ] [
    g.connect (g.output "wrong-type" "source" "left") (g.input "wrong-type" "sink" "in"),
  ] [] [])
