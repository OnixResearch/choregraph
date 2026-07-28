# Choregraph compatibility publication evidence — 2026-07-28

## Result

The operator authorized publication of the unchanged compiler-compatibility candidate.

```text
rid=rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF
candidate=c493ef88a648eb30b621e74fa5c0f7294df9048501a379395403ad3dcd8a4c65
source-revision=81b260da2256147ec82cb5dd44c0a0c1dcb7500d
source-archive-blake3=41adfbdae4bc5b688ef92ab11cd39ed9c8f572bf6214f522d9b4a1290e5f84d5
source-archive-bytes=1853440
release-bundle=6912c485313908411040fa533bd75a1cfd2611d7589605268ae111b3ba1472bc
identity-revision=fce6d85cbe015b49ddab43db66884f550e66153e
author-sigrefs=525f51fc1746ec6dabeeaed0c7d47e268083493b
signed-refs-feature=parent
rollback-revision=fc47c5f4ecbb7b4341af690fa42199f25d57f54c
```

Radicle `main` and `candidate` both resolve to the exact source revision. The earlier candidate and publication receipt remain unchanged.

## Replication

The Radicle push updated both branches and synchronized with two seeds. A following announcement synchronized with three seeds.

`rad sync status` reports the publisher, Aspen, the desktop seed, and Paintedlife in sync at signed refs `525f51fc1746ec6dabeeaed0c7d47e268083493b`.

A direct fetch from Paintedlife reset during its handshake. The final synchronized status still reports Paintedlife at the current signed refs. Aspen and the desktop seed each returned the current parent-feature signed refs through direct fetch.

## Independent native acquisition

Fresh isolated Radicle profiles cloned only from Aspen and only from the desktop seed. Both clients required parent-feature signed refs.

Each checkout resolved `81b260da2256147ec82cb5dd44c0a0c1dcb7500d`, passed `git fsck --full --strict`, and produced the expected archive BLAKE3 and byte count.

Both isolated profiles observed publisher signed refs `525f51fc1746ec6dabeeaed0c7d47e268083493b` with feature level `parent`.

## HTTPS acquisition and write rejection

A fresh clone from `https://git.onix.computer/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git` resolved the exact source revision. It passed strict Git checks and produced the expected source archive.

The endpoint root, an unknown RID, receive-pack discovery, and receive-pack POST each returned `404`.

## Claim boundary

Publication proves the observed source identity and selected acquisition paths. It does not prove indefinite availability, seed trust, dependency correctness, consumer acceptance, runtime authority, or release eligibility.
