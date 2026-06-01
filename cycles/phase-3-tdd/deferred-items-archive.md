---
document_type: deferred-items-archive
version: "1.0"
status: active
producer: state-manager
cycle: phase-3-tdd
traces_to: STATE.md
---

# Deferred Items Archive — Phase 3 TDD

Items removed from STATE.md live tables and archived here when they are either:
- Externally blocked (upstream plugin, awaiting PO, phase-gated)
- Closed (resolved, covered by a later story, or swept)

STATE.md live tables should only contain items that are cheaply actionable on the next relevant PR.

---

## Archived from STATE.md 2026-05-30

Items below were removed from STATE.md Drift Items and Cycle-Close Follow-Up tables on 2026-05-30
during a deferred-item cleanup burst. No information is lost — everything here is either
codified in policies.yaml, externally blocked at a named gate, or closed with a cross-reference.

### Drift Items — Externally Blocked / Phase-Gated (removed 2026-05-30)

| ID | Finding | Category | Target | Status | Archive Reason |
|----|---------|----------|--------|--------|---------------|
| W9-D2 | [process-gap, ESCALATE-UPSTREAM] story-writer template Task #2 wording "Verify Red Gate" incompatible with brownfield-formalization. NOT fixable in this repo; escalate to plugin maintainer. | process-gap | plugin-maintainer | OPEN — ESCALATE-UPSTREAM | Blocked on upstream plugin maintainer; not actionable in this repo |
| W9-D3 | [process-gap, ESCALATE-UPSTREAM] story template lacks per-AC VP trace column. NOT fixable in this repo; escalate to plugin maintainer. | process-gap | plugin-maintainer | OPEN — ESCALATE-UPSTREAM | Blocked on upstream plugin maintainer; not actionable in this repo |
| W9-D4 | [process-gap, ESCALATE-UPSTREAM] story Token Budget template hardcodes "200K for Sonnet". NOT fixable in this repo; escalate to plugin maintainer. | process-gap | plugin-maintainer | OPEN — ESCALATE-UPSTREAM | Blocked on upstream plugin maintainer; not actionable in this repo |
| W9-D12 | [spec-gap, needs-PO-intent] `packets_dropped_capacity` stats counter absent (BC-2.04.015 PC-6 observability). Awaiting PO adjudication: add counter vs document omission. | spec-gap | phase-5 PO | OPEN — AWAITING-PO | Awaiting PO decision; revisit at Phase-5 |

### Cycle-Close Follow-Up — Externally Blocked / Phase-Gated (removed 2026-05-30)

| ID | Item | Priority | Archive Reason |
|----|------|----------|---------------|
| W1.3/W2.5 **[RECURRING Waves 1-16]** | No pipeline gate advances story status draft/in-progress → completed on merge. Requires plugin-level fix (vsdd-factory story-writer template); not fixable in this repo. This session (F-DRIFT3B-001): 16 stories manually reconciled across Waves 3-13 (STORY-033 + 016/017/018/019/020/021/031/032/005/011/012/013/014/015/066/071). Root cause unfixed (upstream plugin). | P1 — ESCALATE-UPSTREAM | Blocked on upstream plugin; not actionable in this repo |
| W7.1 | No public-API surface gate for `pub fn` additions. Candidate: `cargo public-api` CI job. Deferred: requires nightly + committed baseline, 2-PR setup. Documented in CLAUDE.md. | P2 — DEFERRED | Requires multi-PR setup; documented in CLAUDE.md; revisit at v0.1.0-release |
| Phase-4-ENTRY | [deferred-review] Holdout scenarios HS-* must be semantically re-validated against Wave-18 reachability/arithmetic BC corrections (BC-2.07.002 EC-004 SSL2-ServerHello-rejection, BC-2.07.012 reachability, BC-2.07.029 arithmetic) at Phase-4 holdout-evaluation entry — confirm no scenario asserts pre-correction behavior. Non-blocking for Phase-3 wave close (zero src changes; observable behavior unchanged). | P2 — **CLOSED 2026-06-01** | Consistency audit (cycles/phase-3-tdd/phase-4-entry-consistency-audit.md) confirmed all 100 HS scenarios CLEAR of pre-correction behavior; Phase-3→4 gate passed. |
| F-S058-P13-O4 | [deferred-LOW] test_stop_after_handshake cross-story AC labels + STORY-058 FSR inclusion — pre-existing collision documented in STORY-058 v1.2. Target: wave-gate or Phase-5. | P3 — DEFERRED | Phase-gated; low severity; revisit at wave-gate or Phase-5 |

