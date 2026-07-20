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
creation that includes files under `docs/demo-evidence/` or `.factory/demo-evidence/`.

### Gate Command

Run the following command from the repo root and verify it returns **zero results**:

```bash
grep -rE '/Users/|/home/|~/' docs/demo-evidence/
```

If any results are returned, the push is blocked. Scrub all absolute host paths
and tilde-form home references from the offending files before proceeding.

### `.factory/demo-evidence/` — Extended Scope

The gate command is extended for `.factory/demo-evidence/` — run the path-scrub grep
against both `docs/demo-evidence/` and `.factory/demo-evidence/` when committing new
captures to either tree.

Pre-existing files (92 files, 163 host-path occurrences as of wave-75 close, 2026-07-13)
are documented as a baseline exempt from retroactive remediation. Only files created or
modified AFTER this story's delivery (STORY-166 AC-166-003) are subject to the extended
scope.

Extended gate command covering both trees:

```bash
grep -rE '/Users/|/home/|~/' docs/demo-evidence/ .factory/demo-evidence/
```

**Process-gap reference:** PG-W75-DEMO-EVIDENCE-SCRUB-SCOPE (wave-75 gate W7
observation). **Codification story:** STORY-166 AC-166-003 (project half; engine half —
demo-recorder automatic host-path scrub step — tracked as drbothen/vsdd-factory#636).

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
- **Before opening any PR** that modifies `docs/demo-evidence/` or `.factory/demo-evidence/`:
  include gate output in the PR description or demo-evidence report.

### Optional CI Guard

To enforce this gate in CI, add the following step to `.github/workflows/ci.yml`:

```yaml
- name: Demo-evidence path-scrub gate (PG-W70-DEMO-SCRUB)
  run: |
    FAIL=0
    for d in docs/demo-evidence .factory/demo-evidence; do
      if [ -d "$d" ] && grep -rE '/Users/|/home/|~/' "$d"; then
        echo "FAIL: absolute host paths found in $d"
        FAIL=1
      fi
    done
    exit $FAIL
```

This step fails if any absolute `/Users/`, `/home/`, or tilde-form (`~/`) paths are
present in committed demo evidence, preventing reintroduction at CI time. On `develop`
CI, only `docs/demo-evidence/` exists (`.factory/` lives on the `factory-artifacts`
branch — the same constraint that deferred the input-hash CI gate per CLAUDE.md); the
loop guards whichever trees are present, so the example is safe on both `develop` CI
and full repo-root checkouts.

---

## Changelog

| Date | Change | Reference |
|------|--------|-----------|
| 2026-07-20 | CI-guard example aligned with the extended mandatory gate — tilde + `.factory` tree propagation; sibling-sweep gap from the AC-166-003 edit (F-S166P3-001). Follow-up same-day sweep: the two TRIGGER-PREDICATE loci (mandatory-gate lead-in and "When to Run" PR-opening rule) still scoped to `docs/` only after the F-S166P3-001 command-body extension — harmonized to name both `docs/demo-evidence/` and `.factory/demo-evidence/`. The new-captures-only baseline exemption (92 pre-existing files, 163 host-path occurrences as of wave-75 close) still governs. Second same-day fix: the two-tree grep in the CI-guard example exits 2 (error, not 1) when `.factory/` is absent on a `develop` checkout, so the `if` never fires and the guard is a false-green — orchestrator-verified. Replaced with a path-guarded per-tree loop that only greps directories that exist (F-S166P7-001). | F-S166P3-001 / F-S166P4-002 / F-S166P7-001 |
| 2026-07-19 | Extended gate scope to `.factory/demo-evidence/` for NEW captures (both trees now covered by the gate command). 92 pre-existing files (163 host-path occurrences as of wave-75 close, 2026-07-13) documented as a baseline exempt from retroactive remediation. | PG-W75-DEMO-EVIDENCE-SCRUB-SCOPE / STORY-166 AC-166-003 |
| 2026-07-09 | Extended scrub pattern to also reject tilde-form home paths (`~/`) in tape/evidence text files. Root cause: SEC-W72-001 LOW — STORY-159 demo tapes contained `~/Documents/GITHUB/wirerust` which bypassed the original `/Users/` + `/home/` gate. Fixed via PR #391. | SEC-W72-001 / PR #391 |
| 2026-07-08 | Initial gate document authored (STORY-157 AC-157-002, PG-W70-DEMO-SCRUB). | STORY-157 |

---

## Reference

- **PG-W70-DEMO-SCRUB:** Root process-gap (wave-70, 2026-07-07)
- **F-W70P2-002:** Wave-70 Phase-2 gate finding that identified the scope
- **PR #376:** `docs: scrub absolute host paths from committed demo evidence`
- **STORY-157 AC-157-002:** Factory codification story for this gate
- **PG-W75-DEMO-EVIDENCE-SCRUB-SCOPE:** Wave-75 gate W7 observation — `.factory/demo-evidence/`
  scope extension (project half)
- **STORY-166 AC-166-003:** Codification story for the `.factory/demo-evidence/` extended
  scope (engine half — demo-recorder automatic scrub — tracked as drbothen/vsdd-factory#636)
