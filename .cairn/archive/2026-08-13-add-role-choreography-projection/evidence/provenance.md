# Role choreography provenance

## Trellis semantic dependency

- Published revision: `d65974b14978e1a44b0f5067beb9ef6d83317dc8`.
- Semantic completion commit: `32b5b03115377d232f3c017489c1dd657b446a4f`.
- Package: `verified-logic` version `0.2.0`.
- License: `MPL-2.0` under an additional owner-authorized reuse grant for OnixResearch-controlled source.
- Toolchain: Rust `1.96.0` and `vstd =0.0.0-2026-01-25-0057`.
- Source-set manifest BLAKE3: `28bdef948c196d9ff84268a2028e90d7148e7950bcc4046489f5483b0421c99a`.

Selected source identities:

| Source | BLAKE3 |
| --- | --- |
| `src/choreography_global.rs` | `4f8cdfda9ce711e9f93aebec279c9712aaeb5145ce53e2331ad13d4fd829ad02` |
| `src/choreography_local.rs` | `16486a952c7dbfbfa0cafe54165b9181df0363c6431c5b20957c175a3906db3e` |
| `src/choreography_ownership.rs` | `f5d89e64dd8aeb2f996496ecdababd82a86a0470a01777d709b981894a689afe` |
| `src/choreography_projection.rs` | `e13e55b41ffc7aafe3a29e8d0632c8be602cac5dfbffce227ac17bf175eed764` |
| `src/choreography_step.rs` | `5ee8d321a9202436941f8c73c41f777028c20d0a8a1120421c9969a227b0b818` |
| `src/properties.rs` | `4fbc333bf06622f375c6756eb0312b381d8bd1452d4a4ef5d9da7beec508ad18` |

Selected production APIs are `GlobalChoreo`, `LocalChoreo`, `admit_ownership`, `transfer_owner`, `well_formed_global`, `projectable`, and `project_endpoint`.

Selected theorem references are `successful_transfer_conserves_ownership`, `successful_transfer_rejects_stale_source`, `successful_owned_transfer_step_conserves`, and `choreography_owned_transfer_projection_correspondence`.

Choregraph does not claim that theorem citation transfers proof. Local conformance tests call the pinned production projection and compare every emitted local field. Existing Choregraph Verus rails remain local evidence only.

## Wyzer design reference

- Revision: `7f6acf0f60cf34357e06de50112eb2b1ed833a0c`.
- License disposition: MIT, with direct author confirmation reported by the project operator.
- Selected concepts: explicit role-located values, explicit ownership transfer, native and external role declarations, structured multi-span diagnostics, and negative choreography examples.
- Adopted source fragments: none.
- Rejected components: projection and typechecker code, networking, runtime execution, Perceus, FBIP, compile-time execution, and the C ABI.

The MIT disposition removes the license blocker. It does not promote Wyzer prototype, runtime, safety, or production claims.
