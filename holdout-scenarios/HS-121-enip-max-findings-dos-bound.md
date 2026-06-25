---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f3
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.022.md
  - .factory/stories/STORY-138.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-121"
category: "security-probes"
must_pass: "true"
priority: "must-pass"
epic_id: "E-20"
behavioral_contracts:
  - BC-2.17.022
lifecycle_status: active
introduced: v0.11.0-feature-enip
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
fixture_needed: true
fixture_note: "Requires crafted pcap with more than MAX_FINDINGS=10000 CIP Stop frames in one TCP flow (e.g., 10001 frames). Each Stop frame would normally emit T0858. The fixture must be large but can be generated programmatically."
---

# Holdout Scenario: MAX_FINDINGS DoS Bound — Finding Count Stays at Cap Under Flood

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

The `EnipAnalyzer` enforces a hard cap of `MAX_FINDINGS = 10,000` on the total number of
findings stored in `all_findings`. Under a flood of detecting frames (e.g., CIP Stop frames,
each of which would normally emit T0858), the finding count must never exceed 10,000. The
tool must not crash, not exhaust memory, and not panic — and it must complete analysis
within a reasonable time.

Note: CIP Stop has no one-shot guard (each Stop frame emits one T0858). This makes CIP
Stop the ideal flood vector for testing the MAX_FINDINGS cap, since 10,001 Stop frames
would normally emit 10,001 T0858 findings without the cap.

### Case A — 10,001 CIP Stop Frames Produces Exactly 10,000 T0858 Findings

1. A crafted PCAP is presented: one TCP flow on port 44818; 10,001 ENIP SendRRData frames,
   each carrying CPF 0x00B2 item with CIP service=0x07 (Stop request).
   Note: This is a large fixture (~28 bytes * 10,001 ≈ 280 KB of TCP payload; a few hundred
   KB PCAP). Programmatic generation with scapy or a custom writer is required.
2. The user runs: `wirerust analyze enip_stop_flood_10001.pcap --enip --json`
3. The tool exits 0 (no panic, no crash, no OOM).
4. The evaluator inspects the JSON output. The total count of T0858 findings must be
   EXACTLY 10,000 (not 10,001 or more). The 10,001st Stop frame was suppressed by the cap.
5. Analysis completes in finite time (evaluator should allow up to 60 seconds for a large
   fixture on typical hardware).

### Case B — Findings Cap Does Not Prevent State Counter Updates

The JSON summary statistics (pdu_count, aggregate write_count if exposed) should reflect
the actual number of frames processed, NOT just 10,000. Specifically:
- If the summary exposes `pdu_count` or `frames_analyzed`, it should be >= 10,001 (all
  frames were walked and processed in the frame-walk loop; only finding emission was capped).
- Per BC-2.17.022: state counters are updated even after the cap is reached; only finding
  pushes are suppressed.
- The evaluator checks: `pdu_count >= 10001` OR (if pdu_count is not exposed) that the
  summary does not show obvious truncation at exactly 10,000 frames processed.

### Case C — Tool Does Not OOM or Panic

The evaluator must confirm the tool completes (exit code 0, no SIGKILL due to OOM, no
Rust panic abort) for the 10,001-frame fixture. This is the DoS-resistance verification:
a hostile pcap cannot cause the analyzer to allocate unbounded memory.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.17.022 | Postcondition 1 — no new Finding pushed when all_findings.len() >= MAX_FINDINGS | Case A: 10,001st Stop frame produces no additional T0858 |
| BC-2.17.022 | Postcondition 2 — all_findings.len() stays at MAX_FINDINGS | Case A: exactly 10,000 findings in output |
| BC-2.17.022 | Postcondition 4 — per-flow state counters still updated after cap | Case B: pdu_count reflects true frame count |
| BC-2.17.022 | Invariant — MAX_FINDINGS = 10,000 (hard cap) | Case A: exactly 10,000, not 10,001 |

<!-- HIDDEN TRACEABILITY: BC-2.17.022 Postcondition 3 (dropped_findings incremented for each suppressed finding); BC-2.17.022 Postcondition 5 (one-shot guards NOT set when finding dropped) -->

## Fixture Creation Obligation

**F4 must create:**
`enip_stop_flood_10001.pcap` — TCP flow dst port 44818; 10,001 consecutive ENIP SendRRData
frames (24-byte ENIP header + minimal CPF 0x00B2 item with CIP service=0x07) concatenated
in the TCP byte stream. Python scapy or a raw PCAP writer can generate this programmatically
in seconds. Estimated size: ~280 KB PCAP (28 bytes/frame × 10,001 frames + Ethernet/IP/TCP
headers per packet; can pack many frames per TCP segment to reduce packet count).

## Verification Approach

```bash
wirerust analyze enip_stop_flood_10001.pcap --enip --json 2>&1 | jq '
  (.findings | length) as $n |
  if $n == 10000 then "PASS: exactly 10000 findings"
  else "FAIL: got \($n) findings (expected 10000)"
  end
'
# Also check: no panic on stderr (look for "thread .* panicked at")
# Also check: exit code 0.
```

If the JSON output does not expose findings as an array, the evaluator counts T0858
occurrences by grepping the output for `"T0858"` and counting.

## Evaluation Rubric

- **Cap enforced at 10,000** (weight: 0.50): Exactly 10,000 T0858 findings in output.
  More than 10,000 means the cap check (`all_findings.len() < MAX_FINDINGS`) is missing
  or off-by-one. Fewer than 10,000 means some frames were not processed or a one-shot
  guard was incorrectly applied.
- **No panic or OOM** (weight: 0.30): Tool exits 0 without panic; no crash report; no
  out-of-memory termination. This is the core DoS-resistance property.
- **State counters not capped** (weight: 0.10): pdu_count or equivalent >= 10,001 in
  summary (frames were walked; only findings were suppressed).
- **Performance** (weight: 0.10): Analysis completes within 60 seconds on typical hardware.

## Failure Guidance

"HOLDOUT FAIL: HS-121 — MAX_FINDINGS cap not enforced or tool panics under flood. If finding
count > 10,000, the cap check is missing or the condition is wrong (ensure: if
all_findings.len() < MAX_FINDINGS { push finding; } else { dropped_findings += 1; }). If
the tool panics or OOMs, the findings Vec is growing without bound — verify the cap check
is applied at EVERY detection site (T0858, T0816, T0836, T0846, T0888, T0814, ForwardOpen).
See BC-2.17.022 Invariant (MAX_FINDINGS = 10_000 is a hard cap, not a soft limit)."
