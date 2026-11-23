# Preference annotation implementation tasks and evidence gates

- [x] [serial] Define the annotated source DTO, closed annotation non-claims, fixed limits and exact profile selectors, with the cooperative baseline as the default. r[choregraph.role_choreography.preference.profile] r[choregraph.role_choreography.preference.source]
- [x] [serial] Refuse annotated sources in the baseline `compile_evaluated` and in the annotated entry points unless the preference profile is selected; keep v1 parsing, admission and identity bytes unchanged. r[choregraph.role_choreography.preference.profile]
- [x] [serial] Admit the v1 base with the unchanged admission function, then validate factors, worlds, choice paths, deciders, ownership and usage with safe diagnostics. r[choregraph.role_choreography.preference.source]
- [x] [serial] Attach annotations to the unchanged Trellis local set by checked address correspondence; refuse merged foreign choices and every local mismatch. r[choregraph.role_choreography.preference.projection]
- [x] [serial] Add the four Preserves schemas, encoders over the existing Kamacite codec (no Kamacite adapter source change), `preference-*` identity domains, annotated complete local set, receipt, set validator, local-set verifier and receipt verifier. r[choregraph.role_choreography.preference.identity]
- [x] [serial] Add the report with separate projection status, diagnostics and `not-formalized` game mapping, plus its serializable summary. r[choregraph.role_choreography.preference.report]
- [x] [serial] Add the Nickel contract, the positive two-branch fixture with checked-in export, five negative fixtures, and the Nix contract rail comparison. r[choregraph.role_choreography.preference.source]
- [x] [serial] Refresh the `flake.nix` identities in the single-step, trace-replay and WASM evidence manifests and regenerate their exports with the documented procedure; proof sources and claims are unchanged. r[choregraph.role_choreography.preference.profile]
- [x] [serial] Add positive and negative Rust tests for default refusal, unchanged v1 behavior, skeleton identity, nested and offer-mediated addresses, unaddressable merges, every admission refusal, stale local sets and stale receipts. r[choregraph.role_choreography.preference.projection] r[choregraph.role_choreography.preference.identity] r[choregraph.role_choreography.preference.boundary]
- [x] [serial] Document the consumer contract and non-claims in `docs/role-choreography.md`. r[choregraph.role_choreography.preference.boundary]
- [x] [serial] Record local validation observations ([validation](evidence/validation-2026-11-19.md)). r[choregraph.role_choreography.preference.profile]
- [ ] [serial] Observe Lattice adoption of the published cohort (Lattice owner, [Lattice #16](https://github.com/OnixResearch/lattice/issues/16)). r[choregraph.role_choreography.preference.boundary]
- [ ] [serial] Record the published revision and explicit release authority before any release claim. r[choregraph.role_choreography.preference.identity]

The two unchecked gates are not deferred implementation. They record observations that the separate owners must make after this implementation lands. This slice cannot truthfully claim them.
