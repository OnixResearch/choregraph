# Epoch-Fenced Single-Writer Specification

## Purpose

Define a single-writer, epoch-fenced entity-ownership invariant for Choregraph machine semantics.

## ADDED Requirements

### Requirement: One writer per entity, fenced by an epoch

r[choregraph.epoch_fence.invariant]
The epoch-fenced transition API MUST model one writer for one entity. A request MUST carry the caller-supplied current entity epoch to reach the production mutation transition.

#### Scenario: Writer holds the current epoch

- GIVEN a request carries the supplied current entity epoch
- WHEN `step_epoch_fenced` receives a mutation for that entity
- THEN the fence MUST admit the supplied ownership facts and delegate to the production transition
- AND the decision MUST stay scoped to that entity

#### Scenario: Writer holds a stale epoch

- GIVEN a request carries an epoch that differs from the supplied current entity epoch
- WHEN `step_epoch_fenced` receives a mutation for that entity
- THEN the fence MUST reject the mutation before the production transition
- AND it MUST return no replacement state or effect intent

### Requirement: The core decides from supplied epochs

r[choregraph.epoch_fence.decision]
The pure core MUST return admit or reject from the current and offered entity epochs, exact entity match, admitted-entity membership, and terminal-state fact. It MUST read no storage, clock, or network.

#### Scenario: Mutation epoch matches

- GIVEN a mutation carries the current entity epoch
- WHEN the core decides
- THEN the disposition MUST be admit within one entity

#### Scenario: Mutation epoch does not match

- GIVEN a mutation carries a stale or mismatched epoch
- WHEN the core decides
- THEN the disposition MUST be reject
- AND no cross-entity write MUST occur

### Requirement: Production and proof share one implementation

r[choregraph.epoch_fence.proof]
The fence decision MUST use one implementation for the production wrapper and proof targets. The wrapper MUST delegate admitted requests to the existing production `step` implementation, and Verus proofs MUST import the production decision source.

#### Scenario: Proof covers the transition

- GIVEN a production transition encodes the epoch-fenced invariant
- WHEN the Verus specification and proof evaluate it
- THEN the proof MUST hold over the same decision implementation

#### Scenario: Implementation diverges

- GIVEN production and proof targets use different decision implementations
- WHEN the focused foundation rail runs
- THEN the rail MUST fail
- AND Choregraph MUST NOT claim a shared single implementation

### Requirement: Failure coverage remains explicit

r[choregraph.epoch_fence.verification]
Positive and negative fixtures MUST cover every declared invariant and boundary. The invariant MUST NOT be presented as a proof of current-epoch truth, runtime writer exclusivity, storage isolation, or deployment authority.

#### Scenario: Complete focused matrix passes

- GIVEN invariant, decision, proof, fixtures, and documentation are complete
- WHEN focused foundation, package, workspace, Clippy, Cairn, and Nix verification runs
- THEN current-epoch inputs MUST admit and stale, mismatched, terminal, and cross-entity inputs MUST reject
- AND any runtime-authority or arbitrary-correctness claim MUST fail as declared
