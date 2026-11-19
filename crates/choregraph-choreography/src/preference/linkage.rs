//! Canonical artifacts and receipt for one annotated projection cohort.
//!
//! The bundle wraps the unchanged v1 `crate::linkage::ProjectionBundle`. Its
//! receipt binds the annotation identities to that cohort's global artifact,
//! every local artifact, the complete local set, and the v1 receipt.

use serde::{Deserialize, Serialize};

const SCHEMA_COUNT: usize = 4;
const LINKAGE_CLAIM: &str = "identity-linkage-only-no-claim-promotion";
const LOWERER_PREIMAGE: &str =
    "choregraph-choreography-v0.1.0:preference-annotations-over-direct-trellis-project-endpoint";

/// Exact Preserves schema version, root label, and identity domain of one
/// annotated artifact kind.
struct ArtifactProfile {
    schema: &'static str,
    root: &'static str,
    domain: crate::linkage::digest::IdentityDomain,
}

const SOURCE_PROFILE: ArtifactProfile = ArtifactProfile {
    schema: "choregraph.role-choreography-preference-source-preserves.v1",
    root: "AnnotatedRoleChoreographySource",
    domain: crate::linkage::digest::IdentityDomain::PreferenceStrictDto,
};
const ANNOTATIONS_PROFILE: ArtifactProfile = ArtifactProfile {
    schema: "choregraph.role-choreography-preference-annotations-preserves.v1",
    root: "PreferenceAnnotationSet",
    domain: crate::linkage::digest::IdentityDomain::PreferenceAnnotations,
};
const LOCAL_SET_PROFILE: ArtifactProfile = ArtifactProfile {
    schema: "choregraph.role-choreography-preference-local-set-preserves.v1",
    root: "AnnotatedLocalSetBody",
    domain: crate::linkage::digest::IdentityDomain::PreferenceLocalSet,
};
const RECEIPT_PROFILE: ArtifactProfile = ArtifactProfile {
    schema: "choregraph.role-choreography-preference-receipt-preserves.v1",
    root: "PreferenceProjectionReceipt",
    domain: crate::linkage::digest::IdentityDomain::PreferenceReceipt,
};

/// Status of any mapping from an annotated choreography to a formal game.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "GameMappingStatus", rename_all = "kebab-case")]
pub enum GameMapping {
    NotFormalized,
}

/// Canonical receipt binding preference metadata to one v1 projection cohort.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "PreferenceProjectionReceipt", deny_unknown_fields)]
pub struct ReceiptBody {
    pub profile: String,
    pub schema: String,
    pub base_receipt: crate::linkage::digest::Identity,
    pub global_artifact: crate::linkage::digest::Identity,
    pub local_artifacts: Vec<crate::linkage::digest::Identity>,
    pub complete_local_set: crate::linkage::digest::Identity,
    pub strict_dto: crate::linkage::digest::Identity,
    pub annotations: crate::linkage::digest::Identity,
    pub annotated_local_set: crate::linkage::digest::Identity,
    pub schema_set: crate::linkage::digest::Identity,
    pub lowerer: crate::linkage::digest::Identity,
    pub non_claims: crate::linkage::digest::Identity,
    pub game_mapping: GameMapping,
    pub claim: String,
}

#[derive(Serialize)]
#[serde(rename = "AnnotatedRoleChoreographySource")]
struct StrictView<'a> {
    schema: &'a str,
    base: &'a crate::model::schema::RoleSource,
    annotations: &'a super::model::AnnotationSet,
    non_claims: &'a [String],
}

/// The v1 bundle, its Trellis projection, and the annotation artifacts.
#[derive(Debug)]
pub struct AnnotatedBundle {
    pub(crate) base: crate::linkage::ProjectionBundle,
    pub(crate) projected: crate::endpoint::Projected,
    pub(crate) source: crate::linkage::output::CanonicalArtifact,
    pub(crate) annotations: crate::linkage::output::CanonicalArtifact,
    pub(crate) local_set_body: super::projection::AnnotatedLocalSet,
    pub(crate) local_set: crate::linkage::output::CanonicalArtifact,
    pub(crate) receipt_body: ReceiptBody,
    pub(crate) receipt: crate::linkage::output::CanonicalArtifact,
}

