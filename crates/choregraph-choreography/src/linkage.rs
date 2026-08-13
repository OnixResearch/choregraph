#![allow(
    unknown_lints,
    borrowed_argument_types,
    fragile_exhaustive_enum_match,
    non_trait_imports,
    reason = "the pure bundle builder grows one owned framing buffer, keeps closed identity domains exhaustive, and shows exact artifact owner types at one linkage boundary"
)]

use crate::admission::AdmittedChoreography;
use crate::endpoint::{LocalProtocolArtifactBody, ProjectedChoreography, project_all};
use crate::issue::{ChoreographyDiagnostic, DiagnosticCode};
use crate::model::{MAX_CANONICAL_OUTPUT_BYTES, SourceSpan};
use choregraph_kamacite::{
    CanonicalArtifact, ROLE_CHOREOGRAPHY_DIAGNOSTICS_SCHEMA, ROLE_CHOREOGRAPHY_GLOBAL_SCHEMA,
    ROLE_CHOREOGRAPHY_LOCAL_SCHEMA, ROLE_CHOREOGRAPHY_LOCAL_SET_SCHEMA,
    ROLE_CHOREOGRAPHY_RECEIPT_SCHEMA, ROLE_CHOREOGRAPHY_SOURCE_SCHEMA,
    encode_role_choreography_diagnostics, encode_role_choreography_global,
    encode_role_choreography_local, encode_role_choreography_local_set,
    encode_role_choreography_receipt, encode_role_choreography_source,
};
use serde::{Deserialize, Serialize};

pub const TRELLIS_REVISION: &str = "d65974b14978e1a44b0f5067beb9ef6d83317dc8";
pub const TRELLIS_SEMANTIC_COMMIT: &str = "32b5b03115377d232f3c017489c1dd657b446a4f";
pub const TRELLIS_SOURCE_MANIFEST_BLAKE3: &str =
    "28bdef948c196d9ff84268a2028e90d7148e7950bcc4046489f5483b0421c99a";
pub const TRELLIS_TOOLCHAIN: &str = "rust-1.96.0+vstd-0.0.0-2026-01-25-0057";
pub const CHOREGRAPH_TOOLCHAIN: &str = "rust-1.97.1";
pub const LOWERER_IDENTITY_PREIMAGE: &str =
    "choregraph-choreography-v0.1.0:direct-trellis-project-endpoint";
pub const TRELLIS_PROOF_SET_PREIMAGE: &str = concat!(
    "successful_transfer_conserves_ownership\n",
    "successful_transfer_rejects_stale_source\n",
    "successful_owned_transfer_step_conserves\n",
    "choreography_owned_transfer_projection_correspondence\n",
);

const IDENTITY_CONTEXT: &str = "choregraph role choreography artifact identity v1";
const BLAKE3_HEX_BYTES: usize = 64;
const BLAKE3_PREFIX: &str = "blake3:";
const FIXED_EVIDENCE_BINDING_COUNT: usize = 4;
const ROLE_CHOREOGRAPHY_SCHEMA_COUNT: usize = 6;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChoreographyIdentityDomain {
    SourceEnvelope,
    ImportClosure,
    EvaluatorOutput,
    StrictDto,
    GlobalArtifact,
    LocalArtifact,
    CompleteLocalSet,
    TrellisSourceSet,
    TrellisProofSet,
    Lowerer,
    SchemaSet,
    Bounds,
    Diagnostics,
    Toolchain,
    NonClaims,
    ProjectionReceipt,
}

