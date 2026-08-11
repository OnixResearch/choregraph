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
    claim = "The production-linked indexed fold preserves every finite v1 or v2 accepted prefix, stops at the first denial, and returns no dispatch intents.",
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
    ],
    sources = [
      {
        path = "Cargo.lock",
        blake3 = "d6fcb22b5927639b4b0f543a126c4f43f10fd8dd467ceca6c3860ce634a53652",
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
        path = "crates/choregraph-core/src/concrete.rs",
        blake3 = "46a856dd917648d909170f54f4fc70713cae7bb44dc621527609764c67fe9273",
        role = "production v2 replay event and intent boundary",
      },
      {
        path = "crates/choregraph-core/src/lib.rs",
        blake3 = "f5dfef4f2359c5a7e19e9e9e631613b3d8561295babbcf619e531025f51336aa",
        role = "production module and public replay linkage",
      },
      {
        path = "crates/choregraph-core/src/model.rs",
        blake3 = "7e379a958051bdf1d095da0d9ea4329a3b232f4825e5c1f1d96067ede3c0bf61",
        role = "bounded trace, binding, count, and failure types",
      },
      {
        path = "crates/choregraph-core/src/transition.rs",
        blake3 = "0ceb97566ae5308a1fff9d560fd56c330819e5f359673035ea36365c89c5ac83",
        role = "public replay fold over the production step relation",
      },
      {
        path = "crates/choregraph-core/src/verified.rs",
        blake3 = "0f3e394dfb3eb752a67e91516c1900f0a8845244773d2ea087ee8819a67f8a3a",
        role = "verified primitives and module linkage",
      },
      {
        path = "crates/choregraph-core/src/verified/kernel.rs",
        blake3 = "c4d856e44051469be57f6cce5c5457ab98805e19b4d36ae64462d86fb428eefc",
        role = "production-called verified single-step kernel",
      },
      {
        path = "crates/choregraph-core/src/verified/trace.rs",
        blake3 = "9599b74608ae6cf33de0d8e85f1ca80000af1d02b51e35a1c74e65f070399d8a",
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
        blake3 = "b4bee82af6f5455a888ae37d7dffbf1382349f3a1f21b2972a9a77fc44fa6a3f",
        role = "maintained positive, negative, trust, and evidence gate",
      },
      {
        path = "verus/proofs/trace_replay.rs",
        blake3 = "e436d3c94ea07fb83dee7268b4956fcc2d646287fab69c631204642e27d1321e",
        role = "positive proof root that includes production proof sources",
      },
      {
        path = "verus/trace-failure-fixtures/empty_trace_mutation.rs",
        blake3 = "c795a2d1fe81b8a106e9e2d884324c45d1697606d5a27490d68f321e6c830769",
        role = "negative empty-trace mutation fixture",
      },
      {
        path = "verus/trace-failure-fixtures/failure_index_skips_denial.rs",
        blake3 = "c7d5a262a567a01decb15a905a4ef9252292ba259205b45e2b656653fabb5c24",
        role = "negative first-failure index fixture",
      },
      {
        path = "verus/trace-failure-fixtures/preservation_without_initial_invariant.rs",
        blake3 = "f761278218ec17fe048b6306458be9ea964d2330c9dedfe5eb3ee33f9e4e2425",
        role = "negative missing initial-invariant fixture",
      },
      {
        path = "verus/trace-failure-fixtures/replay_with_dispatch_intent.rs",
        blake3 = "9d47173dbeda220b681deed6e096a9da6f9393c1ae0566d86818bd11512546d7",
        role = "negative dispatch-authority fixture",
      },
      {
        path = "verus/trace-failure-fixtures/suspended_trace_drops_fact.rs",
        blake3 = "13291f4afd5ba61eccde31a4e039db5babde6c410c657b5771224766667f20cc",
        role = "negative suspension-fact replay fixture",
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
