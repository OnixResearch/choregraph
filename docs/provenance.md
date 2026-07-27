# Source provenance

The source baseline is in [`../evidence/source-baseline.ncl`](../evidence/source-baseline.ncl). Its generated JSON and BLAKE3 identity are checked in.

Each tree identity uses this method:

1. Read files from the recorded committed revision and source scopes.
2. Sort each relative path with the C locale.
3. Calculate the BLAKE3 value of each file.
4. Write one line with the relative path and file value.
5. Calculate the BLAKE3 value of the complete manifest.

The Seaglass upstream entry uses the BLAKE3 value of the source file. Execution-graph provenance evidence maps that file to its upstream revision.

The foundation imports no implementation or proof source. All entries have `imported = false` and `working_tree_used = false`.

A source identity does not prove correctness. A Trellis proof reference does not transfer a proof claim to Choregraph.

The cap-root entry has no durable release location yet. Its status is reference-only, and Choregraph cannot use it as a release input.
