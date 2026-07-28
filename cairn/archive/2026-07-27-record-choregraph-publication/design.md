# Design: Choregraph publication evidence

## Success contract

Completion requires one typed receipt that binds the unchanged candidate identity to the RID, exact source revision, source archive, release bundle, producer identity, parent-feature signed refs, three observed replicas, and read-only HTTPS acquisition.

An assigned RID without the exact revision, a root-only signed-ref chain, one local copy, a writable endpoint, or a changed candidate identity is false completion.

## Authority

The operator authorized publication in this session. Radicle signing and source distribution are shell actions outside the pure release core.

The receipt records this authority. It grants no consumer, runtime, deployment, or effect authority.

## Validation

Nickel validates receipt shape. Nix checks exact values and rejects stale revisions, mismatched archives, weakened signed refs, missing replicas, writable routes, and missing non-claims.
