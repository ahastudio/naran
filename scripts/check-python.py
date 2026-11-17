"""Checks that every function of the Python scripts has at most five lines of code.

The same shape as the Rust rule of `tools/quality`: a longer function is split
by extracting methods. Blank lines, comments and the docstring do not count.
Usage: python3 -I scripts/check-python.py
"""

import ast
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
FOLDERS = ["scripts", "release"]
LIMIT = 5


def sources():
    return sorted(path for folder in FOLDERS for path in (ROOT / folder).glob("**/*.py"))


def without_docstring(node):
    """The statements of a function body, leaving out its docstring."""
    first = node.body[0]
    is_text = isinstance(first, ast.Expr) and isinstance(getattr(first, "value", None), ast.Constant)
    return node.body[1:] if is_text and isinstance(first.value.value, str) else node.body


def code_lines(source, node):
    body = without_docstring(node)
    if not body:
        return 0
    lines = source.split("\n")[body[0].lineno - 1 : body[-1].end_lineno]
    return sum(1 for line in lines if line.strip() and not line.strip().startswith("#"))


def functions(tree):
    return [n for n in ast.walk(tree) if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef))]


def problems(path):
    source = path.read_text(encoding="utf-8")
    sizes = [(node, code_lines(source, node)) for node in functions(ast.parse(source))]
    return [f"{path.relative_to(ROOT)}:{n.lineno}: {n.name} has {c} lines" for n, c in sizes if c > LIMIT]


def main():
    found = [line for path in sources() for line in problems(path)]
    for line in found:
        print(line, file=sys.stderr)
    print(f"{len(found)} functions have more than {LIMIT} lines." if found else "Every function has at most five lines.")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
