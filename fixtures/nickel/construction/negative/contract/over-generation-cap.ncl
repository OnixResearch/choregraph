let g = import "./choregraph/graph-construction-v1.ncl" in
let too_many = g.generate 1 ["first", "second"] (fun key => g.operation key "tick" {} []) in
g.finish "over-cap" "lattice-test" "fixture-registry" {} [] (g.parallel "root" too_many)
