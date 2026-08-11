## ADDED Requirements

### Requirement: Choregraph owns active compatibility source

r[choregraph.execution_graph_compatibility.source]

Choregraph MUST own the active `execution-graph-core` package source while the original repository remains a frozen compatibility and rollback source.

#### Scenario: Active source is selected

- **GIVEN** a Choregraph workspace build
- **WHEN** Cargo resolves `execution-graph-core`
- **THEN** it selects the workspace crate without a production dependency on the frozen repository

### Requirement: The compatibility API remains available

r[choregraph.execution_graph_compatibility.api]

The compatibility crate MUST preserve graph admission, query, readiness, lifecycle, resume, cycle-probe, and analytics entry points for admitted compatibility inputs.

#### Scenario: A valid graph uses the compatibility API

- **GIVEN** a valid diamond graph
- **WHEN** a consumer admits and queries the graph
- **THEN** the facade returns stable order, groups, dependencies, and analytics

### Requirement: Decisions delegate to Choregraph

r[choregraph.execution_graph_compatibility.delegation]

The compatibility crate MUST convert values and delegate every graph or lifecycle decision to `choregraph-core` without an independent decision algorithm.

#### Scenario: A lifecycle transition is requested

- **GIVEN** a compatibility state and action
- **WHEN** the facade evaluates the transition
- **THEN** it returns the result from the Choregraph lifecycle relation

### Requirement: Compatibility failures remain fail closed

r[choregraph.execution_graph_compatibility.validation]

The compatibility crate MUST test valid inputs and reject malformed graphs, cycles, invalid transitions, terminal mutations, missing nodes, and inputs beyond selected bounds.

#### Scenario: A malformed graph is supplied

- **GIVEN** duplicate nodes, missing endpoints, and a self-loop
- **WHEN** a consumer requests admission
- **THEN** the facade returns deterministic diagnostics and no admitted plan

### Requirement: Product authority stays outside the facade

r[choregraph.execution_graph_compatibility.boundary]

The compatibility crate MUST NOT execute effects, schedule resources, read clocks, persist runs, issue receipts, or grant product authority.

#### Scenario: A consumer requests a graph decision

- **GIVEN** an admitted compatibility graph
- **WHEN** the facade returns a decision
- **THEN** the result contains no execution, persistence, policy, or authority effect
