let valid = import "../positive/declared-import.ncl" in
let Contracts = import "../../../contracts/nickel/restricted-graph-contracts.ncl" in
let escaped = valid & {
  imports | force = [{
    logical_id = "secret",
    root_id = "source",
    relative_path = "../secret",
    media_class = "utf8-text",
    max_bytes = 16,
    expected_blake3 = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
  }],
} in
escaped | Contracts.EvaluatedGraph
