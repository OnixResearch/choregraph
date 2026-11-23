//! Opt-in bounded quantitative model checking pilot for Choregraph role
//! protocols over pinned PRISM 4.9.
//!
//! Default-off research pilot: does not alter Choregraph's v1 source admission,
//! Trellis endpoint projection, ownership checks, or runtime non-claims.

pub mod mdp;
pub mod model;
pub mod prism;

use choregraph_choreography::endpoint::{LocalProtocolSetBody, project_all};
use choregraph_choreography::model::schema::RoleSource;

use crate::mdp::{
    EnumeratedMdp, enumerate_source_mdp, export_prism_mdp, export_prism_props,
    verify_prism_export_conformance,
};
use crate::model::{
    AnalysisError, AnalysisManifest, AnalysisReport, ProbabilitySidecar, PropertyResult,
    QUANTITATIVE_PILOT_PROFILE, QUANTITATIVE_PILOT_SCHEMA, REQUIRED_NON_CLAIMS, RetainedSubject,
};
use crate::prism::{build_receipt, run_prism};

/// End-to-end opt-in pilot entry point:
/// 1. Requires explicit profile selection (`QUANTITATIVE_PILOT_PROFILE`).
/// 2. Admits and projects the `RoleSource` through Choregraph's unchanged
///    Trellis path (`admit_source` + `project_all`).
/// 3. Independently enumerates the synchronous rendezvous MDP with explicit
///    bounds and probability validation.
/// 4. Exports `.prism` and `.props`, executes pinned PRISM 4.9, and runs the
///    independent PRISM explicit-export conformance check.
/// 5. Emits an `AnalysisReport` + `RetainedSubject` bound by a BLAKE3 receipt.
pub fn analyze_role_source(
    source: RoleSource,
    sidecar: &ProbabilitySidecar,
    manifest: &AnalysisManifest,
) -> Result<(AnalysisReport, RetainedSubject, EnumeratedMdp), AnalysisError> {
    if manifest.profile != QUANTITATIVE_PILOT_PROFILE {
        return Err(AnalysisError::ProfileNotSelected);
    }
    if manifest.schema != QUANTITATIVE_PILOT_SCHEMA {
        return Err(AnalysisError::InvalidSchema(manifest.schema.clone()));
    }
    let source_bytes = serde_json::to_vec(&source)
        .map_err(|err| AnalysisError::MalformedCheckerOutput(format!("serialize source: {err}")))?;
    let admitted = choregraph_choreography::admission::admit_source(source).map_err(|err| {
        AnalysisError::IncompleteEndpointSet(format!("Trellis admission: {err:?}"))
    })?;
    let projected = project_all(&admitted).map_err(|err| {
        AnalysisError::IncompleteEndpointSet(format!("Trellis project_all: {err:?}"))
    })?;

    analyze_projected_local_set(&source_bytes, &projected.locals, sidecar, manifest)
}

/// Runs the quantitative pilot directly on a supplied `LocalProtocolSetBody`
/// (useful for deliberate mismatched-communication / deadlock controls without
/// altering Trellis's accepting projector).
pub fn analyze_projected_local_set(
    source_bytes: &[u8],
    local_set: &LocalProtocolSetBody,
    sidecar: &ProbabilitySidecar,
    manifest: &AnalysisManifest,
) -> Result<(AnalysisReport, RetainedSubject, EnumeratedMdp), AnalysisError> {
    if manifest.profile != QUANTITATIVE_PILOT_PROFILE {
        return Err(AnalysisError::ProfileNotSelected);
    }
    if manifest.schema != QUANTITATIVE_PILOT_SCHEMA {
        return Err(AnalysisError::InvalidSchema(manifest.schema.clone()));
    }

    let enumerated = enumerate_source_mdp(
        &manifest.requested_roles,
        local_set,
        sidecar,
        manifest.ownership_abstraction,
        manifest.bounds,
    )?;
    let prism_model = export_prism_mdp(&enumerated);
    let props_file = export_prism_props(&manifest.properties);

    let run = run_prism(
        &manifest.tool,
        &prism_model,
        &props_file,
        manifest.properties.len(),
        manifest.bounds.timeout_ms,
    )?;

    verify_prism_export_conformance(
        &enumerated,
        &run.states_txt,
        &run.trans_txt,
        &run.labels_txt,
    )?;

    let receipt = build_receipt(source_bytes, local_set, sidecar, manifest, &prism_model)?;
    let mut results = Vec::with_capacity(manifest.properties.len());
    for (prop, prob) in manifest.properties.iter().zip(run.probabilities) {
        results.push(PropertyResult {
            property: prop.clone(),
            formula: prop.pctl_formula(),
            is_bounded: prop.is_bounded(),
            scheduler_assumption: manifest.scheduler_assumption,
            probability: prob,
        });
    }

    let inventory = enumerated.inventory();
    let report = AnalysisReport {
        schema: manifest.schema.clone(),
        profile: manifest.profile.clone(),
        claim_scope: String::from(model::CLAIM_SCOPE),
        semantics: manifest.semantics.clone(),
        scheduler_assumption: manifest.scheduler_assumption,
        provenance_classes: enumerated.provenance_classes.clone(),
        has_nondeterministic_choices: enumerated.nondeterministic_choices > 0,
        has_probabilistic_choices: enumerated.probabilistic_choices > 0,
        inventory,
        conformance_verified: true,
        results,
        receipt: receipt.clone(),
        non_claims: REQUIRED_NON_CLAIMS
            .iter()
            .map(|s| String::from(*s))
            .collect(),
    };
    let retained = RetainedSubject {
        prism_model,
        props_file,
        receipt,
    };
    Ok((report, retained, enumerated))
}
