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
    { path = "crates/choregraph-core/src/ownership/mod.rs", blake3 = "2a915df3b08d4ace07591516e7abf870294a7a93e017e3cce7200cf791f0a501", role = "typed pure gate and production transition wrapper" },
    { path = "crates/choregraph-core/src/verified.rs", blake3 = "b010b26797d52b95c9db0ec6c66f1d40813a78c8300371b660b512fd1993c965", role = "production-linked Verus decision and module linkage" },
    { path = "crates/choregraph-core/src/verified/readiness.rs", blake3 = "5ef37907840372363edf5d6be6699dbb927dff9fbb65c9a3f21ca3238909852e", role = "readiness module included by the positive proof root" },
    { path = "crates/choregraph-core/tests/epoch-fence.rs", blake3 = "336eeae93064789cf06fa9dcd33c3406d2f1b9696b67bd2e4b132b80dd9de697", role = "positive, negative, terminal, and cross-entity tests" },
    { path = "verus/proofs/epoch_fence.rs", blake3 = "32f60db7e922d7351ff8f25131d8d8f71f9af5a2c78c7f761c8b6056d581fbd3", role = "positive formal obligations" },
    { path = "verus/epoch-fence-failure-fixtures/cross_entity_admits.rs", blake3 = "ab7e3c516dd2f45f35a77f939e88119b61f2ce4ce2b41e8f21edd160ab8b632d", role = "cross-entity false-claim fixture" },
    { path = "verus/epoch-fence-failure-fixtures/stale_epoch_admits.rs", blake3 = "ee214e47f22d9dc972f7da0735b51a31f174a6e0068273d47e702b566fa1efa2", role = "stale-epoch false-claim fixture" },
    { path = "verus/epoch-fence-failure-fixtures/terminal_mutation_admits.rs", blake3 = "c6f9d8c0b7549a1dcf6aadee59b1ed083568e17973145661408be8ae56171972", role = "terminal-mutation false-claim fixture" },
    { path = "docs/epoch-fenced-single-writer.md", blake3 = "bb9014fbc27544fa44b4a08d190a6596938d19b4579c2a6db07ad3f3544c6891", role = "API, reference, and claim boundary" },
    { path = "fixtures/epoch-fence/negative/shared-store-coupling.rs", blake3 = "c2512d9826e71237b5c045ce9a5f1563bff7ec8353b39965d54099774deaf8de", role = "shared-mutable-store negative boundary fixture" },
    {
      path = "crates/choregraph-core/src/admission.rs",
      blake3 = "83e4d722a54b9ec4bfffe0984f39ee8e57410e04ea1f9e4e371dcfc56944c153",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/analytics/mod.rs",
      blake3 = "c70b59e1a90badbab1249c59f8471b9b965f70da9a311d74eb4aca3b34cc5fe1",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/boundary.rs",
      blake3 = "22dc9fd98919fe75c1d85b13517d18bf22cc0b857dcee59c3b8f441046c0cd25",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/concrete.rs",
      blake3 = "820549c379f2d011cd1b374d57930c6fb5035fb6672fdf2b01a4ed612ca0bbf1",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/concrete/effect.rs",
      blake3 = "0d7fd859c037ace5cb9af0300fc2b9aafc2b087c6a5824205be307dfa38efeed",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/concrete/event.rs",
      blake3 = "d2d3d4dfc2dc4081d810d45f7ef49881d5ad667f316099d66e3c4e1b71ad530c",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/concrete/machine.rs",
      blake3 = "e703018dec1ecfa60762c730a0eb1ba79d008f6ac667fbd60e13a0494552117a",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/concrete/reference.rs",
      blake3 = "cecf64e525a80857c605003e9a81b394096426569a80d854eeaba69a306bf313",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/concrete/replay.rs",
      blake3 = "5b4fe54c25748fecac5ae2702e861edb4fce514da8475dbfc2748bd57f509a4d",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/id.rs",
      blake3 = "99a0d54542d8f68f767172ffa965b5e8620809866c33807054e43800bc80b0ff",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/id/bounded.rs",
      blake3 = "59c08bf219531624efd96f6aa7974d283e2214a5b4db915b710f275efe82188d",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/id/domains.rs",
      blake3 = "63817140746ea2ab0db2056ea9da921d2026ff32061eee26f21a4f6aae37944c",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/id/reference.rs",
      blake3 = "8f0e75add17ea67253de65c716b68f4eb9efbe4a4b9391dc9bc4836bb6d452a6",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/invariant.rs",
      blake3 = "8774cdce7edf40f04a68ae845ed0e7124cbddddc6c7a2f2aa31fdc97712bef99",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/lib.rs",
      blake3 = "361d463ca5bd725bba0a01f0147f0c0f3d8564e906b1dfe0cc7fad9d7e7f0f03",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/model.rs",
      blake3 = "9c71a510408c83e5b6ebbea845d2d4bd5fb7966d9490565b4287e932d11a469c",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/model/analysis.rs",
      blake3 = "c8b043046e8206a82319ae9a04830fd75989108f55cbd8fda159722ac6cad599",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/model/automaton.rs",
      blake3 = "eb484f15e22ec1b75e995cdebd781564704376d432810b59cb08b2d6f4529e46",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/model/capacity.rs",
      blake3 = "8a17cdcdfc538376e8a10e2a9cae71755f0d2e4e40cf00982fcfeb3cdfce9548",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/model/dispatch.rs",
      blake3 = "f17ec2d2de27798f8d7d62d1ccedd39036a95b65722061cd54faaca80528c300",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/model/graph.rs",
      blake3 = "c09d90e818f66726547529cb999a03cd73a1d7b07d6be3be2e751fb05b9f38a2",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/model/history.rs",
      blake3 = "71cbf18f32c8dbac48273c15b00f2ea8902e50078c47ddb782462429b99ee41b",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/model/outcome.rs",
      blake3 = "cb69664e2468d313768112cbccf99ac124481cdff2dcac68f3e539393061bb48",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/model/pause.rs",
      blake3 = "8d0d6bbcdf6e821126b2672b57bdfe22b94d26ae8ff59d11a3152b91b7bc3cc3",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/model/semantics.rs",
      blake3 = "206ed3d32c41c8f2225dbb41ef47350a9857ee807e77d01d140ac1adc3306fec",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/transition.rs",
      blake3 = "9d8c6d178139361738ca14a325a320afc9107c5b0c4feb2542c1161a614a8ff6",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/codes.rs",
      blake3 = "ddf69829fc81ee1f3812664fe57f10571af644f8d80101593b6e7514b8791243",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/codes/action.rs",
      blake3 = "f5eb60e80ff2323e931ce24ed342b498b92117fb1af8bf274258104ffbacc50a",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/codes/denial.rs",
      blake3 = "0b5d7b69c7dfce76ec89af734038aa4aa632d82ed1604a3b8199595500f14b04",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/codes/denial/suspension.rs",
      blake3 = "9d0ca1068b3a254603832012597b58b37c4583c3d23e943b257c2573db854bcd",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/codes/denial/transition.rs",
      blake3 = "d8cfc4cdfc680cf508b41e55ab34ef40dc8548e0364694331c5f2c3c2e4f52ca",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/codes/profile.rs",
      blake3 = "a037e48fa6ed4ddc03257a61c0aef7eb4556d040c51db32e6a8d595f258adefa",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/codes/status.rs",
      blake3 = "c1f44c669d270468f073a6c300c52bf107a465b0bac8013431ec8ac5c3584f57",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/kernel.rs",
      blake3 = "3fe79821068bde8cd2556986a0b01a6f09d9bac83a7d7d5a52b758578dc1a70e",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/kernel/decision.rs",
      blake3 = "70f98598e1be11a97867421bfa617090e289059ee3ce29034c3e843e770f8efc",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/kernel/snapshot.rs",
      blake3 = "6d5e6e439e982656c27b88237106c0d928c66ab26babcdbaa17a8ebe0ed6de92",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/kernel/target.rs",
      blake3 = "5047dfd1edc9420872caf004f1600ed6f6139e88c68796be0d79e00307fe3632",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/kernel/theorem.rs",
      blake3 = "3b769426a94feb4c06df3dfe994b95c67c1b6bf18cfd7934fda0aec4723b23cf",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/primitive.rs",
      blake3 = "9363729a6f94532d0d5586777bd9885f9fc6e1b67f8b2f7425f8c09baddb5a0f",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/primitive/matching.rs",
      blake3 = "e14bb1d90d62313deff940a95a35576d606c1b7b1656baad99ca7f53b1491750",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/primitive/membership.rs",
      blake3 = "ed8dec62a885a3ce277b117bc60c1e4943f07073b60ce1f69b5b02732938bfe3",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/primitive/ownership.rs",
      blake3 = "00b0bb46e23ca43c1e30bac7fd095009a2d211d004d59c06d898c7a910a47058",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/primitive/support.rs",
      blake3 = "5cfcb696e93cfcd4626743865826b6f0d051343201ece45dcf83ad4bcc803bc9",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/primitive/transition.rs",
      blake3 = "92675113af8a6690befb0b42c65d330f7e9391e0c1e6569f8f17b42ace6f181c",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/primitive/validity.rs",
      blake3 = "1d39160c3c0be1dc85727da6b19956bfc60f5c392518be78dd3f094cd8c1ffbd",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/topology.rs",
      blake3 = "022c97236544c95b6774b4bd1b640b3f8bb28989429df2b444ec75ff395c03e1",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/topology/admission.rs",
      blake3 = "ed5f8541fa702e1bfc22b2359de6462400862d2f17ccaac74e4755cee2c581dd",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/topology/bounds.rs",
      blake3 = "825d8e0e87de04bac0555591ab4e2b53929108b5df8ab635ada2cb4fa1b4dc60",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/topology/cardinality.rs",
      blake3 = "0d9ec7a0bccfee995670aea8be64fc4188dd01a2d44b6c9dd56a401ffb7f1088",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/topology/precedence.rs",
      blake3 = "e4232e8cc9d7101a91592c889d7d1a5aa2aa5266b916c44536829e84cbedc1b1",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/trace.rs",
      blake3 = "89b358d694901d69d2c5fdebb008a73ac418ed2cb96578c8af2445f5204f21bd",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/trace/specification.rs",
      blake3 = "f2eea3b57fdc654cbab178ee20d4cc424bff38990a6f670dec7ec57a6af839ce",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
    {
      path = "crates/choregraph-core/src/verified/trace/theorem.rs",
      blake3 = "40846e70f75aecf578c19090d6c2d1402e6493e42108ec3fc045448aec563af7",
      role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
    },
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