impl AnnotatedBundle {
    /// The unchanged v1 projection bundle of the cohort.
    #[must_use]
    pub const fn base(&self) -> &crate::linkage::ProjectionBundle {
        &self.base
    }

    /// The Trellis projection whose local set the v1 bundle encodes.
    #[must_use]
    pub const fn projected(&self) -> &crate::endpoint::Projected {
        &self.projected
    }

    /// Annotated strict DTO artifact.
    #[must_use]
    pub const fn source(&self) -> &crate::linkage::output::CanonicalArtifact {
        &self.source
    }

    /// Normalized annotation-set artifact.
    #[must_use]
    pub const fn annotations(&self) -> &crate::linkage::output::CanonicalArtifact {
        &self.annotations
    }

    #[must_use]
    pub const fn local_set_body(&self) -> &super::projection::AnnotatedLocalSet {
        &self.local_set_body
    }

    /// Annotated complete local-set artifact.
    #[must_use]
    pub const fn local_set(&self) -> &crate::linkage::output::CanonicalArtifact {
        &self.local_set
    }

    #[must_use]
    pub const fn receipt_body(&self) -> &ReceiptBody {
        &self.receipt_body
    }

    /// Annotated projection receipt artifact.
    #[must_use]
    pub const fn receipt(&self) -> &crate::linkage::output::CanonicalArtifact {
        &self.receipt
    }
}

fn issue(code: crate::issue::DiagnosticCode, note: &'static str) -> Vec<crate::issue::Diagnostic> {
    vec![crate::issue::Diagnostic::error(
        code,
        crate::model::participants::SourceSpan { start: 0, end: 0 },
        note,
    )]
}

fn encode_error(note: &'static str) -> Vec<crate::issue::Diagnostic> {
    issue(crate::issue::DiagnosticCode::ArtifactEncodingFailed, note)
}

fn public_artifact(
    artifact: choregraph_kamacite::codec::CanonicalArtifact,
    domain: crate::linkage::digest::IdentityDomain,
) -> crate::linkage::output::CanonicalArtifact {
    let identity = crate::linkage::digest::compute_identity(domain, &artifact.packed_bytes);
    crate::linkage::output::CanonicalArtifact {
        schema_version: artifact.schema_version,
        packed_bytes: artifact.packed_bytes,
        text_projection: artifact.text_projection,
        identity,
    }
}

fn append_digest(material: &mut Vec<u8>, value: &[u8]) {
    material.extend_from_slice(blake3::hash(value).as_bytes());
}

fn schema_identity() -> crate::linkage::digest::Identity {
    let schemas: [(&str, &[u8]); SCHEMA_COUNT] = [
        (
            SOURCE_PROFILE.schema,
            include_bytes!(
                "../../../../schemas/preserves/choregraph-role-choreography-preference-source-v1.preserves"
            ),
        ),
        (
            ANNOTATIONS_PROFILE.schema,
            include_bytes!(
                "../../../../schemas/preserves/choregraph-role-choreography-preference-annotations-v1.preserves"
            ),
        ),
        (
            LOCAL_SET_PROFILE.schema,
            include_bytes!(
                "../../../../schemas/preserves/choregraph-role-choreography-preference-local-set-v1.preserves"
            ),
        ),
        (
            RECEIPT_PROFILE.schema,
            include_bytes!(
                "../../../../schemas/preserves/choregraph-role-choreography-preference-receipt-v1.preserves"
            ),
        ),
    ];
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
    debug_assert!(!material.is_empty());
    crate::linkage::digest::compute_identity(
        crate::linkage::digest::IdentityDomain::PreferenceSchemaSet,
        &material,
    )
}

