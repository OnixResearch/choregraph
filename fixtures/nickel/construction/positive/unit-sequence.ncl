let g = import "./choregraph/graph-construction-v1.ncl" in
g.finish "unit" "lattice-test" "fixture-registry" {} []
  (g.sequence "root" [
    g.operation "first" "tick" {} [],
    g.identity "empty" [],
    g.operation "second" "tick" {} [],
    g.operation "third" "tick" {} [],
  ])
