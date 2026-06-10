# Phase F5 Adversarial Delta Review — Feature #7 Modbus TCP Analyzer (Combined-Delta)

## Metadata

| Field | Value |
|-------|-------|
| Scope | Complete Modbus analyzer — all 4 stories (STORY-102/103/104/105) delivered Wave 2 |
| develop HEAD at review | `dba5f26` |
| Reviewers | Claude (`vsdd-factory:adversary`) + Gemini CLI (gemini 0.44.1, cross-model — genuine non-Claude family) |
| Review type | Combined-delta (full analyzer, not per-story) |
| Date | 2026-06-09 |
| Stories | STORY-102, STORY-103, STORY-104, STORY-105 |
| Fix PR | `fix/f5-modbus-timestamp-units` |

---

## Convergence Verdict

**CONVERGED** (re-pass after spec + code fix; all findings RESOLVED or DEFERRED with justification).

---

## Findings

### CRITICAL

#### F-DELTA-001 — Timestamp Units Mismatch: microseconds vs seconds

**Severity:** CRITICAL
**Category:** correctness / detection-math
**Filed by:** Claude adversary (primary) + Gemini (independently — cross-model agreement)
**Status:** RESOLVED

**Root cause:** `process_pdu` treated the `on_data(timestamp: u32)` parameter as microseconds when
computing sustained-write-rate and burst-window detection. The pipeline's `StreamHandler::on_data`
delivers `timestamp_secs` (seconds per BC-2.09.007). TLS, HTTP, and the reassembler all confirm
seconds. The Modbus implementation diverged.

**Impact:**
1. Sustained-window math used `elapsed_us / 1_000_000` as if `elapsed_us` were microseconds —
   actually dividing seconds-values by 1,000,000, producing near-zero elapsed time, making the
   sustained-rate detector functionally dead.
2. `DateTime::from_timestamp(ts_us as i64, 0)` attached nanosecond-epoch timestamps to every
   Finding (ts_us of ~1,500,000,000 = year 2017 as seconds; treated as microseconds = year 1970
   + ~25 minutes).
3. Exception burst windows (10-second intent) fired as ~10-microsecond windows (always expired
   immediately).

**Both models independently rated CRITICAL.** Cross-model agreement on root cause and severity.

**Resolution:**
- Code (`fix/f5-modbus-timestamp-units` PR): windows use `timestamp_secs` directly; division by
  1,000,000 eliminated; `DateTime::from_timestamp(ts as i64, 0)` corrected.
- 78 test timestamps legitimately corrected from microsecond-shaped values (e.g., 1_000_000) to
  seconds-shaped values (e.g., 1) — not a weakening; the old values were wrong per the pipeline contract.
- E2E dispatcher test + EC-005 cross-window test bind the corrected behavior.
- Spec: SS-14 BCs reconciled (BC-2.14.016 v2.0->v2.1; BC-2.14.017 v2.1->v2.2; BC-2.14.019 v1.1->v1.2;
  BC-2.14.013 v2.1->v2.2 window math seconds-based; f2-fix-directives §11.5/§11.5b F5-correction banners).

---

### HIGH

#### F-DELTA-002 — Dead Counter: `total_flows_analyzed` (BC-2.14.021 post.3 unsatisfiable)

**Severity:** HIGH
**Category:** correctness / dead-code
**Filed by:** Claude adversary (primary)
**Status:** RESOLVED

**Root cause:** `summarize()` emitted a `total_flows_analyzed` field that was always 0 because the
counter was never incremented anywhere in `on_data`. BC-2.14.021 post-condition 3 cited this field
as a required summary output; the implementation could never satisfy it.

**Resolution:** Counter increment added at the correct site in `on_data`. BC-2.14.021 v1.0->v1.1
revised to document the authoritative 6-key summary struct and correct the post.3 struct mismatch.

---

#### F-DELTA-003 — `length_invalid` Non-Latching: Desync After Partial-ADU

**Severity:** HIGH
**Category:** correctness / security
**Filed by:** Claude adversary (primary)
**Status:** RESOLVED

**Root cause:** When MBAP length field was invalid, `is_non_modbus` was set but not latched. On the
next call to `on_data` with new data the flag reset, allowing subsequent PDUs from a desynchronized
stream to be parsed as valid Modbus — defeating the bail-out invariant.

