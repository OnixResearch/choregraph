let valid = import "../positive/minimal.ncl" in
let Contracts = import "../../../contracts/nickel/restricted-graph-contracts.ncl" in
let duplicate = valid & { events | force = ["cancel", "complete", "fail", "mark-ready", "mark-ready"] } in
duplicate | Contracts.EvaluatedGraph