/// Binds the v1 lowerer identity (which hashes the v1 admission and endpoint
/// sources) and the annotation model, admission, and projection sources.
fn lowerer_identity(base: &crate::linkage::ProjectionBundle) -> crate::linkage::digest::Identity {
    let mut material = Vec::new();
    append_digest(&mut material, LOWERER_PREIMAGE.as_bytes());
    append_digest(&mut material, base.receipt_body().lowerer.hex.as_bytes());
    append_digest(&mut material, include_bytes!("model.rs"));
    append_digest(&mut material, include_bytes!("admission.rs"));
    append_digest(&mut material, include_bytes!("projection.rs"));
    crate::linkage::digest::compute_identity(
        crate::linkage::digest::IdentityDomain::PreferenceLowerer,
        &material,
    )
}

fn non_claims_identity(non_claims: &[String]) -> crate::linkage::digest::Identity {
    crate::linkage::digest::compute_identity(
        crate::linkage::digest::IdentityDomain::PreferenceNonClaims,
        non_claims.join("\n").as_bytes(),
    )
}

struct Encoded {
    source: crate::linkage::output::CanonicalArtifact,
    annotations: crate::linkage::output::CanonicalArtifact,
    local_set: crate::linkage::output::CanonicalArtifact,
}

fn encode_artifact<T: Serialize>(
    value: &T,
    profile: &ArtifactProfile,
    note: &'static str,
) -> Result<crate::linkage::output::CanonicalArtifact, Vec<crate::issue::Diagnostic>> {
    choregraph_kamacite::codec::encode(value, profile.schema, profile.root)
        .map(|artifact| public_artifact(artifact, profile.domain))
        .map_err(|_| encode_error(note))
}

fn encode(
    admitted: &super::admission::Admitted,
    local_set: &super::projection::AnnotatedLocalSet,
) -> Result<Encoded, Vec<crate::issue::Diagnostic>> {
    let view = StrictView {
        schema: super::model::ANNOTATED_SCHEMA,
        base: &admitted.base.source,
        annotations: &admitted.annotations,
        non_claims: &admitted.non_claims,
    };
    let encoded = Encoded {
        source: encode_artifact(
            &view,
            &SOURCE_PROFILE,
            "annotated strict DTO Preserves encoding failed",
        )?,
        annotations: encode_artifact(
            &admitted.annotations,
            &ANNOTATIONS_PROFILE,
            "annotation set Preserves encoding failed",
        )?,
        local_set: encode_artifact(
            local_set,
            &LOCAL_SET_PROFILE,
            "annotated local set Preserves encoding failed",
        )?,
    };
    debug_assert_eq!(encoded.source.schema_version, SOURCE_PROFILE.schema);
    debug_assert_eq!(encoded.local_set.schema_version, LOCAL_SET_PROFILE.schema);
    Ok(encoded)
}

fn receipt_body(
    admitted: &super::admission::Admitted,
    base: &crate::linkage::ProjectionBundle,
    encoded: &Encoded,
) -> ReceiptBody {
    let body = ReceiptBody {
        profile: String::from(super::ANNOTATED_PROFILE),
        schema: String::from(super::model::ANNOTATED_SCHEMA),
        base_receipt: base.receipt().identity.clone(),
        global_artifact: base.global().identity.clone(),
        local_artifacts: base.receipt_body().local_artifacts.clone(),
        complete_local_set: base.local_set().identity.clone(),
        strict_dto: encoded.source.identity.clone(),
        annotations: encoded.annotations.identity.clone(),
        annotated_local_set: encoded.local_set.identity.clone(),
        schema_set: schema_identity(),
        lowerer: lowerer_identity(base),
        non_claims: non_claims_identity(&admitted.non_claims),
        game_mapping: GameMapping::NotFormalized,
        claim: String::from(LINKAGE_CLAIM),
    };
    debug_assert_eq!(body.local_artifacts.len(), base.locals().len());
    debug_assert_eq!(body.complete_local_set, base.local_set().identity);
    body
}

