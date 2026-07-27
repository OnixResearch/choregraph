let valid = import "../positive/minimal.ncl" in
let Contracts = import "../../../contracts/nickel/restricted-graph-contracts.ncl" in
(std.record.remove "effects" valid) | Contracts.EvaluatedGraph
