const MAX_EVENTS: u32 = 8;
const MAX_PARENTS: u32 = 2;
const MAX_METADATA: u32 = 2;
const MAX_LABEL: u32 = 32;
const MAX_PROJECTION: u32 = 8;
const PAYLOAD_SEED: u8 = 81;
const SCHEMA_SEED: u8 = 82;

fn event() -> choregraph_history::EventDraft {
    choregraph_history::EventDraft::v1(
        choregraph_history::EventClass::Application,
        choregraph_history::Digest::from_bytes([PAYLOAD_SEED; choregraph_history::DIGEST_BYTES]),
        std::vec::Vec::new(),
        choregraph_history::Digest::from_bytes([SCHEMA_SEED; choregraph_history::DIGEST_BYTES]),
    )
}

fn admitted_graph() -> Result<choregraph_history::HistoryGraph, choregraph_history::HistoryError> {
    let policy = choregraph_history::HistoryLimits::new(
        MAX_EVENTS,
        MAX_PARENTS,
        MAX_METADATA,
        MAX_LABEL,
        MAX_PROJECTION,
    )?;
    let graph = choregraph_history::HistoryGraph::new(policy)?;
    Ok(graph.admit(event())?.graph)
}

fn main() -> Result<(), choregraph_history::HistoryError> {
    let graph = admitted_graph()?;
    println!("{}", graph.identity());
    Ok(())
}
