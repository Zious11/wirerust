# Demo Evidence Report — STORY-159

**Story:** STORY-159 — Author Public ADR-012: Protocols Catalog and Coverage-Gaps System
**Branch:** docs/STORY-159-public-adr-0012
**HEAD:** cd99a58
**Recorded:** 2026-07-09
**Tool:** VHS (CLI terminal recording)
**Scrub gate:** PASS — zero absolute host paths in committed artifacts (PG-W70-DEMO-SCRUB)

---

## Coverage Map

| AC | Path | Recording(s) | Verdict |
|----|------|-------------|---------|
| AC-159-001 | `docs/adr/0012-protocols-catalog-and-coverage-gaps.md` exists, ADR-0009 format, no internal factory IDs | AC-159-001-adr-exists.gif / .webm (success) | PASS |
| AC-159-001 guard | Internal-factory-ID negative grep returns zero matches | AC-159-001-no-internal-ids.gif / .webm (guard) | PASS |
| AC-159-002 | Ten-decision grep loop exits 0 ("All ten decisions present") | AC-159-002-ten-decisions.gif / .webm (success) | PASS |
| AC-159-003 | Cited-decision resolution loop passes + Dec-zero post-normalization check | AC-159-003-citations-resolvable.gif / .webm (success) | PASS |
| AC-159-004 | `CLAUDE.md` docs/adr/ row contains appended 0012 clause | AC-159-004-claude-md-row.gif / .webm (success) | PASS |

---

## Artifacts

### AC-159-001: Public ADR file exists with correct format (success path)

- **Script:** `verify-ac-159-001-success.sh`
- **Tape:** `AC-159-001-adr-exists.tape`
- **GIF:** `AC-159-001-adr-exists.gif`
- **WEBM:** `AC-159-001-adr-exists.webm`

Demonstrates: `ls` confirms file exists (18K), `head -5` shows no YAML frontmatter (starts
with `# ADR 0012:`), grep confirms **Status**, **Date**, **Context** fields present.
Final line: `PASS: ADR-012 public file exists with correct format`

### AC-159-001 guard: No internal factory IDs (negative path)

- **Script:** `verify-ac-159-001-guard.sh`
- **Tape:** `AC-159-001-no-internal-ids.tape`
- **GIF:** `AC-159-001-no-internal-ids.gif`
- **WEBM:** `AC-159-001-no-internal-ids.webm`

Demonstrates: grep for patterns `BC-2.NN.NNN`, `VP-NNN`, `STORY-NNN`, `F-F*`, `D-NNN`,
`.factory/` finds zero matches in the public ADR.
Final line: `(none found — PASS: zero internal factory IDs in public ADR)`

### AC-159-002: All ten decisions present (success path)

- **Script:** `verify-ac-159-002-success.sh`
- **Tape:** `AC-159-002-ten-decisions.tape`
- **GIF:** `AC-159-002-ten-decisions.gif`
- **WEBM:** `AC-159-002-ten-decisions.webm`

Demonstrates: the story-specified ten-decision verification loop (with right-boundary guard
`(\.|:|,| |\)|\*|\`|$)`) confirms Decision 1–10 all FOUND.
Final line: `All ten decisions present`

### AC-159-003: Cited decisions resolvable + Dec-zero check (success path)

- **Script:** `verify-ac-159-003-success.sh`
- **Tape:** `AC-159-003-citations-resolvable.tape`
- **GIF:** `AC-159-003-citations-resolvable.gif`
- **WEBM:** `AC-159-003-citations-resolvable.webm`

Demonstrates in two phases:
1. Extracts cited decision numbers from `src/` and `tests/` (1 2 3 4 5 6 7 9 10), confirms
   each resolves to a `### Decision N:` heading in the public ADR. Line: `All cited decisions resolvable`
2. Post-normalization check: `grep -roh -E "ADR-012 Dec [0-9]+"` returns zero — the
   abbreviated `ADR-012 Dec 10` form at `tests/integration_tests.rs:1166` has been
   normalized to `ADR-012 Decision 10`. Line: `Abbreviated Dec form count: 0 (normalized)`

### AC-159-004: CLAUDE.md Project References row updated (success path)

- **Script:** `verify-ac-159-004-success.sh`
- **Tape:** `AC-159-004-claude-md-row.tape`
- **GIF:** `AC-159-004-claude-md-row.gif`
- **WEBM:** `AC-159-004-claude-md-row.webm`

Demonstrates: grep for `docs/adr/` in CLAUDE.md shows the full Project References row
ending with `, 0012 protocols catalog and coverage-gaps system`.
Final line: `PASS: 0012 clause present in docs/adr/ Project References row`

---

## Path-Scrub Gate Result (PG-W70-DEMO-SCRUB)

Gate command: `grep -rE '<host-path-pattern>' docs/demo-evidence/STORY-159/`

Result: **zero matches — PASS**. No absolute host paths in any committed evidence artifact.
Tape files use `<REPO-ROOT>/.worktrees/STORY-159` (scrub-marker form per SEC-W72-001;
tilde-expansion paths were scrubbed to this form in the wave-72 gate fix);
VHS output files are named with relative paths only.
