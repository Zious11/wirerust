#!/usr/bin/env bash
# AC-159-002 success path: verify all ten decisions present in public ADR-012
set -e

ADR="docs/adr/0012-protocols-catalog-and-coverage-gaps.md"

echo "=== AC-159-002: All ten decisions present in ADR-012 ==="
echo ""

for n in 1 2 3 4 5 6 7 8 9 10; do
  # right-boundary guard: prevents "Decision 1" matching "Decision 10" as a substring
  if grep -qE "Decision $n(\.|:|,| |\)|\*|\`|\$)" "$ADR"; then
    echo "  Decision $n: FOUND"
  else
    echo "MISSING: Decision $n"
    exit 1
  fi
done

echo ""
echo "All ten decisions present"
