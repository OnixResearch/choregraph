# Source provenance

The source baseline is in [`../evidence/source-baseline.ncl`](../evidence/source-baseline.ncl). Its generated JSON and BLAKE3 identity are checked in.

Each tree identity uses this method:

1. Read files from the recorded committed revision and source scopes.
2. Sort each relative path with the C locale.
3. Calculate the BLAKE3 value of each file.
4. Write one line with the relative path and file value.
5. Calculate the BLAKE3 value of the complete manifest.

The Seaglass upstream entry uses the BLAKE3 value of the source file. Execution-graph provenance evidence maps that file to its upstream revision.

The foundation imported no implementation or proof source. The Kamacite change vendors only the selected canonical codec and its license files from the exact recorded revision.

The public `preserves` dependency is pinned to the selected Kamacite profile version. `tools/sync-kamacite-codec.rs` reproduces the import from the recorded Git object.

The authenticated Kamacite repository is not a Cargo release input. All identities exclude sibling working-tree changes. No ambient sibling path is a release input.

A source identity does not prove correctness. A Trellis proof reference does not transfer a proof claim to Choregraph.

The cap-root entry has no durable release location yet. Its status is reference-only, and Choregraph cannot use it as a release input.
