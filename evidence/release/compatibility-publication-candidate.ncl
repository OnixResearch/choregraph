let Contracts = import "publication-candidate-contract.ncl" in
{
  candidate | Contracts.Candidate = {
    schema = "choregraph.publication-candidate.v1",
    candidate_blake3 = "c493ef88a648eb30b621e74fa5c0f7294df9048501a379395403ad3dcd8a4c65",
    source_revision = "81b260da2256147ec82cb5dd44c0a0c1dcb7500d",
    source_archive_blake3 = "41adfbdae4bc5b688ef92ab11cd39ed9c8f572bf6214f522d9b4a1290e5f84d5",
    source_archive_bytes = 1853440,
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
    release_bundle_blake3 = "6912c485313908411040fa533bd75a1cfd2611d7589605268ae111b3ba1472bc",
    rollback_revision = "fc47c5f4ecbb7b4341af690fa42199f25d57f54c",
    publication_authority = 'none,
    publication_status = 'not_published,
  },
}