impl ChoreographyIdentityDomain {
    const fn label(self) -> &'static str {
        match self {
            Self::SourceEnvelope => "source-envelope",
            Self::ImportClosure => "import-closure",
            Self::EvaluatorOutput => "evaluator-output",
            Self::StrictDto => "strict-dto",
            Self::GlobalArtifact => "global-artifact",
            Self::LocalArtifact => "local-artifact",
            Self::CompleteLocalSet => "complete-local-set",
            Self::TrellisSourceSet => "trellis-source-set",
            Self::TrellisProofSet => "trellis-proof-set",
            Self::Lowerer => "lowerer",
            Self::SchemaSet => "schema-set",
            Self::Bounds => "bounds",
            Self::Diagnostics => "diagnostics",
            Self::Toolchain => "toolchain",
            Self::NonClaims => "non-claims",
            Self::ProjectionReceipt => "projection-receipt",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChoreographyIdentity {
    pub domain: ChoreographyIdentityDomain,
    pub algorithm: String,
    pub hex: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectionEvidenceRole {
    Source,
    GlobalProtocol,
    LocalProtocol,
    CompleteLocalSet,
    ProofReference,
    ProjectionReceipt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionEvidenceBinding {
    pub role: ProjectionEvidenceRole,
    pub identity: ChoreographyIdentity,
    pub claim: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DiagnosticArtifactBody {
    diagnostics: Vec<ChoreographyDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionReceiptBody {
    pub source_envelope: ChoreographyIdentity,
    pub import_closure: ChoreographyIdentity,
    pub evaluator_output: ChoreographyIdentity,
    pub strict_dto: ChoreographyIdentity,
    pub global_artifact: ChoreographyIdentity,
    pub local_artifacts: Vec<ChoreographyIdentity>,
    pub complete_local_set: ChoreographyIdentity,
    pub trellis_revision: String,
    pub trellis_semantic_commit: String,
    pub trellis_source_set: ChoreographyIdentity,
    pub trellis_proof_set: ChoreographyIdentity,
    pub lowerer: ChoreographyIdentity,
    pub schema_set: ChoreographyIdentity,
    pub bounds: ChoreographyIdentity,
    pub diagnostics: ChoreographyIdentity,
    pub toolchain: ChoreographyIdentity,
    pub non_claims: ChoreographyIdentity,
    pub evidence_roles: Vec<ProjectionEvidenceBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalChoreographyArtifact {
    pub schema_version: String,
    pub packed_bytes: Vec<u8>,
    pub text_projection: String,
    pub identity: ChoreographyIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalLocalArtifact {
    pub role: crate::model::RoleId,
    pub artifact: CanonicalChoreographyArtifact,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionBundle {
    pub(crate) source: CanonicalChoreographyArtifact,
    pub(crate) global: CanonicalChoreographyArtifact,
    pub(crate) locals: Vec<CanonicalLocalArtifact>,
    pub(crate) local_set: CanonicalChoreographyArtifact,
    pub(crate) receipt_body: ProjectionReceiptBody,
    pub(crate) receipt: CanonicalChoreographyArtifact,
}

impl ProjectionBundle {
    #[must_use]
    pub const fn source(&self) -> &CanonicalChoreographyArtifact {
        &self.source
    }

    #[must_use]
    pub const fn global(&self) -> &CanonicalChoreographyArtifact {
        &self.global
    }

    #[must_use]
    pub fn locals(&self) -> &[CanonicalLocalArtifact] {
        &self.locals
    }

    #[must_use]
    pub const fn local_set(&self) -> &CanonicalChoreographyArtifact {
        &self.local_set
    }

    #[must_use]
    pub const fn receipt_body(&self) -> &ProjectionReceiptBody {
        &self.receipt_body
    }

    #[must_use]
    pub const fn receipt(&self) -> &CanonicalChoreographyArtifact {
        &self.receipt
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionMaterials {
    source_envelope: ChoreographyIdentity,
    evaluator_output: ChoreographyIdentity,
}

fn issue(code: DiagnosticCode, note: &'static str) -> Vec<ChoreographyDiagnostic> {
    vec![ChoreographyDiagnostic::error(
        code,
        SourceSpan { start: 0, end: 0 },
        note,
    )]
}

#[must_use]
pub fn compute_identity(
    domain: ChoreographyIdentityDomain,
    material: &[u8],
) -> ChoreographyIdentity {
    let domain_label = domain.label().as_bytes();
    let mut hasher = blake3::Hasher::new_derive_key(IDENTITY_CONTEXT);
    hasher.update(domain_label);
    hasher.update(&[0]);
    hasher.update(material);
    ChoreographyIdentity {
        domain,
        algorithm: String::from("blake3"),
        hex: hasher.finalize().to_hex().to_string(),
    }
}

#[must_use]
pub fn identity_is_valid(
    identity: &ChoreographyIdentity,
    expected: ChoreographyIdentityDomain,
) -> bool {
    identity.domain == expected
        && identity.algorithm == "blake3"
        && identity.hex.len() == BLAKE3_HEX_BYTES
        && identity
            .hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

impl ProjectionMaterials {
    #[must_use]
    pub fn reviewed(source_envelope_bytes: &[u8], evaluator_output_bytes: &[u8]) -> Self {
        Self {
            source_envelope: compute_identity(
                ChoreographyIdentityDomain::SourceEnvelope,
                source_envelope_bytes,
            ),
            evaluator_output: compute_identity(
                ChoreographyIdentityDomain::EvaluatorOutput,
                evaluator_output_bytes,
            ),
        }
    }
}

fn public_artifact(
    artifact: CanonicalArtifact,
    domain: ChoreographyIdentityDomain,
) -> CanonicalChoreographyArtifact {
    let identity = compute_identity(domain, &artifact.packed_bytes);
    CanonicalChoreographyArtifact {
        schema_version: artifact.schema_version,
        packed_bytes: artifact.packed_bytes,
        text_projection: artifact.text_projection,
        identity,
    }
}

fn encode_error(note: &'static str) -> Vec<ChoreographyDiagnostic> {
    issue(DiagnosticCode::ArtifactEncodingFailed, note)
}

fn append_digest(material: &mut Vec<u8>, value: &[u8]) {
    material.extend_from_slice(blake3::hash(value).as_bytes());
}

fn schema_identity() -> ChoreographyIdentity {
    let schemas: [(&str, &[u8]); ROLE_CHOREOGRAPHY_SCHEMA_COUNT] = [
        (
            ROLE_CHOREOGRAPHY_SOURCE_SCHEMA,
            include_bytes!(
                "../../../schemas/preserves/choregraph-role-choreography-source-v1.preserves"
            ),
        ),
        (
            ROLE_CHOREOGRAPHY_GLOBAL_SCHEMA,
            include_bytes!(
                "../../../schemas/preserves/choregraph-role-choreography-global-v1.preserves"
            ),
        ),
        (
            ROLE_CHOREOGRAPHY_LOCAL_SCHEMA,
            include_bytes!(
                "../../../schemas/preserves/choregraph-role-choreography-local-v1.preserves"
            ),
        ),
        (
            ROLE_CHOREOGRAPHY_LOCAL_SET_SCHEMA,
            include_bytes!(
                "../../../schemas/preserves/choregraph-role-choreography-local-set-v1.preserves"
            ),
        ),
        (
            ROLE_CHOREOGRAPHY_DIAGNOSTICS_SCHEMA,
            include_bytes!(
                "../../../schemas/preserves/choregraph-role-choreography-diagnostics-v1.preserves"
            ),
        ),
        (
            ROLE_CHOREOGRAPHY_RECEIPT_SCHEMA,
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
    compute_identity(ChoreographyIdentityDomain::SchemaSet, &material)
}

fn lowerer_identity() -> ChoreographyIdentity {
    let mut material = Vec::new();
    append_digest(&mut material, LOWERER_IDENTITY_PREIMAGE.as_bytes());
    append_digest(&mut material, include_bytes!("admission.rs"));
    append_digest(&mut material, include_bytes!("endpoint.rs"));
    compute_identity(ChoreographyIdentityDomain::Lowerer, &material)
}

fn toolchain_identity() -> ChoreographyIdentity {
    let material = format!("{CHOREGRAPH_TOOLCHAIN}\n{TRELLIS_TOOLCHAIN}");
    compute_identity(ChoreographyIdentityDomain::Toolchain, material.as_bytes())
}

fn trellis_source_identity() -> ChoreographyIdentity {
    compute_identity(
        ChoreographyIdentityDomain::TrellisSourceSet,
        format!("{TRELLIS_REVISION}\n{TRELLIS_SOURCE_MANIFEST_BLAKE3}").as_bytes(),
    )
}

fn evidence_roles(
    source: &ChoreographyIdentity,
    global: &ChoreographyIdentity,
    locals: &[ChoreographyIdentity],
    local_set: &ChoreographyIdentity,
    proof_set: &ChoreographyIdentity,
) -> Vec<ProjectionEvidenceBinding> {
    let linkage_claim = String::from("identity-linkage-only-no-claim-promotion");
    let mut bindings =
        Vec::with_capacity(locals.len().saturating_add(FIXED_EVIDENCE_BINDING_COUNT));
    bindings.push(ProjectionEvidenceBinding {
        role: ProjectionEvidenceRole::Source,
        identity: source.clone(),
        claim: linkage_claim.clone(),
    });
    bindings.push(ProjectionEvidenceBinding {
        role: ProjectionEvidenceRole::GlobalProtocol,
        identity: global.clone(),
        claim: linkage_claim.clone(),
    });
    for local in locals {
        bindings.push(ProjectionEvidenceBinding {
            role: ProjectionEvidenceRole::LocalProtocol,
            identity: local.clone(),
            claim: linkage_claim.clone(),
        });
    }
    bindings.push(ProjectionEvidenceBinding {
        role: ProjectionEvidenceRole::CompleteLocalSet,
        identity: local_set.clone(),
        claim: linkage_claim.clone(),
    });
    bindings.push(ProjectionEvidenceBinding {
        role: ProjectionEvidenceRole::ProofReference,
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
) -> Result<CanonicalLocalArtifact, Vec<ChoreographyDiagnostic>> {
    let artifact = encode_role_choreography_local(local)
        .map_err(|_| encode_error("local protocol Preserves encoding failed"))?;
    Ok(CanonicalLocalArtifact {
        role: local.role,
        artifact: public_artifact(artifact, ChoreographyIdentityDomain::LocalArtifact),
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
    source: CanonicalChoreographyArtifact,
    global: CanonicalChoreographyArtifact,
    locals: Vec<CanonicalLocalArtifact>,
    local_identities: Vec<ChoreographyIdentity>,
    local_set: CanonicalChoreographyArtifact,
    diagnostics: CanonicalArtifact,
}

fn encode_projection(
    admitted: &AdmittedChoreography,
) -> Result<EncodedProjection, Vec<ChoreographyDiagnostic>> {
    let ProjectedChoreography { global, locals } = project_all(admitted)?;
    let source = public_artifact(
        encode_role_choreography_source(&admitted.source)
            .map_err(|_| encode_error("strict DTO Preserves encoding failed"))?,
        ChoreographyIdentityDomain::StrictDto,
    );
    let global = public_artifact(
        encode_role_choreography_global(&global)
            .map_err(|_| encode_error("global protocol Preserves encoding failed"))?,
        ChoreographyIdentityDomain::GlobalArtifact,
    );
    let encoded_locals: Vec<CanonicalLocalArtifact> = locals
        .locals
        .iter()
        .map(encode_local)
        .collect::<Result<_, _>>()?;
    let local_identities: Vec<ChoreographyIdentity> = encoded_locals
        .iter()
        .map(|local| local.artifact.identity.clone())
        .collect();
    let local_set = public_artifact(
        encode_role_choreography_local_set(&locals)
            .map_err(|_| encode_error("complete local set Preserves encoding failed"))?,
        ChoreographyIdentityDomain::CompleteLocalSet,
    );
    let diagnostics = encode_role_choreography_diagnostics(&DiagnosticArtifactBody {
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
        ChoreographyIdentityDomain::TrellisProofSet,
        TRELLIS_PROOF_SET_PREIMAGE.as_bytes(),
    );
    let receipt = ProjectionReceiptBody {
        source_envelope: materials.source_envelope.clone(),
        import_closure: compute_identity(
            ChoreographyIdentityDomain::ImportClosure,
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
        bounds: compute_identity(
            ChoreographyIdentityDomain::Bounds,
            &encoded.source.packed_bytes,
        ),
        diagnostics: compute_identity(
            ChoreographyIdentityDomain::Diagnostics,
            &encoded.diagnostics.packed_bytes,
        ),
        toolchain: toolchain_identity(),
        non_claims: compute_identity(
            ChoreographyIdentityDomain::NonClaims,
            &encoded.source.packed_bytes,
        ),
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
    admitted: &AdmittedChoreography,
    materials: &ProjectionMaterials,
) -> Result<ProjectionBundle, Vec<ChoreographyDiagnostic>> {
    if !identity_is_valid(
        &materials.source_envelope,
        ChoreographyIdentityDomain::SourceEnvelope,
    ) || !identity_is_valid(
        &materials.evaluator_output,
        ChoreographyIdentityDomain::EvaluatorOutput,
    ) {
        return Err(issue(
            DiagnosticCode::IdentityMismatch,
            "source or evaluator identity is invalid",
        ));
    }
    let encoded = encode_projection(admitted)?;
    let receipt_body = projection_receipt(materials, &encoded);
    let receipt = public_artifact(
        encode_role_choreography_receipt(&receipt_body)
            .map_err(|_| encode_error("projection receipt Preserves encoding failed"))?,
        ChoreographyIdentityDomain::ProjectionReceipt,
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
) -> Result<(), Vec<ChoreographyDiagnostic>> {
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

#[must_use]
pub fn prefixed_identity(identity: &ChoreographyIdentity) -> String {
    format!("{BLAKE3_PREFIX}{}", identity.hex)
}
