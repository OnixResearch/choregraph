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
  machine_schema | MachineSchema,
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
    claim = "The production-linked indexed fold preserves its invariant across every finite accepted prefix, stops at the first denial, and returns no dispatch intents.",
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
    machine_schema = {
      id = "choregraph.machine.v1",
      source_path = "schemas/preserves/choregraph-machine-v1.preserves",
      source_blake3 = "4cb7079deb17e77fe2806bd24a80f47e5df1ca7c405725429a3ea47236ecb0f6",
    },
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
      "first_failure_is_exact",
      "equal_replay_inputs_have_equal_decisions",
      "replay_never_returns_dispatch_intents",
    ],
    sources = [
      {
        path = "Cargo.lock",
        blake3 = "ede2ada6e66ab5e9ff75ac989805402ea737953d93ff7cd0649d48b77ee00594",
        role = "resolved Rust dependency identity",
      },
      {
        path = "config/project.ncl",
        blake3 = "0c2c754868bd94b6590b2260e46885dc19a440f3b2bba078e88385e38b6eb714",
        role = "reviewed project claim boundary",
      },
      {
        path = "config/toolchain.ncl",
        blake3 = "b2c7874666d71c528c5ce6ca07016d0e57b6f215cedb03350d6b884efa9c05cc",
        role = "reviewed toolchain identity",
      },
      {
        path = "crates/choregraph-core/Cargo.toml",
        blake3 = "b0317889ad7968e1982f4df7fbd81d5bdc40ddea2cbbec852bf07f6fa7c7eb84",
        role = "erased vstd dependency contract",
      },
      {
        path = "crates/choregraph-core/src/lib.rs",
        blake3 = "f491db7f9671a151f7985384cc7a937db21b271d44299455a6211ce7dc377b5a",
        role = "production module and public replay linkage",
      },
      {
        path = "crates/choregraph-core/src/model.rs",
        blake3 = "883ff13ba2ef4c81629767449522519302433f742e984b1d720359b939fc7b76",
        role = "bounded trace, binding, count, and failure types",
      },
      {
        path = "crates/choregraph-core/src/transition.rs",
        blake3 = "7fb94f703451facdad98fc3d4e3b2d4b9c4834df4dde439941074dec6bf49ddb",
        role = "public replay fold over the production step relation",
      },
      {
        path = "crates/choregraph-core/src/verified.rs",
        blake3 = "dca1071b89410ac4193190f78fa68207528712d37a4dafe2a992bf9ef7b43246",
        role = "production-called verified single-step kernel",
      },
      {
        path = "crates/choregraph-core/src/verified/trace.rs",
        blake3 = "16a30a0bfd64357fa2b8ea63d6998f171d224bc01c1c56391b62a743b43a7708",
        role = "production trace specifications and finite induction theorems",
      },
      {
        path = "crates/choregraph-core/tests/trace-replay.rs",
        blake3 = "cb1687418a1b758700b003026c3e066d069ef53d13c7f27100cbcddd35666a32",
        role = "public replay fixtures and repeated-step parity properties",
      },
      {
        path = "schemas/preserves/choregraph-machine-v1.preserves",
        blake3 = "4cb7079deb17e77fe2806bd24a80f47e5df1ca7c405725429a3ea47236ecb0f6",
        role = "canonical machine schema binding",
      },
      {
        path = "evidence/generated/source-baseline.json",
        blake3 = "584be28e6181e7289f69c8ece9a77bd838d6b2261322d1258056e50f8c981de1",
        role = "canonical selected-source baseline",
      },
      {
        path = "evidence/source-baseline.ncl",
        blake3 = "0ec9a6510670e4267fec726ed50cb85f70600e48b3b54c527630a130b93a7ec6",
        role = "reviewed selected-source baseline",
      },
      {
        path = "flake.nix",
        blake3 = "a0d8822131a5a9e965faa2ccb07187e6591a59ced0628f5a71a2c594c3ae9156",
        role = "maintained positive, negative, trust, and evidence gate",
      },
      {
        path = "verus/proofs/trace_replay.rs",
        blake3 = "e436d3c94ea07fb83dee7268b4956fcc2d646287fab69c631204642e27d1321e",
        role = "positive proof root that includes production proof sources",
      },
      {
        path = "verus/trace-failure-fixtures/empty_trace_mutation.rs",
        blake3 = "35e898cbada2c06a7a01108222df2793efd6d5a22a6fd606e3f1635dff12cdf0",
        role = "negative empty-trace mutation fixture",
      },
      {
        path = "verus/trace-failure-fixtures/failure_index_skips_denial.rs",
        blake3 = "77796d593dfee1e40bd20146a32bc9cd171685529489f19e7f57e3b1e9cbd3d6",
        role = "negative first-failure index fixture",
      },
      {
        path = "verus/trace-failure-fixtures/preservation_without_initial_invariant.rs",
        blake3 = "24451cc0d03045f756fa2041b8818fbcb3fe6feec60fd67d10b02bfed8c1867a",
        role = "negative missing initial-invariant fixture",
      },
      {
        path = "verus/trace-failure-fixtures/replay_with_dispatch_intent.rs",
        blake3 = "9400608f314ef3a2316245b667f87848d199454e4ceeed9dcaed3431da5743e1",
        role = "negative dispatch-authority fixture",
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
    ],
    trusted_surface = {
      project_trusted_declarations = [],
      external_assumptions = [
        "The selected Verus binary and Z3 solver implement their documented logic correctly.",
        "The selected Rust compiler erases ghost code and compiles the executable functions correctly.",
        "The vstd sequence and vector contracts match their executable Rust implementations.",
        "The allocator and target platform implement Rust allocation and memory semantics correctly.",
      ],
      adapter_boundary = [
        "Identifier parsing and BTreeMap-to-index conversion are executable Rust and are not Verus-proved.",
        "The public replay fold calls production step and executable property tests compare it with repeated steps.",
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
