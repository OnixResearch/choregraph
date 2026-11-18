// Artifact observations and exact digest representation.
use alloc::string::{String, ToString};

use crate::profile::Role;

pub(crate) const DIGEST_BYTES: usize = 32;
const HEX_CHARACTERS_PER_BYTE: usize = 2;
pub(crate) const DIGEST_HEX_BYTES: usize = 64;
const NIBBLE_SHIFT_BITS: u32 = 4;
const HEX_ALPHA_OFFSET: u8 = 10;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ArtifactStatus {
    Observed,
    Canonical,
    Verified,
    Passed,
}

impl ArtifactStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::Canonical => "canonical",
            Self::Verified => "verified",
            Self::Passed => "passed",
        }
    }
}

fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => byte
            .checked_sub(b'a')
            .and_then(|value| value.checked_add(HEX_ALPHA_OFFSET)),
        _ => None,
    }
}

pub(crate) fn decode_digest_hex(value: &str) -> Option<[u8; DIGEST_BYTES]> {
    if value.len() != DIGEST_HEX_BYTES {
        return None;
    }
    let mut decoded = [0u8; DIGEST_BYTES];
    for (index, pair) in value
        .as_bytes()
        .chunks_exact(HEX_CHARACTERS_PER_BYTE)
        .enumerate()
    {
        let high = decode_hex_nibble(pair[0])?;
        let low = decode_hex_nibble(pair[1])?;
        decoded[index] = high.checked_shl(NIBBLE_SHIFT_BITS)? | low;
    }
    Some(decoded)
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArtifactDigest([u8; DIGEST_BYTES]);

impl ArtifactDigest {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; DIGEST_BYTES]) -> Self {
        Self(bytes)
    }

    #[must_use]
    #[inline(always)]
    pub const fn bytes(self) -> [u8; DIGEST_BYTES] {
        self.0
    }

    #[must_use]
    pub fn from_hex(value: &str) -> Option<Self> {
        decode_digest_hex(value).map(Self)
    }

    #[must_use]
    pub fn hex(self) -> String {
        blake3::Hash::from_bytes(self.0).to_hex().to_string()
    }
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArtifactObservation {
    pub role: Role,
    pub relative_path: String,
    pub schema: String,
    pub status: ArtifactStatus,
    pub digest: ArtifactDigest,
}
