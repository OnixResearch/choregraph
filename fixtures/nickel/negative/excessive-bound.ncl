let valid = import "../positive/minimal.ncl" in
let Contracts = import "../../../contracts/nickel/restricted-graph-contracts.ncl" in
let excessive = valid & { limits.max_nodes | force = 4097 } in
excessive | Contracts.EvaluatedGraph
