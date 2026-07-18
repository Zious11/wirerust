# PR Review — #411 (FIX-F5-001)

**Title:** fix(FIX-F5-001): enrich IEC-104 findings with source_ip + timestamp; scrub stale test prose
**Branch:** fix/FIX-F5-001 (157fa71) → develop (7e95f71)
**Reviewer:** pr-reviewer (fresh-eyes, diff-only)

## Verdict: APPROVE (no blocking findings)

The fix is correct, faithfully mirrors the DNP3/EtherNet/IP house pattern, and is
well-tested. Two non-blocking documentation-accuracy findings noted below.

---

## What I verified (8-item checklist)

1. **Diff coherence** — All changes relate to FIX-F5-001: source_ip/timestamp
   enrichment (src), new `mod fix_f5_001` tests + mechanical caller sweep, stale-prose
   rewrites, count fix, CHANGELOG, demo evidence, holdout sweep. No unrelated changes.
2. **Description accuracy** — Mostly accurate; one count overstatement (see MINOR-1).
3. **Test coverage** — `mod fix_f5_001` adds 10 tests over 9 finding families, each
   asserting `source_ip == Some(expected)` AND `timestamp.is_some()`, driven end-to-end
   through `on_data`. Both C2S (client_ip=10.0.0.1) and S2C (server_ip=10.0.0.2) covered.
4. **Demo evidence** — `docs/demo-evidence/FIX-F5-001/evidence-report.md` present, maps
   ACs to tests with before/after JSON. (Text/markdown evidence; acceptable for a
   library/CLI JSON-output change with no visual surface.)
5. **Commit quality** — 5 conventional commits, all scoped `(FIX-F5-001)`, clear messages.
6. **Diff size** — 986 insertions, but dominated by mechanical `None, None` test-caller
   sweep (~60 sites) and two new doc files. Source delta is only +89. Acceptable.
7. **Missing changes** — None. All 5 findings (F-01…F-05) addressed.
8. **Dependency status** — Builds on FIX-P4-001 (direction key) already on develop.

## Question-by-question

- **All 10 emit sites updated:** Confirmed — `process_u_frame` (T0881, T0814),
  `detect_iec104_threats` (T1692 45-47, T1692 48-51, T0836, T0827, T0814-reserved),
  `track_ns_desync` (T1692), 2 inline in `on_data` (carry-overflow, malformed-LEN).
- **3 signature changes + callers swept:** Confirmed; all test callers updated.
- **DNP3 parity:** Exact — `if flow_key.lower_port() == 2404 { ... } match direction`
  is structurally identical to dnp3.rs (port 20000) and enip.rs (port 44818). Timestamp
  uses the same `chrono::DateTime::from_timestamp(ts as i64, 0)` house call.
- **9 prose rewrites GREEN:** Confirmed; story_173 stub language and the F-04 false
  forward-ref (`enriched in STORY-173` → `passed in by the caller (FIX-F5-001)`) fixed.
- **CHANGELOG:** In `[Unreleased] → Fixed`, traces FIX-F5-001 + BC-2.19.011 PC-3.

---

## Findings

| Severity | Category | Finding | Suggestion |
|----------|----------|---------|------------|
| MINOR | description | CHANGELOG.md and evidence-report.md claim "12 total Finding constructors — 10 via function parameters and 2 inline." Actual is 10 total (8 in helper functions + 2 inline), matching the commit message ("all 10 emit sites") and the demo report's own 10-row table. Overcount by 2. | Correct prose to "10 total (8 function + 2 inline)". Aligns with PG-W74-PRDESC-ROW-VERIFY count cross-check. |
| NIT | description | evidence-report.md and holdout-sweep doc describe `on_data`'s timestamp as a "nanosecond-precision u64 timestamp." Signature is `ts: u32`, interpreted as seconds via `from_timestamp(ts as i64, 0)`. Harmless (tests assert only `.is_some()`). | Reword to "u32 seconds-precision timestamp" to match the code and DNP3 parity. |

Neither finding blocks merge. Recommend a documentation touch-up for the count prose.
