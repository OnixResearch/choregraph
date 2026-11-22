"""Small Rust/Verus token-tree reader for qualification policy (not a type checker)."""

from dataclasses import dataclass
import re


@dataclass(frozen=True)
class Token:
    text: str
    line: int
    children: tuple = ()


class AuditError(ValueError):
    pass


# A lifetime is not a character literal. Keep literal tokens opaque, including
# byte/C strings, raw forms with hashes, and escaped character literals.
LEXEME = re.compile(
    r'\s+|//[^\n]*|/\*|(?:br|cr|r)(?P<hashes>\#*)"'
    r'|(?:b|c)?"(?:[^"\\]|\\[\s\S])*"'
    r"|b?'(?:[^'\\\r\n]|\\(?:u\{[\da-fA-F_]+\}|x[\da-fA-F]{2}|[^\r\n]))'"
    r'|(?:r\#)?[^\W\d]\w*|[0-9]\w*|.',
    re.DOTALL,
)
CLOSERS = {"(": ")", "[": "]", "{": "}"}


def lex(source):
    offset = 0
    line = 1
    levels = [([], "", line)]
    while offset < len(source):
        match = LEXEME.match(source, offset)
        text = match[0]
        end = match.end()
        if text == "/*":
            depth = 1
            for marker in re.finditer(r'/\*|\*/', source[end:]):
                depth += 1 if marker[0] == "/*" else -1
                if depth == 0:
                    end += marker.end()
                    break
            else:
                raise AuditError(f"line {line}: unterminated block comment")
        elif match["hashes"] is not None:
            suffix = '"' + match["hashes"]
            closing = source.find(suffix, end)
            if closing < 0:
                raise AuditError(f"line {line}: unterminated raw string")
            end = closing + len(suffix)
            levels[-1][0].append(Token(source[offset:end], line))
        elif text in CLOSERS:
            levels.append(([], text, line))
        elif text in CLOSERS.values():
            children, opener, first_line = levels[-1]
            if CLOSERS.get(opener) != text:
                raise AuditError(f"line {line}: mismatched delimiter {text}")
            levels.pop()
            levels[-1][0].append(Token(opener, first_line, tuple(children)))
        elif text == '"':
            raise AuditError(f"line {line}: unterminated string")
        elif not (text.isspace() or text.startswith("//")):
            levels[-1][0].append(Token(text.removeprefix("r#"), line))
        line += source.count("\n", offset, end)
        offset = end
    if len(levels) != 1:
        raise AuditError(f"line {levels[-1][2]}: unclosed delimiter")
    return tuple(levels[0][0])


def values(tokens):
    return tuple(token.text for token in tokens)


def attr_at(tokens, index):
    """Return (is_inner, bracket_group, next_index) for Rust attributes only."""
    if tokens[index].text != "#":
        return None
    at = index + 1
    inner = values(tokens[at:at + 1]) == ("!",)
    at += int(inner)
    if values(tokens[at:at + 1]) == ("[",):
        return inner, tokens[at], at + 1
    return None


def comma_parts(tokens):
    start = 0
    for index, token in enumerate(tokens):
        if token.text == ",":
            yield tokens[start:index]
            start = index + 1
    if start < len(tokens):
        yield tokens[start:]
