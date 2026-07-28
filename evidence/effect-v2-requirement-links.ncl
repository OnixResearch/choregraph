# r[impl choregraph.effect_v2.stable_machine]
# r[verify choregraph.effect_v2.stable_machine]
# r[impl choregraph.effect_v2.step]
# r[verify choregraph.effect_v2.step]
# r[impl choregraph.effect_v2.kernel]
# r[verify choregraph.effect_v2.kernel]
# r[impl choregraph.effect_v2.nickel]
# r[verify choregraph.effect_v2.nickel]
# r[impl choregraph.effect_v2.release]
# r[verify choregraph.effect_v2.release]
# r[impl choregraph.effect_v2.non_claims]
# r[verify choregraph.effect_v2.non_claims]
# r[verify choregraph.effect_v2.schema.scenario.v1]
# r[verify choregraph.effect_v2.stable_machine.scenario.runs]
# r[verify choregraph.effect_v2.occurrence_identity.scenario.equal-content]
# r[verify choregraph.effect_v2.declarations.scenario.duplicate]
# r[verify choregraph.effect_v2.bindings.scenario.wrong-slot]
# r[verify choregraph.effect_v2.references.scenario.non-canonical]
# r[verify choregraph.effect_v2.step.scenario.accepted]
# r[verify choregraph.effect_v2.determinism.scenario.order]
# r[verify choregraph.effect_v2.denial.scenario.unknown]
# r[verify choregraph.effect_v2.diagnostics.scenario.redaction]
# r[verify choregraph.effect_v2.replay.scenario.history]
# r[verify choregraph.effect_v2.kernel.scenario.wrapper]
# r[verify choregraph.effect_v2.nickel.scenario.missing]
# r[verify choregraph.effect_v2.kamacite.scenario.stale]
# r[verify choregraph.effect_v2.proof.scenario.mutation]
# r[verify choregraph.effect_v2.seaglass.scenario.authority]
# r[verify choregraph.effect_v2.lattice_handoff.scenario.synthetic]
# r[verify choregraph.effect_v2.fixtures.scenario.negative]
# r[verify choregraph.effect_v2.release.scenario.stale]
# r[verify choregraph.effect_v2.non_claims.scenario.scope]
let Link = {
  sources | Array String,
  checks | Array String,
  non_claim | String,
} in
{
  schema = "choregraph.effect-v2-requirement-links.v1",
  stable_machine | Link = {
    sources = ["crates/choregraph-core/src/concrete.rs", "crates/choregraph-core/tests/concrete-effect-intents.rs"],
    checks = ["workspace-tests", "single-step-verus"],
    non_claim = "Stable immutable machine identity does not identify a run occurrence.",
  },
  step | Link = {
    sources = ["crates/choregraph-core/src/transition.rs", "crates/choregraph-core/src/concrete.rs"],
    checks = ["workspace-tests", "single-step-verus"],
    non_claim = "An emitted intent does not authorize dispatch.",
  },
  kernel | Link = {
    sources = ["crates/choregraph-core/src/transition.rs", "crates/choregraph-core/src/verified.rs"],
    checks = ["single-step-verus", "trace-replay-verus"],
    non_claim = "One production kernel does not prove external effect success.",
  },
  nickel | Link = {
    sources = ["contracts/nickel/restricted-graph-contracts.ncl", "crates/choregraph-nickel/src/lower.rs"],
    checks = ["restricted-nickel-contracts", "workspace-tests"],
    non_claim = "Restricted lowering does not prove arbitrary Nickel equivalence.",
  },
  release | Link = {
    sources = ["evidence/release/generated/choregraph.release.bundle.preserves", "evidence/release/generated/report.txt"],
    checks = ["release-bundle", "foundation-evidence"],
    non_claim = "A verified local bundle does not grant publication authority.",
  },
  non_claims | Link = {
    sources = ["docs/lattice-authority-boundary.md", "fixtures/lattice-handoff/consumer.rs"],
    checks = ["foundation-boundaries", "lattice-downstream"],
    non_claim = "Passing local checks does not prove downstream authority or correctness.",
  },
}
