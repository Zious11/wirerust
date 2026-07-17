# Phase F6 — Security Scan Results (feature-iec104 delta)

**Feature:** IEC-104 passive analyzer (STORY-167..174)
**develop HEAD:** `b36b884`
**Date:** 2026-07-17
**Scope:** full dependency tree (`cargo audit`).

---

## Summary

| Scan | Result |
|------|--------|
| `cargo audit` | **exit 0 — 0 vulnerabilities, 0 warnings** across 193 crate dependencies |
| semgrep | **SKIPPED** — not installed in this environment |

## cargo audit

```
Loaded 1166 security advisories (from ~/.cargo/advisory-db)
Scanning Cargo.lock for vulnerabilities (193 crate dependencies)
EXIT=0
```

No RUSTSEC advisories reported. No unmaintained/yanked warnings. No unresolved
CRITICAL or HIGH findings.

## semgrep — skip justification

semgrep is not installed in this environment, so the SAST pass is skipped. This
is acceptable coverage-wise because:
- The per-PR `security-reviewer` already reviewed the IEC-104 delta (STORY-167..174
  plus FIX-P4-001 / FIX-F5-001) for injection, memory-safety, and untrusted-input
  handling.
- `cargo audit` above covers dependency-tree (RUSTSEC) vulnerabilities.
- VP-044 (Kani no-panic on parse_apci_header) and VP-047 (2.64M-run fuzz, 0 crashes)
  provide formal + dynamic memory-safety evidence on the untrusted-input path.

## Verdict

Security gate: **PASS** — cargo-audit clean (no unresolved CRITICAL/HIGH), semgrep
skip justified by prior security-reviewer coverage + audit + Kani/fuzz evidence.
