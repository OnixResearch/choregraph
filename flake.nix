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
                jq --exit-status '.project.status == "unverified-machine-semantics"' \
                  "$TMPDIR/project.json" > /dev/null
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
                  and ([.baseline.sources[] | select(.imported == true) | .id] == ["kamacite-interchange-boundary"])
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
