#!/usr/bin/env bash
# AC-159-003 success path: all inline source citations resolvable + Dec-zero post-normalization check
set -e

ADR="docs/adr/0012-protocols-catalog-and-coverage-gaps.md"

echo "=== AC-159-003: Extract unique decision numbers cited in src/ and tests/ ==="
CITED=$(grep -roh -E "ADR-012 (Decision|Dec) [0-9]+" src/ tests/ \
  | grep -oE "(Decision|Dec) [0-9]+" | awk '{print $2}' | sort -nu)
echo "Cited decision numbers: $(echo $CITED | tr '\n' ' ')"
echo ""

echo "=== AC-159-003: Verify each cited decision resolves to a section in public ADR ==="
for n in $CITED; do
  # right-boundary guard: prevents "Decision 1" matching "Decision 10" as a substring
  if grep -qE "Decision $n(\.|:|,| |\)|\*|\`|\$)" "$ADR"; then
    echo "  Decision $n: RESOLVED"
  else
    echo "UNRESOLVED: Decision $n"
    exit 1
  fi
done
echo ""
echo "All cited decisions resolvable"

echo ""
echo "=== AC-159-003: Post-normalization check — abbreviated 'Dec' form must be zero ==="
REMAINING=$(grep -roh -E "ADR-012 Dec [0-9]+" src/ tests/ | wc -l | tr -d ' ')
if [ "$REMAINING" -eq 0 ]; then
  echo "Abbreviated Dec form count: 0 (normalized)"
else
  echo "FAIL: $REMAINING abbreviated ADR-012 Dec form(s) remain"
  exit 1
fi
