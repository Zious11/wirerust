---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs: [stories/, behavioral-contracts/, prd.md]
input-hash: "[md5-pending]"
traces_to: ""
id: "HS-090"
category: "real-world-corpus"
must_pass: "true"
priority: "must-pass"
epic_id: "E-9"
behavioral_contracts:
  - BC-2.12.001
  - BC-2.12.008
  - BC-2.12.016
  - BC-2.12.021
  - BC-2.11.001
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: End-to-End pcap → JSON Report Pipeline (Real-World Clean Corpus)

## Scenario

A forensic analyst runs wirerust against a real-world, publicly-available pcap from a known-
good capture session. The tool must produce a valid, well-structured JSON report with accurate
summary statistics, zero false-positive findings, and correct protocol distribution — all
within a reasonable wall-clock time.

**Known-good corpus:** The Wireshark sample captures at
`https://wiki.wireshark.org/SampleCaptures` — specifically the HTTP example capture
(`http.cap`) which is a well-maintained, fully-parsed reference pcap. Expected behavior:
very few or zero security findings, correct packet and byte counts.

1. The tool is invoked: `wirerust analyze --all --json http.cap`
2. The tool exits with code 0.
3. The stdout output is valid JSON that parses without error.
4. The JSON contains a `"summary"` object with:
   - `"total_packets"` > 0 (the capture has packets)
   - `"total_bytes"` > 0
   - `"skipped_packets"` == 0 (clean, well-formed capture)
5. The `"findings"` array contains zero or very few entries (< 3) — this is a known-clean
   capture; many findings would indicate a false-positive problem.
6. The `"protocols"` map in the summary contains `"Tcp"` with a count matching the number
   of TCP packets in the capture.
7. The `"services"` map contains `"HTTP"` with a positive count.
8. Total run time is under 30 seconds for a capture of this size (< 5 MB).

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.12.001 | Postcondition 1: analyze subcommand accepts target | Successful invocation with a real pcap |
| BC-2.12.008 | Postcondition 1: --all enables all three analyzers | All analyzers run in this end-to-end test |
| BC-2.12.016 | Postcondition 1: --json selects JSON output | Output is JSON format |
| BC-2.12.021 | Postcondition 1: summary has correct fields | JSON summary block completeness |
| BC-2.11.001 | Postcondition: JSON output is valid RFC 8259 | Structural validity of output |

## Verification Approach

1. Download `http.cap` from Wireshark sample captures.
2. Run: `wirerust analyze --all --json http.cap`
3. Assert exit code == 0.
4. Parse stdout as JSON; assert the top-level object has keys: `"summary"`, `"findings"`.
5. Assert `summary.total_packets` > 0.
6. Assert `summary.skipped_packets` == 0.
7. Assert `findings` array length < 3 (low false-positive threshold for clean capture).
8. Assert `summary.protocols` contains `"Tcp"` key.
9. Assert run time (wall clock) < 30 seconds.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): Valid JSON with correct structure; exit code 0.
- **Edge case handling** (weight: 0.1): No crashes on real-world pcap content.
- **Error quality** (weight: 0.1): No spurious warnings on a clean capture.
- **Performance** (weight: 0.2): Under 30 seconds for < 5 MB pcap with --all analyzers.
- **Data integrity** (weight: 0.2): False-positive rate < 3 findings on known-clean traffic; skipped_packets = 0.

### Real-World Corpus Metadata

| Field | Description |
|-------|-------------|
| corpus_source | Wireshark Sample Captures: https://wiki.wireshark.org/SampleCaptures (http.cap) |
| corpus_size | ~50 KB, ~43 packets |
| known_edge_cases | Standard HTTP GET/response; no TLS, no DNS; standard Ethernet/IPv4/TCP framing |
| false_positive_threshold | < 3 findings (known-clean traffic) |
| false_negative_threshold | N/A (clean corpus; we test false positives here) |

## Edge Conditions

- If `http.cap` cannot be downloaded, substitute any well-known Wireshark sample capture with HTTP traffic.
- The tool should handle the capture without panicking even if some packets have unusual framing.
- If `skipped_packets > 0`, that indicates a decode problem with a real-world packet — investigate.

## Failure Guidance

"HOLDOUT LOW: HS-090 (satisfaction: 0.XX) -- End-to-end pipeline failed on real-world pcap: either invalid JSON output, non-zero exit code, excessive false positives on known-clean traffic, or run time exceeded 30 seconds."
