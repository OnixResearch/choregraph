let Contracts = import "publication-candidate-contract.ncl" in
{
  candidate | Contracts.Candidate = {
    schema = "choregraph.publication-candidate.v1",
    candidate_blake3 = "e4b5cd7bd77d78c20cf8b96cd0c038144028d2287000757c8ab68f5f5dfdff1a",
    source_revision = "fc47c5f4ecbb7b4341af690fa42199f25d57f54c",
    source_archive_blake3 = "dcb1faf95a7487145496ce986ae1639908600ace5bd77c4d96a37a530a7538a5",
    source_archive_bytes = 1781760,
    license = "AGPL-3.0-or-later",
    packages = [
      "choregraph-cli",
      "choregraph-core",
      "choregraph-kamacite",
      "choregraph-nickel",
      "choregraph-release",
      "choregraph-seaglass",
    ],
    acquisition_adapter = "radicle-rid-exact-revision-v1",
    radicle_rid = 'unassigned,
    release_bundle_blake3 = "a2ac1336679542fa9573b356daddc18266fdda8147ddbe28f85aa37d135030d9",
    rollback_revision = "2772442d14bfc90579027e4fbff545d7f829c76c",
    publication_authority = 'none,
    publication_status = 'not_published,
  },
}
