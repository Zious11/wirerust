#!/usr/bin/env bash
# AC-159-001 guard path: verify no internal factory IDs in the public ADR file
# Negative grep — finding nothing is the passing condition.

ADR="docs/adr/0012-protocols-catalog-and-coverage-gaps.md"

echo "=== AC-159-001 guard: Scan for internal factory IDs in public ADR ==="
echo "Patterns checked: BC-2.NN.NNN, VP-NNN, STORY-NNN, F-F*, D-NNN, .factory/"
echo ""

MATCHES=$(grep -nE "(BC-2\.[0-9]+\.[0-9]+|VP-[0-9]+|STORY-[0-9]+|F-F[A-Z0-9]|D-[0-9]{3}|\.factory/)" "$ADR" 2>/dev/null) || true

if [ -z "$MATCHES" ]; then
    echo "(none found — PASS: zero internal factory IDs in public ADR)"
else
    echo "FAIL: internal factory IDs found:"
    echo "$MATCHES"
    exit 1
fi
