# r[impl choregraph.wasm.identity]
# r[verify choregraph.wasm.identity]
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3Digest = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let Source = {
  path | NonEmpty,
  context | NonEmpty,
  blake3 | Blake3Digest,
  role | NonEmpty,
} in
let ProofReference = {
  manifest_path | NonEmpty,
  exported_path | NonEmpty,
  manifest_blake3 | Blake3Digest,
  exported_blake3 | Blake3Digest,
  claim_boundary | NonEmpty,
} in
let Toolchain = {
  rustc | NonEmpty,
  target | NonEmpty,
  wit_bindgen | NonEmpty,
  wasm_tools | NonEmpty,
  conformance_engine | NonEmpty,
} in
let Component = {
  export | NonEmpty,
  imports | NonEmpty,
  blake3_context | NonEmpty,
  blake3 | Blake3Digest,
  nix_nar_hash | NonEmpty,
  nix_nar_hash_algorithm | NonEmpty,
} in
let Conformance = {
  fixture_count | Number,
  corpus_blake3_context | NonEmpty,
  corpus_blake3 | Blake3Digest,
  fresh_instance_parity | Bool,
  reused_instance_parity | Bool,
} in
let Limits = {
  request_bytes | Number,
  result_bytes | Number,
  nodes | Number,
  edges | Number,
  effect_declarations | Number,
  state_entries | Number,
  effects | Number,
  trace_events | Number,
} in
let WasmEvidence = {
  schema | NonEmpty,
  status | NonEmpty,
  abi | NonEmpty,
  toolchain | Toolchain,
  component | Component,
  conformance | Conformance,
  limits | Limits,
  sources | Array Source,
  proof_references | Array ProofReference,
  non_claims | Array NonEmpty,
} in
{
  evidence | WasmEvidence = {
    schema = "choregraph.wasm-kernel-adapter-evidence.v1",
    status = "verified",
    abi = "onix:choregraph/kernel@1.0.0 call(list<u8>) -> list<u8>",
    toolchain = {
      rustc = "rustc 1.97.1 (8bab26f4f 2026-07-14)",
      target = "wasm32-unknown-unknown",
      wit_bindgen = "0.57.1",
      wasm_tools = "1.253.0",
      conformance_engine = "wasmtime 45.0.0",
    },
    component = {
      export = "call",
      imports = "none",
      blake3_context = "choregraph wasm component bytes v1",
      blake3 = "0ca76c32cb67ebca3efe5334736184ccb771b12c8f7c3829388bf76c566f0971",
      nix_nar_hash = "sha256-qBBNaavQHMSV5GzOvVANJ656lTaOvp1Z869FNxegaOs=",
      nix_nar_hash_algorithm = "sha256 required by the Nix store protocol",
    },
    conformance = {
      fixture_count = 22,
      corpus_blake3_context = "choregraph wasm conformance corpus v1",
      corpus_blake3 = "860ef15ce5e4ef5a2859476321456ce85a4a16fa1eea7039cdaa9af142936a11",
      fresh_instance_parity = true,
      reused_instance_parity = true,
    },
    limits = {
      request_bytes = 1048576,
      result_bytes = 2097152,
      nodes = 4096,
      edges = 16384,
      effect_declarations = 16384,
      state_entries = 4096,
      effects = 64,
      trace_events = 65536,
    },
    sources = [
      { path = "Cargo.toml", context = "choregraph wasm source file v1:Cargo.toml", blake3 = "2d5b228130ad6deb6a6958df2ce21abf47e65a06b9e5a581d1bfe301c1eb7a37", role = "workspace membership and dependency policy" },
      { path = "Cargo.lock", context = "choregraph wasm source file v1:Cargo.lock", blake3 = "5b259ec8f9a0b566b1f4e7d8d6ba2c859d28eeaaa4b14dadb6215f34dfd38e9f", role = "resolved Rust dependency identity" },
      { path = "rust-toolchain.toml", context = "choregraph wasm source file v1:rust-toolchain.toml", blake3 = "7521ae53aea3a381bc60d7f68287e91ae99e8d5f49bcb5b19550dc181ef12223", role = "Rust channel and WASM target identity" },
      { path = "flake.nix", context = "choregraph wasm source file v1:flake.nix", blake3 = "f4725207ff7efa4c2752230b006579d812acef5045a487a0762d47788ae68ea5", role = "reproducible component, conformance, and evidence build graph" },
      { path = "crates/choregraph-core/src/lib.rs", context = "choregraph wasm source file v1:crates/choregraph-core/src/lib.rs", blake3 = "0da67ba00a0c0c628bad97f3f0eabd4641eb2cacd9332a0d1a93cdced1fda3c3", role = "production kernel public entrypoints" },
      { path = "crates/choregraph-core/src/admission.rs", context = "choregraph wasm source file v1:crates/choregraph-core/src/admission.rs", blake3 = "986dbd0f2e4b8ae550899dc0ce4cd188e6ac44dcf2f2d6588c8abdec7e1e0584", role = "production machine admission" },
      { path = "crates/choregraph-core/src/transition.rs", context = "choregraph wasm source file v1:crates/choregraph-core/src/transition.rs", blake3 = "9297582305e61faff177967a1ce87ecc51b76499a78ddea488b6838ee575b310", role = "production abstract transition and replay entrypoints" },
      { path = "crates/choregraph-core/src/concrete.rs", context = "choregraph wasm source file v1:crates/choregraph-core/src/concrete.rs", blake3 = "9100c3c745921a3288e51b98803c5709f63a27d141bd8687a94051357c46b63c", role = "production concrete transition and replay entrypoints" },
      { path = "crates/choregraph-kamacite/src/lib.rs", context = "choregraph wasm source file v1:crates/choregraph-kamacite/src/lib.rs", blake3 = "65b1ef17f155fc6dceb70e661d76ea9a82b1600457ed08cdd0f40381f8130edf", role = "canonical artifact API boundary" },
      { path = "crates/choregraph-kamacite/src/aggregate.rs", context = "choregraph wasm source file v1:crates/choregraph-kamacite/src/aggregate.rs", blake3 = "3451a5b9e557b778a6ee5565e4052250a739eaafc71d6d7ef49a7cb661c42569", role = "canonical machine artifact validation" },
      { path = "crates/choregraph-kamacite/src/codec.rs", context = "choregraph wasm source file v1:crates/choregraph-kamacite/src/codec.rs", blake3 = "dc0436e2f5fa48fd9c459ca20b9485786b6a397235b7cb4ff878a2a223d3d659", role = "canonical packed Preserves implementation" },
      { path = "crates/choregraph-kamacite/src/identity.rs", context = "choregraph wasm source file v1:crates/choregraph-kamacite/src/identity.rs", blake3 = "6ba121aeeed33ff658a5e9bcb682c9545815f38d860b8ca318cbac4018d7a16a", role = "domain-separated artifact identities" },
      { path = "crates/choregraph-wasm/Cargo.toml", context = "choregraph wasm source file v1:crates/choregraph-wasm/Cargo.toml", blake3 = "fecfb93f27c5d537ca2f1dbcd41e6b7d2fbfaf1bd268dc60c57477ff490d1bc0", role = "adapter dependency and component build boundary" },
      { path = "crates/choregraph-wasm/wit/kernel.wit", context = "choregraph wasm source file v1:crates/choregraph-wasm/wit/kernel.wit", blake3 = "4a3164631c392002c91bc560cf61cbcd9bad9fa19db25bc5157c67d45cc41be5", role = "versioned export-only WIT contract" },
      { path = "crates/choregraph-wasm/src/lib.rs", context = "choregraph wasm source file v1:crates/choregraph-wasm/src/lib.rs", blake3 = "83d28a06e6126cb150d73ae00923b5e118005ce7dd1a0cef85f71e66b3955bba", role = "component shell and generated ABI export" },
      { path = "crates/choregraph-wasm/src/schema.rs", context = "choregraph wasm source file v1:crates/choregraph-wasm/src/schema.rs", blake3 = "d661820bafada04552ed453c9f5c182c3fc6a246b20ad4d9857bdded4e434f51", role = "bounded versioned request and response schema" },
      { path = "crates/choregraph-wasm/src/codec.rs", context = "choregraph wasm source file v1:crates/choregraph-wasm/src/codec.rs", blake3 = "9ebd2a8e969c918179245b6d98ae3f20aadae2925868cd684555c412b7b9ef31", role = "canonical packed Preserves codec wrapper" },
      { path = "crates/choregraph-wasm/src/dispatch.rs", context = "choregraph wasm source file v1:crates/choregraph-wasm/src/dispatch.rs", blake3 = "4498fd0d20da9be17ef1f3552f2dc4e88036831879ef4b0c578094321529244f", role = "pure dispatcher and production-kernel delegation" },
      { path = "crates/choregraph-wasm/tests/dispatcher.rs", context = "choregraph wasm source file v1:crates/choregraph-wasm/tests/dispatcher.rs", blake3 = "0f0203a5313b6c2e834677e91ed3621bcdee35bbc669ed101478d32f19145584", role = "positive and negative native adapter tests" },
      { path = "vendor/kamacite-core-selected/preserves/codec.rs", context = "choregraph wasm source file v1:vendor/kamacite-core-selected/preserves/codec.rs", blake3 = "4134ae42e399361447b293c033ce7a785ef21995e2026fddd8e7a398d2c04391", role = "selected canonical Kamacite codec reference" },
      { path = "tools/choregraph-wasm-conformance/Cargo.toml", context = "choregraph wasm source file v1:tools/choregraph-wasm-conformance/Cargo.toml", blake3 = "138417eff7a94ae57a598ec731f53934e21727ae27a02f96d76a6d1f100b8df1", role = "pinned conformance engine dependency" },
      { path = "tools/choregraph-wasm-conformance/src/main.rs", context = "choregraph wasm source file v1:tools/choregraph-wasm-conformance/src/main.rs", blake3 = "c0c9b2b73903677c123753c6b2f8a0f24ed5c53250ccafb012c6f8db0d49ac4b", role = "native and component byte-parity corpus" },
      { path = "tools/choregraph-foundation-check/src/main.rs", context = "choregraph wasm source file v1:tools/choregraph-foundation-check/src/main.rs", blake3 = "480c9568f578a5f07a5d25fb37689beff80142b7b04c349073f6e279ed4156de", role = "foundation-check command shell" },
      { path = "tools/choregraph-foundation-check/src/source.rs", context = "choregraph wasm source file v1:tools/choregraph-foundation-check/src/source.rs", blake3 = "71d33b35f78b566b31aa6bb5194eddcd651990cdc042d033332a12f8fdc1df25", role = "repository source loader" },
      { path = "tools/choregraph-foundation-check/src/tests.rs", context = "choregraph wasm source file v1:tools/choregraph-foundation-check/src/tests.rs", blake3 = "e48d7e252d4a9363db8c256569157083bcb7daa29c06b25adeb83830062bb562", role = "positive and negative boundary check fixtures" },
      { path = "tools/choregraph-foundation-check/src/types.rs", context = "choregraph wasm source file v1:tools/choregraph-foundation-check/src/types.rs", blake3 = "e3702a544c5b151911bfb9333fc765536a3705ea13421106eff3287206d9a179", role = "bounded finding model" },
      { path = "tools/choregraph-foundation-check/src/validator.rs", context = "choregraph wasm source file v1:tools/choregraph-foundation-check/src/validator.rs", blake3 = "7066b358936e5cf44d04746b1b1426b3e9ef06e41fcc24985da6ea24e8ffefa2", role = "static boundary and delegation checks" },
    ],
    proof_references = [
      {
        manifest_path = "evidence/single-step-proof.ncl",
        exported_path = "evidence/generated/single-step-proof.json",
        manifest_blake3 = "5846191fdf6b1681a377a430f3c311de3a3010abc2fc494a3c7a0484098f8a94",
        exported_blake3 = "f389ab80838388c01ce14784585f9cc267b178fd15e68992bad1e5cdfbaae0a4",
        claim_boundary = "production-called single-step transition obligations only",
      },
      {
        manifest_path = "evidence/trace-replay-proof.ncl",
        exported_path = "evidence/generated/trace-replay-proof.json",
        manifest_blake3 = "6de3fec44230b070a70851807b330b5888ebf816ecd57cb0e2b56f1d3a057c23",
        exported_blake3 = "b5799125e3712f0c451cbf791b06b21f2369c1553563bcbec4e6100e11f52ac9",
        claim_boundary = "bounded replay refinement and first-denial obligations only",
      },
    ],
    non_claims = [
      "Component parity does not prove compiler or engine correctness.",
      "The adapter does not execute, authorize, schedule, persist, or attest effects.",
      "Import denial does not prove host sandbox correctness.",
      "Proof references retain their original bounded theorem and trust boundaries.",
      "This evidence does not establish release eligibility by itself.",
    ],
  },
}
