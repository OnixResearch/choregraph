# r[impl choregraph.proof.trace.evidence]
# r[impl choregraph.proof.trace.validation]
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3Digest = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let MaxTraceEvents = 65536 in
let TraceBindingBytes = 32 in
let Source = {
  path | NonEmpty,
  blake3 | Blake3Digest,
  role | NonEmpty,
} in
let NegativeFixture = {
  path | NonEmpty,
  expected_diagnostic | NonEmpty,
  violated_obligation | NonEmpty,
} in
let TrellisSource = {
  path | NonEmpty,
  blake3 | Blake3Digest,
  adapted_claim | NonEmpty,
} in
let TrustedSurface = {
  project_trusted_declarations | Array NonEmpty,
  external_assumptions | Array NonEmpty,
  adapter_boundary | Array NonEmpty,
} in
let Limits = {
  max_trace_events | PositiveNumber,
  trace_binding_bytes | PositiveNumber,
  consumed_count_type | NonEmpty,
  failure_index_type | NonEmpty,
} in
let MachineSchema = {
  id | NonEmpty,
  source_path | NonEmpty,
  source_blake3 | Blake3Digest,
} in
let ProofManifest = {
  schema | NonEmpty,
  status | NonEmpty,
  claim | NonEmpty,
  verified_count | PositiveNumber,
  positive_command_argv | Array NonEmpty,
  verifier_release | NonEmpty,
  verifier_revision | GitRevision,
  verifier_binary_blake3 | Blake3Digest,
  verifier_nix_closure_graph_blake3 | Blake3Digest,
  rust_toolchain | NonEmpty,
  rustc_blake3 | Blake3Digest,
  solver | NonEmpty,
  solver_blake3 | Blake3Digest,
  vstd_crate | NonEmpty,
  trellis_revision | GitRevision,
  trellis_sources | Array TrellisSource,
  machine_schemas | Array MachineSchema,
  limits | Limits,
  theorems | Array NonEmpty,
  sources | Array Source,
  negative_fixtures | Array NegativeFixture,
  trusted_surface | TrustedSurface,
  non_claims | Array NonEmpty,
} in
{
  proof | ProofManifest = {
    schema = "choregraph.trace-replay-proof.v1",
    status = "verified",
    claim = "The production-linked indexed fold derives invariant preservation for every finite accepted prefix from the step guards, checks completed dependencies before each Start, stops at the first denial, and returns no dispatch intents.",
    verified_count = 121,
    positive_command_argv = [
      "octet-production-verus",
      "--crate-type=lib",
      "verus/proofs/trace_replay.rs",
      "-o",
      "$TMPDIR/trace-replay.rlib",
    ],
    verifier_release = "0.2026.05.17.e479cce",
    verifier_revision = "e479cce36490b8fa4b0fd7755aa742aec354372c",
    verifier_binary_blake3 = "2aafa795a63160e2b262e0d8df00ad237766ed87fcd7fe9a981034207b1367c7",
    verifier_nix_closure_graph_blake3 = "5f20a8243ed93278d1574e4269838858e99489abd1158f23ab2ffa8471a8d43b",
    rust_toolchain = "1.95.0-x86_64-unknown-linux-gnu",
    rustc_blake3 = "4b001492d51b266eef82aeb98469d135d60f63d2720d24d9712c1e83fa10c9e3",
    solver = "z3@verus-0.2026.05.17.e479cce",
    solver_blake3 = "42e48a26bf3b38ca33d84749f4317cc179791de0d578d4d98a67f51782b54331",
    vstd_crate = "vstd@0.0.0-2026-01-25-0057-no-default-features-alloc",
    trellis_revision = "aead4527bfc3c714e71669bce656719dd2a6b2e2",
    trellis_sources = [
      {
        path = "src/trace_relation.rs",
        blake3 = "99dabc9c41d3a47ec52ae013c0d36d54312c6a44e3d5e9dcee76749ba6abfcab",
        adapted_claim = "finite trace claims require an explicit empty base and stable induction step",
      },
      {
        path = "src/state_machine_workflow.rs",
        blake3 = "55d696cd692e7e74cbac3c0fbfb2bc6808e7356b0e4c8867042fe2f7e23f24af",
        adapted_claim = "trace admission binds an explicit step count to a reviewed maximum",
      },
    ],
    machine_schemas = [
      {
        id = "choregraph.machine.v1",
        source_path = "schemas/preserves/choregraph-machine-v1.preserves",
        source_blake3 = "4cb7079deb17e77fe2806bd24a80f47e5df1ca7c405725429a3ea47236ecb0f6",
      },
      {
        id = "choregraph.machine.v2",
        source_path = "schemas/preserves/choregraph-machine-v2.preserves",
        source_blake3 = "2c77b25b6f4f90033e42f6b84b5764a4b503ee56e80b098b254ad3a0ea125305",
      },
    ],
    limits = {
      max_trace_events = MaxTraceEvents,
      trace_binding_bytes = TraceBindingBytes,
      consumed_count_type = "u64",
      failure_index_type = "u64",
    },
    theorems = [
      "empty_trace_base",
      "admitted_trace_respects_limit",
      "replay_prefix_length_bounded",
      "finite_replay_preserves_invariant",
      "accepted_suspended_prefix_remains_consistent",
      "first_failure_is_exact",
      "equal_replay_inputs_have_equal_decisions",
      "replay_never_returns_dispatch_intents",
      "accepted_trace_starts_after_dependencies",
    ],
    sources = [
      { path = ".cargo/config.toml", blake3 = "a3a61c8e85aab80b6612fd9b5c111c9c184bb879ee5cd9489a7cead1e2b67683", role = "scoped Rust bootstrap exception for the Trellis dependency" },
      { path = "tools/choregraph-foundation-check/proof-policy.json", blake3 = "8354f239d0f6288e4b1175d82a51f78a094c3434f07f2d22135ec4aa6c1047a0", role = "independent proof item and theorem inventory" },
      {
        path = "Cargo.lock",
        blake3 = "19dc051cfbec627d81445a0236bcf503fcaad40ca8cc243e0092f305012bbf44",
        role = "resolved Rust dependency identity",
      },
      {
        path = "config/project.ncl",
        blake3 = "d078f3f80af66c0be83acb97ee0d6ba998ff445fd28ed116bd365483952be664",
        role = "reviewed project claim boundary",
      },
      {
        path = "config/toolchain.ncl",
        blake3 = "913b564a4797e32a1fe4421d4869f27ee98ca517c0625b3213ea09761d8cfd95",
        role = "reviewed toolchain identity",
      },
      {
        path = "crates/choregraph-core/Cargo.toml",
        blake3 = "b0317889ad7968e1982f4df7fbd81d5bdc40ddea2cbbec852bf07f6fa7c7eb84",
        role = "erased vstd dependency contract",
      },
      {
        path = "crates/choregraph-core/src/concrete.rs",
        blake3 = "9a05ee11e384ad5e7263a7fd05f0913623a5606677f5f2ab0f817e93c4dd7636",
        role = "production v2 replay event and intent boundary",
      },
      {
        path = "crates/choregraph-core/src/lib.rs",
        blake3 = "361d463ca5bd725bba0a01f0147f0c0f3d8564e906b1dfe0cc7fad9d7e7f0f03",
        role = "production module and public replay linkage",
      },
      {
        path = "crates/choregraph-core/src/model.rs",
        blake3 = "9c71a510408c83e5b6ebbea845d2d4bd5fb7966d9490565b4287e932d11a469c",
        role = "bounded trace, binding, count, and failure types",
      },
      {
        path = "crates/choregraph-core/src/transition.rs",
        blake3 = "90323de4a7615d6fc0e5d9c88e890ba13b3fff7b2f8efac67fb589dc02a65073",
        role = "public replay fold over the production step relation",
      },
      {
        path = "crates/choregraph-core/src/verified.rs",
        blake3 = "b010b26797d52b95c9db0ec6c66f1d40813a78c8300371b660b512fd1993c965",
        role = "verified primitives and module linkage",
      },
      {
        path = "crates/choregraph-core/src/verified/kernel.rs",
        blake3 = "3fe79821068bde8cd2556986a0b01a6f09d9bac83a7d7d5a52b758578dc1a70e",
        role = "production-called verified single-step kernel",
      },
      {
        path = "crates/choregraph-core/src/verified/readiness.rs",
        blake3 = "5ef37907840372363edf5d6be6699dbb927dff9fbb65c9a3f21ca3238909852e",
        role = "verified dependency readiness included by the replay proof root",
      },
      {
        path = "crates/choregraph-core/src/verified/trace.rs",
        blake3 = "89b358d694901d69d2c5fdebb008a73ac418ed2cb96578c8af2445f5204f21bd",
        role = "production trace specifications and finite induction theorems",
      },
      {
        path = "crates/choregraph-core/tests/trace-replay.rs",
        blake3 = "4e55222861f0cfdc0cddb92e1ee6a0c9c3c057e0c49dfb460aa9e505627e21b1",
        role = "public replay fixtures and repeated-step parity properties",
      },
      {
        path = "schemas/preserves/choregraph-machine-v1.preserves",
        blake3 = "4cb7079deb17e77fe2806bd24a80f47e5df1ca7c405725429a3ea47236ecb0f6",
        role = "canonical machine schema binding",
      },
      {
        path = "evidence/generated/source-baseline.json",
        blake3 = "9bea1c4bd45f91be4a4e2d40ceaedd824bdb74fd869c45974f5d2b72d0df5186",
        role = "canonical selected-source baseline",
      },
      {
        path = "evidence/source-baseline.ncl",
        blake3 = "0ec9a6510670e4267fec726ed50cb85f70600e48b3b54c527630a130b93a7ec6",
        role = "reviewed selected-source baseline",
      },
      {
        path = "flake.nix",
        blake3 = "c0fa05b45bb67635fabe812ce6605eef2f9530345f649daecc32d9229daed0d8",
        role = "maintained positive, negative, trust, and evidence gate",
      },
      {
        path = "verus/proofs/trace_replay.rs",
        blake3 = "e436d3c94ea07fb83dee7268b4956fcc2d646287fab69c631204642e27d1321e",
        role = "positive proof root that includes production proof sources",
      },
      {
        path = "verus/trace-failure-fixtures/empty_trace_mutation.rs",
        blake3 = "28c90cd4aaf4bb1de774456735353db23ba33e5c713a8ce4c245d618a113a37e",
        role = "negative empty-trace mutation fixture",
      },
      {
        path = "verus/trace-failure-fixtures/failure_index_skips_denial.rs",
        blake3 = "55fd297ee783afd03436d92713bb0fcca8e41fcbff671f32b97b5abe63451efa",
        role = "negative first-failure index fixture",
      },
      {
        path = "verus/trace-failure-fixtures/preservation_without_initial_invariant.rs",
        blake3 = "5f7881f844df71e070803a5f2a2ad36f09b1efda1a34fc05e2db209b4f5b038a",
        role = "negative missing initial-invariant fixture",
      },
      {
        path = "verus/trace-failure-fixtures/replay_with_dispatch_intent.rs",
        blake3 = "ef2868ec47d264cc88c3ab6f5d354ca1efc8fe9acd528b92c9048532a1cd4236",
        role = "negative dispatch-authority fixture",
      },
      {
        path = "verus/trace-failure-fixtures/suspended_trace_drops_fact.rs",
        blake3 = "363e965807f298366e633ef8cda0c8600bd9ac531f13c1be2d72d214a929331e",
        role = "negative suspension-fact replay fixture",
      },
      {
        path = "crates/choregraph-core/src/admission.rs",
        blake3 = "c6cc30d4ff82a846e1ef5037d59fd4c11cf23d2078047d4e8d311ffe06085494",
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
        blake3 = "54490a226e32f7dbc544c0056176dd63a354d6e583e2fe278c1395ee503e93f0",
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
        blake3 = "66639142b219dc7af4d9f0f85f1f8186fcc01a8e4553bbec113aa317e3063904",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/invariant.rs",
        blake3 = "b35a2615c571a960bf93e3d594f6b05244ed6c00b34a5ca817351a9188d0a866",
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
        blake3 = "1ded3f9810619398ccdd02fd9cea28cac43b4c1782ba37572410acc7f953824c",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/model/pause.rs",
        blake3 = "8d0d6bbcdf6e821126b2672b57bdfe22b94d26ae8ff59d11a3152b91b7bc3cc3",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/model/semantics.rs",
        blake3 = "d1b0242e4b997e43e7fd42945200d81d8133e5ac7524742adc44c957506de3a8",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/ownership/mod.rs",
        blake3 = "5dc5e4da9e063f69c3328380a037306ae634c69155871fad4ce8fb7c1956e462",
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
        blake3 = "567f8a79960fd611b381cd3e7fcac3383529e61de6384bac20b85bec286cd20e",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/verified/codes/status.rs",
        blake3 = "c1f44c669d270468f073a6c300c52bf107a465b0bac8013431ec8ac5c3584f57",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/verified/kernel/decision.rs",
        blake3 = "4ed830075a3b3e6f766c4ddcabcb959c6087fa955a73f56003132d5dbc5b4247",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/verified/kernel/snapshot.rs",
        blake3 = "dee13990db834962613bb01353da74f885bfe52f907e175fffe6d97ee6ffc4fa",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/verified/kernel/target.rs",
        blake3 = "367030b671d4d0c5d0752e55e6efa5a6731454d9b128736d3d91f564d970ff80",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/verified/kernel/theorem.rs",
        blake3 = "dc5774402bf701fcb16b9426af16ed6aa276e90eddb4d1a4f929841f7b88cae1",
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
        blake3 = "5d51f9ef054c88d4bd4e17af05fd89386700bcaf3985f2e414af5f7ae1451640",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/verified/primitive/transition.rs",
        blake3 = "1d82dab149c5a479a9b044c62297f4453397c6eb89b1ab6d919ae946f9200f9b",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/verified/primitive/validity.rs",
        blake3 = "fea7371a6a1527ade3322ed1748221c6d98ddeddca0d36050e61250f1090d391",
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
        path = "crates/choregraph-core/src/verified/trace/specification.rs",
        blake3 = "2faf8891b7c1f3b94440b29dfc0438a2a2dfed5b441c49e1849222b42f61a9ea",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
      {
        path = "crates/choregraph-core/src/verified/trace/theorem.rs",
        blake3 = "86f1267b38748824908e08decc1ea7acec522bf5c4e8b9f64f254ea8c3b67307",
        role = "production core module in the bound source closure; theorem and adapter boundaries remain explicit",
      },
    ],
    negative_fixtures = [
      {
        path = "verus/trace-failure-fixtures/empty_trace_mutation.rs",
        expected_diagnostic = "postcondition not satisfied: replay_final_states_from_spec",
        violated_obligation = "an empty trace mutates the initial state",
      },
      {
        path = "verus/trace-failure-fixtures/failure_index_skips_denial.rs",
        expected_diagnostic = "postcondition not satisfied: replay_failure_index_from_spec",
        violated_obligation = "failure reporting skips the first denied event",
      },
      {
        path = "verus/trace-failure-fixtures/preservation_without_initial_invariant.rs",
        expected_diagnostic = "postcondition not satisfied: indexed_invariant_spec",
        violated_obligation = "trace safety omits the invariant initial-state premise",
      },
      {
        path = "verus/trace-failure-fixtures/replay_with_dispatch_intent.rs",
        expected_diagnostic = "postcondition not satisfied: replay_dispatch_intents_spec",
        violated_obligation = "replay returns fresh dispatch authority",
      },
      {
        path = "verus/trace-failure-fixtures/suspended_trace_drops_fact.rs",
        expected_diagnostic = "postcondition not satisfied: replay_final_suspensions_from_spec",
        violated_obligation = "replay drops the suspension fact recorded by an accepted suspend",
      },
    ],
    trusted_surface = {
      project_trusted_declarations = [],
      external_assumptions = [
        "The selected Verus binary and Z3 solver implement their documented logic correctly.",
        "The selected Rust compiler erases ghost code and compiles the executable functions correctly.",
        "The vstd sequence and vector contracts match their executable Rust implementations.",
        "The allocator and target platform implement Rust allocation and memory semantics correctly.",
        "Normal Cargo builds enable RUSTC_BOOTSTRAP only for the pinned verified_logic dependency; the selected Verus toolchain is separate.",
      ],
      adapter_boundary = [
        "Identifier parsing and BTreeMap-to-index conversion are executable Rust and are not Verus-proved.",
        "The public replay folds call production step or step_concrete; executable property tests compare them with repeated steps.",
        "Caller-supplied machine and initial-state binding derivation is not proved by Choregraph core.",
        "The proof starts after bounded indexed trace admission, not at storage, source text, or observed history.",
      ],
    },
    non_claims = [
      "This evidence does not prove Verus, Z3, rustc, vstd, the allocator, or the operating system.",
      "This evidence does not prove event truth, authorization, history completeness, or canonical storage order.",
      "This evidence does not prove storage durability, crash recovery, liveness, fairness, or scheduler progress.",
      "This evidence does not prove runtime effect success, fresh authorization, or external exactly-once behavior.",
      "This evidence does not prove consumer BLAKE3 binding derivation or arbitrary Nickel evaluation.",
      "Trellis source identities record adapted proof inputs and do not transfer Trellis certification.",
    ],
  },
}
