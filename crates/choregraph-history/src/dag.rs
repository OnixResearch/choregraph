// r[impl choregraph.history.event_dag]
// r[impl choregraph.history.event_dag.append]
// r[impl choregraph.history.event_dag.rejects]
// r[impl choregraph.history.merge]
// r[impl choregraph.history.merge.valid]
// r[impl choregraph.history.merge.rejects_implicit]
const GRAPH_IDENTITY_DOMAIN: &[u8] = b"choregraph.history.graph.v1";
const ADJACENT_FIELD_COUNT: usize = 2;

/// One successful immutable graph extension.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Admission {
    /// New graph value. The input graph remains unchanged.
    pub graph: HistoryGraph,
    /// Identity of the admitted event.
    pub event: crate::identity::EventId,
}

/// An immutable admitted event DAG.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryGraph {
    limits: crate::model::HistoryLimits,
    event_count: u32,
    events: alloc::collections::BTreeMap<crate::identity::EventId, crate::model::HistoryEvent>,
}

impl HistoryGraph {
    /// Creates an empty graph under explicit positive limits.
    pub fn new(limits: crate::model::HistoryLimits) -> Result<Self, crate::model::HistoryError> {
        limits.validate()?;
        Ok(Self {
            limits,
            event_count: 0,
            events: alloc::collections::BTreeMap::new(),
        })
    }

    /// Returns the selected limits.
    #[must_use]
    pub const fn limits(&self) -> crate::model::HistoryLimits {
        self.limits
    }

