---
wave: 70
pass: W2
verdict: FINDINGS_REMEDIATED
develop_head_at_pass: 116100d3096e66723c825f8157935b08a37c48c2
develop_head_post_remediation: 83196242f81024dd127ba7caafb76a1990297290
date: 2026-07-07
---

# Wave 70 — Adversarial Pass W2

**Verdict: FINDINGS — remediated. Consecutive-clean count reset to 0.**

## Summary

Pass W2 ran against develop HEAD `116100d`. Two blocking findings discovered and
remediated; one observation registered as tech-debt (PERF-003/004/005 already in
register from maint-2026-07-01; registered as wave-70 gate observation O-W70P2-002).

## Findings

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-W70P2-001 | MEDIUM | STORY-150 anchor drift: BC reference in story v1.0 pointed at stale anchor site in tls.rs that was updated by STORY-149 carry-path restructure | Fixed: story updated to v1.1 at commit 9273b85; anchor re-targeted to correct post-restructure site |
| F-W70P2-002 | MEDIUM | Path leak: committed demo-evidence files contained absolute host filesystem paths (e.g. `/Users/zious/...`) that should not be present in version-controlled artifacts | Fixed: PR #376 (8319624) scrubs absolute paths from all committed demo evidence in docs/demo-evidence/ |
| O-W70P2-001 | LOW | Informational: minor formatting observation in STORY-150 draft | No action; noted |
| O-W70P2-002 | LOW/OBS | PERF-003/004/005 (TLS tidy-pass candidates) confirmed still open on 116100d, were cited in STORY-149 AC-149-004 (optional scope, not exercised) | Registered tech-debt register v1.6 wave-70 gate annotation; no code change required |

## Remediation Actions

1. **9273b85** — STORY-150 v1.1: anchor corrected to post-restructure tls.rs site; story input-hash rebaselined.
2. **PR #376 (8319624)** — `docs: scrub absolute host paths from committed demo evidence (F-W70P2-002)` — squash-merged to develop.

## Consecutive-Clean Count After Pass

0 of 3 required (findings remediated; counter reset).
