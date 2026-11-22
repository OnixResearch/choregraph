let valid = import "../positive/durable-cancellation.ncl" in
valid & { events | force = ["cancel", "complete", "fail", "mark-ready", "start"] }
