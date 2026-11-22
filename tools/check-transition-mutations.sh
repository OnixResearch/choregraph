#!/usr/bin/env bash
# Run from the source root after the unmodified single-step proof passes.
set -euo pipefail

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
for mutation in cancel-completes mark-ready-ignores-dependencies; do
    root="$work/$mutation"
    mkdir -p "$root/crates/choregraph-core" "$root/verus"
    cp -R crates/choregraph-core/src "$root/crates/choregraph-core/"
    cp -R verus/proofs "$root/verus/"
    chmod -R u+w "$root"
    patch --batch --fuzz=0 -p1 -d "$root" < "verus/mutations/$mutation.patch"
    if (cd "$root" && octet-production-verus --triggers-mode silent --crate-type=lib \
        verus/proofs/single_step.rs -o "$root/mutant.rlib") > "$root/verus.log" 2>&1; then
        printf '%s verified despite changing its independent obligation\n' "$mutation" >&2
        exit 1
    fi
    case "$mutation" in
        cancel-completes)
            obligation='ensures lifecycle_target_spec(profile, from, ACTION_CANCEL) != Some(STATE_COMPLETED)'
            ;;
        mark-ready-ignores-dependencies)
            obligation='pub proof fn mark_ready_acceptance_is_exact('
            ;;
    esac
    if ! grep -E '^verification results:: [0-9]+ verified, [1-9][0-9]* errors$' "$root/verus.log" \
        || ! grep -F "$obligation" "$root/verus.log"; then
        cat "$root/verus.log" >&2
        printf '%s failed for the wrong reason\n' "$mutation" >&2
        exit 1
    fi
done
