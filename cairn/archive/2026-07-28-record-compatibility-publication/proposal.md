## Why

The operator authorized publication of the immutable compiler-compatibility candidate. Consumers need a receipt that binds the unchanged candidate to the existing Radicle RID, current parent-feature signed refs, observed replicas, and read-only HTTPS acquisition.

## What Changes

- Record the authorized Radicle publication without changing the candidate identity.
- Preserve the earlier candidate and publication receipt as immutable historical evidence.
- Verify exact native and HTTPS acquisition from the published source revision.
- Record write rejection and bounded publication non-claims.

## Impact

- **Files**: compatibility publication receipt, publication evidence, release documentation, README, release-evidence specification, and Cairn lifecycle artifacts
- **Testing**: Nickel receipt validation, exact Radicle refs, three-replica status, isolated native clones, HTTPS clone and archive identity, strict Git checks, and write-route rejection
