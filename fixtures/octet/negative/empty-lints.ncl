let Contracts = import "../../../contracts/octet/deny-all.ncl" in
{
  config | Contracts.DenyAllConfig = {
    schema = "choregraph.octet-deny-all.v1",
    mode = "deny-all",
    octet_revision = "5b1e119c1dad202be7987eeff21e35770b5a4fff",
    hook_blake3 = "193d21dbbf8fc6250344a7548e481a7589cb8cd72b7389cee13e773ad8e00d75",
    lints = [],
  },
}
