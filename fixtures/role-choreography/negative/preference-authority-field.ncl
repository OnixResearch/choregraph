# The annotated source shape is closed: preferences cannot carry authority.
let valid = import "../positive/annotated-two-branch-choice.ncl" in
let Contract = (import "../../../contracts/nickel/role-choreography-preference-contracts.ncl").AnnotatedRoleChoreography in
(valid & { authority = "ambient" }) | Contract
