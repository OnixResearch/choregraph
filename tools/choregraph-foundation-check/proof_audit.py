"""Audit the local positive Verus source closure and its advertised identities."""

from collections import Counter
import posixpath
import re

from rust_tokens import AuditError, attr_at, lex, values

CORE = "crates/choregraph-core/src/verified"
ROOTS = {
    "epoch-fence": "verus/proofs/epoch_fence.rs",
    "single-step": "verus/proofs/single_step.rs",
    "trace-replay": "verus/proofs/trace_replay.rs",
}
TEST_MODULES = {CORE + ".rs", CORE + "/topology.rs"}

# No trust-changing verifier attributes are permitted. No verifier::* attribute
# is needed today. The only verifier annotation in use is the trigger syntax.
VERIFIER_ATTRIBUTES = frozenset()
FORBIDDEN = {
    "admit", "assume", "assume_specification", "axiom", "external_fn_specification",
    "fn_spec", "macro_rules", "uninterp", "unsafe",
}
TEST_MACROS = {"assert", "assert_eq", "vec"}
IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*\Z")


def require(condition, detail):
    if not condition:
        raise AuditError(detail)


def core_file(path):
    return path == CORE + ".rs" or (path.startswith(CORE + "/") and path.endswith(".rs"))


def local_target(source, literal):
    # Existing path attributes are ordinary ASCII literals. Forbid escapes, raw
    # literals, absolute paths and external roots rather than reinterpret them.
    require(re.fullmatch(r'"[a-zA-Z0-9_./-]+\.rs"', literal), "expected a plain Rust module path")
    path = literal[1:-1]
    require(not path.startswith("/"), "absolute proof module path")
    target = posixpath.normpath(posixpath.join(posixpath.dirname(source), path))
    require(core_file(target), f"proof module outside the reviewed core: {target}")
    return target


def read_regular(root, relative):
    path = root / relative
    for part in (path, *path.parents):
        if part == root:
            break
        require(not part.is_symlink(), f"unexpected source symlink: {part}")
    return path.read_text(encoding="utf-8")


def proof_attr(inner, attr, in_verus, in_tests):
    words = values(attr.children)
    head = words[:1]
    if words[1:3] == (":", ":"):
        require(words in VERIFIER_ATTRIBUTES, "unreviewed qualified proof attribute: " + " ".join(words))
    elif head == ("trigger",):
        require(in_verus and (len(words) > 1 if inner else len(words) == 1), "malformed Verus trigger")
    elif words == ("no_std",):
        require(inner and not in_verus, "misplaced no_std attribute")
    elif words == ("test",):
        require(not inner and in_tests, "test attribute outside a reviewed test module")
    elif head in {("allow",), ("derive",)}:
        require(words in {("allow", "("), ("derive", "(")}, "malformed proof attribute")
        if head == ("derive",):
            derives = values(attr.children[1].children)
            require(not inner and set(derives) <= {",", "Clone", "Copy", "Debug", "Eq", "PartialEq"},
                    "unreviewed proof derive")
    else:
        raise AuditError("unreviewed proof attribute: " + " ".join(words))


def module_after(tokens, index):
    """Read an optional visibility then `mod NAME` and its body marker."""
    if values(tokens[index:index + 1]) == ("pub",):
        index += 1
        if values(tokens[index:index + 1]) == ("(",):
            index += 1
    if values(tokens[index:index + 1]) == ("mod",):
        tail = tokens[index + 1:index + 3]
        if len(tail) == 2 and IDENTIFIER.fullmatch(tail[0].text):
            return index + 3, tail[0].text, tail[1]
    raise AuditError("expected attribute immediately attached to a module declaration")


