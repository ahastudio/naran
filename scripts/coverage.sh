#!/bin/bash
# Runs every test of the workspace with coverage turned on and prints the
# report for the source files of every crate. Fails when a line, a region or
# a function is not covered.
#
# Uses the `llvm-tools` component of the toolchain and python3, so nothing
# else needs to be installed. Set SHOW=1 to also print the source with its
# counts. The full data is written to target/coverage/coverage.json.
# Set CARGO to pick a specific cargo (default: the one in PATH).
set -euo pipefail
cd "$(dirname "$0")/.."

CARGO=${CARGO:-cargo}
TOOLS="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"
OUT=target/coverage
TESTS=(--workspace --all-targets)
objects=()
sources=()

prepare_output() {
  rm -rf "$OUT/profiles"
  mkdir -p "$OUT"
  export RUSTFLAGS="-C instrument-coverage"
  export CARGO_TARGET_DIR="$OUT/target"
  export LLVM_PROFILE_FILE="$PWD/$OUT/profiles/%p-%m.profraw"
}

# The test executables the coverage is read from.
build_test_objects() {
  while IFS= read -r file; do
    objects+=(-object "$file")
  done < <(
    $CARGO test "${TESTS[@]}" --no-run --message-format=json |
      sed -n 's/.*"executable":"\([^"]*\)".*/\1/p'
  )
}

run_tests() {
  $CARGO test "${TESTS[@]}" --no-fail-fast
  "$TOOLS/llvm-profdata" merge -sparse "$OUT"/profiles/*.profraw -o "$OUT/naran.profdata"
}

# The source files of every crate. Test-only files are left out, and so is
# `app/src/main.rs`: it only starts the platform's event loop, which a test
# cannot run.
collect_sources() {
  while IFS= read -r file; do
    sources+=("$file")
  done < <(find core/src app/src tools/*/src -name '*.rs' ! -name tests.rs ! -path '*/tests/*' ! -path app/src/main.rs | sort)
}

show_source() {
  [[ ${SHOW:-} == 1 ]] || return 0
  "$TOOLS/llvm-cov" show -show-line-counts-or-regions -show-instantiations=false \
    -use-color=false "${filters[@]}" -sources "${sources[@]}"
}

write_reports() {
  "$TOOLS/llvm-cov" export -format=text "${filters[@]}" -sources "${sources[@]}" >"$OUT/coverage.json"
  "$TOOLS/llvm-cov" report "${filters[@]}" -sources "${sources[@]}" | tee "$OUT/report.txt"
}

# Lines and functions come from the report. A region counts as run when any
# test executable ran it: the core is compiled into several of them, and the
# report counts each copy on its own.
list_uncovered_regions() {
  python3 scripts/uncovered.py "$OUT/coverage.json" "${sources[@]}" | sed "s|^$PWD/||" >"$OUT/uncovered.txt"
}

fail_unless_covered() {
  local total missed regions
  total=$(tail -1 "$OUT/report.txt")
  missed=$(awk '{print $6, $9}' <<<"$total")
  regions=$(wc -l <"$OUT/uncovered.txt" | tr -d ' ')
  if [[ $missed != "0 0" || $regions != 0 ]]; then
    cat "$OUT/uncovered.txt" >&2
    echo "Not everything is covered (functions and lines missed: $missed, regions: $regions)." >&2
    exit 1
  fi
  echo "Every line, function and region is covered."
}

main() {
  prepare_output
  build_test_objects
  run_tests
  collect_sources
  filters=(-instr-profile="$OUT/naran.profdata" "${objects[@]}")
  show_source
  write_reports
  list_uncovered_regions
  fail_unless_covered
}

main
