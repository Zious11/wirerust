---
document_type: adversarial-spec-review
cycle: feature-pcapng-reader
phase: F2
pass: 8
verdict: CLEAN
clean_pass_number: 1
clean_passes_required: 3
timestamp: 2026-06-20T00:00:00Z
findings:
  critical: 0
  high: 0
  medium: 3
  low: 5
  total: 8
---

# F2 Adversarial Spec Review — Pass 8

**Verdict:** CLEAN of HIGH/CRITICAL — 0C / 0H / 3M / 5L

**Clean-pass counter:** 1 of 3 required (BC-5.39.001). Two more consecutive clean passes
required before F3 story decomposition is unblocked.

**Trajectory:** P1:23 / P2:24 / P3:17 / P4:13 / P5:13 / P6:13 / P7:12 / P8:8

---

## Convergence Observations

All framing BCs (BC-2.01.009..018) have VP coverage + holdout scenarios. Per-block constants
agree across 6 documents (BC, ADR-009, HS-index, error-taxonomy, VP-INDEX, verification-
architecture). Holdout arithmetic is self-consistent across HS-101..109. All 4 pass-7 fixes
propagated with zero stale siblings detected.

---

## Medium Findings (all remediated in this burst)

### M-1: error-taxonomy SPB-fixed-min prose gap (FIXED)

**Finding:** error-taxonomy v3.4 E-INP-008 scope note described SPB body-too-short as
"SPB body < 4 bytes (original_len)" without mentioning the canonical SPB minimum frame size
(`SPB_FIXED_MIN = 16 bytes: btl=12 → body=0; btl=16 → body=4`). The prose was technically
correct but omitted the anchoring minimum, making it harder for an implementer to derive
the constructible-fixture window without cross-referencing BC-2.01.013.

**Fix:** error-taxonomy v3.4→v3.5: E-INP-008 row SPB body-too-short entry amended to
explicitly cite `SPB_FIXED_MIN=16` and clarify that btl=12 is the crate-minimum (body=0,
all 0 bytes < 4 → E-INP-008) while btl=16 is the minimum valid SPB (body=4 = exactly
SPB fixed-field size, no error from body-decode alone). Boundary note cross-references
BC-2.01.013 EC-008.

**Status: FIXED** — error-taxonomy v3.5. D-161.

### M-2: IDB body-decode holdout gap (FIXED)

**Finding:** IDB (BC-2.01.011) was the only framing BC without a dedicated body-decode
error-path holdout scenario. HS-101..108 covered SHB (HS-103), EPB (HS-104), SPB (HS-107),
and block-walk (HS-105), but no holdout exercised the IDB body-decode error paths
(constructible body-short window 12<=btl<20, malformed option TLV, if_tsresol option_length
mismatch). This gap meant VP-026 (IDB linktype whitelist) had holdout coverage (HS-106)
but the body-decode error cluster did not.

**Fix:** HS-109 authored (IDB body-decode framing error paths — 5 cases):
(a) btl=16 → body=4 bytes < 8 IDB fixed-field bytes → E-INP-008;
(b) reserved field != 0 → E-INP-008 (semantic validation);
(c) options-TLV option_length exceeds remaining body bytes → E-INP-008;
(d) if_tsresol option_length = 4 (not 1) → E-INP-008 (malformed option format per AC-005);
(e) positive control: valid IDB btl=20 (body=8, minimal options) → idb parsed, if_tsresol
extracted correctly.
HS-INDEX v2.3→v2.4: greenfield total 108→109; all-namespace total 181→182; must_pass
count 108 (HS-109 is must_pass: true).

**Status: FIXED** — HS-109 v1.0 authored; HS-INDEX v2.4. D-161.

### M-3: BC-2.01.013 AC-001 test name drift (DF-AC-TEST-NAME-SYNC-001) (FIXED)

**Finding:** BC-2.01.013 AC-001 specified test name
`test_BC_2_01_013_snaplen_lookup_guarded`. The "snaplen" reference is stale — snaplen was
removed from the SPB path in ADR-009 rev 8 Decision 9 amendment (D-153). The AC-001 guard
is solely for the empty-interface-table path (E-INP-009), not a snaplen guard. The stale
test name violates DF-AC-TEST-NAME-SYNC-001 and would produce a test suite where the test
name contradicts the tested behavior.

**Fix:** BC-2.01.013 v1.7→v1.8: AC-001 test name renamed
`test_BC_2_01_013_snaplen_lookup_guarded` → `test_BC_2_01_013_empty_interface_table_guarded`.
Scope note clarified: AC-001 guards the empty-interface-table (E-INP-009) path; body-too-short
(btl=12 → body=0 < 4 → E-INP-008) is handled distinctly by AC-004a/EC-008, making these
ACs non-redundant. No normative behavior change.

