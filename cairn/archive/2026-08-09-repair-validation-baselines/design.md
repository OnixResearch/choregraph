# Design

## Decisions

- Treat production Verus signatures as authoritative for negative fixtures.
- Preserve each fixture's false postcondition so a successful verification remains a test failure.
- Vendor the current Cairn policy contract and default as a revision-bound source snapshot.
- Apply Choregraph-specific common fields with Nickel `force` priorities.
- Inherit new upstream policy fields without copying them into the project overlay.
- Split oversized proof and test modules at stable responsibility boundaries.
- Keep compatibility-only lint exceptions local, documented, and explicit.

## Rejected alternatives

- Do not edit generated JSON by hand.
- Do not replace Choregraph-specific evidence, traceability, or receipt settings with generic Cairn defaults.
- Do not migrate the lifecycle directory as part of a validation-baseline repair.

## Tests

Run positive production proofs, all negative proof fixtures, policy export freshness, invalid policy fixtures, current Cairn policy parsing, Cargo checks, Octet, and Nix checks.

## Risks and boundaries

A copied policy snapshot can become stale again. Provenance binds it to one source revision. The overlay preserves project policy but does not prove Cairn or Nickel correctness.
