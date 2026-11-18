#![allow(
    unknown_lints,
    borrowed_argument_types,
    non_trait_imports,
    reason = "the pure bundle builder grows one owned framing buffer and shows exact artifact owner types at one linkage boundary"
)]

pub mod digest;
pub mod evidence;
pub mod output;
pub mod provenance;

use self::digest::{Identity, IdentityDomain, compute_identity, identity_is_valid};
use self::evidence::{ProjectionBinding, ProjectionReceiptBody, ProjectionRole};
use self::output::{CanonicalArtifact, CanonicalLocalArtifact};
use self::provenance::{
    CHOREGRAPH_TOOLCHAIN, LOWERER_IDENTITY_PREIMAGE, TRELLIS_PROOF_SET_PREIMAGE, TRELLIS_REVISION,
    TRELLIS_SEMANTIC_COMMIT, TRELLIS_SOURCE_MANIFEST_BLAKE3, TRELLIS_TOOLCHAIN,
};
use crate::admission::Admitted;
use crate::endpoint::{LocalProtocolArtifactBody, Projected, project_all};
use crate::issue::{Diagnostic, DiagnosticCode};
use crate::model::bounds::MAX_CANONICAL_OUTPUT_BYTES;
use crate::model::participants::SourceSpan;
use choregraph_kamacite::profile::{observation, protocol, source};
use serde::{Deserialize, Serialize};

