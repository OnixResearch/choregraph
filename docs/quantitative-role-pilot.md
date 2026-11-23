# Quantitative model checking pilot for bounded role protocols

`examples/quantitative-role-pilot/` is a default-off research pilot that exports a supported subset of Trellis-projected Choregraph role protocols to synchronous Markov Decision Processes (MDPs) and checks them with pinned PRISM 4.9.

This pilot is inspired by *Model Checking Probabilistic Multiparty Session Types* (arXiv:2611.14795v1), which explicitly leaves asynchronous sessions as future work. Its outputs describe a **synchronous source-model abstraction only** and do not establish runtime reliability guarantees for asynchronous Lattice or Aspen-Molten deployments.

## Pinning and tool provenance

- **Model checker:** `prism-model-checker` version `4.9`
- **License:** `GPL-2.0-or-later`
- **Pinned Nix source:** `github:NixOS/nixpkgs/20535e48e12c86043b577b8518234ff5dbb26957#prism-model-checker`
- **Choregraph projector:** unchanged `choregraph-choreography` (`admission::admit_source` + `endpoint::project_all` over pinned Trellis). No second accepting endpoint projector exists.

## Opt-in profile and abstraction boundary

- **Schema:** `choregraph.role-choreography.quantitative-pilot.v1`
- **Explicit selector:** `choregraph.role-choreography.profile.quantitative-mdp-pilot.v1`
- Existing unannotated `choregraph.role-choreography.v1` inputs remain untouched and produce byte-identical canonical artifacts and local-set identities.
- **Exact probabilities:** declared as exact nonnegative rationals (`RationalProb { numerator, denominator }`) summing strictly to `1` in a separate `ProbabilitySidecar`. Each choice annotation records its provenance (`hypothesis`, `externally-estimated`, or `validated-observation`). Unannotated choices remain **nondeterministic**; uniform probabilities are never invented.
- **Ownership abstraction:** `SendOwned` / `ReceiveOwned` ownership transfers are explicitly rejected (`OwnershipAbstraction::RejectOwnershipTransfer`) rather than silently dropped.
- **Termination vs. deadlock:** global `End` (`all_ended`) receives an explicit `[terminated]` self-loop so PRISM's built-in `deadlock` state predicate never conflates normal termination with `protocol_deadlock`.

## Conformance evidence and receipts

Because upstream papers do not automatically verify a local adapter, `mdp::verify_prism_export_conformance` independently enumerates the synchronous rendezvous MDP directly from `LocalProtocolSetBody` and compares it against PRISM 4.9's own `-exportstates`, `-exporttrans`, and `-exportlabels` output. Mutation tests verify that swapped participants/labels, omitted branches, altered probabilities, and an exporter that labels `End` as deadlock are all caught.

`AnalysisReceipt` binds domain-separated BLAKE3 digests of the admitted source and local endpoints, probability annotations, abstraction policy, requested properties, scheduler assumption (`unrestricted-adversarial`), pinned tool configuration, and exported PRISM model bytes. Any change invalidates the receipt; `prism::replay_retained_subject` replays retained `.prism` and `.props` subjects directly through PRISM.

## Explicit non-claims

Every `AnalysisReport` carries the closed `REQUIRED_NON_CLAIMS` set and does **not** establish:

- asynchronous delivery or message queuing semantics;
- faults, crashes, or retries;
- dynamic membership;
- real-world probability calibration (preferences and LLM confidence scores are not calibrated branch probabilities);
- a formal correspondence theorem between the synchronous MDP and a host implementation;
- runtime execution or authorization authority.

## Running the real PRISM 4.9 pilot tests

```sh
nix shell 'github:NixOS/nixpkgs/20535e48e12c86043b577b8518234ff5dbb26957#prism-model-checker' \
  -c nix develop --builders '' \
  -c cargo test --manifest-path examples/quantitative-role-pilot/Cargo.toml
```
