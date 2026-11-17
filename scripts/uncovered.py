"""Prints every code region no test ran, from the JSON of `llvm-cov export`.

A region counts as run when any test executable ran it. Only the given
source files are listed. Usage:
    python3 scripts/uncovered.py target/coverage/coverage.json core/src/lib.rs ...
"""

import collections
import json
import sys


def function_regions(function):
    """The code regions of a function: their place, and how often they ran."""
    for region in function["regions"]:
        start_line, start_column, end_line, end_column, count, file_id, _, kind = region[:8]
        if kind == 0:
            yield (function["filenames"][file_id], start_line, start_column, end_line, end_column), count


def counts(export):
    """The highest count of each region, keyed by file and place."""
    found = collections.defaultdict(int)
    for function in export["data"][0]["functions"]:
        for place, count in function_regions(function):
            found[place] = max(found[place], count)
    return found


def place_text(name, start_line, start_column, end_line, end_column):
    return f"{name}:{start_line}:{start_column}-{end_line}:{end_column}"


def uncovered(found, sources):
    """The places no test ran, in the given source files, in order."""
    return [
        place_text(*place)
        for place, count in sorted(found.items())
        if count == 0 and place[0].endswith(sources)
    ]


def main(path, sources):
    with open(path, encoding="utf-8") as file:
        found = counts(json.load(file))
    for line in uncovered(found, sources):
        print(line)


if __name__ == "__main__":
    main(sys.argv[1], tuple("/" + source for source in sys.argv[2:]))