---

## Closed Items (removed 2026-05-30)

### DF-16.B CLOSED 2026-05-30

DF-16.B CLOSED 2026-05-30 — 209 BC files swept to `domain/capabilities/cap-NN-<slug>.md` form (broken `capabilities.md §CAP-NN` citations replaced); commit b17c5f0; grep 0 remaining broken citations after sweep. SS-01 (8 files) fixed 2026-05-29; remaining SS-02..SS-13 (209 files) fixed in b17c5f0 bulk sweep.

### OBS-7 CLOSED 2026-05-30

OBS-7 CLOSED 2026-05-30 — covered by STORY-076 BC-2.11.003 ("JsonReporter Escapes C0 Control Bytes per RFC 8259 via serde") / tests `test_BC_2_11_003_c0_esc_escaped_in_json` + `test_BC_2_11_003_c0_roundtrip`; PR #157 → e5cb2b1. Source BC-2.07.020 inv2 ("JSON reporter receives raw lossy summary; serde_json escapes C0 per RFC 8259") is satisfied. Previously deferred from STORY-056 P9 as untestable within tls.rs scope.

---

---

## Archived from STATE.md 2026-05-31

Items archived during the 2026-05-31 drift-remediation sweep.

### Upstream Escalation — CLI-STORY-TEMPLATE [process-gap, ESCALATE-UPSTREAM]

**Archived:** 2026-05-31
**Category:** process-gap, ESCALATE-UPSTREAM
**Finding:** The vsdd-factory plugin's CLI story template seeds a `tests/cli_tests.rs` placeholder. This caused recurring FSR-row drift across STORY-086, STORY-087, and STORY-096 (all cited `tests/cli_tests.rs` instead of per-story `tests/cli_story_NNN_tests.rs`). STORY-088 and STORY-089 are expected to hit the same pattern when delivered (Waves 25/26). The fix is in the plugin's story template — not actionable in this repo.
**Action required:** Escalate to vsdd-factory plugin maintainer: update CLI story template to seed the per-story `tests/cli_story_NNN_tests.rs` form instead of the monolith `tests/cli_tests.rs` placeholder.
**Impact in-repo:** Minimal — FSR citations are cosmetic anchors; behavioral contracts and tests are correct. Each affected story requires a 1-line FSR update at delivery time (low cost workaround).
**Validated per:** DF-VALIDATION-001 (research-agent validation 2026-05-31; report: .factory/research/deferred-validation-2026-05-31/).
**Archive reason:** Engine-side plugin cache; not fixable in this repo.

---

## Resolved Audit-Followup Items — Closed 2026-06-01

### D-001 RESOLVED 2026-06-01

D-001 (consistency-audit drift) RESOLVED — STORY-053 EC behavior corrected; fix commit f368f53 on develop. Confirmed resolved by Phase-3→4 consistency audit.

### D-002 RESOLVED 2026-06-01

D-002 (stale statuses + missing wave rows) RESOLVED — 6 story files (STORY-057/076/077/078/079/080) frontmatter status `draft` → `completed`; STORY-INDEX.md Index Table status corrected for same 6 stories; Wave Delivery Progress table backfilled for waves 3–22 (data sourced from wave-history.md). Resolved in this burst (Phase-3→4 gate).

---

## Revisit Gates

| Item | Gate | Notes |
|------|------|-------|
| W9-D2, W9-D3, W9-D4 | Upstream plugin maintainer | Monitor vsdd-factory plugin releases for story-template fixes |
| W9-D12 | Phase-5 (PO adjudication) | packets_dropped_capacity counter decision |
| W1.3/W2.5 | Upstream plugin maintainer | Monitor vsdd-factory story-writer template for status-transition gate |
| W7.1 | v0.1.0-release cycle | 2-PR setup: nightly baseline + CI step |
| Phase-4-ENTRY | **CLOSED 2026-06-01** | Confirmed CLEAR by Phase-3→4 consistency audit |
| F-S058-P13-O4 | Wave-gate or Phase-5 | test_stop_after_handshake cross-story collision |
| CLI-STORY-TEMPLATE | Upstream plugin maintainer | vsdd-factory CLI story template seeds wrong test filename; escalate to plugin maintainer |
