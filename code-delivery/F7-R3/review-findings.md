# Review Findings — F7-R3 (PR #274)

## Convergence Summary

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1     | 3        | 0        | 0     | 3 (all LOW/NIT) |
| —     | —        | —        | —     | APPROVE — no MEDIUM+ |

**Convergence: APPROVED after 1 cycle. Zero blocking findings.**

---

## Security Review Findings (security-reviewer)

### SEC-001 — `|| true` suppresses grep file-not-found (LOW)
- **CWE:** CWE-390 (Detection of Error Condition Without Action)
- **Location:** `.github/workflows/ci.yml` — help-provenance-gate `run:` step
- **Description:** `VIOLATIONS=$(grep ... src/cli.rs) || true` — if `src/cli.rs` is deleted
  or renamed, `grep` exits with code 2 (error), which `|| true` collapses to success.
  `VIOLATIONS` is empty and the gate silently passes. The provenance-leak guard becomes a no-op.
- **Severity Rationale:** Conditional on a file rename/deletion that would itself be caught in
  code review. Not an active exploit path — brittleness in gate self-integrity.
- **Disposition:** Post-merge hardening item. Proposed fix: add `test -f src/cli.rs || exit 1`
  before the grep line. Not blocking this PR.

### SEC-002 — Broadened regex catches standards IDs (RFC-, ISO-, CVE-) — latent false-positive (LOW)
- **CWE:** CWE-697 (Incorrect Comparison)
- **Location:** `.github/workflows/ci.yml` — help-provenance-gate regex `\b[A-Z]{2,}-[0-9A-Z]`
- **Description:** `RFC-9293`, `ISO-27001`, `CVE-2024-1234` would all match. If a future
  contributor adds a legitimate `///` doc-comment referencing a standards body ID, CI fails
  with a confusing error. Potential for gate-working-around over time.
- **Severity Rationale:** Zero current matches in `src/cli.rs`. Risk only materializes with
  future `///` doc-comment changes. Not an active issue.
- **Disposition:** Post-merge hardening item. Proposed fix: add exclusion for known standards
  prefixes (`grep -v -E '\b(RFC|ISO|CVE|IEC|IEEE)-'`), or add a comment warning maintainers.
  Not blocking this PR.

---

## PR Review Findings (pr-reviewer)

### NIT — MITRE-Txxxx hyphenated form not documented in false-positive analysis
- **Severity:** NIT (non-blocking)
- **Location:** PR description false-positive analysis section
- **Description:** The description enumerates `MITRE`, `JSON`, `TCP`, `ARP` as safe
  space-separated terms, but does not note that a hyphenated form `MITRE-T1046` would
  match the broadened regex. The shipped `src/cli.rs` uses "MITRE ATT&CK" (space form),
  so there is no real false positive today.
- **Disposition:** Noted for awareness. Not blocking. Optional post-merge description update
  or comment in ci.yml warning about `MITRE-Txxxx` form.

---

## Independent Verifications (pr-reviewer)

All of the following were verified against the actual codebase:

1. ADR Decision code block mirrors shipped `terminal.rs` — CONFIRMED
2. Binding Rule 5 Forward-compatibility paragraph Rust semantics — CONFIRMED CORRECT
3. Corrected line anchors `:502` / `:511` — CONFIRMED CORRECT (old :505/:514 were stale)
4. Broadened regex catches all 9 claimed factory-ID prefixes — CONFIRMED
5. False-positive risk for MITRE/JSON/TCP/ARP/non-zero/opt-in — CONFIRMED SAFE
6. Zero matches against actual `src/cli.rs` on PR branch — CONFIRMED
7. CI YAML parses cleanly; checkout SHA pin compliant with action-pin gate — CONFIRMED
8. PR description accuracy — CONFIRMED ACCURATE
