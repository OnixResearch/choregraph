let valid = import "../positive/minimal.ncl" in
let excessive = valid & { limits.max_nodes | force = 4097 } in
excessive
