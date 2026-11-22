"""Run the proof identity/trust or workspace Octet source policy audit."""

import argparse
import json
from pathlib import Path
import sys

import octet_audit
import proof_audit
from rust_tokens import AuditError


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=Path)
    subparsers = parser.add_subparsers(dest="audit", required=True)
    proof = subparsers.add_parser("proof")
    proof.add_argument("rail", choices=proof_audit.ROOTS)
    proof.add_argument("--manifest", required=True, type=Path)
    proof.add_argument("--stdout", required=True, type=Path)
    subparsers.add_parser("octet")
    args = parser.parse_args()
    root = args.root.absolute()
    policy = Path(__file__).parent

    if args.audit == "proof":
        manifest = read_json(args.manifest)
        if args.rail != "epoch-fence":
            manifest = manifest["proof"]
        proof_audit.audit_rail(root, args.rail, manifest, args.stdout.read_text(encoding="utf-8"),
                               read_json(policy / "proof-policy.json"))
    else:
        lints = read_json(root / "config/octet-deny-all.json")["config"]["lints"]
        octet_audit.audit_octet(root, set(lints), read_json(policy / "octet-exemptions.json"))


if __name__ == "__main__":
    try:
        main()
    except (AuditError, OSError, ValueError, KeyError, TypeError) as error:
        print(f"qualification: {error}", file=sys.stderr)
        sys.exit(1)
