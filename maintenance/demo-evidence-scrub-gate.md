# Demo-Evidence Path-Scrub Gate

**Process-gap reference:** PG-W70-DEMO-SCRUB  
**Finding reference:** F-W70P2-002 (MEDIUM, wave-70 Phase-2 gate)  
**Remediation precedent:** PR #376 (`docs: scrub absolute host paths from committed demo evidence`)  
**Added:** 2026-07-08 (STORY-157 AC-157-002)

---

## Background

During STORY-149 wave-70 delivery, the demo-recorder committed evidence transcripts
containing absolute host paths (`/Users/zious/...`). Wave-70 Phase-2 gate finding
F-W70P2-002 (MEDIUM, privacy/process-gap) identified the scope as repo-wide: 196
substitutions across 193 files in 31 directories. PR #376 remediated all occurrences
(`docs: scrub absolute host paths from committed demo evidence`).

Root cause: the demo-recording checklist had no path-scrub step and no CI grep guard
to prevent reintroduction.

---

## Demo-Evidence Path-Scrub Gate (MANDATORY)

Every demo-recording session MUST include this gate before any `git push` or PR
creation that includes files under `docs/demo-evidence/`.

### Gate Command

Run the following command from the repo root and verify it returns **zero results**:

```bash
grep -rE '/Users/|/home/|~/' docs/demo-evidence/
```

If any results are returned, the push is blocked. Scrub all absolute host paths
and tilde-form home references from the offending files before proceeding.

### Common Patterns to Scrub

Replace absolute host paths with repo-relative paths or anonymized placeholders:

| Before | After |
|--------|-------|
| `/Users/zious/Documents/GITHUB/wirerust/` | `<repo>/` |
| `/Users/zious/` | `<home>/` |
| `/home/username/` | `<home>/` |
| `~/Documents/GITHUB/wirerust` | `<repo>/` |
| `~/` | `<home>/` |

Use `sed -i '' 's|/Users/[^/]*/[^/]*/[^/]*/wirerust/||g'` (macOS) or equivalent
to scrub bulk occurrences, then verify the grep returns zero results.

### When to Run

- **Before every demo-evidence push:** run the gate command and confirm zero results.
- **After any VHS recording or Playwright session** that captures CLI or browser output:
  paths leak from prompt strings, error messages, and shell output.
- **Before opening any PR** that modifies `docs/demo-evidence/`: include gate output
  in the PR description or demo-evidence report.

### Optional CI Guard

To enforce this gate in CI, add the following step to `.github/workflows/ci.yml`:

```yaml
- name: Demo-evidence path-scrub gate (PG-W70-DEMO-SCRUB)
  run: |
    if grep -rE '/Users/|/home/' docs/demo-evidence/ 2>/dev/null; then
      echo "FAIL: absolute host paths found in demo-evidence"
      exit 1
    fi
```

This step fails if any absolute `/Users/`, `/home/`, or tilde-form (`~/`) paths are
present in committed demo evidence, preventing reintroduction at CI time.

---

## Changelog

| Date | Change | Reference |
|------|--------|-----------|
| 2026-07-09 | Extended scrub pattern to also reject tilde-form home paths (`~/`) in tape/evidence text files. Root cause: SEC-W72-001 LOW — STORY-159 demo tapes contained `~/Documents/GITHUB/wirerust` which bypassed the original `/Users/` + `/home/` gate. Fixed via PR #391. | SEC-W72-001 / PR #391 |
| 2026-07-08 | Initial gate document authored (STORY-157 AC-157-002, PG-W70-DEMO-SCRUB). | STORY-157 |

---

## Reference

- **PG-W70-DEMO-SCRUB:** Root process-gap (wave-70, 2026-07-07)
- **F-W70P2-002:** Wave-70 Phase-2 gate finding that identified the scope
- **PR #376:** `docs: scrub absolute host paths from committed demo evidence`
- **STORY-157 AC-157-002:** Factory codification story for this gate
