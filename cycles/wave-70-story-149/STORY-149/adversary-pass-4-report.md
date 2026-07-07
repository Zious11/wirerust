---
document_type: adversary-pass-report
level: ops
version: "1.0"
story: STORY-149
cycle: wave-70-story-149
pass: 4
date: "2026-07-07"
worktree_head_reviewed: 208b2d4
story_spec_at_review: v1.3 (0841cce)
classification: FINDINGS
producer: adversary (wave-70)
traces_to: STORY-149
bc_gating: BC-5.39.001
---

# Adversary Pass 4 Report — STORY-149

## Checkout Guard

**Result: PASS**

Worktree head reviewed: `208b2d4`. Branch `feature/STORY-149-tls-carry-perf` confirmed at
correct head. No uncommitted changes.

## Classification

**FINDINGS** — 1 finding: 1 MEDIUM. Clean streak reset to 0.

## Pass-3 Nitpick Fix Verification

Both Pass-3 story-wording nitpick fixes (F-S149P3-001 and F-S149P3-002) confirmed present
in story v1.3:

| Finding | Status | Notes |
|---------|--------|-------|
| F-S149P3-001 — bench name slug | FIXED | Naming convention for `tls_carry_*` bench functions documented in v1.3 |
| F-S149P3-002 — WARNING/MAX_BUF conflation | FIXED | Two thresholds correctly disambiguated in v1.3 acceptance criteria |

## Findings Detail

### MEDIUM

**F-S149P4-001** — Architecture Mapping phantom field `TlsFlowState.carry`

The story's Architecture Mapping section (the table that maps story concepts to struct
fields in the implementation) cited `TlsFlowState.carry` as the carry buffer field. No
such field exists in the implementation at `208b2d4`. The 636-line restructure introduced
two per-direction carry buffers: `client_hs_carry` and `server_hs_carry`.

The phantom field reference would cause the following downstream failures:

- A formal-hardening agent reading the Architecture Mapping would attempt to verify a
  property on `TlsFlowState.carry`, find no such field, and either error out or silently
  skip the verification.
- A future implementer adding a feature that touches the carry path could introduce an
  incorrectly-named field based on the stale spec guidance.
- A Kani harness targeting the Architecture Mapping would reference a non-existent path.

Additionally, two prose references in the story body (lines 194 and 233) used the generic
phrase "carry buffer" in contexts where the per-direction field distinction is
architecturally significant — e.g., "the carry buffer is cleared on parse error" should
read "client_hs_carry and server_hs_carry are cleared independently per direction on parse
error." These were harmonized as part of the same remediation.

Remediated in: `d68af34` (story-writer; STORY-149 bumped to v1.4). Architecture Mapping
corrected to `client_hs_carry` / `server_hs_carry`; sibling prose at lines 194/233
harmonized to use per-direction field names.

## Convergence State

Pass 4 complete. Clean streak reset to **0 / 3** required (FINDINGS raised).
Pass 5 pending against head `208b2d4` with story at v1.4 (`d68af34`).
