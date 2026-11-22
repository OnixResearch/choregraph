"""Ratchet Octet source suppressions by file, scope, level, and lint."""

from collections import Counter
import re

from proof_audit import read_regular, require
from rust_tokens import AuditError, attr_at, comma_parts, lex, values

LEVELS = {"allow", "expect", "warn"}
BLANKET = {"warnings", "octet", "octet::all", "clippy::all", "clippy::restriction",
           "clippy::pedantic", "clippy::nursery", "clippy::cargo"}
LINT_PATH = re.compile(r"(?:[a-zA-Z_][a-zA-Z_0-9]*)(?:::[a-zA-Z_][a-zA-Z_0-9]*)*\Z")


def suppressions(meta, scope, lints):
    words = values(meta)
    if words[:1] == ("cfg_attr",):
        require(words == ("cfg_attr", "("), "malformed cfg_attr")
        parts = list(comma_parts(meta[1].children))
        require(len(parts) >= 2, "cfg_attr requires a condition and at least one attribute")
        for part in parts[1:]:
            yield from suppressions(part, "conditional-" + scope, lints)
    elif words and words[0] in LEVELS:
        require(len(words) == 2 and words[1] == "(", f"malformed {words[0]} attribute")
        for part in comma_parts(meta[1].children):
            pieces = values(part)
            if len(pieces) == 3 and pieces[:2] == ("reason", "="):
                continue
            lint = "".join(pieces)
            require(LINT_PATH.fullmatch(lint), f"malformed lint name: {lint!r}")
            require(lint not in BLANKET, f"blanket lint suppression is forbidden: {lint}")
            bare = lint.removeprefix("octet::")
            if bare in lints or lint.startswith("octet::"):
                yield f"{scope} {words[0]} {bare}"


def source_exemptions(tokens, lints):
    counts = Counter()

    def walk(values_here, *, at_file=False, in_verus=False):
        index = 0
        while index < len(values_here):
            token = values_here[index]
            attribute = attr_at(values_here, index)
            if attribute is not None:
                inner, group, index = attribute
                scope = "file" if inner and at_file else "block" if inner else "item"
                if in_verus:
                    scope = "verus-" + scope
                try:
                    counts.update(suppressions(group.children, scope, lints))
                except AuditError as error:
                    raise AuditError(f"line {token.line}: {error}") from error
                continue
            if token.text == "verus" and values(values_here[index + 1:index + 3]) == ("!", "{"):
                walk(values_here[index + 2].children, in_verus=True)
                index += 3
                continue
            if token.children:
                walk(token.children, in_verus=in_verus)
            index += 1

    walk(tokens, at_file=True)
    return counts


def rust_sources(root):
    # Own-source trees, including build scripts, tests, examples and standalone
    # Rust tooling. Vendored upstream and intentionally false proof fixtures
    # are not first-party Octet qualification inputs.
    for name in ("crates", "tools", "verus/proofs"):
        require((root / name).is_dir(), f"missing Octet source tree: {name}")
        pending = [root / name]
        while pending:
            directory = pending.pop()
            require(not directory.is_symlink(), f"unexpected Octet source directory symlink: {directory}")
            for path in sorted(directory.iterdir()):
                if path.is_dir():
                    if path.name not in {"target", ".git"}:
                        pending.append(path)
                elif path.suffix == ".rs":
                    yield path.relative_to(root).as_posix()


def inventory(root, lints):
    observed = {}
    file_count = 0
    for path in rust_sources(root):
        file_count += 1
        try:
            result = source_exemptions(lex(read_regular(root, path)), lints)
        except AuditError as error:
            raise AuditError(f"{path}: {error}") from error
        if result:
            observed[path] = dict(result)
    return observed, file_count


def verify_baseline(observed, baseline):
    changes = []
    for path in sorted(set(observed) | set(baseline)):
        old, new = baseline.get(path, {}), observed.get(path, {})
        for key in sorted(set(old) | set(new)):
            pinned, found = old.get(key, 0), new.get(key, 0)
            if pinned != found:
                action = "policy review required" if found > pinned else "lower the baseline"
                changes.append(f"{path}: {key}: {pinned} -> {found} ({action})")
    require(not changes, "Octet source exemptions differ from the ratchet:\n" + "\n".join(changes))


def audit_octet(root, lints, baseline):
    observed, scanned = inventory(root, lints)
    verify_baseline(observed, baseline)
    file_scope = [(path, key, count) for path, entries in observed.items()
                  for key, count in entries.items() if key.split()[0].endswith("file")]
    item_scope = [(path, key, count) for path, entries in observed.items()
                  for key, count in entries.items() if not key.split()[0].endswith("file")]
    print(f"octet-audit: {scanned} scanned Rust files, {len(observed)} files with exemptions; "
          f"{sum(count for _, _, count in file_scope)} file-scope lint occurrences in "
          f"{len({path for path, _, _ in file_scope})} files; "
          f"{sum(count for _, _, count in item_scope)} item/block occurrences in "
          f"{len({path for path, _, _ in item_scope})} files")
