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
            || pkgs.lib.hasSuffix ".wit" (toString path)
            || pkgs.lib.hasInfix "/contracts/nickel/" (toString path)
            || pkgs.lib.hasInfix "/evidence/generated/" (toString path)
            || pkgs.lib.hasInfix "/fixtures/kamacite/" (toString path)
            || pkgs.lib.hasInfix "/fixtures/nickel/" (toString path)
            || pkgs.lib.hasInfix "/fixtures/role-choreography/" (toString path)
            || pkgs.lib.hasInfix "/schemas/preserves/" (toString path)
            || pkgs.lib.hasInfix "/vendor/bounded-exec-selected/" (toString path)
            || pkgs.lib.hasInfix "/vendor/cap-root-core-selected/" (toString path)
            || pkgs.lib.hasInfix "/vendor/execution-graph-core-selected/" (toString path)
            || pkgs.lib.hasSuffix "/rust-toolchain.toml" (toString path);
        };
        workspaceVersion = "0.1.0";
        defaultMaxTraceEvents = 65536;
        traceBindingBytes = 32;
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
        roleChoreographyCargoArtifacts = craneLib.buildDepsOnly (
          commonCargoArgs
          // {
            pname = "choregraph-role-choreography-dependencies";
            cargoExtraArgs = "--locked -p choregraph-choreography";
            doCheck = false;
          }
        );
        roleChoreographyPackage = craneLib.buildPackage (
          commonCargoArgs
          // {
            cargoArtifacts = roleChoreographyCargoArtifacts;
            pname = "choregraph-role-choreography";
            cargoExtraArgs = "--locked -p choregraph-choreography";
            doCheck = true;
          }
        );
        roleChoreographyContractsCheck =
          pkgs.runCommand "choregraph-role-choreography-contracts"
            {
              nativeBuildInputs = [ pkgs.nickel ];
              src = cargoSource;
            }
            ''
              set -eu
              cd "$src"
              nickel typecheck contracts/nickel/role-choreography-contracts.ncl
              nickel export --format json fixtures/role-choreography/positive/two-role-transfer.ncl \
                > "$TMPDIR/two-role-transfer.json"
              for fixture in fixtures/role-choreography/negative/*.ncl; do
                if nickel export --format json "$fixture" \
                  > "$TMPDIR/negative.json" 2> "$TMPDIR/negative.stderr"; then
                  echo "invalid role choreography unexpectedly exported: $fixture" >&2
                  exit 1
                fi
              done
              mkdir -p "$out"
              cp "$TMPDIR/two-role-transfer.json" "$out/two-role-transfer.json"
            '';
        wasmConformanceCargoArtifacts = craneLib.buildDepsOnly (
          commonCargoArgs
          // {
            pname = "choregraph-wasm-conformance-dependencies";
            cargoExtraArgs = "--locked -p choregraph-wasm-conformance";
            doCheck = false;
          }
        );
        wasmConformanceChecker = craneLib.buildPackage (
          commonCargoArgs
          // {
            cargoArtifacts = wasmConformanceCargoArtifacts;
            pname = "choregraph-wasm-conformance";
            cargoExtraArgs = "-p choregraph-wasm-conformance";
            doCheck = true;
          }
        );
        wasmCargoArtifacts = craneLib.buildDepsOnly (
          commonCargoArgs
          // {
            pname = "choregraph-wasm-dependencies";
            cargoExtraArgs = "--locked -p choregraph-wasm --target wasm32-unknown-unknown";
            doCheck = false;
          }
        );
        # r[impl choregraph.wasm.module]
        # r[impl choregraph.wasm.identity]
        wasmKernelCore = craneLib.cargoBuild (
          commonCargoArgs
          // {
            cargoArtifacts = wasmCargoArtifacts;
            pname = "choregraph-wasm-core";
            cargoExtraArgs = "--locked -p choregraph-wasm --target wasm32-unknown-unknown";
            installPhaseCommand = ''
              mkdir -p "$out/lib"
              cp target/wasm32-unknown-unknown/release/choregraph_wasm.wasm \
                "$out/lib/choregraph-wasm-core.wasm"
            '';
          }
        );
        # r[verify choregraph.wasm.boundary]
        # r[verify choregraph.wasm.identity]
        wasmKernelComponent =
          pkgs.runCommand "choregraph-wasm-component-${workspaceVersion}"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.coreutils
                pkgs.gnugrep
                pkgs.wasm-tools
              ];
            }
            ''
              set -eu
              mkdir -p "$out/lib" "$out/share/choregraph"
              wasm-tools component new \
                ${wasmKernelCore}/lib/choregraph-wasm-core.wasm \
                -o "$TMPDIR/component-a.wasm"
              wasm-tools component new \
                ${wasmKernelCore}/lib/choregraph-wasm-core.wasm \
                -o "$TMPDIR/component-b.wasm"
              cmp "$TMPDIR/component-a.wasm" "$TMPDIR/component-b.wasm"
              wasm-tools validate "$TMPDIR/component-a.wasm"
              wasm-tools component wit "$TMPDIR/component-a.wasm" \
                > "$TMPDIR/component.wit"
              wasm-tools print "$TMPDIR/component-a.wasm" \
                > "$TMPDIR/component.wat"
              if grep -F '(import ' "$TMPDIR/component.wat"; then
                echo 'WASM component gained a host import' >&2
                exit 1
              fi
              test "$(grep -c 'export call:' "$TMPDIR/component.wit")" -eq 1
              test "$(grep -c '^[[:space:]]*export ' "$TMPDIR/component.wit")" -eq 1
              cp "$TMPDIR/component-a.wasm" "$out/lib/choregraph-wasm-component.wasm"
              cp "$TMPDIR/component.wit" "$out/share/choregraph/component.wit"
              b3sum --derive-key 'choregraph wasm component bytes v1' \
                "$out/lib/choregraph-wasm-component.wasm" \
                | cut -d ' ' -f 1 > "$out/share/choregraph/component.blake3"
              printf '%s\n' \
                'non-claim=no-compiler-engine-sandbox-effect-or-release-correctness' \
                > "$out/share/choregraph/non-claim.txt"
            '';
        # r[verify choregraph.wasm.parity]
        # r[verify choregraph.wasm.bounds]
        wasmConformanceCheck =
          pkgs.runCommand "choregraph-wasm-conformance-check"
            {
              nativeBuildInputs = [ wasmConformanceChecker ];
            }
            ''
              set -eu
              mkdir -p "$out"
              choregraph-wasm-conformance \
                ${wasmKernelComponent}/lib/choregraph-wasm-component.wasm \
                | tee "$out/report.txt"
              grep -F 'imports=none exports=call verdict=pass' "$out/report.txt" > /dev/null
              grep -F 'non-claim=engine-compiler-sandbox-effect-and-release-correctness' \
                "$out/report.txt" > /dev/null
            '';
        # r[verify choregraph.wasm.identity]
        wasmEvidenceCheck =
          pkgs.runCommand "choregraph-wasm-evidence-check"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.coreutils
                pkgs.jq
                pkgs.nickel
              ];
              src = self;
            }
            ''
              set -eu
              cd "$src"
              nickel export evidence/wasm-kernel-adapter.ncl --format json \
                > "$TMPDIR/manifest.json"
              cmp evidence/generated/wasm-kernel-adapter.json "$TMPDIR/manifest.json"

              expected_component="$(jq --raw-output '.evidence.component.blake3' \
                "$TMPDIR/manifest.json")"
              observed_component="$(cat \
                ${wasmKernelComponent}/share/choregraph/component.blake3)"
              test "$observed_component" = "$expected_component"

              expected_corpus="$(jq --raw-output '.evidence.conformance.corpus_blake3' \
                "$TMPDIR/manifest.json")"
              observed_corpus="$(sed -n 's/^corpus-identity=//p' \
                ${wasmConformanceCheck}/report.txt)"
              test "$observed_corpus" = "$expected_corpus"
              expected_fixtures="$(jq --raw-output '.evidence.conformance.fixture_count' \
                "$TMPDIR/manifest.json")"
              observed_fixtures="$(grep -c '^fixture=' \
                ${wasmConformanceCheck}/report.txt)"
              test "$observed_fixtures" = "$expected_fixtures"

              jq --raw-output \
                '.evidence.sources[] | [.path, .context, .blake3] | @tsv' \
                "$TMPDIR/manifest.json" \
                | while IFS=$'\t' read -r path context expected; do
                    observed="$(b3sum --derive-key "$context" "$path")"
                    test "''${observed%% *}" = "$expected"
                  done

              jq --raw-output \
                '.evidence.proof_references[] | [.manifest_path, .manifest_blake3, .exported_path, .exported_blake3] | @tsv' \
                "$TMPDIR/manifest.json" \
                | while IFS=$'\t' read -r manifest manifest_expected exported exported_expected; do
                    manifest_observed="$(b3sum \
                      --derive-key "choregraph wasm source file v1:$manifest" \
                      "$manifest")"
                    exported_observed="$(b3sum \
                      --derive-key "choregraph wasm source file v1:$exported" \
                      "$exported")"
                    test "''${manifest_observed%% *}" = "$manifest_expected"
                    test "''${exported_observed%% *}" = "$exported_expected"
                  done

              mkdir -p "$out"
              cp "$TMPDIR/manifest.json" "$out/manifest.json"
              cp ${wasmConformanceCheck}/report.txt "$out/conformance-report.txt"
              printf '%s\n' \
                'non-claim=identity-linkage-and-bounded-parity-observations-only' \
                > "$out/non-claim.txt"
            '';
        octetPackages = octet.packages.${system};
        octetDenyConfig = (builtins.fromJSON (builtins.readFile ./config/octet-deny-all.json)).config;
        octetDenyFlags = builtins.concatStringsSep " " (map (lint: "-D ${lint}") octetDenyConfig.lints);
        octetDenyAllRunner = pkgs.writeShellApplication {
          name = "octet-deny-all";
          runtimeInputs = [ octetPackages.cargo-octet ];
          text = ''
            export DYLINT_RUSTFLAGS=${pkgs.lib.escapeShellArg octetDenyFlags}
            exec cargo-octet check "$@"
          '';
        };
        # r[impl choregraph.quality.octet.scope]
        octetDenyAllCheck =
          (octet.lib.mkConsumerCheck {
            inherit system;
            src = self;
            cargoLock = ./Cargo.lock;
          }).overrideAttrs
            (_old: {
              DYLINT_RUSTFLAGS = octetDenyFlags;
            });
        productionVerus = octetPackages."octet-production-verus";
        productionVerusClosureManifest = octetPackages."octet-production-verus-nix-closure-manifest";
        verusToolchainProfile = octetPackages."octet-verus-toolchain-profile";
      in
      {
        packages = {
          default = choregraphPackage;
          choregraph = choregraphPackage;
          foundation-check = foundationChecker;
          role-choreography = roleChoreographyPackage;
          choregraph-wasm-component = wasmKernelComponent;
          choregraph-wasm-conformance = wasmConformanceChecker;
          octet-deny-all = octetDenyAllRunner;
        };

        checks = {
          role-choreography = roleChoreographyPackage;
          role-choreography-contracts = roleChoreographyContractsCheck;
          octet-deny-all = octetDenyAllCheck;
          wasm-kernel-component = wasmKernelComponent;
          wasm-kernel-conformance = wasmConformanceCheck;
          wasm-kernel-evidence = wasmEvidenceCheck;

          # r[impl choregraph.quality.octet.severity]
          octet-deny-all-policy =
            pkgs.runCommand "choregraph-octet-deny-all-policy"
              {
                nativeBuildInputs = [
                  pkgs.b3sum
                  pkgs.coreutils
                  pkgs.diffutils
                  pkgs.gawk
                  pkgs.gnugrep
                  pkgs.jq
                  pkgs.nickel
                ];
                src = self;
              }
              ''
                set -eu
                cd "$src"

                nickel export --format json config/octet-deny-all.ncl \
                  > "$TMPDIR/octet-deny-all-raw.json"
                jq -S . "$TMPDIR/octet-deny-all-raw.json" \
                  > "$TMPDIR/octet-deny-all.json"
                cmp config/octet-deny-all.json "$TMPDIR/octet-deny-all.json"

                jq --exit-status '
                  .config.schema == "choregraph.octet-deny-all.v1"
                  and .config.mode == "deny-all"
                  and .config.octet_revision == "374bd16b26cee2af34211a29bfa531c016811f51"
                  and (.config.lints | length) > 0
                  and (.config.lints | length) == (.config.lints | unique | length)
                  and (.config.lints[0] == "unknown_lints")
                ' "$TMPDIR/octet-deny-all.json" > /dev/null

                expected_hook_hash="$(jq --raw-output '.config.hook_blake3' \
                  "$TMPDIR/octet-deny-all.json")"
                observed_hook_hash="$(b3sum ${octet}/hooks/octet-deny-all.sh)"
                test "''${observed_hook_hash%% *}" = "$expected_hook_hash"

                awk '$1 == "-D" { print $2 }' ${octet}/hooks/octet-deny-all.sh \
                  > "$TMPDIR/upstream-lints.txt"
                jq --raw-output '.config.lints[]' "$TMPDIR/octet-deny-all.json" \
                  > "$TMPDIR/local-lints.txt"
                diff --unified "$TMPDIR/upstream-lints.txt" "$TMPDIR/local-lints.txt"

                for fixture in \
                  fixtures/octet/negative/warn-mode.ncl \
                  fixtures/octet/negative/empty-lints.ncl \
                  fixtures/octet/negative/disabled-lint.ncl; do
                  if nickel export --format json "$fixture" \
                    > "$TMPDIR/invalid.json" 2> "$TMPDIR/invalid.stderr"; then
                    echo "invalid Octet policy fixture unexpectedly exported: $fixture" >&2
                    exit 1
                  fi
                done

                nickel export --format json fixtures/octet/negative/duplicate-lint.ncl \
                  > "$TMPDIR/duplicate.json"
                jq --exit-status '
                  (.config.lints | length) != (.config.lints | unique | length)
                ' "$TMPDIR/duplicate.json" > /dev/null

                grep --fixed-strings '[octet]' dylint.toml > /dev/null
                grep --fixed-strings 'excessive_file_length_max = 1100' dylint.toml > /dev/null
                grep --fixed-strings 'module_file_count_max = 9' dylint.toml > /dev/null
                if grep -E '^[[:space:]]*(disabled_lints|\[octet\.lint_levels\]|[a-z_]+[[:space:]]*=[[:space:]]*"warn")' dylint.toml; then
                  echo 'dylint.toml weakens the Octet deny-all policy' >&2
                  exit 1
                fi
                grep --fixed-strings 'default_scope = ["--workspace"]' Cargo.toml > /dev/null
                grep --fixed-strings 'cargo_check_args = ["--all-targets"]' Cargo.toml > /dev/null

                mkdir -p "$out"
                cp "$TMPDIR/octet-deny-all.json" "$out/policy.json"
                cp "$TMPDIR/local-lints.txt" "$out/lints.txt"
                printf '%s\n' 'non-claim=static-source-lint-observations-only' \
                  > "$out/non-claim.txt"
              '';

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

          # r[verify choregraph.epoch_fence.invariant]
          # r[verify choregraph.epoch_fence.decision]
          # r[verify choregraph.epoch_fence.proof]
          # r[verify choregraph.epoch_fence.verification]
          epoch-fence-verus =
            pkgs.runCommand "choregraph-epoch-fence-verus"
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

                octet-production-verus --crate-type=lib verus/proofs/epoch_fence.rs \
                  -o "$TMPDIR/epoch-fence.rlib" \
                  > "$TMPDIR/positive.stdout" 2> "$TMPDIR/positive.stderr"
                grep -E 'verification results:: [1-9][0-9]* verified, 0 errors' \
                  "$TMPDIR/positive.stdout" > /dev/null

                expected_negative_count=3
                observed_negative_count="$(find verus/epoch-fence-failure-fixtures \
                  -maxdepth 1 -type f -name '*.rs' | wc -l)"
                test "$observed_negative_count" -eq "$expected_negative_count"
                for fixture in verus/epoch-fence-failure-fixtures/*.rs; do
                  fixture_name="$(basename "$fixture")"
                  case "$fixture_name" in
                    cross_entity_admits.rs|stale_epoch_admits.rs|terminal_mutation_admits.rs)
                      expected='epoch_fence_decision_spec'
                      ;;
                    *)
                      echo "unclassified epoch-fence failure fixture: $fixture_name" >&2
                      exit 1
                      ;;
                  esac
                  set +e
                  octet-production-verus --crate-type=lib "$fixture" \
                    > "$TMPDIR/negative.stdout" 2> "$TMPDIR/negative.stderr"
                  negative_status="$?"
                  set -e
                  if [ "$negative_status" -eq 0 ]; then
                    echo "negative epoch-fence fixture unexpectedly passed: $fixture_name" >&2
                    exit 1
                  fi
                  grep -F 'postcondition not satisfied' "$TMPDIR/negative.stderr" > /dev/null
                  grep -F "$expected" "$TMPDIR/negative.stderr" > /dev/null
                done

                if grep -E '#\[verifier::external_body\]|assume_specification|\bassume[[:space:]]*\(|\baxiom\b' \
                  crates/choregraph-core/src/verified.rs; then
                  echo 'epoch-fence proof core gained an unrecorded trust escape' >&2
                  exit 1
                fi
                grep -F 'crate::verified::epoch_fence_decision(' \
                  crates/choregraph-core/src/ownership/mod.rs > /dev/null
                grep -F 'Some(crate::step(' \
                  crates/choregraph-core/src/ownership/mod.rs > /dev/null
                grep -F '../../crates/choregraph-core/src/verified.rs' \
                  verus/proofs/epoch_fence.rs > /dev/null
                if grep -E 'std::|SystemTime|Instant|TcpStream|UdpSocket|File::|Command::|Mutex|RwLock|OnceLock' \
                  crates/choregraph-core/src/ownership/mod.rs; then
                  echo 'epoch-fence core gained ambient I/O or shared mutable state' >&2
                  exit 1
                fi
                grep -E 'std::|Mutex|RwLock|OnceLock' \
                  fixtures/epoch-fence/negative/shared-store-coupling.rs > /dev/null

                nickel export --format json evidence/epoch-fence-proof.ncl \
                  > "$TMPDIR/evidence-raw.json"
                jq -S . "$TMPDIR/evidence-raw.json" > "$TMPDIR/evidence.json"
                diff --unified evidence/generated/epoch-fence-proof.json \
                  "$TMPDIR/evidence.json"
                expected_manifest_hash="$(cat evidence/generated/epoch-fence-proof.blake3)"
                observed_manifest_hash="$(b3sum evidence/generated/epoch-fence-proof.json)"
                test "''${observed_manifest_hash%% *}" = "$expected_manifest_hash"
                jq --argjson expected_negative_count "$expected_negative_count" --exit-status '
                  .schema == "choregraph.epoch-fence-proof.v1"
                  and .status == "verified"
                  and (.negative_fixtures | length) == $expected_negative_count
                  and (.sources | length) > 0
                  and (.non_claims | length) > 0
                  and .reference.revision == "553ae73f83c87c3f7c7a5f73c32c2211d9d7341f"
                  and .reference.license == "Apache-2.0"
                ' "$TMPDIR/evidence.json" > /dev/null
                jq --raw-output '.sources[] | [.path, .blake3] | @tsv' \
                  "$TMPDIR/evidence.json" \
                  | while IFS="$(printf '\t')" read -r source_path expected_hash; do
                      observed_hash="$(b3sum "$source_path")"
                      test "''${observed_hash%% *}" = "$expected_hash"
                    done

                mkdir -p "$out"
                cp "$TMPDIR/positive.stdout" "$out/positive.stdout"
                cp "$TMPDIR/evidence.json" "$out/evidence.json"
                printf '%s\n' \
                  'non-claim=supplied-epoch-decision-only-not-runtime-writer-authority' \
                  > "$out/non-claim.txt"
              '';

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
                    concrete_reference_mutation.rs) expected='result.payload_reference' ;;
                    denial_with_intent.rs) expected='result.intent_indices@.len()' ;;
                    premature_readiness.rs) expected='dependencies_complete_spec' ;;
                    state_map_mismatch.rs) expected='indexed_invariant_spec' ;;
                    suspended_mark_ready.rs) expected='lifecycle_target_spec' ;;
                    suspension_effect_intent.rs) expected='indexed_step_accepts_spec' ;;
                    suspension_resume_mismatch.rs) expected='indexed_step_accepts_spec' ;;
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
                  crates/choregraph-core/src/verified/topology.rs; then
                  echo 'single-step proof core gained an unrecorded trust escape' >&2
                  exit 1
                fi
                grep -F 'STATE_SUSPENDED' crates/choregraph-core/src/verified.rs > /dev/null
                grep -F 'ACTION_RESUME_SIGNAL' crates/choregraph-core/src/verified.rs > /dev/null
                grep -F 'suspensions_consistent_spec' crates/choregraph-core/src/verified.rs > /dev/null
                test "$(grep -F -c 'crate::verified::indexed_step(' \
                  crates/choregraph-core/src/transition.rs)" -eq 2
                grep -F 'crate::verified::lifecycle_target(' \
                  crates/choregraph-core/src/transition.rs > /dev/null
                test "$(grep -F -c 'crate::verified::concrete_binding_accepts(' \
                  crates/choregraph-core/src/concrete.rs)" -eq 2
                grep -F 'request_reference: request_binding.reference' \
                  crates/choregraph-core/src/concrete.rs > /dev/null
                grep -F 'payload_reference: payload_binding.reference' \
                  crates/choregraph-core/src/concrete.rs > /dev/null
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
                  and (.proof.theorems | length) == 13
                  and (.proof.negative_fixtures | length) == 10
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

          # r[verify choregraph.proof.trace.lemmas]
          # r[verify choregraph.proof.trace.preservation]
          # r[verify choregraph.proof.trace.determinism]
          # r[verify choregraph.proof.trace.failure]
          # r[verify choregraph.proof.trace.replay]
          # r[verify choregraph.proof.trace.evidence]
          # r[verify choregraph.proof.trace.validation]
          trace-replay-verus =
            pkgs.runCommand "choregraph-trace-replay-verus"
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

                octet-production-verus --crate-type=lib verus/proofs/trace_replay.rs \
                  -o "$TMPDIR/trace-replay.rlib" \
                  > "$TMPDIR/positive.stdout" 2> "$TMPDIR/positive.stderr"
                grep -E 'verification results:: [1-9][0-9]* verified, 0 errors' \
                  "$TMPDIR/positive.stdout" > /dev/null

                for fixture in verus/trace-failure-fixtures/*.rs; do
                  fixture_name="$(basename "$fixture")"
                  case "$fixture_name" in
                    empty_trace_mutation.rs) expected='replay_final_states_from_spec' ;;
                    failure_index_skips_denial.rs) expected='replay_failure_index_from_spec' ;;
                    preservation_without_initial_invariant.rs) expected='indexed_invariant_spec' ;;
                    replay_with_dispatch_intent.rs) expected='replay_dispatch_intents_spec' ;;
                    suspended_trace_drops_fact.rs) expected='replay_final_suspensions_from_spec' ;;
                    *) echo "unclassified trace failure fixture: $fixture_name" >&2; exit 1 ;;
                  esac
                  set +e
                  octet-production-verus --crate-type=lib "$fixture" \
                    > "$TMPDIR/negative.stdout" 2> "$TMPDIR/negative.stderr"
                  negative_status="$?"
                  set -e
                  if [ "$negative_status" -eq 0 ]; then
                    echo "negative trace fixture unexpectedly passed: $fixture_name" >&2
                    exit 1
                  fi
                  grep -F 'postcondition not satisfied' "$TMPDIR/negative.stderr" > /dev/null
                  grep -F "$expected" "$TMPDIR/negative.stderr" > /dev/null
                done

                if grep -R -E '#\[verifier::external_body\]|assume_specification|\bassume[[:space:]]*\(|\baxiom\b' \
                  crates/choregraph-core/src/verified.rs \
                  crates/choregraph-core/src/verified/trace.rs; then
                  echo 'trace proof core gained an unrecorded trust escape' >&2
                  exit 1
                fi
                grep -F 'match step(input.machine, &last_valid_state, event)' \
                  crates/choregraph-core/src/transition.rs > /dev/null
                grep -F 'Decision::Accept { next, intents: _ }' \
                  crates/choregraph-core/src/transition.rs > /dev/null
                grep -F 'match step_concrete(input.machine, &last_valid_state, event)' \
                  crates/choregraph-core/src/transition.rs > /dev/null
                grep -F 'ConcreteDecision::Accept { next, intents: _ }' \
                  crates/choregraph-core/src/transition.rs > /dev/null
                grep -F '../../crates/choregraph-core/src/verified.rs' \
                  verus/proofs/trace_replay.rs > /dev/null
                grep -F '../../crates/choregraph-core/src/verified/trace.rs' \
                  verus/proofs/trace_replay.rs > /dev/null
                if grep -A12 -F 'pub enum ReplayDecision' \
                  crates/choregraph-core/src/model.rs | grep -E 'intents|dispatch'; then
                  echo 'replay decision gained dispatch authority' >&2
                  exit 1
                fi
                if grep -F '(STATE_PENDING, ACTION_MARK_READY)' \
                  crates/choregraph-core/src/verified/trace.rs; then
                  echo 'trace proof regained an independent lifecycle table' >&2
                  exit 1
                fi

                nickel export --format json evidence/trace-replay-proof.ncl \
                  > "$TMPDIR/proof-manifest-raw.json"
                jq -S . "$TMPDIR/proof-manifest-raw.json" \
                  > "$TMPDIR/proof-manifest.json"
                diff --unified evidence/generated/trace-replay-proof.json \
                  "$TMPDIR/proof-manifest.json"
                expected_manifest_hash="$(cat evidence/generated/trace-replay-proof.blake3)"
                observed_manifest_hash="$(b3sum evidence/generated/trace-replay-proof.json)"
                test "''${observed_manifest_hash%% *}" = "$expected_manifest_hash"
                jq --exit-status '
                  .proof.schema == "choregraph.trace-replay-proof.v1"
                  and .proof.status == "verified"
                  and .proof.limits.max_trace_events == ${toString defaultMaxTraceEvents}
                  and .proof.limits.trace_binding_bytes == ${toString traceBindingBytes}
                  and (.proof.machine_schemas | length) == 2
                  and (.proof.theorems | length) == 8
                  and (.proof.negative_fixtures | length) == 5
                  and .proof.trusted_surface.project_trusted_declarations == []
                  and (.proof.non_claims | length) >= 5
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
                    forbidden-external-path.ncl|forbidden-unreviewed-cap-root.ncl)
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
                  choregraphPackage
                  pkgs.b3sum
                  pkgs.coreutils
                  pkgs.gnugrep
                  pkgs.jq
                  pkgs.nickel
                  rustToolchain
                ];
                src = self;
              }
              ''
                set -eu
                cd "$src"
                nickel typecheck contracts/nickel/restricted-graph-contracts.ncl
                nickel typecheck contracts/nickel/evaluated-graph-contract.ncl
                nickel export --format json evidence/cap-root-adoption.ncl \
                  > "$TMPDIR/cap-root-adoption.json"
                jq --exit-status '
                  .adoption.schema == "choregraph.cap-root-adoption.v1"
                  and .adoption.repository_rid == "rad:z3cWBCK7VPbEmbL5MAvKJWKjCE745"
                  and .adoption.revision == "8991e2572578c16e590c2f1c0915db6c46c5d9cb"
                  and .adoption.public_repository_available == true
                  and .adoption.exact_revision_clone_verified == true
                  and .adoption.cargo_source_admitted == true
                  and .adoption.nix_source_aligned == true
                  and .adoption.capability_shell_ready == true
                ' "$TMPDIR/cap-root-adoption.json" > /dev/null
                nickel export --format json vendor/cap-root-core-selected/SOURCE.ncl \
                  > "$TMPDIR/cap-root-source.json"
                expected_source_hash="$(jq --raw-output '.source.source_blake3' "$TMPDIR/cap-root-source.json")"
                observed_source_hash="$(b3sum vendor/cap-root-core-selected/src/lib.rs)"
                test "''${observed_source_hash%% *}" = "$expected_source_hash"
                expected_license_hash="$(jq --raw-output '.source.license_blake3' "$TMPDIR/cap-root-source.json")"
                observed_license_hash="$(b3sum vendor/cap-root-core-selected/LICENSE)"
                test "''${observed_license_hash%% *}" = "$expected_license_hash"
                rustc tools/sync-cap-root-core.rs -o "$TMPDIR/sync-cap-root-core"
                "$TMPDIR/sync-cap-root-core" --self-test
                grep --fixed-strings 'cap-root-core = { path = "vendor/cap-root-core-selected", version = "=0.1.0" }' Cargo.toml > /dev/null
                grep --fixed-strings 'cap-std = "=4.0.2"' Cargo.toml > /dev/null
                if grep -R -E 'std::(fs|net|process)|SystemTime|Command::new|env::' \
                  crates/choregraph-nickel/src --include='*.rs'; then
                  echo 'choregraph-nickel gained ambient authority' >&2
                  exit 1
                fi
                for fixture in fixtures/nickel/positive/*.ncl; do
                  fixture_name="$(basename "$fixture" .ncl)"
                  nickel export --format json \
                    --apply-contract contracts/nickel/evaluated-graph-contract.ncl \
                    "$fixture" > "$TMPDIR/raw.json"
                  jq -S . "$TMPDIR/raw.json" > "$TMPDIR/normalized.json"
                  cmp "fixtures/nickel/evaluated/$fixture_name.json" "$TMPDIR/normalized.json"
                done
                for fixture in fixtures/nickel/negative/*.ncl; do
                  if test "$(basename "$fixture")" = forbidden-runtime-import.ncl; then
                    continue
                  fi
                  if nickel export --format json \
                    --apply-contract contracts/nickel/evaluated-graph-contract.ncl \
                    "$fixture" > "$TMPDIR/negative.json"; then
                    echo "negative Nickel fixture unexpectedly exported: $fixture" >&2
                    exit 1
                  fi
                done

                mkdir -p "$TMPDIR/scratch" "$TMPDIR/output"
                evaluator="$(command -v nickel)"
                choregraph compile-nickel \
                  "$evaluator" \
                  "$src/fixtures/nickel" positive/minimal.ncl \
                  "$src/contracts/nickel" "$TMPDIR/scratch" \
                  "$TMPDIR/output" minimal.preserves \
                  > "$TMPDIR/minimal-report.txt"
                grep --fixed-strings 'status=compiled' "$TMPDIR/minimal-report.txt" > /dev/null
                test -s "$TMPDIR/output/minimal.preserves"

                choregraph compile-nickel \
                  "$evaluator" \
                  "$src/fixtures/nickel" positive/declared-import.ncl \
                  "$src/contracts/nickel" "$TMPDIR/scratch" \
                  "$TMPDIR/output" declared-import.preserves \
                  "source=$src/fixtures/nickel" \
                  > "$TMPDIR/import-report.txt"
                grep --fixed-strings 'imports=1' "$TMPDIR/import-report.txt" > /dev/null
                test -s "$TMPDIR/output/declared-import.preserves"

                if choregraph compile-nickel \
                  "$evaluator" \
                  "$src/fixtures/nickel" negative/forbidden-runtime-import.ncl \
                  "$src/contracts/nickel" "$TMPDIR/scratch" \
                  "$TMPDIR/output" forbidden.preserves \
                  > "$TMPDIR/forbidden.stdout" 2> "$TMPDIR/forbidden.stderr"; then
                  echo 'forbidden Nickel import unexpectedly compiled' >&2
                  exit 1
                fi
                grep --fixed-strings 'ForbiddenNickelAuthority' "$TMPDIR/forbidden.stderr" > /dev/null
                if grep --fixed-strings 'missing-declared-import.ncl' "$TMPDIR/forbidden.stderr"; then
                  echo 'safe diagnostic leaked source text' >&2
                  exit 1
                fi
                test ! -e "$TMPDIR/output/forbidden.preserves"

                if choregraph compile-nickel \
                  "$evaluator" \
                  "$src/fixtures/nickel" negative/path-escape.ncl \
                  "$src/contracts/nickel" "$TMPDIR/scratch" \
                  "$TMPDIR/output" path-escape.preserves \
                  > "$TMPDIR/path.stdout" 2> "$TMPDIR/path.stderr"; then
                  echo 'path-escape fixture unexpectedly compiled' >&2
                  exit 1
                fi
                grep --fixed-strings 'EvaluatorExecutionRejected' "$TMPDIR/path.stderr" > /dev/null
                if grep --fixed-strings '/home/operator/secret' "$TMPDIR/path.stderr"; then
                  echo 'safe diagnostic leaked a host path' >&2
                  exit 1
                fi
                test ! -e "$TMPDIR/output/path-escape.preserves"

                mkdir -p "$out"
                printf '%s\n' 'claim=capability-relative-bounded-frontend-not-nickel-semantics' > "$out/result.txt"
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
                expected_concrete_machine_schema_hash="$(jq --raw-output '.profile.concrete_machine_schema_source_blake3' "$TMPDIR/kamacite-corpus.json")"
                observed_concrete_machine_schema_hash="$(b3sum schemas/preserves/choregraph-machine-v2.preserves)"
                test "''${observed_concrete_machine_schema_hash%% *}" = "$expected_concrete_machine_schema_hash"
                expected_concrete_intent_schema_hash="$(jq --raw-output '.profile.concrete_intent_schema_source_blake3' "$TMPDIR/kamacite-corpus.json")"
                observed_concrete_intent_schema_hash="$(b3sum schemas/preserves/choregraph-concrete-intent-v2.preserves)"
                test "''${observed_concrete_intent_schema_hash%% *}" = "$expected_concrete_intent_schema_hash"
                expected_link_schema_hash="$(jq --raw-output '.profile.link_receipt_schema_source_blake3' "$TMPDIR/kamacite-corpus.json")"
                observed_link_schema_hash="$(b3sum schemas/preserves/choregraph-link-receipt-v1.preserves)"
                test "''${observed_link_schema_hash%% *}" = "$expected_link_schema_hash"
                if grep -R -E 'std::(fs|net|process)|SystemTime|Command::new|env::' \
                  crates/choregraph-kamacite/src --include='*.rs'; then
                  echo 'choregraph-kamacite gained ambient authority' >&2
                  exit 1
                fi
                jq --exit-status '
                  .project.status == "finite-trace-verified-indexed-kernel"
                  and .project.claim_scope == "named-single-step-and-finite-trace-properties-over-production-called-indexed-kernel"
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

                nickel export --format json vendor/bounded-exec-selected/SOURCE.ncl \
                  > "$TMPDIR/bounded-exec-source.json"
                expected_bounded_root_hash="$(jq --raw-output '.source.root_manifest_blake3' "$TMPDIR/bounded-exec-source.json")"
                observed_bounded_root_hash="$(b3sum vendor/bounded-exec-selected/Cargo.toml)"
                test "''${observed_bounded_root_hash%% *}" = "$expected_bounded_root_hash"
                expected_bounded_core_manifest_hash="$(jq --raw-output '.source.core_manifest_blake3' "$TMPDIR/bounded-exec-source.json")"
                observed_bounded_core_manifest_hash="$(b3sum vendor/bounded-exec-selected/crates/bounded-exec-core/Cargo.toml)"
                test "''${observed_bounded_core_manifest_hash%% *}" = "$expected_bounded_core_manifest_hash"
                expected_bounded_core_source_hash="$(jq --raw-output '.source.core_source_blake3' "$TMPDIR/bounded-exec-source.json")"
                observed_bounded_core_source_hash="$(b3sum vendor/bounded-exec-selected/crates/bounded-exec-core/src/lib.rs)"
                test "''${observed_bounded_core_source_hash%% *}" = "$expected_bounded_core_source_hash"
                expected_bounded_shell_manifest_hash="$(jq --raw-output '.source.shell_manifest_blake3' "$TMPDIR/bounded-exec-source.json")"
                observed_bounded_shell_manifest_hash="$(b3sum vendor/bounded-exec-selected/crates/bounded-exec/Cargo.toml)"
                test "''${observed_bounded_shell_manifest_hash%% *}" = "$expected_bounded_shell_manifest_hash"
                expected_bounded_shell_source_hash="$(jq --raw-output '.source.shell_source_blake3' "$TMPDIR/bounded-exec-source.json")"
                observed_bounded_shell_source_hash="$(b3sum vendor/bounded-exec-selected/crates/bounded-exec/src/lib.rs)"
                test "''${observed_bounded_shell_source_hash%% *}" = "$expected_bounded_shell_source_hash"
                expected_bounded_license_hash="$(jq --raw-output '.source.license_blake3' "$TMPDIR/bounded-exec-source.json")"
                observed_bounded_license_hash="$(b3sum vendor/bounded-exec-selected/LICENSE)"
                test "''${observed_bounded_license_hash%% *}" = "$expected_bounded_license_hash"
                rustc tools/sync-bounded-exec.rs -o "$TMPDIR/sync-bounded-exec"
                "$TMPDIR/sync-bounded-exec" --self-test
                grep --fixed-strings 'bounded-exec = { path = "vendor/bounded-exec-selected/crates/bounded-exec", version = "=0.1.0" }' \
                  Cargo.toml > /dev/null

                nickel export --format json vendor/execution-graph-core-selected/SOURCE.ncl \
                  > "$TMPDIR/execution-graph-source.json"
                expected_manifest_hash="$(jq --raw-output '.source.manifest_blake3' "$TMPDIR/execution-graph-source.json")"
                observed_manifest_hash="$(b3sum vendor/execution-graph-core-selected/Cargo.toml)"
                test "''${observed_manifest_hash%% *}" = "$expected_manifest_hash"
                expected_oracle_hash="$(jq --raw-output '.source.source_blake3' "$TMPDIR/execution-graph-source.json")"
                observed_oracle_hash="$(b3sum vendor/execution-graph-core-selected/src/lib.rs)"
                test "''${observed_oracle_hash%% *}" = "$expected_oracle_hash"
                expected_oracle_license_hash="$(jq --raw-output '.source.license_blake3' "$TMPDIR/execution-graph-source.json")"
                observed_oracle_license_hash="$(b3sum vendor/execution-graph-core-selected/LICENSE)"
                test "''${observed_oracle_license_hash%% *}" = "$expected_oracle_license_hash"
                rustc tools/sync-execution-graph-oracle.rs -o "$TMPDIR/sync-execution-graph-oracle"
                "$TMPDIR/sync-execution-graph-oracle" --self-test
                grep --fixed-strings 'legacy-execution-graph = { package = "execution-graph-core", path = "../../vendor/execution-graph-core-selected", version = "=0.1.0" }' \
                  crates/choregraph-seaglass/Cargo.toml > /dev/null

                jq --exit-status '
                  (.baseline.sources | map(.id)) as $ids
                  | .baseline.schema == "choregraph.source-baseline.v1"
                  and ($ids == ($ids | sort))
                  and (($ids | length) == ($ids | unique | length))
                  and ([.baseline.sources[].working_tree_used] | all(. == false))
                  and ([.baseline.sources[] | select(.imported == true) | .id] == ["bounded-exec-process-mechanism", "cap-root-lexical-boundary", "execution-graph-parity-oracle", "kamacite-interchange-boundary", "trellis-proof-reference"])
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
                observed_codec_hash="$(b3sum vendor/kamacite-core-selected/preserves/codec.rs)"
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
            pkgs.wasm-tools
            octetDenyAllRunner
            productionVerus
            rustToolchain
            verusToolchainProfile
          ];
        };

        formatter = pkgs.nixfmt;
      }
    );
}
