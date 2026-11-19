# Every annotated source must declare the complete closed non-claim set.
let valid = import "../positive/annotated-two-branch-choice.ncl" in
let Contract = (import "../../../contracts/nickel/role-choreography-preference-contracts.ncl").AnnotatedRoleChoreography in
(valid & { non_claims | force = ["no-solver"] }) | Contract
