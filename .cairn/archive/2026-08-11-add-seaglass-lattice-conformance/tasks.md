## Tasks

- [x] [serial] Record exact execution-graph, Seaglass, and Lattice source revisions, behavior entry points, fixtures, package sources, and rollback states. r[choregraph.conformance.sources] r[choregraph.conformance.inventory]
- [x] [serial] Implement conversion-only Seaglass compatibility mappings over Choregraph production types. r[choregraph.conformance.adapter]
- [x] [depends:compatibility-adapter] Dual-run admission, cycle, order, dependency, group, readiness, lifecycle, resume, critical-path, bottleneck, state, and event behavior. r[choregraph.conformance.parity]
- [x] [depends:nickel-lowering] Add the Lattice diamond Nickel-to-machine end-to-end positive fixture. r[choregraph.conformance.lattice]
- [x] [parallel] Add malformed graph, invalid lifecycle, replay, undeclared-effect, authorization-denial-result, stale-source, receipt-mismatch, and no-fallback negative fixtures. r[choregraph.conformance.negative]
- [x] [depends:lattice-fixture] Add a disposable downstream integration check against one exact reviewed Lattice revision. r[choregraph.conformance.downstream]
- [x] [serial] Record cross-repository cutover, source alignment, duplicate-removal, and VCS rollback evidence requirements. r[choregraph.conformance.cutover] r[choregraph.conformance.single_implementation]
- [x] [serial] Verify Choregraph emits pure intents while Lattice retains every runtime and product authority. r[choregraph.conformance.authority]
- [x] [serial] Run parity, downstream, negative, proof-linkage, Cargo, Clippy, Cairn, Octet, and relevant Nix checks. r[choregraph.conformance.validation]
