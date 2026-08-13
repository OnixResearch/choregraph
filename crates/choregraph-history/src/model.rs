/// First canonical history event version.
pub const HISTORY_EVENT_VERSION_V1: u16 = 1;
/// A merge must have at least two distinct parents.
pub const MINIMUM_MERGE_PARENTS: u32 = 2;
const EVENT_IDENTITY_DOMAIN: &[u8] = b"choregraph.history.event.v1";
const LIMITS_IDENTITY_DOMAIN: &[u8] = b"choregraph.history.limits.v1";
const APPLICATION_EVENT_TAG: u8 = 0;
const MACHINE_V1_EVENT_TAG: u8 = 1;
const MACHINE_V2_EVENT_TAG: u8 = 2;
const MERGE_EVENT_TAG: u8 = 3;

/// Generic structural class of one event.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EventClass {
    /// Opaque application event.
    Application,
    /// Aggregate v1 production-machine event.
    MachineV1,
    /// Concrete v2 production-machine event.
    MachineV2,
    /// Application-supplied merge resolution.
    Merge,
}

impl EventClass {
    pub(crate) const fn tag(self) -> u8 {
        if let Self::Application = self {
            APPLICATION_EVENT_TAG
        } else if let Self::MachineV1 = self {
            MACHINE_V1_EVENT_TAG
        } else if let Self::MachineV2 = self {
            MACHINE_V2_EVENT_TAG
        } else {
            debug_assert!(matches!(self, Self::Merge));
            MERGE_EVENT_TAG
        }
    }
}

/// One parent identity and its explicit application role.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ParentLink {
    /// Bounded role text. Canonical parent order uses this role, then the event identity.
    pub role: alloc::string::String,
    /// Immutable parent event.
    pub event: crate::identity::EventId,
}

/// One bounded safe metadata key and opaque value identity.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MetadataIdentity {
    /// Metadata key. Raw metadata values are not stored.
    pub key: alloc::string::String,
    /// Identity of the consumer-owned metadata value.
    pub value: crate::identity::Digest,
}

/// Named bounds for one history graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoryLimits {
    /// Maximum admitted events.
    pub max_events: u32,
    /// Maximum parents for one event.
    pub max_parents: u32,
    /// Maximum safe metadata entries for one event.
    pub max_metadata_entries: u32,
    /// Maximum bytes in each role, key, or branch name.
    pub max_label_bytes: u32,
    /// Maximum events in one projection.
    pub max_projection_events: u32,
}

impl HistoryLimits {
    /// Creates a positive bounded policy.
    ///
    /// # Errors
    ///
    /// Returns [`HistoryError::InvalidLimits`] if a bound is zero.
    pub fn new(
        max_events: u32,
        max_parents: u32,
        max_metadata_entries: u32,
        max_label_bytes: u32,
        max_projection_events: u32,
    ) -> Result<Self, HistoryError> {
        let policy = Self {
            max_events,
            max_parents,
            max_metadata_entries,
            max_label_bytes,
            max_projection_events,
        };
        policy.validate()?;
        Ok(policy)
    }

    /// Validates all selected bounds.
    pub const fn validate(self) -> Result<(), HistoryError> {
        if self.max_events == 0 {
            return Err(HistoryError::InvalidLimits {
                field: "max_events",
            });
        }
        if self.max_parents == 0 {
            return Err(HistoryError::InvalidLimits {
                field: "max_parents",
            });
        }
        if self.max_metadata_entries == 0 {
            return Err(HistoryError::InvalidLimits {
                field: "max_metadata_entries",
            });
        }
        if self.max_label_bytes == 0 {
            return Err(HistoryError::InvalidLimits {
                field: "max_label_bytes",
            });
        }
        if self.max_projection_events == 0 {
            return Err(HistoryError::InvalidLimits {
                field: "max_projection_events",
            });
        }
        Ok(())
    }

    /// Returns the domain-separated policy identity.
    #[must_use]
    pub fn identity(self) -> crate::identity::Digest {
        let mut hasher = blake3::Hasher::new();
        crate::identity::update_bytes(&mut hasher, LIMITS_IDENTITY_DOMAIN);
        for bound_count in [
            self.max_events,
            self.max_parents,
            self.max_metadata_entries,
            self.max_label_bytes,
            self.max_projection_events,
        ] {
            crate::identity::update_count(&mut hasher, bound_count);
        }
        crate::identity::Digest::from_bytes(*hasher.finalize().as_bytes())
    }
}

/// Untrusted immutable event content before identity admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventDraft {
    /// Canonical event schema version.
    pub version: u16,
    /// Structural event class.
    pub class: EventClass,
    /// Opaque consumer-owned payload identity.
    pub payload_identity: crate::identity::Digest,
    /// Explicit canonical parent links.
    pub parents: alloc::vec::Vec<ParentLink>,
    /// Opaque schema identity for the payload class.
    pub schema_identity: crate::identity::Digest,
    /// Optional reducer or projection identity.
    pub projection_identity: Option<crate::identity::Digest>,
    /// Bounded safe metadata identities.
    pub metadata: alloc::vec::Vec<MetadataIdentity>,
}

impl EventDraft {
    /// Creates a v1 draft without metadata or projection identity.
    #[must_use]
    pub const fn v1(
        class: EventClass,
        payload_identity: crate::identity::Digest,
        parents: alloc::vec::Vec<ParentLink>,
        schema_identity: crate::identity::Digest,
    ) -> Self {
        Self {
            version: HISTORY_EVENT_VERSION_V1,
            class,
            payload_identity,
            parents,
            schema_identity,
            projection_identity: None,
            metadata: alloc::vec::Vec::new(),
        }
    }
}

