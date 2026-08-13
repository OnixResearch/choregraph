let valid = import "../positive/two-role-transfer.ncl" in
let Contract = (import "../../../contracts/nickel/role-choreography-contracts.ncl").RoleChoreography in
(valid & { limits.max_roles | force = 65 }) | Contract
