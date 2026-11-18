let g = import "./choregraph/graph-construction-v1.ncl" in
g.finish "linear-drop" "lattice-test" "fixture-registry" {} []
  (g.wire "root" [g.operation "source" "linear-source" {} []] [] [] [])
