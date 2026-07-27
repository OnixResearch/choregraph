let Contracts = import "../contracts.ncl" in
(import "../default.ncl") & {
  traceability_policy.profiles = [{
    id = "cairn-default",
    requirement_sources = [
      { root = "cairn/specs", source_kind = "accepted_specs", extensions = ["md"] },
    ],
    evidence_sources = [
      { root = "src", source_kind = "ambient_source", extensions = ["rs"] },
    ],
    marker_verbs = ["impl", "verify", "depends", "related"],
  }],
} | Contracts.Policy
