/// Number of bytes in each project-owned BLAKE3 identity.
pub const DIGEST_BYTES: usize = 32;
const HEX_CHARACTERS_PER_BYTE: usize = 2;
const HEX_CAPACITY: usize = DIGEST_BYTES.saturating_mul(HEX_CHARACTERS_PER_BYTE);
const LENGTH_U64_TAG: u8 = 0;
const LENGTH_NATIVE_TAG: u8 = 1;

/// One exact BLAKE3 identity.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Digest([u8; DIGEST_BYTES]);

impl Digest {
    /// Creates an identity from exact digest bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; DIGEST_BYTES]) -> Self {
        Self(bytes)
    }

    /// Hashes one explicitly framed domain and byte sequence.
    #[must_use]
    pub fn derive(domain: &[u8], bytes: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        update_bytes(&mut hasher, domain);
        update_bytes(&mut hasher, bytes);
        Self(*hasher.finalize().as_bytes())
    }

    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn bytes(self) -> [u8; DIGEST_BYTES] {
        self.0
    }

    /// Returns lowercase hexadecimal text.
    #[must_use]
    pub fn to_hex(self) -> alloc::string::String {
        let mut result = alloc::string::String::with_capacity(HEX_CAPACITY);
        for byte in self.0 {
            let write_result =
                core::fmt::Write::write_fmt(&mut result, core::format_args!("{byte:02x}"));
            assert!(write_result.is_ok(), "writing to a String must succeed");
        }
        result
    }
}

impl core::fmt::Debug for Digest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_tuple("Digest")
            .field(&self.to_hex())
            .finish()
    }
}

impl core::fmt::Display for Digest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

/// Identity of one immutable history event.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventId(Digest);

impl EventId {
    /// Creates an event identity from a digest.
    #[must_use]
    pub const fn new(digest: Digest) -> Self {
        Self(digest)
    }

    /// Returns the underlying digest.
    #[must_use]
    pub const fn digest(self) -> Digest {
        self.0
    }

    /// Returns exact identity bytes.
    #[must_use]
    pub const fn bytes(self) -> [u8; DIGEST_BYTES] {
        self.0.bytes()
    }
}

impl core::fmt::Display for EventId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, formatter)
    }
}

pub(crate) fn update_length(hasher: &mut blake3::Hasher, length: usize) {
    match u64::try_from(length) {
        Ok(framed) => {
            hasher.update(&[LENGTH_U64_TAG]);
            hasher.update(&framed.to_le_bytes());
        }
        Err(_) => {
            hasher.update(&[LENGTH_NATIVE_TAG]);
            hasher.update(&length.to_le_bytes());
        }
    }
}

pub(crate) fn update_count(hasher: &mut blake3::Hasher, count: u32) {
    hasher.update(&[LENGTH_U64_TAG]);
    hasher.update(&u64::from(count).to_le_bytes());
}

pub(crate) fn update_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    update_length(hasher, bytes.len());
    hasher.update(bytes);
}

pub(crate) fn update_digest(hasher: &mut blake3::Hasher, digest: Digest) {
    hasher.update(&digest.bytes());
}

pub(crate) fn count_from_length(length: usize) -> Option<u32> {
    u32::try_from(length).ok()
}
