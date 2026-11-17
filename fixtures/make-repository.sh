#!/bin/sh
# Makes a repository for measuring. Usage: make-repository.sh <folder> <commit count>
# Writes a linear history with `git fast-import`, and lets 11 branch names
# take turns moving forward.
set -eu
directory=${1:?give a folder}
count=${2:?give a commit count}

init_repository() {
  mkdir -p "$directory"
  git -C "$directory" init -q -b main
}

# The `git fast-import` stream: one commit per number, on a branch picked by
# the number.
history_stream() {
  awk -v count="$count" 'BEGIN {
    for (i = 1; i <= count; i++) {
      branch = (i % 50 < 10) ? "refs/heads/topic" (i % 10) : "refs/heads/main"
      printf "commit %s\nmark :%d\ncommitter Naran <naran@example.com> %d +0000\n", branch, i, 1600000000 + i
      message = "Commit " i
      printf "data %d\n%s\n", length(message) + 0, message
      if (i > 1) printf "from :%d\n", i - 1
      printf "M 100644 inline file.txt\ndata 7\n%06d\n\n", i % 1000000
    }
  }'
}

check_out_main() {
  git -C "$directory" symbolic-ref HEAD refs/heads/main
  git -C "$directory" reset -q --hard main
}

init_repository
history_stream | git -C "$directory" fast-import --quiet
check_out_main
