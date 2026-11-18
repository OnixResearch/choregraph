let g = import "./choregraph/graph-construction-v1.ncl" in
g.finish "malformed" "lattice-test" "fixture-registry" {} [] {
  key = "missing-effects",
  kind = {tag = "operation", operation = "tick", configuration = {}},
}
