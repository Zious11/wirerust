# Review Findings — STORY-153

**PR:** #352 — feat(dispatcher): unclassified-protocol gap counters (TCP + UDP)
**Branch:** feature/story-153-unclassified-counters @ ff91fd8
**Date:** 2026-07-03

## Convergence Tracking

| Cycle | Reviewer | Findings | Blocking | Fixed | Remaining |
|-------|---------|----------|----------|-------|-----------|
| 1 | pr-reviewer | 4 INFO | 0 | 0 | 0 INFO (no action needed) |
| 1 | security-reviewer | 1 LOW | 0 | 0 deferred | 1 LOW deferred to STORY-154 |
| — | — | **APPROVE** | **0** | — | **0 blocking** |

**Status: CONVERGED in 1 cycle. 0 blocking findings.**

---

## AI PR Review (Cycle 1)

**Verdict: APPROVE**

### Invariant Verification (all 9 pass)

| # | Invariant | Status |
|---|-----------|--------|
| 1 | TCP gap key uses lower_port().min(upper_port()) | PASS |
| 2 | unclassified_flows gated on analyzer-present only; port counter nested inside coverage_gaps_enabled | PASS |
| 3 | udp_gap_key is pub fn free function | PASS |
| 4 | classify() and DispatchTarget unchanged | PASS |
| 5 | TransportProto defined in dispatcher.rs, not protocols.rs | PASS |
| 6 | saturating_add via correct let c/c.saturating_add(1) pattern | PASS |
| 7 | VP-042 has exactly 3 sub-harnesses | PASS |
| 8 | #[allow(non_snake_case)] at module scope | PASS |
| 9 | coverage_gaps is new scalar on run_analyze(), hard-passed false | PASS |

### INFO Observations (non-blocking, no action required)

| # | Observation | Triage | Route |
|---|------------|--------|-------|
| INFO-1 | No explicit test for coverage_gaps_enabled=true + zero analyzers registered | ACCEPTED — outer guard already tested elsewhere | No fix |
| INFO-2 | unclassified_port_counts worst-case 65k entries, acceptable for pcap analytics | ACCEPTED — bounded by design, document at STORY-154 enablement | Deferred to STORY-154 |
| INFO-3 | coverage_gaps dead until STORY-154 wires CLI flag | BY DESIGN — comment in code | No fix |
| INFO-4 | Accessor doc slightly loose ("empty map" rather than "reference to unpopulated field") | ACCEPTED — not misleading | No fix |

---

## Security Review (Cycle 1)

**Verdict: APPROVE**

### Findings

| ID | Severity | CWE | Description | Disposition |
|----|---------|-----|-------------|-------------|
| SEC-001 | LOW | CWE-400 | HashMap accumulation bounded by u16 key space (max 65535 keys/~3MB total). No capacity cap in code. | DEFERRED to STORY-154 — no code change required this wave; document ceiling at CLI flag enablement |

### Clean Areas
- Integer overflow: saturating_add correctly applied (CWE-190: N/A)
- Unsafe code: none added (CWE-119: N/A)
- Injection: no user-controlled input to system calls (CWE-78: N/A)
- Information disclosure: udp_gap_key is pure/stateless (CWE-200: N/A)
- Gate bypass: coverage_gaps_enabled immutable post-construction (CWE-284: N/A)
- ADR-012 D10 compliance: dns_analyzer.can_decode() correctly called regardless of enable_dns

---

## Final Disposition

**READY-TO-MERGE: YES** (subject to CI and human approval)

- 0 blocking findings from AI PR review
- 0 CRITICAL/HIGH security findings
- 1 LOW security finding deferred to STORY-154 (no code change this wave)
- All 9 load-bearing invariants verified in diff
- Diff confirmed: exactly 3 files (src/dispatcher.rs +110, src/main.rs +29, tests/dispatcher_tests.rs +654)
