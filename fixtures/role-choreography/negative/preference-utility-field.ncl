# A utility weight is not part of the closed annotation shape.
let valid = import "../positive/annotated-two-branch-choice.ncl" in
let Contract = (import "../../../contracts/nickel/role-choreography-preference-contracts.ncl").AnnotatedRoleChoreography in
(valid & { annotations.factors | force = [{ id = 1, role = 1, name = "cost", weight = 3, span = { start = 18, end = 19 } }] }) | Contract
