## Context

The local candidate `c493ef88a648eb30b621e74fa5c0f7294df9048501a379395403ad3dcd8a4c65` already binds exact source, archive, bundle, and rollback identities. Publication must not change those values. The existing receipt covers the earlier published candidate and must remain unchanged.

## Decisions

### Decision: Publish the exact candidate revision on main and candidate

**Choice:** Update both Radicle branches to `81b260da2256147ec82cb5dd44c0a0c1dcb7500d` and require parent-feature signed refs.

**Rationale:** Equal branch targets remove ambiguity for exact consumers. Parent-feature signed refs preserve replay and graft protections.

### Decision: Add a separate compatibility receipt

**Choice:** Keep the earlier candidate and receipt unchanged. Add a new receipt that binds the compatibility candidate to the same RID and current signed refs.

**Rationale:** Publication history remains append-only. A new observation must not rewrite earlier evidence.

### Decision: Verify independent acquisition paths

**Choice:** Require three in-sync native replicas, isolated Aspen and desktop clones, one fresh HTTPS clone, exact archive equality, strict Git checks, and receive-pack rejection.

**Rationale:** One publisher-local observation cannot establish the selected distribution paths.

## Risks / Trade-offs

- Replica status is an observation at one time. It does not prove future availability.
- HTTPS read-only checks do not prove the absence of every external write path.
- Publication grants source acquisition only. It grants no consumer, runtime, deployment, or effect authority.
