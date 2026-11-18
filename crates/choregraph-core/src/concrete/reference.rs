use crate::id::bounded::{SlotId, SourceRef};
use crate::id::reference::PayloadRef;
use alloc::string::String;

pub const CANONICAL_REFERENCE_PROFILE_V1: &str = "choregraph.packed-preserves.v1";

/// The closed canonical encoding profile accepted by the pure core.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalReferenceProfile {
    PackedPreservesV1,
}

impl CanonicalReferenceProfile {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PackedPreservesV1 => CANONICAL_REFERENCE_PROFILE_V1,
        }
    }

    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            CANONICAL_REFERENCE_PROFILE_V1 => Some(Self::PackedPreservesV1),
            _ => None,
        }
    }
}

/// One untrusted canonical reference supplied by a schema or event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteReferenceSpec {
    pub profile: String,
    pub identity: String,
}

/// One admitted canonical packed Preserves reference.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConcreteReference {
    pub profile: CanonicalReferenceProfile,
    pub identity: PayloadRef,
}

/// One untrusted exact or runtime-slot binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceBindingSpec {
    Exact { reference: ConcreteReferenceSpec },
    Slot { slot_id: String, source_ref: String },
}

/// One admitted exact or runtime-slot binding.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReferenceBinding {
    Exact(ConcreteReference),
    Slot {
        slot_id: SlotId,
        source_ref: SourceRef,
    },
}

/// One concrete reference and its selected slot facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteBindingInput {
    pub slot_id: Option<String>,
    pub source_ref: Option<String>,
    pub reference: ConcreteReferenceSpec,
}
