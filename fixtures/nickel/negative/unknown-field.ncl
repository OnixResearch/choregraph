let valid = import "../positive/minimal.ncl" in
let Contracts = import "../../../contracts/nickel/restricted-graph-contracts.ncl" in
(valid & { ambient_path = "/home/operator/secret" }) | Contracts.EvaluatedGraph
