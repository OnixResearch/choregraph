# A choice names at most one exogenous world variable, never a list.
let valid = import "../positive/annotated-two-branch-choice.ncl" in
let Contract = (import "../../../contracts/nickel/role-choreography-preference-contracts.ncl").AnnotatedRoleChoreography in
(valid & { annotations.choices | force = [{ path = [], decider = 1, factors = [1, 2], world = [1], span = { start = 24, end = 25 } }] }) | Contract
