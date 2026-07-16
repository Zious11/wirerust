# PR #408 Fresh-Eyes Review — STORY-173 IEC-104 Dispatcher Integration

**Reviewer:** pr-reviewer (fresh-eyes, different model family)
**PR:** #408 — feat: STORY-173 IEC-104 dispatcher integration + T0881 catalog + --iec104 flag + findings cap (wave-82)
**Base:** develop ← feature/STORY-173-iec104-dispatcher-integration
**Date:** 2026-07-15

## Overall Verdict: APPROVE

No CRITICAL or HIGH findings. The 6-subsystem diff is coherent, well-traced, and internally
consistent. Two LOW observability findings and one INFO note are recorded below; none block merge.

## What I Verified

- **Dispatcher (`src/dispatcher.rs`):** `DispatchTarget::Iec104` variant added; `classify()`
  Rule 8 (`ports.contains(&2404) → Iec104`) placed correctly after Rule 7 (ENIP) and before the
  `None` fallthrough; the `#[cfg(kani)]` `classify_oracle` mirrors production exactly (same arm
  order). Early-exit guard correctly extended with `&& self.iec104.is_none()`; `on_data` and
  `on_flow_close` Iec104 arms mirror the ENIP arms; `new()` extended to 6 params with all call
  sites updated (benches, `main.rs` f6 test, every dispatcher/bc test).
- **MITRE (`src/mitre.rs`):** `technique_info("T0881") → ("Service Stop",
  IcsInhibitResponseFunction)`. Tactic mapping verified against MITRE ATT&CK for ICS — T0881
  "Service Stop" → Inhibit Response Function = TA0107. `SEEDED_TECHNIQUE_IDS` (28→29),
  `SEEDED_TECHNIQUE_ID_COUNT=29`, and `EMITTED_IDS` (20→21) bumped in lockstep; drift-guard tests
  cover both directions. T0881 is genuinely emitted (dispatcher STOPDT-act test produces a real
  T0881 finding), so its inclusion in `EMITTED_IDS` is correct.
- **CLI/main (`src/cli.rs`, `src/main.rs`):** `--iec104` flag default-off, `*iec104 || *all`
  wiring, `needs_reassembly` includes `enable_iec104`, `--no-reassemble` warning path present,
  analyzer constructed only when `enable_iec104 && !skip_reassembly`, and
  `take_iec104_analyzer()`/`summarize()` collected post-finalize (mirrors ENIP).
- **Protocols (`src/protocols.rs`):** Port 2404 added to `SUPPORTED_PORTS` (9 entries);
  `supported_protocols()` → 8, `unsupported_protocols()` → 22; promoted-in-place via port-filter
  with the partition invariant preserved.
- **Findings cap (`src/analyzer/iec104.rs`):** `MAX_IEC104_FINDINGS = 10_000`,
  `dropped_findings: u64`, cap enforced at the `on_data` extend step via `saturating_sub` +
  `truncate` + `saturating_add`. Boundary (MAX-1 → MAX, no drop) and multi-call behavior correct
  and tested.

## Findings

| # | Severity | Category | Location | Finding | Suggested Fix |
|---|----------|----------|----------|---------|---------------|
| 1 | LOW | observability | `src/analyzer/iec104.rs` `summarize()` | `flows_analyzed` is set to `self.flows.len()` (open flows at summarize time). In the production pipeline `summarize()` runs after `reassembler.finalize()`, which drives `on_flow_close()` for every flow, and `on_flow_close` does `self.flows.remove(&flow_key)`. So this metric will report ~0 in the real CLI run regardless of flows analyzed. ENIP avoids this with a persistent cumulative counter. Not a BC violation (only `dropped_findings` is mandated), but the emitted value is misleading. | Track a cumulative flow counter incremented in `on_flow_close`, or drop the key. Non-blocking. |
| 2 | LOW | consistency | `src/analyzer/iec104.rs` `summarize()` | `packets_analyzed` is set to `self.all_findings.len()` (a findings count), whereas every other analyzer reports an actual packet/frame count. Documented as an intentional proxy, but it is a semantic mismatch that can confuse summary consumers. | Use a real frame counter or a neutral value in a follow-up. Non-blocking. |
| 3 | INFO | demo-evidence | `docs/demo-evidence/STORY-173/` | Evidence is markdown (CLI `--help` excerpts, source confirmation, test transcripts) — no `.gif`/`.webm`. For a dispatcher-wiring + catalog-registration change there is no meaningful visual surface, and this matches the established convention for the IEC-104 series (STORY-167–172). Evidence is substantive and per-AC. | Accepted for non-visual infra story. Not blocking. |
| — | ACCEPTED | cosmetic | `tests/iec104_analyzer_tests.rs` | **A-12-01** — pre-implementation "stub"-tense wording in FAILURE-ONLY assert messages. Confirmed present, confirmed cosmetic-only. Not re-raised as blocking. | Pre-accepted advisory. |

## Additional Notes

- Diff coherence: all changes map to STORY-173; no unrelated changes. ADR-0013 is committed on
  the branch. CHANGELOG has a proper `[Unreleased] → Added` entry satisfying the changelog-gate.
- Diff size >500 lines, but the bulk is mechanical `None` param additions to existing
  `StreamDispatcher::new()` call sites plus 18 new tests and demo evidence — reasonable for scope.
- Test evidence: 2602/2602 PASS; 18 new tests across dispatcher/mitre/protocols/iec104 modules,
  row-verified against the per-AC coverage map.
- Findings #1 and #2 are pre-existing-pattern deviations, not regressions, and neither affects
  detection correctness or the BC-mandated `dropped_findings` surfacing.

**Verdict: APPROVE.**
