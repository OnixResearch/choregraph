# The v1 cooperative baseline contract stays closed: annotations cannot be
# smuggled into an unannotated source.
let valid = import "../positive/two-role-transfer.ncl" in
let Contract = (import "../../../contracts/nickel/role-choreography-contracts.ncl").RoleChoreography in
(valid & { annotations = { factors = [], worlds = [], choices = [] } }) | Contract
