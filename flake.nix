{
  description = "Choregraph bounded execution-graph compiler and verified machine kernel";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/20535e48e12c86043b577b8518234ff5dbb26957";
    flake-utils.url = "github:numtide/flake-utils/11707dc2f618dd54ca8739b309ec4fc024de578b";
    rust-overlay = {
      url = "github:oxalica/rust-overlay/3c38e1e1ba9c8d7030f7b5a801398ea7d8a6fdc0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane/b556d7bbae5ff86e378451511873dfd07e4504cd";
    octet = {
      url = "github:OnixResearch/octet?rev=374bd16b26cee2af34211a29bfa531c016811f51";
      inputs.crane.follows = "crane";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      octet,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        cargoSource = pkgs.lib.cleanSourceWith {
          src = self;
          filter =
            path: type:
            craneLib.filterCargoSources path type
            || pkgs.lib.hasInfix "/fixtures/kamacite/" (toString path)
            || pkgs.lib.hasInfix "/fixtures/nickel/" (toString path);
        };
        workspaceVersion = "0.1.0";
        commonCargoArgs = {
          pname = "choregraph-workspace";
          version = workspaceVersion;
          src = cargoSource;
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonCargoArgs;
        choregraphPackage = craneLib.buildPackage (
          commonCargoArgs
          // {
            inherit cargoArtifacts;
            pname = "choregraph";
            cargoExtraArgs = "-p choregraph-cli";
            doCheck = true;
          }
        );
        foundationChecker = craneLib.buildPackage (
          commonCargoArgs
          // {
            inherit cargoArtifacts;
            pname = "choregraph-foundation-check";
            cargoExtraArgs = "-p choregraph-foundation-check";
            doCheck = true;
          }
        );
        octetPackages = octet.packages.${system};
        productionVerus = octetPackages."octet-production-verus";
        productionVerusClosureManifest = octetPackages."octet-production-verus-nix-closure-manifest";
        verusToolchainProfile = octetPackages."octet-verus-toolchain-profile";
      in
      {
        packages = {
          default = choregraphPackage;
          choregraph = choregraphPackage;
          foundation-check = foundationChecker;
        };

        checks = {
          workspace-format = craneLib.cargoFmt {
            src = cargoSource;
          };

          workspace-tests = craneLib.cargoTest (
            commonCargoArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--workspace --all-targets";
            }
          );

          workspace-clippy = craneLib.cargoClippy (
            commonCargoArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--workspace --all-targets -- --deny warnings";
            }
          );

          core-wasm = craneLib.cargoBuild (
            commonCargoArgs
            // {
              inherit cargoArtifacts;
              cargoExtraArgs = "-p choregraph-core --target wasm32-unknown-unknown";
            }
          );

          core-thumb = craneLib.cargoBuild (
            commonCargoArgs
            // {
              inherit cargoArtifacts;
              cargoExtraArgs = "-p choregraph-core --target thumbv7em-none-eabihf";
            }
          );

          # r[verify choregraph.proof.single_step.baseline]
          # r[verify choregraph.proof.single_step.negative]
          # r[verify choregraph.proof.single_step.evidence]
          # r[verify choregraph.proof.single_step.trust]
          # r[verify choregraph.proof.single_step.toolchain]
          # r[verify choregraph.proof.single_step.validation]
          single-step-verus =
            pkgs.runCommand "choregraph-single-step-verus"
              {
                nativeBuildInputs = [
                  pkgs.b3sum
                  pkgs.coreutils
                  pkgs.diffutils
                  pkgs.gnugrep
                  pkgs.jq
                  pkgs.nickel
                  productionVerus
                ];
                src = self;
              }
              ''
                set -eu
                cd "$src"

                octet-production-verus --identity > "$TMPDIR/verus-identity.txt"
                grep -F 'proof-verifier: verus@0.2026.05.17.e479cce' \
                  "$TMPDIR/verus-identity.txt" > /dev/null
                grep -F 'proof-verifier-source-revision: e479cce36490b8fa4b0fd7755aa742aec354372c' \
                  "$TMPDIR/verus-identity.txt" > /dev/null
                grep -F 'proof-verifier-binary-blake3: 2aafa795a63160e2b262e0d8df00ad237766ed87fcd7fe9a981034207b1367c7' \
                  "$TMPDIR/verus-identity.txt" > /dev/null
                grep -F 'proof-toolchain: 1.95.0-x86_64-unknown-linux-gnu' \
                  "$TMPDIR/verus-identity.txt" > /dev/null
                grep -F 'proof-toolchain-blake3: 4b001492d51b266eef82aeb98469d135d60f63d2720d24d9712c1e83fa10c9e3' \
                  "$TMPDIR/verus-identity.txt" > /dev/null
                grep -F 'proof-solver-blake3: 42e48a26bf3b38ca33d84749f4317cc179791de0d578d4d98a67f51782b54331' \
                  "$TMPDIR/verus-identity.txt" > /dev/null
                closure_digest="$(cat ${productionVerusClosureManifest}/octet-production-verus-closure.graph.blake3)"
                test "$closure_digest" = '5f20a8243ed93278d1574e4269838858e99489abd1158f23ab2ffa8471a8d43b'

                octet-production-verus --crate-type=lib verus/proofs/single_step.rs \
                  -o "$TMPDIR/single-step.rlib" \
                  > "$TMPDIR/positive.stdout" 2> "$TMPDIR/positive.stderr"
                grep -E 'verification results:: [1-9][0-9]* verified, 0 errors' \
                  "$TMPDIR/positive.stdout" > /dev/null

                for fixture in verus/failure-fixtures/*.rs; do
                  fixture_name="$(basename "$fixture")"
                  case "$fixture_name" in
                    denial_with_intent.rs) expected='result.intent_indices@.len()' ;;
                    premature_readiness.rs) expected='dependencies_complete_spec' ;;
                    state_map_mismatch.rs) expected='indexed_invariant_spec' ;;
                    terminal_mutation.rs) expected='terminal_states_preserved_spec' ;;
                    topological_edge_reversal.rs) expected='topological_order_spec' ;;
                    undeclared_intent.rs) expected='effects_allowed_spec' ;;
                    *) echo "unclassified Verus failure fixture: $fixture_name" >&2; exit 1 ;;
                  esac
                  set +e
                  octet-production-verus --crate-type=lib "$fixture" \
                    > "$TMPDIR/negative.stdout" 2> "$TMPDIR/negative.stderr"
                  negative_status="$?"
                  set -e
                  if [ "$negative_status" -eq 0 ]; then
                    echo "negative Verus fixture unexpectedly passed: $fixture_name" >&2
                    exit 1
                  fi
                  grep -F 'postcondition not satisfied' "$TMPDIR/negative.stderr" > /dev/null
                  grep -F "$expected" "$TMPDIR/negative.stderr" > /dev/null
                done

                if grep -R -E '#\[verifier::external_body\]|assume_specification|\bassume[[:space:]]*\(|\baxiom\b' \
                  crates/choregraph-core/src/verified.rs \
                  crates/choregraph-core/src/verified_topology.rs; then
                  echo 'single-step proof core gained an unrecorded trust escape' >&2
                  exit 1
                fi
                test "$(grep -F -c 'crate::verified::indexed_step(' \
                  crates/choregraph-core/src/transition.rs)" -eq 2
                grep -F 'crate::verified::lifecycle_target(' \
                  crates/choregraph-core/src/transition.rs > /dev/null
                grep -F 'crate::verified_topology::admitted_graph_valid(' \
                  crates/choregraph-core/src/invariant.rs > /dev/null
                if grep -F '(LocalState::Pending, Action::MarkReady)' \
                  crates/choregraph-core/src/transition.rs; then
                  echo 'transition shell regained an independent lifecycle table' >&2
                  exit 1
                fi

                nickel export --format json evidence/single-step-proof.ncl \
                  > "$TMPDIR/proof-manifest-raw.json"
                jq -S . "$TMPDIR/proof-manifest-raw.json" \
                  > "$TMPDIR/proof-manifest.json"
                diff --unified evidence/generated/single-step-proof.json \
                  "$TMPDIR/proof-manifest.json"
                expected_manifest_hash="$(cat evidence/generated/single-step-proof.blake3)"
                observed_manifest_hash="$(b3sum evidence/generated/single-step-proof.json)"
                test "''${observed_manifest_hash%% *}" = "$expected_manifest_hash"
                jq --exit-status '
                  .proof.schema == "choregraph.single-step-proof.v1"
                  and .proof.status == "verified"
                  and (.proof.theorems | length) == 6
                  and (.proof.negative_fixtures | length) == 6
                  and .proof.trusted_surface.project_trusted_declarations == []
                  and (.proof.non_claims | length) > 0
                ' "$TMPDIR/proof-manifest.json" > /dev/null
                jq --raw-output '.proof.sources[] | [.path, .blake3] | @tsv' \
                  "$TMPDIR/proof-manifest.json" \
                  | while IFS="$(printf '\t')" read -r source_path expected_hash; do
                      observed_hash="$(b3sum "$source_path")"
                      test "''${observed_hash%% *}" = "$expected_hash"
                    done

                mkdir -p "$out"
                cp "$TMPDIR/verus-identity.txt" "$out/verus-identity.txt"
                cp "$TMPDIR/positive.stdout" "$out/positive.stdout"
                cp "$TMPDIR/proof-manifest.json" "$out/proof-manifest.json"
                printf '%s\n' "$closure_digest" > "$out/verus-closure.blake3"
              '';

          foundation-boundaries =
            pkgs.runCommand "choregraph-foundation-boundaries"
              {
                nativeBuildInputs = [
                  foundationChecker
                  pkgs.coreutils
                  pkgs.gnugrep
                  pkgs.nickel
                  rustToolchain
                ];
                src = self;
              }
              ''
                set -eu
                cd "$src"
                choregraph-foundation-check --root "$src"

                nickel export --format json fixtures/foundation/positive/allowed-boundary.ncl \
                  > "$TMPDIR/allowed-boundary.json"
                choregraph-foundation-check --model-json "$TMPDIR/allowed-boundary.json"

                for fixture in fixtures/foundation/negative/*.ncl; do
                  fixture_name="$(basename "$fixture")"
                  case "$fixture_name" in
                    forbidden-core-io.ncl|forbidden-core-evaluator.ncl|forbidden-core-runtime.ncl|forbidden-core-receipt.ncl|forbidden-core-policy.ncl)
                      expected_code="foundation.forbidden-core-dependency"
                      ;;
                    forbidden-external-path.ncl)
                      expected_code="foundation.path-dependency"
                      ;;
                    forbidden-reverse-edge.ncl)
                      expected_code="foundation.dependency-direction"
                      ;;
                    forbidden-core-source.ncl)
                      expected_code="foundation.forbidden-core-source"
                      ;;
                    *)
                      echo "unclassified negative foundation fixture: $fixture_name" >&2
                      exit 1
                      ;;
                  esac
                  nickel export --format json "$fixture" > "$TMPDIR/negative.json"
                  if choregraph-foundation-check --model-json "$TMPDIR/negative.json" \
                    > "$TMPDIR/negative.stdout" 2> "$TMPDIR/negative.stderr"; then
                    echo "negative foundation fixture unexpectedly passed: $fixture_name" >&2
                    exit 1
                  fi
                  grep -F "$expected_code" "$TMPDIR/negative.stderr" > /dev/null
                done

                if nickel export --format json \
                  fixtures/foundation/malformed/missing-package-name.ncl \
                  > "$TMPDIR/malformed.json"; then
                  echo "malformed foundation fixture unexpectedly exported" >&2
                  exit 1
                fi

                mkdir -p "$out"
                printf '%s\n' 'claim-scope=repository-boundary-only-unverified' \
                  > "$out/result.txt"
              '';

          restricted-nickel-contracts =
            pkgs.runCommand "choregraph-restricted-nickel-contracts"
              {
                nativeBuildInputs = [
                  pkgs.coreutils
                  pkgs.gnugrep
                  pkgs.jq
                  pkgs.nickel
                ];
                src = self;
              }
              ''
                set -eu
                cd "$src"
                nickel typecheck contracts/nickel/restricted-graph-contracts.ncl
                nickel export --format json evidence/cap-root-adoption-blocker.ncl \
                  > "$TMPDIR/cap-root-blocker.json"
                jq --exit-status '
                  .blocker.capability_shell_ready == false
                  and .blocker.cargo_source_admitted == false
                  and .blocker.nix_source_aligned == false
                ' "$TMPDIR/cap-root-blocker.json" > /dev/null
                if grep -R -E 'std::(fs|net|process)|SystemTime|Command::new|env::' \
                  crates/choregraph-nickel/src --include='*.rs'; then
                  echo 'choregraph-nickel gained ambient authority' >&2
                  exit 1
                fi
                for fixture in fixtures/nickel/positive/*.ncl; do
                  fixture_name="$(basename "$fixture" .ncl)"
                  nickel export --format json "$fixture" > "$TMPDIR/raw.json"
                  jq -S . "$TMPDIR/raw.json" > "$TMPDIR/normalized.json"
                  cmp "fixtures/nickel/evaluated/$fixture_name.json" "$TMPDIR/normalized.json"
                done
                for fixture in fixtures/nickel/negative/*.ncl; do
                  if nickel export --format json "$fixture" > "$TMPDIR/negative.json"; then
                    echo "negative Nickel fixture unexpectedly exported: $fixture" >&2
                    exit 1
                  fi
                done
                mkdir -p "$out"
                printf '%s\n' 'claim=bounded-contract-shape-only-not-nickel-semantics' > "$out/result.txt"
              '';

          foundation-evidence =
            pkgs.runCommand "choregraph-foundation-evidence"
              {
                nativeBuildInputs = [
                  pkgs.b3sum
                  pkgs.coreutils
                  pkgs.diffutils
                  pkgs.gnugrep
                  pkgs.jq
                  pkgs.nickel
                  pkgs.nix
                  productionVerus
                  rustToolchain
                ];
                src = self;
              }
              ''
                set -eu
                cd "$src"

                nickel export --format json config/project.ncl > "$TMPDIR/project.json"
                nickel export --format json config/toolchain.ncl > "$TMPDIR/toolchain.json"
                nickel export --format json fixtures/kamacite/corpus.ncl \
                  > "$TMPDIR/kamacite-corpus.json"
                jq --exit-status '
                  ([.cases[] | select(.expected == "accept")] | length) > 0
                  and ([.cases[] | select(.expected == "reject")] | length) > 0
                  and .profile.authority == "canonical-packed-preserves"
                  and .profile.fsm_determinism_policy == "deterministic"
                  and .profile.fsm_receipt_kinds == ["exhaustiveness", "trace"]
                  and .profile.fsm_denial_no_effect_class == "no-side-effect"
                ' "$TMPDIR/kamacite-corpus.json" > /dev/null
                expected_machine_schema_hash="$(jq --raw-output '.profile.machine_schema_source_blake3' "$TMPDIR/kamacite-corpus.json")"
                observed_machine_schema_hash="$(b3sum schemas/preserves/choregraph-machine-v1.preserves)"
                test "''${observed_machine_schema_hash%% *}" = "$expected_machine_schema_hash"
                expected_link_schema_hash="$(jq --raw-output '.profile.link_receipt_schema_source_blake3' "$TMPDIR/kamacite-corpus.json")"
                observed_link_schema_hash="$(b3sum schemas/preserves/choregraph-link-receipt-v1.preserves)"
                test "''${observed_link_schema_hash%% *}" = "$expected_link_schema_hash"
                if grep -R -E 'std::(fs|net|process)|SystemTime|Command::new|env::' \
                  crates/choregraph-kamacite/src --include='*.rs'; then
                  echo 'choregraph-kamacite gained ambient authority' >&2
                  exit 1
                fi
                jq --exit-status '
                  .project.status == "single-step-verified-indexed-kernel"
                  and .project.claim_scope == "named-single-step-properties-over-production-called-indexed-kernel"
                ' "$TMPDIR/project.json" > /dev/null
                jq --exit-status '.toolchain.claim_scope == "pinned-tool-identity-only-not-tool-soundness"' \
                  "$TMPDIR/toolchain.json" > /dev/null

                nickel export --format json evidence/source-baseline.ncl \
                  > "$TMPDIR/source-baseline-raw.json"
                jq -S . "$TMPDIR/source-baseline-raw.json" \
                  > "$TMPDIR/source-baseline.json"
                cmp evidence/generated/source-baseline.json "$TMPDIR/source-baseline.json"
                expected_hash="$(cat evidence/generated/source-baseline.blake3)"
                observed_hash="$(b3sum evidence/generated/source-baseline.json)"
                test "''${observed_hash%% *}" = "$expected_hash"
                jq --exit-status '
                  (.baseline.sources | map(.id)) as $ids
                  | .baseline.schema == "choregraph.source-baseline.v1"
                  and ($ids == ($ids | sort))
                  and (($ids | length) == ($ids | unique | length))
                  and ([.baseline.sources[].working_tree_used] | all(. == false))
                  and ([.baseline.sources[] | select(.imported == true) | .id] == ["kamacite-interchange-boundary", "trellis-proof-reference"])
                ' "$TMPDIR/source-baseline.json" > /dev/null

                if nickel export --format json \
                  evidence/source-baseline-fixtures/invalid-missing-revision.ncl \
                  > "$TMPDIR/invalid-source-baseline.json"; then
                  echo "invalid source baseline unexpectedly exported" >&2
                  exit 1
                fi

                grep -F 'GNU AFFERO GENERAL PUBLIC LICENSE' LICENSE > /dev/null
                grep -F '20535e48e12c86043b577b8518234ff5dbb26957' flake.nix > /dev/null
                grep -F '374bd16b26cee2af34211a29bfa531c016811f51' flake.nix > /dev/null
                rustc --version > "$TMPDIR/rust-version.txt"
                grep -F 'rustc 1.97.1 ' "$TMPDIR/rust-version.txt" > /dev/null
                nickel export --format json vendor/kamacite-core-selected/SOURCE.ncl \
                  > "$TMPDIR/kamacite-codec-source.json"
                expected_codec_hash="$(jq --raw-output '.source.source_blake3' "$TMPDIR/kamacite-codec-source.json")"
                observed_codec_hash="$(b3sum vendor/kamacite-core-selected/preserves_codec.rs)"
                test "''${observed_codec_hash%% *}" = "$expected_codec_hash"
                expected_apache_hash="$(jq --raw-output '.source.apache_license_blake3' "$TMPDIR/kamacite-codec-source.json")"
                observed_apache_hash="$(b3sum vendor/kamacite-core-selected/LICENSE-APACHE)"
                test "''${observed_apache_hash%% *}" = "$expected_apache_hash"
                expected_mit_hash="$(jq --raw-output '.source.mit_license_blake3' "$TMPDIR/kamacite-codec-source.json")"
                observed_mit_hash="$(b3sum vendor/kamacite-core-selected/LICENSE-MIT)"
                test "''${observed_mit_hash%% *}" = "$expected_mit_hash"
                rustc tools/sync-kamacite-codec.rs -o "$TMPDIR/sync-kamacite-codec"
                "$TMPDIR/sync-kamacite-codec" --self-test

                nickel --version > "$TMPDIR/nickel-version.txt"
                nix --version > "$TMPDIR/nix-version.txt"
                octet-production-verus --identity > "$TMPDIR/verus-identity.txt"
                test -s "$TMPDIR/verus-identity.txt"
                test -s ${verusToolchainProfile}/share/octet/verus-toolchain/profile.json

                mkdir -p "$out"
                cp "$TMPDIR/project.json" "$out/project.json"
                cp "$TMPDIR/toolchain.json" "$out/toolchain.json"
                cp "$TMPDIR/rust-version.txt" "$out/rust-version.txt"
                cp "$TMPDIR/nickel-version.txt" "$out/nickel-version.txt"
                cp "$TMPDIR/nix-version.txt" "$out/nix-version.txt"
                cp "$TMPDIR/verus-identity.txt" "$out/verus-identity.txt"
                printf '%s\n' 'non-claim=no-machine-or-proof-correctness' > "$out/non-claim.txt"
              '';

          cairn-policy =
            pkgs.runCommand "choregraph-cairn-policy"
              {
                nativeBuildInputs = [
                  pkgs.coreutils
                  pkgs.jq
                  pkgs.nickel
                ];
                src = self;
              }
              ''
                set -eu
                cd "$src"
                nickel export --format json cairn-policy/default.ncl \
                  > "$TMPDIR/fresh-policy-raw.json"
                jq -S . "$TMPDIR/fresh-policy-raw.json" > "$TMPDIR/fresh-policy.json"
                cmp cairn-policy/generated/cairn-policy.json "$TMPDIR/fresh-policy.json"
                for fixture in cairn-policy/fixtures/invalid-*.ncl; do
                  if nickel export --format json "$fixture" > "$TMPDIR/invalid-policy.json"; then
                    echo "invalid Cairn policy fixture unexpectedly exported: $fixture" >&2
                    exit 1
                  fi
                done
                mkdir -p "$out"
              '';
        };

        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.b3sum
            pkgs.jq
            pkgs.nickel
            pkgs.nixfmt
            productionVerus
            rustToolchain
            verusToolchainProfile
          ];
        };

        formatter = pkgs.nixfmt;
      }
    );
}
