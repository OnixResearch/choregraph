# Release authority audit — 2026-07-27

## Result

The release commands have local read and bounded local write authority only.

The command surface is:

- `write-release-bundle`
- `verify-release-bundle`
- `validate-publication-candidate`

`write-release-bundle` reads fixed relative artifacts and writes one manifest without replacement under an operator-opened capability root.

`verify-release-bundle` and `validate-publication-candidate` only read through operator-opened capability roots. Candidate validation does not write.

## Static audit

A focused search of `crates/choregraph-cli/src/release.rs` found no process spawn, TCP, UDP, HTTP client, SSH, signing, Git, Jujutsu, Radicle command, remote mutation, or direct ambient-authority call.

The release shell calls `open_capability_dir`, `read_relative`, and `publish_atomic`. The capability module owns the sole `ambient_authority()` call that opens operator-supplied absolute roots.

`publish_atomic` is a local file-publication helper. Its name does not mean repository or network publication.

## Repository effects

No Radicle initialization, publication, push, signing, remote change, or rollback command ran. The local candidate records:

```text
radicle-rid=unassigned
publication-status=not-published
publication-authority=none
```

## Boundary

A future operator can use separate tools to publish or sign. Those actions require explicit authorization and separate receipts. This release command surface cannot perform them.
