# Compatibility source

This crate preserves the wire encodings, ordering, diagnostic messages and decision behavior from `execution-graph-core` revision `03736f1ec46c377ff86b451260ad68aa70ff3b0b`. Rust APIs now live in `structure`, `admission`, `aggregate`, `lifecycle` and `queries` modules without root aliases. Topology types use `Node`, `Edge`, `Topology`, `Group` and `Bottleneck`; admission diagnostics use `DiagnosticCode` and `Diagnostic`; the admitted plan is `aggregate::Plan` and validation is `admission::validate`.

The frozen source repository has RID `rad:z2oYsb9jGTyp68BKYhzpivY1eK58a`. Active compatibility development now belongs to Choregraph.

The facade converts compatibility values and delegates all graph decisions to `choregraph-core`. It does not schedule work, execute effects, allocate resources, persist runs, or grant authority.
