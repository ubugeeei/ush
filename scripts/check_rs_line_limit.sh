#!/bin/sh
set -eu

limit=250
failed=0

tmp_file="${TMPDIR:-/tmp}/ush_rs_line_limit.$$"
trap 'rm -f "$tmp_file"' EXIT INT TERM

find apps crates -type f -name '*.rs' | sort > "$tmp_file"

while IFS= read -r file; do
  lines=$(wc -l < "$file")
  if [ "$lines" -gt "$limit" ]; then
    printf '%s has %s lines (limit: %s)\n' "$file" "$lines" "$limit" >&2
    failed=1
  fi
done < "$tmp_file"

if [ "$failed" -ne 0 ]; then
  exit 1
fi
