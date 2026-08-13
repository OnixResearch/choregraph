# Adversarial role-choreography audit

Date: 2026-08-13

Verdict: pass after two fixes.

## Search portfolio

The audit used separate passes for semantic duplication, partial output, stale identity, diagnostic leakage, authority transfer, and overclaim.

### Semantic duplication

An AST search found one production call to `project_endpoint`:

`crates/choregraph-choreography/src/endpoint.rs`

The call uses each normalized declared role and the admitted Trellis global value. No second endpoint-projection function exists. The local conversion only converts the returned Trellis enum into the public artifact DTO.

Admission calls Trellis `projectable`. Ownership admission calls `admit_ownership`, `owner_of`, and `transfer_owner`.

### Partial output

`project_all` builds all local values in memory. It validates the exact count and normalized order before it returns.

A missing role kind, a Trellis rejection, an encoding failure, or a size failure returns an error. The crate owns no file output or publication operation.

Negative tests reject missing, duplicate, and reordered local values.

### Stale identity

The receipt verifier requires the exact expected body and packed Preserves bytes.

Negative tests reject stale Trellis, schema, lowerer, proof-set, local, local-order, and complete-set identities.

The lowerer identity frames the lowerer profile plus `admission.rs` and `endpoint.rs`. The schema-set identity frames every schema version and reviewed Preserves schema source.

A changed local continuation changes the local and complete-set identities. The prior receipt then fails.

### Diagnostic leakage

A source search found no filesystem, process, network, clock, shared-mutation, or unsafe API in the adapter.

Safe diagnostics reject Unix paths, Windows paths, assignment text, and secret, token, credential, or environment terms. They also enforce fixed text, count, span, and ordering limits.

The audit strengthened this gate after the first pass found that only `/home/` paths were rejected.

### Authority transfer

External roles contain only a contract identity and input and output schema identities. Strict DTO parsing rejects unknown fields.

No adapter, host, credential, capability, transport, effect, execution, persistence, or authorization field exists in the accepted role type.

The audit made admitted choreography and projection bundles opaque. External callers can no longer forge an admitted value or mutate the expected receipt bundle.

### Overclaim

The receipt contains the closed non-claim set and linkage-only evidence roles.

Negative tests reject proof transfer, deadlock, delivery, and release-eligibility claims. Documentation keeps runtime and release authority with consumers and operators.

## Bounded result

The audit found no remaining semantic duplication, partial projection, stale-link acceptance, diagnostic leakage path, authority transfer, or claim promotion in the reviewed surface.

This is source, test, and identity-linkage evidence. It is not a proof of external role behavior, transport delivery, runtime correctness, fault liveness, or release eligibility.
