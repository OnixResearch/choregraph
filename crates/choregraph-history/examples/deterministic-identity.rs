const MAX_EVENTS: u32 = 8;
const MAX_PARENTS: u32 = 2;
const MAX_METADATA: u32 = 2;
const MAX_LABEL: u32 = 32;
const MAX_PROJECTION: u32 = 8;
const PAYLOAD_SEED: u8 = 81;
const SCHEMA_SEED: u8 = 82;

fn event() -> choregraph_history::model::EventDraft {
    choregraph_history::model::EventDraft::v1(
        choregraph_history::model::EventClass::Application,
        choregraph_history::identity::Digest::from_bytes(
            [PAYLOAD_SEED; choregraph_history::identity::DIGEST_BYTES],
        ),
        std::vec::Vec::new(),
        choregraph_history::identity::Digest::from_bytes(
            [SCHEMA_SEED; choregraph_history::identity::DIGEST_BYTES],
        ),
    )
}

fn admitted_graph() -> Result<choregraph_history::dag::Graph, choregraph_history::model::Error> {
    let policy = choregraph_history::model::Limits {
        max_events: MAX_EVENTS,
        max_parents: MAX_PARENTS,
        max_metadata_entries: MAX_METADATA,
        max_label_bytes: MAX_LABEL,
        max_projection_events: MAX_PROJECTION,
    };
    let graph = choregraph_history::dag::Graph::new(policy)?;
    Ok(graph.admit(event())?.graph)
}

fn main() -> Result<(), choregraph_history::model::Error> {
    let graph = admitted_graph()?;
    println!("{}", graph.identity());
    Ok(())
}
