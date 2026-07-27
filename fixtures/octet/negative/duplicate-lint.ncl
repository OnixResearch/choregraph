let Contracts = import "../../../contracts/octet/deny-all.ncl" in
{
  config | Contracts.DenyAllConfig = {
    schema = "choregraph.octet-deny-all.v1",
    mode = "deny-all",
    octet_revision = "374bd16b26cee2af34211a29bfa531c016811f51",
    hook_blake3 = "41603febce44635585c6010557f6991008976b7e2f7e417c9c78f5299690522f",
    lints = ["magic_number", "magic_number"],
  },
}
