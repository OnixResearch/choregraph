# r[verify choregraph.epoch_fence.invariant]
# r[verify choregraph.epoch_fence.decision]
# r[verify choregraph.epoch_fence.proof]
# r[verify choregraph.epoch_fence.verification]
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3Reference = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^blake3:[0-9a-f]{64}$" value
) in
let Blake3Digest = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^[0-9a-f]{64}$" value
) in
let Source = {
  path | NonEmpty,
  blake3 | Blake3Digest,
  role | NonEmpty,
} in
let Evidence = {
  schema | NonEmpty,
  status | NonEmpty,
  contract_identity | Blake3Reference,
  implementation_identity | Blake3Reference,
  proof_tool | NonEmpty,
  positive_proof | NonEmpty,
  negative_fixtures | Array NonEmpty,
  sources | Array Source,
  reference | {
    repository | NonEmpty,
    revision | NonEmpty,
    license | NonEmpty,
    ownership_source | NonEmpty,
    decision_source | NonEmpty,
    claim_boundary | NonEmpty,
  },
  non_claims | Array NonEmpty,
} in
{
  schema = "choregraph.epoch-fence-proof.v1",
  status = "verified",
  contract_identity = "blake3:791e39edd60b0ae0520bc57b08cc13e1b6d36fe0679faf798d8f939995afe63c",
  implementation_identity = "blake3:208455a5a682f658e7a0c4fa0cdd3224a2cc0e44d11ca5615450224c972717c7",
  proof_tool = "octet-production-verus@0.2026.05.17.e479cce",
  positive_proof = "verus/proofs/epoch_fence.rs",
  negative_fixtures = [
    "verus/epoch-fence-failure-fixtures/cross_entity_admits.rs",
    "verus/epoch-fence-failure-fixtures/stale_epoch_admits.rs",
    "verus/epoch-fence-failure-fixtures/terminal_mutation_admits.rs",
  ],
  sources = [
    { path = "crates/choregraph-core/src/ownership/mod.rs", blake3 = "f7c1d682962d757b0919893ff1f96b56475adff09cd200232886cb4fdbe4beea", role = "typed pure gate and production transition wrapper" },
    { path = "crates/choregraph-core/src/verified.rs", blake3 = "ea5716274215e5b55d63132e532b27bd92a658d56b002ae91391d15232dae5a4", role = "production-linked Verus decision" },
    { path = "crates/choregraph-core/tests/epoch-fence.rs", blake3 = "bc1fa42d2d80e13fd189758797894efd995404e8c150ea1e374bc42aa61f3aa7", role = "positive, negative, terminal, and cross-entity tests" },
    { path = "verus/proofs/epoch_fence.rs", blake3 = "48558d89808eb179c4b26efac62ecb6f3dad64799e7f71cefafe4e6991f61fa9", role = "positive formal obligations" },
    { path = "verus/epoch-fence-failure-fixtures/cross_entity_admits.rs", blake3 = "ecc2a83316e229734126c275dd5af81067ac5aaa16f014e757f395f183ea0d09", role = "cross-entity false-claim fixture" },
    { path = "verus/epoch-fence-failure-fixtures/stale_epoch_admits.rs", blake3 = "687399285d7c094dc6e5397e29733c27a2dd8f6ab3a882de90857bbae1b59117", role = "stale-epoch false-claim fixture" },
    { path = "verus/epoch-fence-failure-fixtures/terminal_mutation_admits.rs", blake3 = "414de55f22b94720e040097b4e3d1482f04d49258e6c668ccfd9433cc9f2a624", role = "terminal-mutation false-claim fixture" },
    { path = "docs/epoch-fenced-single-writer.md", blake3 = "4a6ed2c8f95639b3b8cfd1bb52356365e26819fa85ddf22838db689e84994af3", role = "API, reference, and claim boundary" },
    { path = "fixtures/epoch-fence/negative/shared-store-coupling.rs", blake3 = "c2512d9826e71237b5c045ce9a5f1563bff7ec8353b39965d54099774deaf8de", role = "shared-mutable-store negative boundary fixture" },
  ],
  reference = {
    repository = "https://github.com/denoland/celld",
    revision = "553ae73f83c87c3f7c7a5f73c32c2211d9d7341f",
    license = "Apache-2.0",
    ownership_source = "crates/celld/ownership_store.rs blob 63f1fa367876003f442fa06959fb73965e1a78ce",
    decision_source = "crates/logic/lib.rs blob 3b7a469639cc979bb56655a7d8af227e6b8ac382",
    claim_boundary = "bounded mechanism comparison only; no code, parity, equivalence, interoperability, or correctness transfer",
  },
  non_claims = [
    "the supplied current epoch is not independently observed or authorized",
    "epoch equality does not acquire or renew a lease",
    "the decision does not provide storage atomicity or durability",
    "the decision does not provide consensus or membership",
    "the decision does not prove a runtime has exactly one writer",
    "one-entity isolation does not prove cross-entity transaction safety",
    "the Celld reference does not establish parity or equivalence",
    "passing evidence does not establish release eligibility",
  ],
} | Evidence
