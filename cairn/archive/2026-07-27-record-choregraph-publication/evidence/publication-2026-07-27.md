# Choregraph publication evidence — 2026-07-27

## Result

The exact local candidate is published through Radicle.

```text
rid=rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF
candidate=e4b5cd7bd77d78c20cf8b96cd0c038144028d2287000757c8ab68f5f5dfdff1a
source-revision=fc47c5f4ecbb7b4341af690fa42199f25d57f54c
source-archive-blake3=dcb1faf95a7487145496ce986ae1639908600ace5bd77c4d96a37a530a7538a5
release-bundle=a2ac1336679542fa9573b356daddc18266fdda8147ddbe28f85aa37d135030d9
identity-revision=fce6d85cbe015b49ddab43db66884f550e66153e
author-sigrefs=70148d3d6d0f5d5995fc7c4b78002189701748b5
signed-refs-feature=parent
```

The operator explicitly authorized publication. The candidate branch points to the same exact revision as `main`.

## Replication and acquisition

Aspen, the desktop seed, and Paintedlife report the publisher signed refs in sync.

Fresh isolated clients cloned from Aspen alone and from the desktop alone. Both clients required parent-feature signed refs and resolved the exact candidate revision.

A fresh HTTPS clone from `https://git.onix.computer/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git` passed `git fsck --full --strict`. Its source archive matched the candidate BLAKE3 and byte count.

The endpoint root, an unknown RID, receive-pack discovery, and receive-pack POST returned `404`.

## Claim boundary

Publication proves the observed source identity and acquisition paths. It does not prove indefinite availability, seed trust, consumer correctness, runtime authority, or consumer release eligibility.