fn total_bytes(bundle: &AnnotatedBundle) -> usize {
    let base = &bundle.base;
    let artifacts = [
        base.source(),
        base.global(),
        base.local_set(),
        base.receipt(),
        &bundle.source,
        &bundle.annotations,
        &bundle.local_set,
        &bundle.receipt,
    ];
    let locals = base
        .locals()
        .iter()
        .map(|local| local.artifact.packed_bytes.len());
    let total = artifacts
        .iter()
        .map(|artifact| artifact.packed_bytes.len())
        .chain(locals)
        .fold(0_usize, usize::saturating_add);
    debug_assert!(total >= bundle.receipt.packed_bytes.len());
    debug_assert!(total >= bundle.base.receipt().packed_bytes.len());
    total
}

/// Projects the admitted base with Trellis and binds the annotations to it.
///
/// Any failure emits no annotated artifact.
pub fn build_bundle(
    admitted: &super::admission::Admitted,
    materials: &crate::linkage::ProjectionMaterials,
) -> Result<AnnotatedBundle, Vec<crate::issue::Diagnostic>> {
    let projected = crate::endpoint::project_all(&admitted.base)?;
    let base = crate::linkage::build_projection_bundle(&admitted.base, materials)?;
    let local_set_body = super::projection::annotate(admitted, &projected, &base)?;
    let encoded = encode(admitted, &local_set_body)?;
    let body = receipt_body(admitted, &base, &encoded);
    let receipt = encode_artifact(
        &body,
        &RECEIPT_PROFILE,
        "annotated projection receipt Preserves encoding failed",
    )?;
    let bundle = AnnotatedBundle {
        base,
        projected,
        source: encoded.source,
        annotations: encoded.annotations,
        local_set_body,
        local_set: encoded.local_set,
        receipt_body: body,
        receipt,
    };
    if total_bytes(&bundle) > crate::model::bounds::MAX_CANONICAL_OUTPUT_BYTES {
        return Err(issue(
            crate::issue::DiagnosticCode::CanonicalOutputLimitExceeded,
            "canonical annotated output exceeds the reviewed limit",
        ));
    }
    debug_assert_eq!(
        bundle.local_set_body.locals.len(),
        bundle.base.locals().len()
    );
    debug_assert_eq!(
        bundle.receipt_body.annotated_local_set,
        bundle.local_set.identity
    );
    Ok(bundle)
}

/// Verifies a candidate receipt against the complete annotated bundle.
pub fn verify_receipt(
    expected: &AnnotatedBundle,
    candidate_body: &ReceiptBody,
    candidate_packed: &[u8],
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    let is_exact = candidate_body == &expected.receipt_body
        && candidate_packed == expected.receipt.packed_bytes.as_slice();
    if is_exact {
        Ok(())
    } else {
        Err(issue(
            crate::issue::DiagnosticCode::IdentityMismatch,
            "annotated projection receipt differs from the complete artifact set",
        ))
    }
}

/// Verifies a candidate annotated local set against the bundle: it must be a
/// complete, ordered set over the bundle's v1 cohort and encode to exactly the
/// bundle's annotated local-set bytes.
pub fn verify_local_set(
    expected: &AnnotatedBundle,
    candidate: &super::projection::AnnotatedLocalSet,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    super::projection::validate_set(&expected.base, candidate)?;
    let encoded = encode_artifact(
        candidate,
        &LOCAL_SET_PROFILE,
        "annotated local set Preserves encoding failed",
    )?;
    if encoded.packed_bytes == expected.local_set.packed_bytes {
        Ok(())
    } else {
        Err(issue(
            crate::issue::DiagnosticCode::IdentityMismatch,
            "annotated local set differs from the bundle artifact",
        ))
    }
}