    /// Returns the number of admitted events.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.event_count
    }

    /// Returns true when no events are admitted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Returns one immutable event.
    #[must_use]
    pub fn event(&self, identity: crate::identity::EventId) -> Option<&crate::model::HistoryEvent> {
        self.events.get(&identity)
    }

    /// Returns true when the identity exists.
    #[must_use]
    pub fn contains(&self, identity: crate::identity::EventId) -> bool {
        self.events.contains_key(&identity)
    }

    /// Iterates events in identity order.
    pub fn events(
        &self,
    ) -> impl Iterator<Item = (&crate::identity::EventId, &crate::model::HistoryEvent)> {
        self.events.iter()
    }

    /// Returns the canonical identity expected for a draft.
    #[must_use]
    pub fn expected_identity(draft: &crate::model::EventDraft) -> crate::identity::EventId {
        crate::model::event_identity(draft)
    }

    /// Admits one canonical draft and returns a new graph value.
    pub fn admit(
        &self,
        draft: crate::model::EventDraft,
    ) -> Result<Admission, crate::model::HistoryError> {
        let claimed = crate::model::event_identity(&draft);
        self.admit_claimed(claimed, draft)
    }

    /// Admits one application-supplied merge resolution.
    pub fn admit_merge(
        &self,
        envelope: crate::model::MergeEnvelope,
    ) -> Result<Admission, crate::model::HistoryError> {
        self.admit(envelope.into_draft()?)
    }

    /// Admits one event with an explicit claimed identity.
    ///
    /// This entrypoint supports canonical envelope validation. A duplicate claim
    /// with different content fails before identity recomputation.
    pub fn admit_claimed(
        &self,
        claimed: crate::identity::EventId,
        draft: crate::model::EventDraft,
    ) -> Result<Admission, crate::model::HistoryError> {
        self.validate_draft(claimed, &draft)?;
        if let Some(existing) = self.events.get(&claimed) {
            if existing.draft == draft {
                return Err(crate::model::HistoryError::NoChange);
            }
            return Err(crate::model::HistoryError::ConflictingDuplicate { event: claimed });
        }
        if self.event_count >= self.limits.max_events {
            return Err(crate::model::HistoryError::EventLimitExceeded {
                maximum: self.limits.max_events,
            });
        }
        let next_event_count =
            self.event_count
                .checked_add(1)
                .ok_or(crate::model::HistoryError::CountOverflow {
                    field: "graph_events",
                })?;
        let expected = crate::model::event_identity(&draft);
        if claimed != expected {
            return Err(crate::model::HistoryError::IdentityMismatch { claimed, expected });
        }
        let mut events = self.events.clone();
        let replaced = events.insert(
            claimed,
            crate::model::HistoryEvent {
                identity: claimed,
                draft,
            },
        );
        debug_assert!(replaced.is_none());
        debug_assert!(events.len() == self.events.len().saturating_add(1));
        Ok(Admission {
            graph: Self {
                limits: self.limits,
                event_count: next_event_count,
                events,
            },
            event: claimed,
        })
    }

    /// Returns a domain-separated identity over all admitted event identities.
    #[must_use]
    pub fn identity(&self) -> crate::identity::Digest {
        let mut hasher = blake3::Hasher::new();
        crate::identity::update_bytes(&mut hasher, GRAPH_IDENTITY_DOMAIN);
        crate::identity::update_count(&mut hasher, self.event_count);
        for identity in self.events.keys() {
            hasher.update(&identity.bytes());
        }
        crate::identity::Digest::from_bytes(*hasher.finalize().as_bytes())
    }

    fn validate_draft(
        &self,
        claimed: crate::identity::EventId,
        draft: &crate::model::EventDraft,
    ) -> Result<(), crate::model::HistoryError> {
        if draft.version != crate::HISTORY_EVENT_VERSION_V1 {
            return Err(crate::model::HistoryError::UnsupportedVersion {
                version: draft.version,
            });
        }
        let parent_count = crate::identity::count_from_length(draft.parents.len())
            .ok_or(crate::model::HistoryError::CountOverflow { field: "parents" })?;
        if draft.class == crate::EventClass::Merge {
            if parent_count < crate::model::MINIMUM_MERGE_PARENTS {
                return Err(crate::model::HistoryError::MergeNeedsMultipleParents);
            }
            if draft.projection_identity.is_none() {
                return Err(crate::model::HistoryError::MissingMergePolicy);
            }
        }
        if parent_count > self.limits.max_parents {
            return Err(crate::model::HistoryError::ParentLimitExceeded {
                observed: parent_count,
                maximum: self.limits.max_parents,
            });
        }
        let metadata_count = crate::identity::count_from_length(draft.metadata.len())
            .ok_or(crate::model::HistoryError::CountOverflow { field: "metadata" })?;
        if metadata_count > self.limits.max_metadata_entries {
            return Err(crate::model::HistoryError::MetadataLimitExceeded {
                observed: metadata_count,
                maximum: self.limits.max_metadata_entries,
            });
        }
        self.validate_ordered_fields(draft)?;
        for parent in &draft.parents {
            if parent.event == claimed {
                return Err(crate::model::HistoryError::Cycle { event: claimed });
            }
            if !self.events.contains_key(&parent.event) {
                return Err(crate::model::HistoryError::MissingParent {
                    event: parent.event,
                });
            }
        }
        Ok(())
    }

    fn validate_ordered_fields(
        &self,
        draft: &crate::model::EventDraft,
    ) -> Result<(), crate::model::HistoryError> {
        for parent in &draft.parents {
            self.validate_label("parent_role", &parent.role)?;
        }
        let mut parent_ids = alloc::collections::BTreeSet::new();
        for parent in &draft.parents {
            if !parent_ids.insert(parent.event) {
                return Err(crate::model::HistoryError::DuplicateParent {
                    event: parent.event,
                });
            }
        }
        if !draft
            .parents
            .windows(ADJACENT_FIELD_COUNT)
            .all(|pair| pair[0] < pair[1])
        {
            return Err(crate::model::HistoryError::NonCanonicalParents);
        }
        let mut metadata_keys = alloc::collections::BTreeSet::new();
        for metadata in &draft.metadata {
            self.validate_label("metadata_key", &metadata.key)?;
            if !metadata_keys.insert(metadata.key.as_str()) {
                return Err(crate::model::HistoryError::NonCanonicalMetadata);
            }
        }
        if !draft
            .metadata
            .windows(ADJACENT_FIELD_COUNT)
            .all(|pair| pair[0] < pair[1])
        {
            return Err(crate::model::HistoryError::NonCanonicalMetadata);
        }
        Ok(())
    }

    fn validate_label(
        &self,
        field: &'static str,
        value: &str,
    ) -> Result<(), crate::model::HistoryError> {
        let observed = crate::identity::count_from_length(value.len())
            .ok_or(crate::model::HistoryError::CountOverflow { field })?;
        if value.is_empty() || observed > self.limits.max_label_bytes {
            return Err(crate::model::HistoryError::LabelTooLong {
                field,
                observed,
                maximum: self.limits.max_label_bytes,
            });
        }
        Ok(())
    }
}
