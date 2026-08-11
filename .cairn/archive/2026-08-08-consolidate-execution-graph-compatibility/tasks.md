## Implementation

- [x] [serial] I1 Add the active compatibility crate to the Choregraph workspace. r[choregraph.execution_graph_compatibility.api]
- [x] [serial] I2 Route every compatibility decision through `choregraph-core`. r[choregraph.execution_graph_compatibility.delegation]
- [x] [parallel] I3 Preserve source identity and explicit authority boundaries. r[choregraph.execution_graph_compatibility.source] r[choregraph.execution_graph_compatibility.boundary]
- [x] [parallel] I4 Preserve positive and negative API tests in the active crate. r[choregraph.execution_graph_compatibility.validation]

## Closeout

- [x] [serial] C1 Run focused Cargo, Clippy, no-std, foundation, Cairn, and Nix validation. r[choregraph.execution_graph_compatibility.validation]
- [x] [serial] C2 Sync the accepted requirement and archive this change after all evidence passes. r[choregraph.execution_graph_compatibility.source] r[choregraph.execution_graph_compatibility.boundary]
