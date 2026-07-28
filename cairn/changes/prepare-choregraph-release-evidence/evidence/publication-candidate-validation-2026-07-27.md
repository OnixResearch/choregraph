# Local publication candidate validation — 2026-07-27

## Result

The local publication candidate passes exact identity validation.

```text
candidate-blake3=e4b5cd7bd77d78c20cf8b96cd0c038144028d2287000757c8ab68f5f5dfdff1a
source-revision=fc47c5f4ecbb7b4341af690fa42199f25d57f54c
source-archive-blake3=dcb1faf95a7487145496ce986ae1639908600ace5bd77c4d96a37a530a7538a5
source-archive-bytes=1781760
release-bundle-blake3=a2ac1336679542fa9573b356daddc18266fdda8147ddbe28f85aa37d135030d9
rollback-revision=2772442d14bfc90579027e4fbff545d7f829c76c
license=AGPL-3.0-or-later
acquisition-adapter=radicle-rid-exact-revision-v1
radicle-rid=unassigned
publication-status=not-published
```

The exact source archive came from:

```text
git archive --format=tar --output=/tmp/choregraph-candidate-source.tar fc47c5f4ecbb7b4341af690fa42199f25d57f54c
```

`jj` does not provide an archive command in this environment. `git archive` was used only for this missing operation against the Git-compatible object ID selected by `jj`.

The validator reopened and hashed the archive through a capability root. It also reopened all release artifacts and the checked-in manifest. It rebuilt the bundle before it constructed the candidate.

The package set is exactly:

- `choregraph-cli`
- `choregraph-core`
- `choregraph-kamacite`
- `choregraph-nickel`
- `choregraph-release`
- `choregraph-seaglass`

## Rollback

The rollback revision is the archived, fully validated Seaglass and Lattice conformance state. Rollback execution is not part of candidate validation.

## Authority

The candidate has no publication authority. No Radicle repository was created. No RID was assigned. No signing, push, remote mutation, or rollback occurred.

After explicit operator publication, the acquisition record must replace `radicle-rid=unassigned` with the assigned RID. It must still select the exact source revision.
