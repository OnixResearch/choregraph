# r[impl choregraph.proof.single_step.evidence]
# r[impl choregraph.proof.single_step.trust]
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3Digest = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
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
  theorems | Array NonEmpty,
  sources | Array Source,
  negative_fixtures | Array NegativeFixture,
  trusted_surface | TrustedSurface,
  non_claims | Array NonEmpty,
} in
{
  proof | ProofManifest = {
    schema = "choregraph.single-step-proof.v1",
    status = "verified",
    claim = "The production-called indexed kernel preserves bounded state, declared intent membership, concrete binding decisions, reference witnesses, durable suspension facts, exact resume binding, and effect-free suspension transitions.",
    positive_command_argv = [
      "octet-production-verus",
      "--crate-type=lib",
      "verus/proofs/single_step.rs",
      "-o",
      "$TMPDIR/single-step.rlib",
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
        path = "src/job_dag.rs",
        blake3 = "33d61c877ea238302c843a16647667d8703ede28c1f2888a4d5b8fc4164fcc70",
        adapted_claim = "all declared dependency indexes must identify completed states",
      },
      {
        path = "src/job_state.rs",
        blake3 = "3236af6e78b333e8c9dbe61f3e898647d24c747c9186a79a0fbc5506f71495a1",
        adapted_claim = "the closed six-state lifecycle has no terminal outgoing transition",
      },
      {
        path = "src/topo_sort.rs",
        blake3 = "1e4665e37556c5549dccc23cea8fe62ea8619573e8827ea5a66e7c6e2362e8a8",
        adapted_claim = "an admitted finite order contains each node once and places each edge forward",
      },
    ],
    theorems = [
      "accepted_resume_requires_exact_fact_and_clears",
      "accepted_step_preserves_invariant_and_effects",
      "accepted_suspend_records_exact_fact",
      "admitted_topology_has_each_node_once_and_edges_forward",
      "concrete binding and reference preservation witnesses",
      "denial_has_no_replacement_or_intents",
      "equal_explicit_inputs_have_equal_decisions",
      "indexed_step executable refinement postconditions",
      "mismatched_resume_is_denied",
      "readiness_is_sound",
      "suspended_state_blocks_execution_actions",
      "suspension_transitions_emit_no_effects",
      "suspension_update_preserves_consistency",
    ],
    sources = [
      {
        path = "Cargo.lock",
        blake3 = "cfcb76953e7991d53b201b1fd660fc3f94ad792e8b7065a1f9d2a2a32783f114",
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
        path = "crates/choregraph-core/src/admission.rs",
        blake3 = "347384983b7d59136791a5f3006974683b8bd7c51d838b585fc06fc8b332a43e",
        role = "production indexed graph construction",
      },
      {
        path = "crates/choregraph-core/src/concrete.rs",
        blake3 = "dea59aed4710cdee14460206942c907f575e172685fa4e167cc45852e8ba8801",
        role = "production concrete binding resolver and exact intent construction",
      },
      {
        path = "crates/choregraph-core/src/invariant.rs",
        blake3 = "13078fde3e2b5c6c3227d211c36b52e60b29cc223e00c47da5b2f51867ee41c7",
        role = "production graph and post-step gates",
      },
      {
        path = "crates/choregraph-core/src/lib.rs",
        blake3 = "1b009daaa9a4604be776c522cb5d626bdf7de6061deaeaed9d1bffd4b54ef021",
        role = "production module linkage",
      },
      {
        path = "crates/choregraph-core/src/model.rs",
        blake3 = "873b159c57c4a956730f52179948b1dce27bb12c8e19c72d714112d12a9ba71b",
        role = "production indexed machine fields",
      },
      {
        path = "crates/choregraph-core/src/transition.rs",
        blake3 = "d0e18f721280b48dfd88130ca86a3134588aa9cae52678962ce75165aab9cbd5",
        role = "production transition shell and kernel delegation",
      },
      {
        path = "crates/choregraph-core/src/verified.rs",
        blake3 = "49d30d9bb40c0b3554a0174bd6afffe819639d67b477845114de154982ef1894",
        role = "production-called verified transition kernel",
      },
      {
        path = "crates/choregraph-core/src/verified/topology.rs",
        blake3 = "4d28285eab8a9e1374a73b78d165340b285f79002bf9223aed7ec41655bbbb2d",
        role = "production-called verified graph admission gate",
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
        blake3 = "f913942d1e4533d2b0fb98e0919a13370816cfd0e3ad64a0be49494462a58e80",
        role = "maintained proof and failure-fixture gate",
      },
      {
        path = "verus/failure-fixtures/concrete_reference_mutation.rs",
        blake3 = "5a9a398c120f48e41c5e7243d63c7c28515b5f357f8e35f0fcf2dda50e22f00e",
        role = "negative concrete reference preservation fixture",
      },
      {
        path = "verus/failure-fixtures/denial_with_intent.rs",
        blake3 = "4de64276c0f609796ffe2160dfa4222b800fa75321545f4ae8eb43378025f5bb",
        role = "negative denial fixture",
      },
      {
        path = "verus/failure-fixtures/premature_readiness.rs",
        blake3 = "d866a8659ca27339098f51b4916d03570ea2c85b30ad5542431e60fb77e1240c",
        role = "negative readiness fixture",
      },
      {
        path = "verus/failure-fixtures/state_map_mismatch.rs",
        blake3 = "97b7d87f487b7a1463d50acefefb51355929b3a4f7f10e6394810718d6dd3cf3",
        role = "negative state-shape fixture",
      },
      {
        path = "verus/failure-fixtures/suspended_mark_ready.rs",
        blake3 = "a7c93e4f77686e4815b02eac13c8997eddc3a5de535889da8a4fb7933016df9b",
        role = "negative suspended readiness fixture",
      },
      {
        path = "verus/failure-fixtures/suspension_effect_intent.rs",
        blake3 = "e8362e129f79dc3bda08d6bef9c1acc5bbea879ede45a0fccd7563c6e93300bc",
        role = "negative suspension effect fixture",
      },
      {
        path = "verus/failure-fixtures/suspension_resume_mismatch.rs",
        blake3 = "23b4763f2c1955f9de0b4f885ed75afc889cbb45f8525b740c20408e123addca",
        role = "negative resume-binding fixture",
      },
      {
        path = "verus/failure-fixtures/terminal_mutation.rs",
        blake3 = "9b4db193e53e8edadfb4fcdc64c49fad043bf08597d9f8ecf0e5442a84a6d12b",
        role = "negative terminal-state fixture",
      },
      {
        path = "verus/failure-fixtures/topological_edge_reversal.rs",
        blake3 = "86b8cc37f5aca10041ed0aa13d6092289740309e397a701e4592161241488589",
        role = "negative graph-order fixture",
      },
      {
        path = "verus/failure-fixtures/undeclared_intent.rs",
        blake3 = "c346a7e621ea6a938975188950831d26fbf40b654a9d42673b2ff39991bf4bd2",
        role = "negative effect-intent fixture",
      },
      {
        path = "verus/proofs/single_step.rs",
        blake3 = "6b255a130a0a20d2e7369e4f1e0764129efadb6cf6c1c97ce322c7e4b65fd24e",
        role = "positive proof root that includes production sources",
      },
    ],
    negative_fixtures = [
      {
        path = "verus/failure-fixtures/concrete_reference_mutation.rs",
        expected_diagnostic = "postcondition not satisfied: result.payload_reference",
        violated_obligation = "a concrete payload reference changes after acceptance",
      },
      {
        path = "verus/failure-fixtures/denial_with_intent.rs",
        expected_diagnostic = "postcondition not satisfied: result.intent_indices@.len()",
        violated_obligation = "denial emits an intent",
      },
      {
        path = "verus/failure-fixtures/premature_readiness.rs",
        expected_diagnostic = "postcondition not satisfied: dependencies_complete_spec",
        violated_obligation = "readiness accepts a running dependency",
      },
      {
        path = "verus/failure-fixtures/state_map_mismatch.rs",
        expected_diagnostic = "postcondition not satisfied: indexed_invariant_spec",
        violated_obligation = "dependency and state vectors have different lengths",
      },
      {
        path = "verus/failure-fixtures/suspended_mark_ready.rs",
        expected_diagnostic = "postcondition not satisfied: lifecycle_target_spec",
        violated_obligation = "a suspended node starts without a resume signal",
      },
      {
        path = "verus/failure-fixtures/suspension_effect_intent.rs",
        expected_diagnostic = "postcondition not satisfied: indexed_step_accepts_spec",
        violated_obligation = "a suspend transition emits a product effect intent",
      },
      {
        path = "verus/failure-fixtures/suspension_resume_mismatch.rs",
        expected_diagnostic = "postcondition not satisfied: indexed_step_accepts_spec",
        violated_obligation = "a resume signal with a mismatched fact moves state",
      },
      {
        path = "verus/failure-fixtures/terminal_mutation.rs",
        expected_diagnostic = "postcondition not satisfied: terminal_states_preserved_spec",
        violated_obligation = "a completed state changes to running",
      },
      {
        path = "verus/failure-fixtures/topological_edge_reversal.rs",
        expected_diagnostic = "postcondition not satisfied: topological_order_spec",
        violated_obligation = "an edge target appears before its source",
      },
      {
        path = "verus/failure-fixtures/undeclared_intent.rs",
        expected_diagnostic = "postcondition not satisfied: effects_allowed_spec",
        violated_obligation = "an undeclared intent is emitted",
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
        "Canonical references are abstract proof tokens; Kamacite checks bytes and BLAKE3 identities outside Verus.",
        "The public shell accepts only an accepted indexed decision and reruns invariant and effect checks after conversion.",
        "The proof starts at admitted indexed graph and state views, not at arbitrary source text or Nickel evaluation.",
      ],
    },
    non_claims = [
      "This evidence does not prove Verus, Z3, rustc, vstd, the allocator, or the operating system.",
      "This evidence does not prove arbitrary Nickel evaluation or source-to-index semantic equivalence.",
      "This evidence does not prove runtime authorization, effect execution, persistence, scheduling, or liveness. Finite-trace claims require the separate trace rail.",
      "Trellis source identities record adapted proof inputs and do not transfer Trellis certification.",
    ],
  },
}