**Status: FIXED** — BC-2.01.013 v1.8. D-161.

---

## Low Findings (5 items — all verified CLEAN or CONVERGED)

### L-1: VP-INDEX total_vps=31 count propagation

**Status: CONVERGED GREEN** — verified 31 VPs on disk; VP-INDEX v2.8 count correct. No
action required.

### L-2: error-taxonomy next_free E-INP-014 confirmed

**Status: CONVERGED GREEN** — confirmed E-INP-014 is the next free code; no new error
codes added in pass-7 remediation. No action required.

### L-3: BC count 302 propagation

**Status: CONVERGED GREEN** — 302 active BCs confirmed in BC-INDEX; no BC additions or
retirements in pass-7/8 remediation. No action required.

### L-4: HS-INDEX all-namespace count verified

**Status: FIXED (via M-2)** — HS-INDEX v2.4 updated all-namespace to 182 (was 181 before
HS-109 addition). Greenfield count now 109.

### L-5: ADR-009 status field confirmed accepted

**Status: CONVERGED GREEN** — ADR-009 `status: accepted` confirmed on disk (rev 9). No
action required.

---

## Process-Gap Observations

### O-1: No machine-checkable framing-constant validator (DEFERRED TO F3)

**Observation:** Six documents independently state the same per-block constants
(EPB_FIXED_OVERHEAD_BYTES=28, SPB fixed-field bytes=4, SHB minimum body=16, etc.). There
is no cross-document validator that would catch a future drift in any one of these constants.
The same class of error was the root cause of H-2 (pass-1), C-3 (pass-2), and the SPB
three-way-min propagation failures.

**Disposition:** DEFERRED TO F3 — a cross-branch `bin/` script that grepped all 6 documents
for constant values would be a useful process improvement but requires a bin/ addition on
the develop branch, which is out of scope for the current F2 spec-only phase. The F3 story
decomposition checklist (STATE.md Section D) should include an item to evaluate whether a
framing-constant validator should be added to `bin/` as part of the implementation stories.

**Status: DEFERRED-TO-F3** — no remediation required for F2 convergence.

### O-2: ADR-009 status was `proposed` (FIXED in this burst)

**Observation:** ADR-009 had `status: proposed` despite having been adopted as the
architectural decision for pcapng library selection (pcap-file 2.0.0, Option A) since rev 1.
Pass-8 first-pass review confirmed this field was stale; it should be `accepted`.

**Fix:** ADR-009 rev 9 `status: proposed` → `status: accepted`. No other content changed.

**Status: FIXED** — ADR-009 status=accepted. D-161.

---

## Verification Summary

| Check | Result |
|-------|--------|
| All framing BCs (BC-2.01.009..018) have VP + holdout | PASS |
| Per-block constants agree across 6 docs | PASS |
| Holdout fixtures arithmetically self-consistent | PASS |
| All 4 pass-7 fixes propagated (no stale siblings) | PASS |
| error-taxonomy next_free E-INP-014 | PASS |
| VP-INDEX total 31 | PASS |
| BC count 302 | PASS |
| HS count 109 greenfield / 182 all-namespace | PASS (post M-2 fix) |
| ADR-009 status accepted | PASS (post O-2 fix) |

---

## Remediation Summary

**Artifacts updated in this burst (4 artifacts):**

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| error-taxonomy | v3.4 | v3.5 | M-1 (SPB-fixed-min prose gap) |
| HS-109 | — | v1.0 (new) | M-2 (IDB body-decode holdout gap) |
| HS-INDEX | v2.3 | v2.4 | M-2 (HS-109 added; greenfield 109; all-namespace 182) |
| BC-2.01.013 | v1.7 | v1.8 | M-3 (AC-001 test name renamed; scope note clarified) |
| ADR-009 | rev 9 | rev 9 (status field only) | O-2 (status: proposed → accepted) |
| BC-INDEX | v1.65 | v1.66 | BC-2.01.013 v1.7→v1.8 annotation sync |
| spec-changelog | — | — | [pcapng-f2-pass8-clean-and-medium-remediation-2026-06-20] prepended |
| STATE.md | — | — | phase_status updated; D-161 decision log entry; clean-pass counter 1/3 |
| f2-review-remediation-tracker.md | — | — | M-1/M-2/M-3/O-2 FIXED; O-1 DEFERRED-TO-F3; CLEAN-PASS 1/3 |

**Pass-8 verdict:** CLEAN — 0C / 0H / 3M / 5L. This is **clean-pass 1 of 3 required**.
All 3 MEDIUM findings remediated in this burst. O-1 DEFERRED-TO-F3 (process improvement,
non-blocking). Adversary pass-9 pending (targeting clean-pass 2/3). F3 remains BLOCKED
until 3 consecutive clean passes achieved.
