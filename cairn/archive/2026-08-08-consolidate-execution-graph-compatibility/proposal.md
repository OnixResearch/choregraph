## Why

The published `execution-graph-core` package is now a conversion-only facade over Choregraph. Separate active repositories force lockstep source updates, validation, and release coordination for one implementation.

Choregraph must own active facade development. The original repository must remain a frozen compatibility and rollback source until consumers adopt the Choregraph package.

## What Changes

- Add `execution-graph-core` to the Choregraph workspace as a conversion-only compatibility crate. r[choregraph.execution_graph_compatibility.api]
- Delegate every graph and lifecycle decision to `choregraph-core`. r[choregraph.execution_graph_compatibility.delegation]
- Preserve the public compatibility types and explicit non-claims. r[choregraph.execution_graph_compatibility.boundary]
- Preserve positive and negative compatibility tests in the active source repository. r[choregraph.execution_graph_compatibility.validation]
- Record the frozen source RID and reviewed revision without using it as a production dependency. r[choregraph.execution_graph_compatibility.source]

## Success Criteria

- Choregraph builds and audits the compatibility crate in its normal workspace.
- The crate has no independent graph decision algorithm.
- Existing valid and invalid compatibility cases retain their outcomes.
- The original repository remains available as a frozen source and rollback point.

## False Completion

Copying the old independent algorithm into Choregraph is not completion. Keeping active facade development in both repositories is not completion.

## Out of Scope

This change does not publish a new Choregraph revision. It does not change the old RID, downstream pins, runtime authority, or graph semantics.
