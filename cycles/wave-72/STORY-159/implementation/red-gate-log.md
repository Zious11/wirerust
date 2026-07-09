---
document_type: red-gate-log
level: ops
version: "1.0"
status: complete
producer: test-writer
timestamp: 2026-07-09T00:00:00Z
phase: 3
inputs: []
input-hash: "d41d8cd"
traces_to: ".factory/stories/STORY-159.md"
stub_architect_agent: "n/a — DOCS-ONLY story, no Rust stubs needed"
stub_compile_verified: true
test_writer_agent: "test-writer (this session)"
red_gate_verified: true
---

# Red Gate Log: Wave-72 / STORY-159

STORY-159 is a DOCS-ONLY story (E-11 convention). No `todo!()` stubs are
required — the acceptance-criteria verification scripts serve as the tests.
Red Gate is established by confirming each verification script FAILS against
the current HEAD where the output artifacts do not yet exist.

## Summary

| Story | Tests (AC scripts) | All Fail (Red)? | Gate |
|-------|--------------------|-----------------|------|
| STORY-159 | AC-159-001 (implicit), AC-159-002, AC-159-003 (both loops), AC-159-004 | YES — all fail | RED |

## Stubs Created

None. DOCS-ONLY story — no Rust API surface change. `cargo check` confirms
the baseline compiles cleanly at HEAD before any implementation work begins.

## Red Gate Verification

All checks run from worktree `/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-159`.

---

### Check 0 — Baseline compile (prerequisite)

**Command:**
```bash
cargo check
```

**Outcome:** GREEN (prerequisite satisfied)

```
Checking wirerust v0.11.5 (…/.worktrees/STORY-159)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.55s
```

The worktree compiles cleanly. No pre-existing build failures that could
contaminate Red Gate results.

---

### Check 1 — AC-159-001 (public ADR file existence)

**Command:**
```bash
ls docs/adr/0012-protocols-catalog-and-coverage-gaps.md
```

**Outcome:** RED (file absent — `ls: …: No such file or directory`)

`docs/adr/0012-protocols-catalog-and-coverage-gaps.md` does not exist at HEAD.
All downstream AC checks (AC-159-002, AC-159-003) also fail as a direct
consequence of this absence.

---

### Check 2 — AC-159-002 (all ten decisions covered)

**Command:**
```bash
for n in 1 2 3 4 5 6 7 8 9 10; do
  grep -qE "Decision $n(\.|:|,| |\)|\*|\`|$)" docs/adr/0012-protocols-catalog-and-coverage-gaps.md \
    || { echo "MISSING: Decision $n"; exit 1; }
done
echo "All ten decisions present"
```

**Outcome:** RED — exits 1

```
MISSING: Decision 1
```

The script fails immediately on Decision 1 because the target file does not
exist (grep returns non-zero on a missing file, triggering the `exit 1`
branch). All ten decisions are unresolvable.

---

### Check 3 — AC-159-003 (CITED extraction)

**Command (extraction only — establishes CITED set):**
```bash
CITED=$(grep -roh -E "ADR-012 (Decision|Dec) [0-9]+" src/ tests/ \
  | grep -oE "(Decision|Dec) [0-9]+" | awk '{print $2}' | sort -nu)
echo "CITED decision numbers: $CITED"
```

**Outcome of extraction:** GREEN (informational — extraction succeeds)

```
CITED decision numbers: 1
2
3
4
5
6
7
9
10
```

Nine unique decision numbers are cited in source: **1, 2, 3, 4, 5, 6, 7, 9,
10**. Decision 8 has no source citation, consistent with the story's
Background section (Decision 8 is documented in the public ADR for
completeness only). The abbreviated `ADR-012 Dec 10` form at
`tests/integration_tests.rs:1166` is correctly captured (maps to decision
number 10).

---

### Check 4 — AC-159-003 (resolution loop against public doc)

**Command:**
```bash
CITED=$(grep -roh -E "ADR-012 (Decision|Dec) [0-9]+" src/ tests/ \
  | grep -oE "(Decision|Dec) [0-9]+" | awk '{print $2}' | sort -nu)
for n in $CITED; do
  grep -qE "Decision $n(\.|:|,| |\)|\*|\`|$)" docs/adr/0012-protocols-catalog-and-coverage-gaps.md \
    || { echo "UNRESOLVED: Decision $n"; exit 1; }
done
echo "All cited decisions resolvable"
```

**Outcome:** RED — exits 1

```
UNRESOLVED: Decision 1
```

