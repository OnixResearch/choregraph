# Repair validation baselines

## Why

The durable-suspension proof API added suspension and profile arguments, but four negative Verus fixtures still use the old signature. The repository-owned Cairn policy also predates the current workflow and policy fields.

## What Changes

- Update negative trace fixtures without weakening their intended failures.
- Rebase the Cairn policy contract on a pinned Cairn source snapshot.
- Keep Choregraph-specific policy fields as explicit forced overrides.
- Refresh generated policy and validation evidence.
- Restore strict formatting and Octet quality checks for the durable-suspension and compatibility surfaces.

## Impact

Full proof, policy, formatting, and Octet checks can evaluate the current source instead of failing on stale inputs.

## Out of Scope

This change does not migrate the repository from `cairn/` to `.cairn/`, alter runtime graph semantics, or grant release authority.
