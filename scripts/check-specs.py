"""Checks the feature documents in `specs/` against the FBPW templates.

A document that no longer matches the code misleads the next reader, so this
runs in `mise run check`. It reports every problem it finds and exits with 1
when there is any. Usage: python3 -I scripts/check-specs.py
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
FILES = ["README", "spec", "plan", "tasks", "findings", "progress"]
HEADINGS = {
    "README": ["Background", "Goal", "How it works", "Related Documents"],
    "spec": [
        "Overview",
        "User Scenarios & Testing (mandatory)",
        "Functional Requirements (mandatory)",
        "Constraints (mandatory)",
        "Success Criteria (mandatory)",
    ],
    "plan": [
        "Summary",
        "Requirements",
        "Critical Files",
        "Architecture",
        "Implementation Steps",
        "Verification",
        "Considerations",
    ],
    "tasks": ["Goal", "Current Phase", "Phases", "Notes"],
    "findings": [
        "Requirements",
        "Research Findings",
        "Resources",
        "Technical Decisions",
        "Issues Encountered",
        "Learnings",
    ],
    "progress": ["Session 2026-11-16", "Test Results", "Error Log", "5-Question Reboot Check"],
}
PHASES = [
    "Phase 1: Requirements & Discovery",
    "Phase 2: Planning & Structure",
    "Phase 3: Implementation",
    "Phase 4: Testing",
]
BANNED = ["backfill", "written afterward", "after the fact", "not run"]
ALLOWED_FENCES = {"text", "bash", ""}
SCENARIO = r"Scenario \d+:"


def read_text(path):
    return path.read_text(encoding="utf-8")


def read_lines(path):
    return read_text(path).split("\n")


def outside_code(lines):
    """The lines that are not inside a code fence, with their numbers."""
    inside = False
    for number, line in enumerate(lines, 1):
        inside ^= line.startswith("```")
        if not inside and not line.startswith("```"):
            yield number, line


def headings(lines, level):
    prefix = "#" * level + " "
    return [line[len(prefix):] for _, line in outside_code(lines) if line.startswith(prefix)]


def test_names():
    """Every test function name of the repository."""
    names = set()
    for path in ROOT.glob("**/*.rs"):
        if "target" not in path.parts:
            names.update(re.findall(r"fn (\w+)\(", read_text(path)))
    return names


def undated(titles):
    return [re.sub(r" \d{4}-\d\d-\d\d$", "", title) for title in titles]


def sections(path, name):
    """The problem with the sections of one document, if any."""
    got, want = undated(headings(read_lines(path), 2)), undated(HEADINGS[name])
    return [] if got == want else [f"{path}: sections {got}, expected {want}"]


def check_structure(folder):
    paths = {name: folder / f"{name}.md" for name in FILES}
    missing = [f"{path}: missing" for path in paths.values() if not path.exists()]
    present = [(path, name) for name, path in paths.items() if path.exists()]
    return missing + [found for path, name in present for found in sections(path, name)]


def phases(path):
    got = [re.sub(r" [^\w ]+$", "", title) for title in headings(read_lines(path), 3)]
    shown = [title for title in got if title.startswith("Phase ")]
    return [] if shown == PHASES else [f"{path}: phases {shown}, expected {PHASES}"]


def check_phases(folder):
    return [found for name in ("tasks", "progress") for found in phases(folder / f"{name}.md")]


def banned_in(path):
    text = read_text(path).lower()
    return [f"{path}: contains '{word}'" for word in BANNED if word in text]


def check_banned_words(folder):
    return [found for path in sorted(folder.glob("*.md")) for found in banned_in(path)]


def code_blocks(path, lines):
    fences = [(n, line[3:].strip()) for n, line in enumerate(lines, 1) if line.startswith("```")]
    return [f"{path}:{n}: code block '{kind}'" for n, kind in fences if kind not in ALLOWED_FENCES]


def is_identifier(span):
    return "::" in span or re.search(r"\w\(", span) is not None


def identifiers(path, lines):
    spans = [(n, s) for n, line in outside_code(lines) for s in re.findall(r"`([^`]+)`", line)]
    return [f"{path}:{n}: identifier `{span}`" for n, span in spans if is_identifier(span)]


def step_texts(plan):
    """The text of each '### Step N:' of a plan, with its number."""
    found = re.finditer(r"### Step (\d+): .*?(?=\n### |\n## |\Z)", plan, re.S)
    return [(int(m.group(1)), m.group(0)) for m in found]


def check_plan_steps(folder):
    path = folder / "plan.md"
    numbers = [number for number, _ in step_texts(read_text(path))]
    ok = numbers == list(range(len(numbers))) and "Acceptance Tests" in read_text(path).split("### Step 1")[0]
    return [] if ok else [f"{path}: steps {numbers}, expected Step 0 (Acceptance Tests) then 1, 2, ..."]


def spec_tests(path):
    """Every test name that the Test lines of a spec give."""
    lines = [line for line in read_lines(path) if line.startswith("- Test:")]
    return [n.split("::")[-1] for line in lines for n in re.findall(r"`([\w:]+)`", line.split(" in `")[0])]


def names_test(text, name):
    """Whether the text gives the test by its whole name, in backticks."""
    return re.search(rf"`(?:[\w]+::)*{re.escape(name)}`", text) is not None


def check_plan_covers_tests(folder):
    plan = read_text(folder / "plan.md")
    missing = sorted({n for n in spec_tests(folder / "spec.md") if not names_test(plan, n)})
    return [f"{folder / 'plan.md'}: test '{name}' is in no step" for name in missing]


def check_plan_has_no_code(folder):
    path = folder / "plan.md"
    lines = read_lines(path)
    return code_blocks(path, lines) + identifiers(path, lines)


def block_body(lines, start):
    """The lines after a scenario up to the next heading or scenario."""
    rest = lines[start + 1:]
    ends = (i for i, line in enumerate(rest) if line.startswith("#") or re.match(SCENARIO, line))
    return rest[: next(ends, len(rest))]


def scenario_blocks(lines):
    """Each 'Scenario N:' line with its number, its text and the lines that follow it."""
    starts = [n for n, line in enumerate(lines) if re.match(SCENARIO, line)]
    return [(n + 1, lines[n], block_body(lines, n)) for n in starts]


def unknown_tests(line, names):
    """The test names a Test line gives that no test of the repository has."""
    found = re.findall(r"`([\w:]+)`", line.split(" in `")[0])
    return [name for name in (n.split("::")[-1] for n in found) if name not in names]


def scenario_problems(path, block, names):
    number, title, body = block
    tests = [line for line in body if line.startswith("- Test:")]
    missing = [] if tests else [f"{path}:{number}: '{title}' has no Test line"]
    unknown = [f"{path}:{number}: test '{n}' not found" for t in tests for n in unknown_tests(t, names)]
    return missing + unknown


def check_scenarios(folder, names):
    path = folder / "spec.md"
    blocks = scenario_blocks(read_lines(path))
    return [found for block in blocks for found in scenario_problems(path, block, names)]


def table_rows(lines, section):
    """The rows of the first table under the '## section' heading."""
    rest = lines[lines.index(f"## {section}") + 1:]
    end = next((i for i, line in enumerate(rest) if line.startswith("## ")), len(rest))
    rows = [line for line in rest[:end] if line.startswith("|")]
    return [[cell.strip() for cell in row.strip("|").split("|")] for row in rows][2:]


def result_name(row):
    return row[0].strip("`").split(" ")[0].strip("`").split("::")[-1]


def result_problems(path, row, names):
    name = result_name(row)
    unknown = [] if name in names else [f"{path}: test '{name}' not found"]
    incomplete = row[-2] in ("", "-") or row[-1] not in ("✅", "❌")
    return unknown + ([f"{path}: incomplete result for '{name}'"] if incomplete else [])


def check_results(folder, names):
    path = folder / "progress.md"
    rows = table_rows(read_lines(path), "Test Results")
    return [found for row in rows for found in result_problems(path, row, names)]


def check_content(folder, names):
    checks = [check_phases, check_banned_words, check_plan_has_no_code, check_plan_steps]
    shown = [found for check in checks for found in check(folder)]
    return shown + check_scenarios(folder, names) + check_results(folder, names) + check_plan_covers_tests(folder) + check_result_order(folder)


def home_step(plan, name):
    """The number of the first plan step that names the test, or a large one."""
    return next((number for number, text in step_texts(plan) if names_test(text, name)), 99)


def check_result_order(folder):
    plan = read_text(folder / "plan.md")
    rows = table_rows(read_lines(folder / "progress.md"), "Test Results")
    steps = [home_step(plan, result_name(row)) for row in rows]
    return [] if steps == sorted(steps) else [f"{folder / 'progress.md'}: test results are not in plan step order"]


def check_folder(folder, names):
    problems = check_structure(folder)
    return problems or check_content(folder, names)


def folders():
    return sorted(path for path in (ROOT / "specs").iterdir() if path.is_dir())


def print_problems(problems):
    for problem in problems:
        print(problem.replace(str(ROOT) + "/", ""), file=sys.stderr)


def summary(problems):
    """The closing line, on stderr when there are problems."""
    text = f"{len(problems)} problems in specs/." if problems else "specs/ matches the templates and the tests."
    print(text, file=sys.stderr if problems else sys.stdout)


def main():
    names = test_names()
    problems = [found for folder in folders() for found in check_folder(folder, names)]
    print_problems(problems)
    summary(problems)
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
