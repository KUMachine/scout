#!/usr/bin/env bash
# Fail if subprocess spawning appears outside the audited gateway modules.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
violations=0

while IFS= read -r file; do
  echo "security check: unexpected Command::new in $file"
  violations=$((violations + 1))
done < <(rg -l 'Command::new' "$ROOT/src" \
  --glob '!gh.rs' \
  --glob '!util.rs' || true)

if [[ "$violations" -ne 0 ]]; then
  echo "Found $violations file(s) spawning subprocesses outside src/gh.rs and src/util.rs"
  exit 1
fi

# Ban direct network crates in Cargo.toml (scout delegates to gh for all HTTP).
if rg -q 'reqwest|ureq|hyper|curl|tokio' "$ROOT/Cargo.toml"; then
  echo "security check: disallowed network dependency in Cargo.toml"
  exit 1
fi

echo "security checks passed"
