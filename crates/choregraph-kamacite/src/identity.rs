// Identity code keeps explicit domain matches and fixed-size BLAKE3 arithmetic.
#![allow(
    unknown_lints,
    ambiguous_params,
    fragile_exhaustive_enum_match,
    missing_const_fn,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    reason = "identity code keeps explicit domain matches and fixed-size BLAKE3 arithmetic"
)]

// r[impl choregraph.interchange.kamacite.identity]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub const BLAKE3_ALGORITHM: &str = "blake3";
pub const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARACTERS_PER_BYTE;
const HEX_CHARACTERS_PER_BYTE: usize = 2;
const IDENTITY_CONTEXT: &str = "choregraph artifact identity v1";
const DOMAIN_LENGTH_BYTES: usize = size_of::<u64>();

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactDomain {
    Machine,
    Lifecycle,
    AdmittedIr,
    ProofSet,
    Generator,
    Adapter,
    Profile,
    Toolchain,
    Receipt,
}

impl ArtifactDomain {
    const fn label(self) -> &'static str {
        match self {
            Self::Machine => "machine",
            Self::Lifecycle => "lifecycle",
            Self::AdmittedIr => "admitted-ir",
            Self::ProofSet => "proof-set",
            Self::Generator => "generator",
            Self::Adapter => "adapter",
            Self::Profile => "profile",
            Self::Toolchain => "toolchain",
            Self::Receipt => "receipt",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KamaciteIdentityDomain {
    Source,
    CanonicalValue,
    Adapter,
    Receipt,
    FsmSpec,
    FsmUniverse,
    FsmCoverage,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ArtifactIdentity {
    pub domain: ArtifactDomain,
    pub algorithm: String,
    pub hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct KamaciteIdentity {
    pub domain: KamaciteIdentityDomain,
    pub algorithm: String,
    pub hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IdentityChain {
    pub machine: ArtifactIdentity,
    pub lifecycle: ArtifactIdentity,
    pub admitted_ir: ArtifactIdentity,
    pub proof_set: ArtifactIdentity,
    pub generator: ArtifactIdentity,
    pub adapter: ArtifactIdentity,
    pub profile: ArtifactIdentity,
    pub toolchain: ArtifactIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityError {
    EmptyMaterial {
        domain: ArtifactDomain,
    },
    MaterialLengthOverflow {
        domain: ArtifactDomain,
    },
    WrongAlgorithm {
        actual: String,
    },
    InvalidLength {
        actual: usize,
        expected: usize,
    },
    InvalidHex,
    WrongDomain {
        actual: ArtifactDomain,
        expected: ArtifactDomain,
    },
    WrongKamaciteDomain {
        actual: KamaciteIdentityDomain,
        expected: KamaciteIdentityDomain,
    },
}

impl Display for IdentityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMaterial { domain } => {
                write!(formatter, "identity material for {domain:?} is empty")
            }
            Self::MaterialLengthOverflow { domain } => {
                write!(
                    formatter,
                    "identity framing length for {domain:?} exceeds u64"
                )
            }
            Self::WrongAlgorithm { actual } => {
                write!(formatter, "identity algorithm {actual:?} is not blake3")
            }
            Self::InvalidLength { actual, expected } => {
                write!(formatter, "identity hash length {actual} != {expected}")
            }
            Self::InvalidHex => formatter.write_str("identity hash is not lowercase hexadecimal"),
            Self::WrongDomain { actual, expected } => {
                write!(formatter, "identity domain {actual:?} is not {expected:?}")
            }
            Self::WrongKamaciteDomain { actual, expected } => {
                write!(
                    formatter,
                    "Kamacite identity domain {actual:?} is not {expected:?}"
                )
            }
        }
    }
}

impl std::error::Error for IdentityError {}

impl ArtifactIdentity {
    pub fn validate(&self, expected: ArtifactDomain) -> Result<(), IdentityError> {
        validate_hash_shape(&self.algorithm, &self.hex)?;
        if self.domain != expected {
            return Err(IdentityError::WrongDomain {
                actual: self.domain,
                expected,
            });
        }
        Ok(())
    }
}

impl KamaciteIdentity {
    pub fn validate(&self, expected: KamaciteIdentityDomain) -> Result<(), IdentityError> {
        validate_hash_shape(&self.algorithm, &self.hex)?;
        if self.domain != expected {
            return Err(IdentityError::WrongKamaciteDomain {
                actual: self.domain,
                expected,
            });
        }
        Ok(())
    }
}

fn validate_hash_shape(algorithm: &str, hex: &str) -> Result<(), IdentityError> {
    if algorithm != BLAKE3_ALGORITHM {
        return Err(IdentityError::WrongAlgorithm {
            actual: algorithm.to_owned(),
        });
    }
    if hex.len() != BLAKE3_HEX_LENGTH {
        return Err(IdentityError::InvalidLength {
            actual: hex.len(),
            expected: BLAKE3_HEX_LENGTH,
        });
    }
    if !hex.bytes().all(is_lower_hex_byte) {
        return Err(IdentityError::InvalidHex);
    }
    Ok(())
}

fn is_lower_hex_byte(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

pub fn compute_identity(
    domain: ArtifactDomain,
    material: &[u8],
) -> Result<ArtifactIdentity, IdentityError> {
    if material.is_empty() {
        return Err(IdentityError::EmptyMaterial { domain });
    }
    let domain_label = domain.label().as_bytes();
    let domain_length = u64::try_from(domain_label.len())
        .map_err(|_| IdentityError::MaterialLengthOverflow { domain })?;
    let mut hasher = blake3::Hasher::new_derive_key(IDENTITY_CONTEXT);
    hasher.update(&domain_length.to_le_bytes()[..DOMAIN_LENGTH_BYTES]);
    hasher.update(domain_label);
    hasher.update(material);
    Ok(ArtifactIdentity {
        domain,
        algorithm: String::from(BLAKE3_ALGORITHM),
        hex: hasher.finalize().to_hex().to_string(),
    })
}

#[must_use]
pub fn compute_kamacite_identity(
    domain: KamaciteIdentityDomain,
    material: &[u8],
) -> KamaciteIdentity {
    KamaciteIdentity {
        domain,
        algorithm: String::from(BLAKE3_ALGORITHM),
        hex: blake3::hash(material).to_hex().to_string(),
    }
}
