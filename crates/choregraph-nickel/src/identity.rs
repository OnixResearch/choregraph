// r[impl choregraph.nickel.identity]
// r[impl choregraph.nickel.claim_boundary]
use crate::diagnostic::{FrontendFailure, FrontendIssueCode};
use serde::{Deserialize, Serialize};

const IDENTITY_CONTEXT: &str = "choregraph nickel translation identity v1";
const LENGTH_FRAME_BYTES: usize = size_of::<u64>();

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TranslationDomain {
    Source,
    ImportClosure,
    Evaluator,
    EvaluatedOutput,
    Lowerer,
    CanonicalDescriptor,
    AdmittedIr,
    ProofSet,
    Toolchain,
}

impl TranslationDomain {
    const fn label(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::ImportClosure => "import-closure",
            Self::Evaluator => "evaluator",
            Self::EvaluatedOutput => "evaluated-output",
            Self::Lowerer => "lowerer",
            Self::CanonicalDescriptor => "canonical-descriptor",
            Self::AdmittedIr => "admitted-ir",
            Self::ProofSet => "proof-set",
            Self::Toolchain => "toolchain",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranslationIdentity {
    pub domain: TranslationDomain,
    pub algorithm: String,
    pub hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranslationIdentityChain {
    pub source: TranslationIdentity,
    pub import_closure: TranslationIdentity,
    pub evaluator: TranslationIdentity,
    pub evaluated_output: TranslationIdentity,
    pub lowerer: TranslationIdentity,
    pub canonical_descriptor: TranslationIdentity,
    pub admitted_ir: TranslationIdentity,
    pub proof_set: TranslationIdentity,
    pub toolchain: TranslationIdentity,
}

#[derive(Clone, Copy, Debug)]
pub struct TranslationMaterials<'a> {
    pub source: &'a [u8],
    pub import_closure: &'a [u8],
    pub evaluator: &'a [u8],
    pub evaluated_output: &'a [u8],
    pub lowerer: &'a [u8],
    pub canonical_descriptor: &'a [u8],
    pub admitted_ir: &'a [u8],
    pub proof_set: &'a [u8],
    pub toolchain: &'a [u8],
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceReceipt {
    pub schema: String,
    pub source: TranslationIdentity,
    pub import_closure: TranslationIdentity,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvaluatorReceipt {
    pub schema: String,
    pub identities: TranslationIdentityChain,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslationReceipts {
    pub source: SourceReceipt,
    pub evaluator: EvaluatorReceipt,
}

fn compute(
    domain: TranslationDomain,
    material: &[u8],
) -> Result<TranslationIdentity, FrontendFailure> {
    if material.is_empty() {
        return Err(FrontendFailure::one(
            FrontendIssueCode::EmptyIdentityMaterial,
        ));
    }
    let label = domain.label().as_bytes();
    let label_length = u64::try_from(label.len())
        .map_err(|_| FrontendFailure::one(FrontendIssueCode::EmptyIdentityMaterial))?;
    let mut hasher = blake3::Hasher::new_derive_key(IDENTITY_CONTEXT);
    hasher.update(&label_length.to_le_bytes()[..LENGTH_FRAME_BYTES]);
    hasher.update(label);
    hasher.update(material);
    Ok(TranslationIdentity {
        domain,
        algorithm: String::from("blake3"),
        hex: hasher.finalize().to_hex().to_string(),
    })
}

pub fn build_translation_receipts(
    materials: TranslationMaterials<'_>,
) -> Result<TranslationReceipts, FrontendFailure> {
    let identities = TranslationIdentityChain {
        source: compute(TranslationDomain::Source, materials.source)?,
        import_closure: compute(TranslationDomain::ImportClosure, materials.import_closure)?,
        evaluator: compute(TranslationDomain::Evaluator, materials.evaluator)?,
        evaluated_output: compute(
            TranslationDomain::EvaluatedOutput,
            materials.evaluated_output,
        )?,
        lowerer: compute(TranslationDomain::Lowerer, materials.lowerer)?,
        canonical_descriptor: compute(
            TranslationDomain::CanonicalDescriptor,
            materials.canonical_descriptor,
        )?,
        admitted_ir: compute(TranslationDomain::AdmittedIr, materials.admitted_ir)?,
        proof_set: compute(TranslationDomain::ProofSet, materials.proof_set)?,
        toolchain: compute(TranslationDomain::Toolchain, materials.toolchain)?,
    };
    let source = SourceReceipt {
        schema: String::from("choregraph.nickel-source-receipt.v1"),
        source: identities.source.clone(),
        import_closure: identities.import_closure.clone(),
        checked_laws: vec![
            String::from("source-bytes-identity-recorded"),
            String::from("declared-import-closure-identity-recorded"),
        ],
        non_claims: vec![
            String::from("source receipt does not prove Nickel source meaning"),
            String::from("source receipt does not grant file or runtime authority"),
        ],
    };
    let evaluator = EvaluatorReceipt {
        schema: String::from("choregraph.nickel-evaluator-receipt.v1"),
        identities,
        checked_laws: vec![
            String::from("observed-evaluator-input-output-linkage"),
            String::from("canonical-admission-artifacts-linked"),
        ],
        non_claims: vec![
            String::from("receipt does not prove arbitrary Nickel semantic equivalence"),
            String::from("receipt does not prove evaluator soundness"),
            String::from("receipt does not prove transition correctness"),
            String::from("receipt does not authorize or execute effects"),
            String::from("receipt does not establish release eligibility"),
        ],
    };
    Ok(TranslationReceipts { source, evaluator })
}

pub fn validate_translation_receipts(
    receipts: &TranslationReceipts,
    materials: TranslationMaterials<'_>,
) -> Result<(), FrontendFailure> {
    let expected = build_translation_receipts(materials)?;
    if receipts != &expected {
        return Err(FrontendFailure::one(
            FrontendIssueCode::StaleTranslationReceipt,
        ));
    }
    Ok(())
}
