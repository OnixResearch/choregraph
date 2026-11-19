# The two-branch choice of `projectable_choice_includes_uninvolved_role`, with
# declared preference factors and one exogenous world variable on decider 1.
# Factors are names only; they choose no branch and grant no authority.
let Base = import "../../../contracts/nickel/role-choreography-contracts.ncl" in
let Contract = (import "../../../contracts/nickel/role-choreography-preference-contracts.ncl").AnnotatedRoleChoreography in
{
  schema = "choregraph.role-choreography.preference-annotations.v1",
  base = {
    schema = "choregraph.role-choreography.v1",
    limits = Base.ReviewedLimits,
    imports = [],
    roles = [
      { id = 1, kind = { kind = "native" }, span = { start = 0, end = 1 } },
      { id = 2, kind = { kind = "native" }, span = { start = 2, end = 3 } },
      { id = 3, kind = { kind = "native" }, span = { start = 4, end = 5 } },
    ],
    values = [{ id = 10, payload_tag = 20, initial_owner = 1, span = { start = 6, end = 7 } }],
    protocol = {
      kind = "choice",
      decider = 1,
      branches = [
        {
          label = 30,
          body = {
            kind = "communication",
            from = 1,
            to = 2,
            label = 30,
            payload_tag = 20,
            next = { kind = "end", span = { start = 12, end = 13 } },
            span = { start = 10, end = 13 },
          },
          span = { start = 9, end = 13 },
        },
        {
          label = 31,
          body = {
            kind = "communication",
            from = 1,
            to = 2,
            label = 31,
            payload_tag = 21,
            next = { kind = "end", span = { start = 16, end = 17 } },
            span = { start = 15, end = 17 },
          },
          span = { start = 14, end = 17 },
        },
      ],
      span = { start = 8, end = 17 },
    },
    non_claims = Base.RequiredNonClaims,
  },
  annotations = {
    factors = [
      { id = 1, role = 1, name = "cost", span = { start = 18, end = 19 } },
      { id = 2, role = 1, name = "latency", span = { start = 20, end = 21 } },
    ],
    worlds = [{ id = 1, name = "market", span = { start = 22, end = 23 } }],
    choices = [
      { path = [], decider = 1, factors = [1, 2], world = 1, span = { start = 24, end = 25 } },
    ],
  },
  non_claims = [
    "no-branch-selection",
    "no-equilibrium-analysis",
    "no-game-semantics",
    "no-label-boolean-conversion",
    "no-preference-authority",
    "no-self-interested-projection-soundness",
    "no-solver",
    "no-utility-function",
    "no-world-distribution",
  ],
} | Contract
