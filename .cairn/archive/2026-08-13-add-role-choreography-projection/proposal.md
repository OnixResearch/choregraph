## Why

Choregraph lowers bounded Nickel execution graphs into verified machine artifacts. It does not offer a global role protocol that can be projected into one exact local protocol per participant.

Trellis already owns verified finite choreography, endpoint projection, branch merge, choice agreement, and step correspondence. The new authoring boundary must reuse that semantic source instead of adding another unproved projection pass.

Wyzer at revision `7f6acf0f60cf34357e06de50112eb2b1ed833a0c` is an MIT-licensed design reference for role-located values, explicit transfers, native and external roles, structured diagnostics, and negative choreography examples. The project operator reports direct MIT confirmation from the author. Suitable source can be reused after review, but Wyzer runtime, Perceus, compile-time execution, and C ABI code are outside this change.

## What Changes

- Add a typed Nickel role-choreography profile with abstract roles, role kinds, owned values, transfers, choices, and termination.
- Add strict Rust DTO revalidation with named limits and deterministic normalization.
- Add a pure `choregraph-choreography` adapter that lowers into the selected Trellis choreography model and uses its projectability and endpoint projection semantics.
- Emit one canonical global artifact and one canonical local protocol artifact for every declared role.
- Emit canonical Kamacite Preserves artifacts and a projection receipt that binds source, global, local, proof-reference, lowerer, schema, and toolchain identities.
- Add stable structured diagnostics with primary and secondary source-relative spans, notes, hints, and bounded safe fixes.
- Distinguish native roles from external roles without granting host, adapter, credential, transport, or effect authority.
- Add positive and negative source, lowering, projection, diagnostic, identity, and claim-boundary fixtures.

## Success Criteria

- The source profile cannot express an undeclared role, duplicate role, self-transfer, duplicate branch label, stale value owner, or open external contract.
- The pure adapter returns no local artifacts when projectability or owned-transfer admission fails.
- Every successful result contains exactly one local artifact for each declared role.
- Local sender, receiver, choice, and offer heads match the selected Trellis projection exactly.
- Canonical local artifacts are ordered by role identity and have domain-separated BLAKE3 identities.
- Choregraph-local evidence revalidates every imported or adapted Trellis claim.
- Diagnostics remain bounded and do not expose host paths, credentials, environment values, or unbounded source text.

## False Completion

A second projection implementation that only resembles Trellis is not completion. A Nickel contract without strict Rust revalidation is not completion.

Generating local files without exact source-to-global-to-local identity linkage is not completion. A native or external role label that grants runtime authority is not completion.

## Impact

- **Workspace**: add `crates/choregraph-choreography/` and connect it to the existing Nickel, Kamacite, CLI, release, and foundation rails.
- **Contracts**: add typed Nickel role-choreography contracts and canonical Preserves schemas.
- **Evidence**: add exact Wyzer reference, Trellis source and proof identities, projection receipts, and Valence-friendly roles.
- **Testing**: add positive and negative global, local, branch, ownership, external-role, diagnostics, identity, and stale-artifact fixtures.
- **Compatibility**: existing graph and machine formats remain unchanged during the first pilot.

## Dependencies

- Trellis change `prove-choreography-value-transfer` must publish the selected value-ownership and transfer semantics.
- Existing Choregraph Nickel, Kamacite, proof-reference, release, and capability boundaries remain authoritative.
- No Kamacite, Valence, Basalt, Host Control, or current machine-schema change is required for the first compiler pilot.

## Out of Scope

- Executing a role, sending a message, opening a network connection, persisting a session, retrying a transfer, or reporting external success.
- Mapping roles to physical hosts or operator inventories.
- Adding dynamic branch semantics to the current Choregraph DAG machine.
- Adopting Wyzer networking, Perceus, FBIP, unrestricted compile-time execution, or its C ABI.
- Claiming whole-protocol deadlock freedom, transport delivery, external-role correctness, or release eligibility.

## Affected Specs

- `role-choreography`: authoring, admission, Trellis-backed projection, artifacts, diagnostics, role kinds, evidence, and non-claims.
