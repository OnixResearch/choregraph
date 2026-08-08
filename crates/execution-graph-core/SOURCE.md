# Compatibility source

This crate preserves the public API from `execution-graph-core` revision `03736f1ec46c377ff86b451260ad68aa70ff3b0b`.

The frozen source repository has RID `rad:z2oYsb9jGTyp68BKYhzpivY1eK58a`. Active compatibility development now belongs to Choregraph.

The facade converts compatibility values and delegates all graph decisions to `choregraph-core`. It does not schedule work, execute effects, allocate resources, persist runs, or grant authority.
