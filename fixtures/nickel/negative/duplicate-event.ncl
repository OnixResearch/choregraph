let valid = import "../positive/minimal.ncl" in
let duplicate = valid & { events | force = ["cancel", "complete", "fail", "mark-ready", "mark-ready"] } in
duplicate