def inspect_source(source, tokens):
    """Return explicit module edges and checked functions, or reject the source."""
    children = {}
    functions = {}

    def walk(tokens, *, in_verus=False, in_tests=False, item_scope=True):
        index = 0
        while index < len(tokens):
            token = tokens[index]
            text = token.text
            try:
                attribute = attr_at(tokens, index)
                if attribute is not None:
                    inner, attr, index = attribute
                    words = values(attr.children)
                    if words[:1] == ("path",):
                        require(item_scope and not in_verus and not in_tests and not inner
                                and len(words) == 3 and words[1] == "=", "malformed proof path attribute")
                        index, name, body = module_after(tokens, index)
                        require(body.text == ";" and name not in children, "invalid proof path module")
                        children[name] = local_target(source, words[2])
                    elif words == ("cfg", "(") and values(attr.children[1].children) == ("test",):
                        require(source in TEST_MODULES and not in_tests and not in_verus
                                and item_scope and not inner, "unreviewed conditional proof scope")
                        index, name, body = module_after(tokens, index)
                        require(name == "tests" and body.text == "{", "only the local tests module may use cfg(test)")
                        walk(body.children, in_tests=True, item_scope=False)
                    else:
                        proof_attr(inner, attr, in_verus, in_tests)
                        # Trigger expressions are real proof tokens, not opaque metadata.
                        if words[:1] == ("trigger",):
                            walk(attr.children[1:], in_verus=True, item_scope=False)
                    continue

                require(text not in FORBIDDEN, f"forbidden proof trust token: {text}")
                tail = tokens[index + 1:index + 3]
                if len(tail) == 2 and tail[0].text == "!" and tail[1].text in {"{", "[", "("}:
                    if text == "verus":
                        require(item_scope and not in_verus and not in_tests and tail[1].text == "{"
                                and (index == 0 or tokens[index - 1].text != ":"), "unreviewed verus! scope")
                        walk(tail[1].children, in_verus=True)
                        index += 3
                        continue
                    if IDENTIFIER.fullmatch(text):
                        require((in_tests and text in TEST_MACROS) or (in_verus and text == "seq"),
                                f"unreviewed proof macro: {text}!")
                if text == "mod":
                    raise AuditError("proof modules require an audited literal path")
                if text == "use" and not in_tests:
                    imported = []
                    for part in tokens[index + 1:]:
                        if part.text == ";":
                            break
                        imported.append(part)
                    path = values(imported)
                    prelude = ("vstd", ":", ":", "prelude", ":", ":", "*")
                    local = (path[:3] == ("super", ":", ":")
                             or path[:6] == ("crate", ":", ":", "verified", ":", ":")
                             or (source == ROOTS["epoch-fence"] and path[:3] == ("verified", ":", ":")))
                    require(item_scope and not in_verus and (path == prelude or local), "unreviewed proof import")
                    candidates = list(imported)
                    while candidates:
                        part = candidates.pop()
                        require(part.text not in {"as", "verus"}, "proof import alias or macro shadow")
                        candidates.extend(part.children)
                if text == "extern":
                    require(item_scope and not in_verus and not in_tests
                            and values(tokens[index:index + 4]) == ("extern", "crate", "alloc", ";"),
                            "only the pinned alloc infrastructure may be linked here")
                if text == "fn" and not in_tests:
                    require(in_verus and item_scope, "function outside a checked top-level verus! item scope")
                    name = tokens[index + 1].text if tail else ""
                    require(IDENTIFIER.fullmatch(name) and name not in functions, "invalid or repeated proof function name")
                    mode = tokens[index - 1].text
                    mode = mode if mode in {"proof", "spec"} else "exec"
                    # Maintained functions use no struct-literal brace groups
                    # in the item header before `ensures`; the Verus parser still
                    # independently checks the contracts and bodies.
                    header = []
                    for part in tokens[index + 2:]:
                        if part.text in {"{", ";", "fn"}:
                            break
                        header.append(part.text)
                    require(mode == "spec" or "ensures" in header,
                            f"checked {mode} function {name} must declare an ensures clause")
                    functions[name] = mode
                if token.children:
                    walk(token.children, in_verus=in_verus, in_tests=in_tests, item_scope=False)
            except AuditError as error:
                raise AuditError(f"{source}:{token.line}: {error}") from error
            index += 1

    walk(tokens)
    return children, functions


def inventory(root):
    sources = set(ROOTS.values()) | {CORE + ".rs"}
    sources.update(path.relative_to(root).as_posix() for path in (root / CORE).rglob("*.rs"))
    return {path: inspect_source(path, lex(read_regular(root, path))) for path in sorted(sources)}


def rail_closure(parsed, rail):
    visited = set()
    functions = {}

    def visit(path, prefix, ancestors):
        require(path in parsed, f"missing proof source: {path}")
        require(path not in ancestors, f"cycle in proof modules: {path}")
        visited.add(path)
        modules, items = parsed[path]
        for name, mode in items.items():
            full_name = f"{prefix}::{name}"
            require(full_name not in functions, f"duplicate checked item: {full_name}")
            functions[full_name] = mode
        for name, target in modules.items():
            visit(target, f"{prefix}::{name}", ancestors | {path})

    visit(ROOTS[rail], "crate", set())
    return visited, functions


def verify_inventory(parsed, pins):
    observed = {path: {"modules": modules, "functions": functions}
                for path, (modules, functions) in parsed.items()}
    for path in sorted(set(pins) | set(observed)):
        require(pins.get(path) == observed.get(path),
                f"reviewed module/function inventory changed in {path}: pinned={pins.get(path)}, observed={observed.get(path)}")
    reached = set().union(*(rail_closure(parsed, rail)[0] for rail in ROOTS))
    require(reached == set(parsed), f"proof files not reached by any maintained rail: {sorted(set(parsed) - reached)}")


def verify_theorems(manifest, mapping, functions):
    labels = manifest.get("theorems")
    require(isinstance(labels, list) and all(isinstance(label, str) for label in labels),
            "expected a list of theorem labels")
    require(Counter(labels) == Counter(mapping.keys()),
            f"advertised theorem labels changed: missing={sorted(set(mapping) - set(labels))}, "
            f"extra={sorted(set(labels) - set(mapping))}; duplicates are not permitted")
    for label, items in mapping.items():
        require(bool(items), f"theorem label has no checked items: {label}")
        for item, mode in items.items():
            require(mode in {"exec", "proof"} and functions.get(item) == mode,
                    f"{label}: expected checked {mode} item {item} in this rail's source closure")


def verify_summary(manifest, stdout):
    expected = manifest.get("verified_count")
    require(type(expected) is int and expected > 0, "verified_count must be a positive integer")
    summaries = [line for line in stdout.splitlines() if line.startswith("verification results::")]
    require(summaries == [f"verification results:: {expected} verified, 0 errors"],
            f"expected exactly {expected} verified and 0 errors; summary was {summaries!r}")


def audit_rail(root, rail, manifest, stdout, policy):
    parsed = inventory(root)
    verify_inventory(parsed, policy["sources"])
    visited, functions = rail_closure(parsed, rail)
    evidenced = Counter(item["path"] for item in manifest["sources"])
    require(all(evidenced[path] == 1 for path in visited),
            f"proof closure paths must each have one manifest digest: {sorted(path for path in visited if evidenced[path] != 1)}")
    verify_theorems(manifest, policy["theorems"][rail], functions)
    verify_summary(manifest, stdout)
    print(f"proof-audit: {rail}: {len(visited)} files, {len(functions)} named functions, "
          f"{len(manifest['theorems'])} advertised labels; Verus verified_count={manifest['verified_count']}")
