use crate::id::reference::BLAKE3_REFERENCE_BYTES;

pub const DEFAULT_MAX_NODES: usize = 4_096;
pub const DEFAULT_MAX_EDGES: usize = 16_384;
pub const DEFAULT_MAX_OPERATIONS: usize = 16_384;
pub const DEFAULT_MAX_EFFECT_DECLARATIONS: usize = 16_384;
pub const DEFAULT_MAX_IDENTIFIER_BYTES: usize = 256;
pub const DEFAULT_MAX_REFERENCE_BYTES: usize = BLAKE3_REFERENCE_BYTES;
pub const DEFAULT_MAX_EVENT_BYTES: usize = 65_536;
pub const DEFAULT_MAX_INTENT_BYTES: usize = 2_048;
pub const DEFAULT_MAX_INTENTS_PER_TRANSITION: usize = 64;
pub const DEFAULT_MAX_TRACE_EVENTS: u64 = 65_536;

/// Bounded admission and transition limits selected by the caller.
///
/// Suspension classes share `max_identifier_bytes` (a class is a bounded
/// identifier) and suspension references share `max_reference_bytes` (a
/// reference is a canonical BLAKE3 reference). No separate suspension limit
/// exists, so v1 limit descriptors stay byte-identical.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmissionLimits {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub max_operations: usize,
    pub max_effect_declarations: usize,
    pub max_identifier_bytes: usize,
    pub max_reference_bytes: usize,
    pub max_event_bytes: usize,
    pub max_intent_bytes: usize,
    pub max_intents_per_transition: usize,
}

impl Default for AdmissionLimits {
    fn default() -> Self {
        Self {
            max_nodes: DEFAULT_MAX_NODES,
            max_edges: DEFAULT_MAX_EDGES,
            max_operations: DEFAULT_MAX_OPERATIONS,
            max_effect_declarations: DEFAULT_MAX_EFFECT_DECLARATIONS,
            max_identifier_bytes: DEFAULT_MAX_IDENTIFIER_BYTES,
            max_reference_bytes: DEFAULT_MAX_REFERENCE_BYTES,
            max_event_bytes: DEFAULT_MAX_EVENT_BYTES,
            max_intent_bytes: DEFAULT_MAX_INTENT_BYTES,
            max_intents_per_transition: DEFAULT_MAX_INTENTS_PER_TRANSITION,
        }
    }
}
