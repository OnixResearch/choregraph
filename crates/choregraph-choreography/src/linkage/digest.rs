use serde::{Deserialize, Serialize};

const IDENTITY_CONTEXT: &str = "choregraph role choreography artifact identity v1";
const BLAKE3_HEX_BYTES: usize = 64;
const BLAKE3_PREFIX: &str = "blake3:";

// One closed table owns both Serde names and BLAKE3 framing labels.
macro_rules! identity_domains {
    ($($variant:ident => $label:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(rename = "ChoreographyIdentityDomain")]
        pub enum IdentityDomain {
            $(#[serde(rename = $label)] $variant,)+
        }

        impl IdentityDomain {
            const fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label,)+
                }
            }
        }
    };
}

identity_domains! {
    SourceEnvelope => "source-envelope",
    ImportClosure => "import-closure",
    EvaluatorOutput => "evaluator-output",
    StrictDto => "strict-dto",
    GlobalArtifact => "global-artifact",
    LocalArtifact => "local-artifact",
    CompleteLocalSet => "complete-local-set",
    TrellisSourceSet => "trellis-source-set",
    TrellisProofSet => "trellis-proof-set",
    Lowerer => "lowerer",
    SchemaSet => "schema-set",
    Bounds => "bounds",
    Diagnostics => "diagnostics",
    Toolchain => "toolchain",
    NonClaims => "non-claims",
    ProjectionReceipt => "projection-receipt",
    PreferenceStrictDto => "preference-strict-dto",
    PreferenceAnnotations => "preference-annotations",
    PreferenceLocalSet => "preference-local-set",
    PreferenceSchemaSet => "preference-schema-set",
    PreferenceLowerer => "preference-lowerer",
    PreferenceNonClaims => "preference-non-claims",
    PreferenceReceipt => "preference-receipt",
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "ChoreographyIdentity", deny_unknown_fields)]
pub struct Identity {
    pub domain: IdentityDomain,
    pub algorithm: String,
    pub hex: String,
}

#[must_use]
pub fn compute_identity(domain: IdentityDomain, material: &[u8]) -> Identity {
    let domain_label = domain.label().as_bytes();
    let mut hasher = blake3::Hasher::new_derive_key(IDENTITY_CONTEXT);
    hasher.update(domain_label);
    hasher.update(&[0]);
    hasher.update(material);
    Identity {
        domain,
        algorithm: String::from("blake3"),
        hex: hasher.finalize().to_hex().to_string(),
    }
}

#[must_use]
pub fn identity_is_valid(identity: &Identity, expected: IdentityDomain) -> bool {
    identity.domain == expected
        && identity.algorithm == "blake3"
        && identity.hex.len() == BLAKE3_HEX_BYTES
        && identity
            .hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[must_use]
pub fn prefixed_identity(identity: &Identity) -> String {
    format!("{BLAKE3_PREFIX}{}", identity.hex)
}
