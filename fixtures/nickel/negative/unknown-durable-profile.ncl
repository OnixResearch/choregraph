let valid = import "../positive/durable-cancellation.ncl" in
valid & { lifecycle_profile | force = "choregraph.lifecycle.durable-suspension.v3" }
