# Lattice Nickel authoring profile

The first consumer profile was reviewed at Lattice revision `c8d56ad83a57287064edb85f58808f53e4bf4536`.

## Current Lattice shape

Lattice workflow Nickel files contain these relevant values:

- numeric node identifiers;
- numeric input and output port identifiers;
- edges from an output port to an input port;
- one runtime configuration per node;
- product fields for labels, coordinates, commands, environment references, HTTP data, file paths, agents, retries, triggers, and output policy.

Lattice edges do not directly contain node identifiers. The lowerer must resolve each port to its owning node before it creates a Choregraph edge.

The reviewed Lattice evaluator uses `nickel-lang-core` `0.17.0`. Its current file loader can include host paths in raw errors. Choregraph does not adopt that loader as a safe diagnostic or capability boundary.

## Selected projection

`choregraph.lattice-workflow-projection.v1` admits only the topology required by the first compatibility corpus:

- `nodes[].id`;
- `nodes[].ports[].id` and direction;
- `edges[].source` and `edges[].target` port identifiers;
- `configs[].node_id` and a bounded operation identifier;
- the fixed lifecycle event set;
- explicit machine limits;
- declared content-addressed imports.

The lowerer converts node `N` to `lattice-node/N`. It converts operation `K` to `lattice/K`. It rejects duplicate ports, missing ports, wrong edge direction, missing or duplicate node configuration, cycles, and normal machine-admission failures.

Labels, coordinates, display metadata, commands, credentials, environment values, URLs, bodies, host paths, triggers, and runtime policy do not enter canonical topology identity in this profile.

## Fixtures

The maintained diamond fixture is [`../fixtures/nickel/positive/lattice-diamond.ncl`](../fixtures/nickel/positive/lattice-diamond.ncl). It models one root, two parallel branches, and one join.

Nickel exports are normalized under `fixtures/nickel/evaluated/`. Rust tests lower those exact evaluated bytes. Nix rebuilds each export and compares it with the checked-in value.

## Claim boundary

The projection records selected evaluated data. It does not prove that general Lattice workflow source has equivalent meaning. Lattice retains evaluator, product, effect, scheduling, persistence, trigger, credential, and runtime authority.
