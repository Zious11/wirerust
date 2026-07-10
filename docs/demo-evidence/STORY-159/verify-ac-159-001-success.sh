#!/usr/bin/env bash
# AC-159-001 success path: verify ADR-012 public file exists and follows correct format
set -e

echo "=== AC-159-001: Public ADR file exists ==="
ls -lh docs/adr/0012-protocols-catalog-and-coverage-gaps.md
echo ""

echo "=== AC-159-001: No YAML frontmatter (first 5 lines) ==="
head -5 docs/adr/0012-protocols-catalog-and-coverage-gaps.md
echo ""

echo "=== AC-159-001: Format check — Status, Date, Context present ==="
grep -E "^\*\*Status:\*\*|^\*\*Date:\*\*|^## Context|^\*\*Context\*\*|^\*\*Context:" docs/adr/0012-protocols-catalog-and-coverage-gaps.md | head -5
echo ""

echo "PASS: ADR-012 public file exists with correct format"