/// One admitted immutable history event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryEvent {
    pub(crate) identity: crate::identity::EventId,
    pub(crate) draft: EventDraft,
}

impl HistoryEvent {
    /// Returns the immutable event identity.
    #[must_use]
    pub const fn identity(&self) -> crate::identity::EventId {
        self.identity
    }

    /// Returns the canonical event content.
    #[must_use]
    pub const fn content(&self) -> &EventDraft {
        &self.draft
    }
}

/// An application-supplied merge. Choregraph validates shape but not meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeEnvelope {
    /// Every concurrent parent and its explicit role.
    pub parents: alloc::vec::Vec<ParentLink>,
    /// Consumer-supplied resolution identity. Absence requests no automatic merge.
    pub resolution_identity: Option<crate::identity::Digest>,
    /// Schema of the supplied resolution.
    pub schema_identity: crate::identity::Digest,
    /// Exact application merge-policy identity.
    pub merge_policy_identity: crate::identity::Digest,
    /// Safe metadata identities only.
    pub metadata: alloc::vec::Vec<MetadataIdentity>,
}

impl MergeEnvelope {
    /// Converts a supplied resolution into an immutable merge event draft.
    pub fn into_draft(self) -> Result<EventDraft, HistoryError> {
        let parent_count = crate::identity::count_from_length(self.parents.len())
            .ok_or(HistoryError::CountOverflow { field: "parents" })?;
        if parent_count < MINIMUM_MERGE_PARENTS {
            return Err(HistoryError::MergeNeedsMultipleParents);
        }
        let resolution_identity = self
            .resolution_identity
            .ok_or(HistoryError::MissingMergeResolution)?;
        Ok(EventDraft {
            version: HISTORY_EVENT_VERSION_V1,
            class: EventClass::Merge,
            payload_identity: resolution_identity,
            parents: self.parents,
            schema_identity: self.schema_identity,
            projection_identity: Some(self.merge_policy_identity),
            metadata: self.metadata,
        })
    }
}

/// Typed fail-closed history errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryError {
    InvalidLimits {
        field: &'static str,
    },
    CountOverflow {
        field: &'static str,
    },
    UnsupportedVersion {
        version: u16,
    },
    EventLimitExceeded {
        maximum: u32,
    },
    ParentLimitExceeded {
        observed: u32,
        maximum: u32,
    },
    MetadataLimitExceeded {
        observed: u32,
        maximum: u32,
    },
    LabelTooLong {
        field: &'static str,
        observed: u32,
        maximum: u32,
    },
    NonCanonicalParents,
    NonCanonicalMetadata,
    DuplicateParent {
        event: crate::identity::EventId,
    },
    MissingParent {
        event: crate::identity::EventId,
    },
    Cycle {
        event: crate::identity::EventId,
    },
    IdentityMismatch {
        claimed: crate::identity::EventId,
        expected: crate::identity::EventId,
    },
    ConflictingDuplicate {
        event: crate::identity::EventId,
    },
    NoChange,
    HeadMissing {
        event: crate::identity::EventId,
    },
    ProjectionLimitExceeded {
        observed: u32,
        maximum: u32,
    },
    AmbiguousLineage {
        event: crate::identity::EventId,
    },
    InvalidParentChoice {
        event: crate::identity::EventId,
        selected: crate::identity::EventId,
    },
    InvalidBranchName,
    StaleGeneration {
        expected: u64,
        observed: u64,
    },
    StaleHead,
    GenerationOverflow,
    MissingMergeResolution,
    MissingMergePolicy,
    MergeNeedsMultipleParents,
    CursorMismatch,
    CursorOutOfBounds {
        position: u32,
        length: u32,
    },
}

pub(crate) fn event_identity(draft: &EventDraft) -> crate::identity::EventId {
    debug_assert!(!EVENT_IDENTITY_DOMAIN.is_empty());
    debug_assert_eq!(crate::identity::DIGEST_BYTES, blake3::OUT_LEN);
    let mut hasher = blake3::Hasher::new();
    crate::identity::update_bytes(&mut hasher, EVENT_IDENTITY_DOMAIN);
    hasher.update(&draft.version.to_le_bytes());
    hasher.update(&[draft.class.tag()]);
    crate::identity::update_digest(&mut hasher, draft.payload_identity);
    crate::identity::update_digest(&mut hasher, draft.schema_identity);
    match draft.projection_identity {
        Some(identity) => {
            hasher.update(&[1]);
            crate::identity::update_digest(&mut hasher, identity);
        }
        None => {
            hasher.update(&[0]);
        }
    }
    crate::identity::update_length(&mut hasher, draft.parents.len());
    for parent in &draft.parents {
        crate::identity::update_bytes(&mut hasher, parent.role.as_bytes());
        hasher.update(&parent.event.bytes());
    }
    crate::identity::update_length(&mut hasher, draft.metadata.len());
    for metadata in &draft.metadata {
        crate::identity::update_bytes(&mut hasher, metadata.key.as_bytes());
        crate::identity::update_digest(&mut hasher, metadata.value);
    }
    crate::identity::EventId::new(crate::identity::Digest::from_bytes(
        *hasher.finalize().as_bytes(),
    ))
}