All nine cited decision numbers (1, 2, 3, 4, 5, 6, 7, 9, 10) are unresolvable
because the public doc does not exist. The post-normalization Dec-form
zero-check is not run here (it is only meaningful after Task 3 normalizes
`ADR-012 Dec 10` → `ADR-012 Decision 10`).

---

### Check 5 — Pre-normalization state (abbreviated citation at line 1166)

**Command:**
```bash
grep -n "ADR-012 Dec 10" tests/integration_tests.rs
```

**Outcome:** Exactly 1 occurrence found (pre-normalization state confirmed)

```
1166://   (decode-loop path, ADR-012 Dec 10).
```

`tests/integration_tests.rs:1166` contains the abbreviated form
`ADR-012 Dec 10`. Task 3 must normalize this to `ADR-012 Decision 10`.
After that normalization the post-normalization check must return zero.

---

### Check 6 — AC-159-004 (CLAUDE.md Project References row)

**Command:**
```bash
grep "docs/adr/" CLAUDE.md
```

**Outcome:** RED — `0012` absent from the row

```
| `docs/adr/` | Architecture Decision Records (0001 stream dispatch, 0002 modular
analyzers, 0003 reporting pipeline, 0004 process-wide warning atomics, 0005
binary ICS protocol integration, 0006 multi-technique finding attribution,
0007 DNP3 stream dispatch and parser design, 0009 pcapng reader design,
0010 EtherNet/IP CIP stream dispatch, 0011 TLS handshake reassembly) |
```

The row ends at `0011 TLS handshake reassembly`. The required entry
`, 0012 protocols catalog and coverage-gaps system` is not present.

## Regression Check

| Existing Tests | Status |
|----------------|--------|
| Full test suite (cargo test --all-targets) | not re-run — baseline `cargo check` confirms compilation; no Rust source changed at this stage |

No Rust source modifications are planned for this story (DOCS-ONLY). The
only Rust-adjacent change is a comment normalization in
`tests/integration_tests.rs:1166` (no behavioral effect). Pre-existing tests
are not at risk.

## Red Gate Summary

| Check | AC | Command | Result |
|-------|----|---------|--------|
| 0 — Baseline compile | prereq | `cargo check` | GREEN (compiles) |
| 1 — ADR file existence | AC-159-001 | `ls docs/adr/0012-…` | RED — file absent |
| 2 — Ten decisions covered | AC-159-002 | ten-decision grep loop | RED — MISSING: Decision 1 |
| 3 — CITED extraction | AC-159-003 | grep + awk pipeline | GREEN — set: {1,2,3,4,5,6,7,9,10} |
| 4 — Resolution loop | AC-159-003 | CITED loop vs public doc | RED — UNRESOLVED: Decision 1 |
| 5 — Pre-normalization state | AC-159-003 (post) | grep "ADR-012 Dec 10" | 1 occurrence at line 1166 (awaiting Task 3) |
| 6 — CLAUDE.md row | AC-159-004 | grep "docs/adr/" CLAUDE.md | RED — 0012 absent |

**Red Gate: VERIFIED.** All three acceptance-criteria verification scripts
fail against current HEAD. The CITED decision-number set is
{1, 2, 3, 4, 5, 6, 7, 9, 10}. Implementer may proceed.

## Hand-Off to Implementer

Stories ready for implementation: **STORY-159**

Implementation guidance:

1. Read the factory ADR at
   `.factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md`
   in full before authoring.
2. Author `docs/adr/0012-protocols-catalog-and-coverage-gaps.md` following the
   format of `docs/adr/0009-pcapng-reader-design.md`. Cover all ten decisions
   plus the Decision 6 Clarification. Strip all internal factory IDs
   (`BC-*`, `VP-*`, `STORY-*`, `F-F*`, `D-NNN`, `.factory/` paths).
   Use `### Decision N: <title>` as the canonical section heading form for every
   decision — the AC-159-002 boundary guard targets this exact form.
3. Normalize `tests/integration_tests.rs:1166`: `ADR-012 Dec 10` →
   `ADR-012 Decision 10`. Run the AC-159-003 post-normalization check afterward
   to confirm zero abbreviated Dec forms remain.
4. Amend `CLAUDE.md` Project References table: append
   `, 0012 protocols catalog and coverage-gaps system` to the `docs/adr/` row
   (after `0011 TLS handshake reassembly`).
5. Run both verification loops in full to confirm green before opening the PR.
6. PR title must use `docs:` semantic prefix
   (e.g., `docs: add ADR-012 protocols catalog and coverage-gaps system`).