const FIXED_EVIDENCE_BINDING_COUNT: usize = 4;
const ROLE_CHOREOGRAPHY_SCHEMA_COUNT: usize = 6;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DiagnosticArtifactBody {
    diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionBundle {
    pub(crate) source: CanonicalArtifact,
    pub(crate) global: CanonicalArtifact,
    pub(crate) locals: Vec<CanonicalLocalArtifact>,
    pub(crate) local_set: CanonicalArtifact,
    pub(crate) receipt_body: ProjectionReceiptBody,
    pub(crate) receipt: CanonicalArtifact,
}

impl ProjectionBundle {
    #[must_use]
    pub const fn source(&self) -> &CanonicalArtifact {
        &self.source
    }

    #[must_use]
    pub const fn global(&self) -> &CanonicalArtifact {
        &self.global
    }

    #[must_use]
    pub fn locals(&self) -> &[CanonicalLocalArtifact] {
        &self.locals
    }

    #[must_use]
    pub const fn local_set(&self) -> &CanonicalArtifact {
        &self.local_set
    }

    #[must_use]
    pub const fn receipt_body(&self) -> &ProjectionReceiptBody {
        &self.receipt_body
    }

    #[must_use]
    pub const fn receipt(&self) -> &CanonicalArtifact {
        &self.receipt
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionMaterials {
    source_envelope: Identity,
    evaluator_output: Identity,
}

fn issue(code: DiagnosticCode, note: &'static str) -> Vec<Diagnostic> {
    vec![Diagnostic::error(
        code,
        SourceSpan { start: 0, end: 0 },
        note,
    )]
}

impl ProjectionMaterials {
    #[must_use]
    pub fn reviewed(source_envelope_bytes: &[u8], evaluator_output_bytes: &[u8]) -> Self {
        Self {
            source_envelope: compute_identity(
                IdentityDomain::SourceEnvelope,
                source_envelope_bytes,
            ),
            evaluator_output: compute_identity(
                IdentityDomain::EvaluatorOutput,
                evaluator_output_bytes,
            ),
        }
    }
}

fn public_artifact(
    artifact: choregraph_kamacite::codec::CanonicalArtifact,
    domain: IdentityDomain,
) -> CanonicalArtifact {
    let identity = compute_identity(domain, &artifact.packed_bytes);
    CanonicalArtifact {
        schema_version: artifact.schema_version,
        packed_bytes: artifact.packed_bytes,
        text_projection: artifact.text_projection,
        identity,
    }
}

fn encode_error(note: &'static str) -> Vec<Diagnostic> {
    issue(DiagnosticCode::ArtifactEncodingFailed, note)
}

fn append_digest(material: &mut Vec<u8>, value: &[u8]) {
    material.extend_from_slice(blake3::hash(value).as_bytes());
}

fn schema_identity() -> Identity {
    let schemas: [(&str, &[u8]); ROLE_CHOREOGRAPHY_SCHEMA_COUNT] = [
        (
            source::SCHEMA,
            include_bytes!(
                "../../../schemas/preserves/choregraph-role-choreography-source-v1.preserves"
            ),
        ),
        (
            protocol::GLOBAL_SCHEMA,
            include_bytes!(
                "../../../schemas/preserves/choregraph-role-choreography-global-v1.preserves"
            ),
        ),
        (
            protocol::LOCAL_SCHEMA,
            include_bytes!(
                "../../../schemas/preserves/choregraph-role-choreography-local-v1.preserves"
            ),
        ),
        (
            protocol::LOCAL_SET_SCHEMA,
            include_bytes!(
                "../../../schemas/preserves/choregraph-role-choreography-local-set-v1.preserves"
            ),
        ),
        (
            observation::DIAGNOSTICS_SCHEMA,
            include_bytes!(
                "../../../schemas/preserves/choregraph-role-choreography-diagnostics-v1.preserves"
            ),
        ),
        (
            observation::RECEIPT_SCHEMA,
            include_bytes!(
                "../../../schemas/preserves/choregraph-role-choreography-receipt-v1.preserves"
            ),
        ),
    ];
    debug_assert_eq!(schemas.len(), ROLE_CHOREOGRAPHY_SCHEMA_COUNT);
    debug_assert!(
        schemas
            .iter()
            .all(|(version, source)| !version.is_empty() && !source.is_empty())
    );
    let mut material = Vec::new();
    for (version, source) in schemas {
        append_digest(&mut material, version.as_bytes());
        append_digest(&mut material, source);
    }
    compute_identity(IdentityDomain::SchemaSet, &material)
}

fn lowerer_identity() -> Identity {
    let mut material = Vec::new();
    append_digest(&mut material, LOWERER_IDENTITY_PREIMAGE.as_bytes());
    append_digest(&mut material, include_bytes!("admission.rs"));
    append_digest(&mut material, include_bytes!("endpoint.rs"));
    compute_identity(IdentityDomain::Lowerer, &material)
}

fn toolchain_identity() -> Identity {
    let material = format!("{CHOREGRAPH_TOOLCHAIN}\n{TRELLIS_TOOLCHAIN}");
    compute_identity(IdentityDomain::Toolchain, material.as_bytes())
}

fn trellis_source_identity() -> Identity {
    compute_identity(
        IdentityDomain::TrellisSourceSet,
        format!("{TRELLIS_REVISION}\n{TRELLIS_SOURCE_MANIFEST_BLAKE3}").as_bytes(),
    )
}

fn evidence_roles(
    source: &Identity,
    global: &Identity,
    locals: &[Identity],
    local_set: &Identity,
    proof_set: &Identity,
) -> Vec<ProjectionBinding> {
    let linkage_claim = String::from("identity-linkage-only-no-claim-promotion");
    let mut bindings =
        Vec::with_capacity(locals.len().saturating_add(FIXED_EVIDENCE_BINDING_COUNT));
    bindings.push(ProjectionBinding {
        role: ProjectionRole::Source,
        identity: source.clone(),
        claim: linkage_claim.clone(),
    });
    bindings.push(ProjectionBinding {
        role: ProjectionRole::GlobalProtocol,
        identity: global.clone(),
        claim: linkage_claim.clone(),
    });
    for local in locals {
        bindings.push(ProjectionBinding {
            role: ProjectionRole::LocalProtocol,
            identity: local.clone(),
            claim: linkage_claim.clone(),
        });
    }
    bindings.push(ProjectionBinding {
        role: ProjectionRole::CompleteLocalSet,
        identity: local_set.clone(),
        claim: linkage_claim.clone(),
    });
    bindings.push(ProjectionBinding {
        role: ProjectionRole::ProofReference,
        identity: proof_set.clone(),
        claim: linkage_claim,
    });
    debug_assert_eq!(
        bindings.len(),
        locals.len().saturating_add(FIXED_EVIDENCE_BINDING_COUNT)
    );
    debug_assert!(
        bindings
            .iter()
            .all(|binding| binding.claim == "identity-linkage-only-no-claim-promotion")
    );
    bindings
}

fn encode_local(
    local: &LocalProtocolArtifactBody,
) -> Result<CanonicalLocalArtifact, Vec<Diagnostic>> {
    let artifact = protocol::encode_local(local)
        .map_err(|_| encode_error("local protocol Preserves encoding failed"))?;
    Ok(CanonicalLocalArtifact {
        role: local.role,
        artifact: public_artifact(artifact, IdentityDomain::LocalArtifact),
    })
}

fn total_artifact_bytes(bundle: &ProjectionBundle) -> usize {
    bundle
        .source
        .packed_bytes
        .len()
        .saturating_add(bundle.global.packed_bytes.len())
        .saturating_add(bundle.local_set.packed_bytes.len())
        .saturating_add(bundle.receipt.packed_bytes.len())
        .saturating_add(
            bundle
                .locals
                .iter()
                .map(|local| local.artifact.packed_bytes.len())
                .sum::<usize>(),
        )
}

struct EncodedProjection {
    source: CanonicalArtifact,
    global: CanonicalArtifact,
    locals: Vec<CanonicalLocalArtifact>,
    local_identities: Vec<Identity>,
    local_set: CanonicalArtifact,
    diagnostics: choregraph_kamacite::codec::CanonicalArtifact,
}

fn encode_projection(admitted: &Admitted) -> Result<EncodedProjection, Vec<Diagnostic>> {
    let Projected { global, locals } = project_all(admitted)?;
    let source = public_artifact(
        source::encode(&admitted.source)
            .map_err(|_| encode_error("strict DTO Preserves encoding failed"))?,
        IdentityDomain::StrictDto,
    );
    let global = public_artifact(
        protocol::encode_global(&global)
            .map_err(|_| encode_error("global protocol Preserves encoding failed"))?,
        IdentityDomain::GlobalArtifact,
    );
    let encoded_locals: Vec<CanonicalLocalArtifact> = locals
        .locals
        .iter()
        .map(encode_local)
        .collect::<Result<_, _>>()?;
    let local_identities: Vec<Identity> = encoded_locals
        .iter()
        .map(|local| local.artifact.identity.clone())
        .collect();
    let local_set = public_artifact(
        protocol::encode_local_set(&locals)
            .map_err(|_| encode_error("complete local set Preserves encoding failed"))?,
        IdentityDomain::CompleteLocalSet,
    );
    let diagnostics = observation::encode_diagnostics(&DiagnosticArtifactBody {
        diagnostics: Vec::new(),
    })
    .map_err(|_| encode_error("diagnostic Preserves encoding failed"))?;
    debug_assert_eq!(encoded_locals.len(), admitted.roles.len());
    debug_assert_eq!(local_identities.len(), admitted.roles.len());
    Ok(EncodedProjection {
        source,
        global,
        locals: encoded_locals,
        local_identities,
        local_set,
        diagnostics,
    })
}

fn projection_receipt(
    materials: &ProjectionMaterials,
    encoded: &EncodedProjection,
) -> ProjectionReceiptBody {
    let proof_set = compute_identity(
        IdentityDomain::TrellisProofSet,
        TRELLIS_PROOF_SET_PREIMAGE.as_bytes(),
    );
    let receipt = ProjectionReceiptBody {
        source_envelope: materials.source_envelope.clone(),
        import_closure: compute_identity(
            IdentityDomain::ImportClosure,
            &encoded.source.packed_bytes,
        ),
        evaluator_output: materials.evaluator_output.clone(),
        strict_dto: encoded.source.identity.clone(),
        global_artifact: encoded.global.identity.clone(),
        local_artifacts: encoded.local_identities.clone(),
        complete_local_set: encoded.local_set.identity.clone(),
        trellis_revision: String::from(TRELLIS_REVISION),
        trellis_semantic_commit: String::from(TRELLIS_SEMANTIC_COMMIT),
        trellis_source_set: trellis_source_identity(),
        trellis_proof_set: proof_set.clone(),
        lowerer: lowerer_identity(),
        schema_set: schema_identity(),
        bounds: compute_identity(IdentityDomain::Bounds, &encoded.source.packed_bytes),
        diagnostics: compute_identity(
            IdentityDomain::Diagnostics,
            &encoded.diagnostics.packed_bytes,
        ),
        toolchain: toolchain_identity(),
        non_claims: compute_identity(IdentityDomain::NonClaims, &encoded.source.packed_bytes),
        evidence_roles: evidence_roles(
            &encoded.source.identity,
            &encoded.global.identity,
            &encoded.local_identities,
            &encoded.local_set.identity,
            &proof_set,
        ),
    };
    debug_assert_eq!(receipt.local_artifacts.len(), encoded.locals.len());
    debug_assert_eq!(receipt.trellis_revision, TRELLIS_REVISION);
    receipt
}

pub fn build_projection_bundle(
    admitted: &Admitted,
    materials: &ProjectionMaterials,
) -> Result<ProjectionBundle, Vec<Diagnostic>> {
    if !identity_is_valid(&materials.source_envelope, IdentityDomain::SourceEnvelope)
        || !identity_is_valid(&materials.evaluator_output, IdentityDomain::EvaluatorOutput)
    {
        return Err(issue(
            DiagnosticCode::IdentityMismatch,
            "source or evaluator identity is invalid",
        ));
    }
    let encoded = encode_projection(admitted)?;
    let receipt_body = projection_receipt(materials, &encoded);
    let receipt = public_artifact(
        observation::encode_receipt(&receipt_body)
            .map_err(|_| encode_error("projection receipt Preserves encoding failed"))?,
        IdentityDomain::ProjectionReceipt,
    );
    let bundle = ProjectionBundle {
        source: encoded.source,
        global: encoded.global,
        locals: encoded.locals,
        local_set: encoded.local_set,
        receipt_body,
        receipt,
    };
    if total_artifact_bytes(&bundle) > MAX_CANONICAL_OUTPUT_BYTES {
        return Err(issue(
            DiagnosticCode::CanonicalOutputLimitExceeded,
            "canonical choreography output exceeds the reviewed limit",
        ));
    }
    debug_assert!(total_artifact_bytes(&bundle) <= MAX_CANONICAL_OUTPUT_BYTES);
    debug_assert_eq!(bundle.locals.len(), admitted.roles.len());
    Ok(bundle)
}

pub fn verify_projection_receipt(
    expected: &ProjectionBundle,
    candidate_body: &ProjectionReceiptBody,
    candidate_packed: &[u8],
) -> Result<(), Vec<Diagnostic>> {
    if candidate_body != &expected.receipt_body
        || candidate_packed != expected.receipt.packed_bytes.as_slice()
    {
        return Err(issue(
            DiagnosticCode::IdentityMismatch,
            "projection receipt differs from the complete artifact set",
        ));
    }
    Ok(())
}