**Resolution:** `is_non_modbus` latch made sticky (never reset after first invalid-length detection).
BC-2.14.019 v1.1->v1.2 updated to reflect the latching invariant.

---

#### F-DELTA-004 — Flush Granularity: Window Fidelity on `on_close`

**Severity:** HIGH
**Category:** correctness
**Filed by:** Gemini (cross-model, primary on this finding)
**Status:** RESOLVED

**Root cause:** `on_close` flushed the sustained-rate window using the last-seen timestamp rather than
a sentinel that forces any in-progress window to be evaluated. For a flow that closes mid-window,
the accumulated writes could be lost without generating a finding.

**Resolution:** `on_close` now forces window evaluation at the last-seen timestamp, ensuring in-flight
sustained-rate accumulations are checked before the flow is dropped.

---

#### F-DELTA-005 — `source_ip` Drawn from Non-Existent `flow_key.client_ip()`

**Severity:** HIGH
**Category:** correctness / spec defect
**Filed by:** Claude adversary (primary); Gemini confirmed
**Status:** RESOLVED

**Root cause:** Multiple BCs (BC-2.14.013, BC-2.14.016, BC-2.14.019) cited `flow_key.client_ip()`
as the source of `source_ip` for emitted Findings. The `FlowKey` type has no `client_ip()` method;
the correct field is `Direction`-based lookup. The implementation used the wrong path, causing
`source_ip = None` on all Findings.

**Resolution:** `source_ip` now drawn from `Direction` at the correct call site. BCs reconciled
to reference `Direction` rather than the non-existent `flow_key.client_ip()`. This also closes
the D-041 "source_ip=None on all findings" defect previously noted.

---

### Spec Defect (Non-Finding Category)

**SD-001 — BCs cite non-existent `flow_key.client_ip()` method.** Propagated across BC-2.14.013,
BC-2.14.016, BC-2.14.019 and BC-2.14.021. All corrected as part of F-DELTA-005 resolution above.

---

## Deferred / Out-of-Scope

**Sub-second rate precision:** Using `timestamp_secs` (u32 seconds) means the minimum measurable
burst window is 1 second. Sub-second precision would require `timestamp_usecs` threaded through
`on_data` — not currently in the pipeline contract. Deferred; would be a new BC addition.

---

## Process-Gap Observation (Codification Candidate)

**[process-gap]:** The per-story adversarial perimeter (F4 wave-by-wave reviews) had no axis
requiring a test driving through the REAL `StreamHandler::on_data` entry point with
pipeline-realistic (seconds) timestamps. The seconds-vs-microseconds seam was structurally
undetectable in per-story isolation — each story's tests wired timestamps internally with
arbitrary values, none shaped by the pipeline contract.

The F5 combined-delta view (full analyzer, fresh context, both models seeing the complete
implementation at once) surfaced this as CRITICAL because it could compare `on_data` contract
(seconds per BC-2.09.007) against the implementation's actual arithmetic.

**Candidate codification:** Any analyzer consuming `on_data(timestamp: u32)` MUST have at least
one test driving through the dispatcher/reassembler boundary with `timestamp_secs`-shaped values
(i.e., wall-clock epoch seconds, not microseconds). This test class should be an explicit
adversarial axis in the per-story review checklist for binary-protocol analyzers.

---

## Summary Table

| ID | Severity | Category | Filed By | Status |
|----|----------|----------|----------|--------|
| F-DELTA-001 | CRITICAL | correctness/detection-math | Claude + Gemini (independent) | RESOLVED |
| F-DELTA-002 | HIGH | correctness/dead-code | Claude | RESOLVED |
| F-DELTA-003 | HIGH | correctness/security | Claude | RESOLVED |
| F-DELTA-004 | HIGH | correctness | Gemini | RESOLVED |
| F-DELTA-005 | HIGH | correctness/spec-defect | Claude + Gemini | RESOLVED |
| SD-001 | Spec defect | non-existent API reference | Claude + Gemini | RESOLVED |

**Totals:** 1 CRITICAL + 4 HIGH + 1 spec defect. ALL RESOLVED. Re-pass: CONVERGED.
