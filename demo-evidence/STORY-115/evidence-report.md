# STORY-115 Demo Evidence Report

**Story:** STORY-115 — D3 ARP Storm Detection + `--arp-storm-rate` + `storm_findings` Summary Key  
**Epic:** E-16 (ARP Security Analyzer)  
**Product type:** CLI (Rust) — VHS recordings  
**Recording date:** 2026-06-15  
**Binary:** `wirerust 0.6.0` built from `.worktrees/STORY-115` (worktree branch `worktree-issue-9-story-115-arp-d3-storm`)  
**Evidence path:** `.factory/demo-evidence/STORY-115/` (factory-artifacts branch)  

---

## Coverage Map

| Recording | AC(s) | BC(s) | Command | Observed |
|-----------|-------|--------|---------|----------|
| [AC-011-012-arp-storm-rate-flag.gif/.webm](#ac-011-012) | AC-011, AC-012 | BC-2.16.013 PC1/PC2/EC-006 | see below | flag accepted; storm fires at rate=10; default rate=50 suppresses; flag accepted without `--arp` |
| [AC-003-013-d3-storm-finding.gif/.webm](#ac-003-013) | AC-003, AC-013, AC-014 | BC-2.16.008 PC3, BC-2.16.010 | see below | D3 finding: confidence=Medium, category=Anomaly, mitre_techniques absent, storm_findings=1 |
| [AC-TEST-storm-unit-tests.gif/.webm](#ac-test) | AC-001–AC-015 | BC-2.16.008, BC-2.16.013 | `cargo test` | 13 unit + 1 integration + 2 CLI tests: all pass |

---

## AC-011 + AC-012: `--arp-storm-rate` Flag {#ac-011-012}

**Recording:** `AC-011-012-arp-storm-rate-flag.gif` / `.webm`  
**Tape:** `AC-011-012-arp-storm-rate-flag.tape`

### Scene 1 — Explicit `--arp-storm-rate 10` (AC-011, BC-2.16.013 PC1)

```
wirerust-s115 analyze /tmp/arp_storm_10fps.pcap --arp --arp-storm-rate 10 --no-color
```

**Observed output (terminal format):**
```
WIRERUST TRIAGE REPORT
────────────────────────────────────────
  Packets: 0  Bytes: 0  Hosts: 0

FINDINGS
────────────────────────────────────────
  [Anomaly] POSSIBLE (MEDIUM) - D3: ARP storm detected — high ARP frame rate from source MAC AA:BB:CC:DD:EE:FF
    > source_mac=AA:BB:CC:DD:EE:FF
    > frame_count=10
    > window_secs=0
    > rate_pps=10

ANALYZER: ARP
────────────────────────────────────────
  Packets analyzed: 10
  ...
  storm_findings: 1
```

**Verdict:** `--arp-storm-rate 10` is accepted; D3 storm finding fires (10 frames/s >= 10 threshold).

### Scene 2 — Default rate (50) — storm suppressed (AC-011, BC-2.16.013 PC2)

```
wirerust-s115 analyze /tmp/arp_storm_10fps.pcap --arp --no-color
```

**Observed output (ARP summary section):**
```
ANALYZER: ARP
────────────────────────────────────────
  ...
  storm_findings: 0
```

**Verdict:** When `--arp-storm-rate` is absent, default 50 applies. 10 frames/s < 50 → no storm → `storm_findings: 0`.

### Scene 3 — Flag accepted without `--arp` (AC-012, BC-2.16.013 EC-006)

```
wirerust-s115 analyze /tmp/arp_storm_10fps.pcap --arp-storm-rate 25 --no-color
```

**Observed output:**
```
WIRERUST TRIAGE REPORT
────────────────────────────────────────
  Packets: 0  Bytes: 0  Hosts: 0
```

**Verdict:** No parse error. `--arp-storm-rate` is accepted without `--arp`. No ARP analyzer section (not invoked).

---

## AC-003 + AC-013 + AC-014: D3 Storm Finding Structure {#ac-003-013}

**Recording:** `AC-003-013-d3-storm-finding.gif` / `.webm`  
**Tape:** `AC-003-013-d3-storm-finding.tape`

```
wirerust-s115 analyze /tmp/arp_storm_10fps.pcap --arp --arp-storm-rate 10 --json
```

**Observed JSON output:**
```json
{
  "analyzers": [
    {
      "analyzer_name": "ARP",
      "detail": {
        "bindings_tracked": 1,
        "frames_analyzed": 10,
        "garp_findings": 0,
        "malformed_findings": 0,
        "malformed_frames": 0,
        "mismatch_findings": 0,
        "other_opcode_count": 0,
        "reply_count": 0,
        "request_count": 10,
        "spoof_findings": 0,
        "storm_findings": 1
      },
      "packets_analyzed": 10
    }
  ],
  "findings": [
    {
      "category": "Anomaly",
      "confidence": "Medium",
      "evidence": [
        "source_mac=AA:BB:CC:DD:EE:FF",
        "frame_count=10",
        "window_secs=0",
        "rate_pps=10"
      ],
      "summary": "D3: ARP storm detected — high ARP frame rate from source MAC AA:BB:CC:DD:EE:FF",
      "verdict": "Possible"
    }
  ],
  ...
}
```

**AC-003 verification:** D3 finding emitted with `confidence: Medium`, `category: Anomaly`, evidence fields `source_mac`, `frame_count`, `window_secs`, `rate_pps` present. PASS.

**AC-013 verification:** `storm_findings: 1` in analyzer detail (was 0 before D3 detection; now non-zero). PASS.

**AC-014 verification:** `mitre_techniques` key is absent from the finding JSON (JSON finding has no `mitre_techniques` key, confirming empty/absent per DF-VALIDATION-001; T0814 NOT present). PASS.

**Synthetic pcap:** 10 ARP Request frames from `AA:BB:CC:DD:EE:FF` all at `ts_sec=100`. Rate = `10 / max(1, 100-100) = 10/1 = 10 >= 10` → storm fires. (no real-world ARP-heavy pcap in `tests/fixtures/`; synthetic pcap used per BC-2.16.008 test vector pattern.)

---

## AC-001 through AC-015: Unit and Integration Test Evidence {#ac-test}

**Recording:** `AC-TEST-storm-unit-tests.gif` / `.webm`  
**Tape:** `AC-TEST-storm-unit-tests.tape`

### Storm unit tests (AC-001–AC-010, AC-013, AC-014)

```
cargo test --all-targets test_storm 2>&1 | grep -E 'test test_storm|test result.*passed'
```

**Observed test names passing:**
```
test analyzer::arp::story_115::test_storm_first_observation_no_finding ... ok
test analyzer::arp::story_115::test_storm_in_window_increments_count ... ok
test analyzer::arp::story_115::test_storm_finding_emitted_at_threshold ... ok
test analyzer::arp::story_115::test_storm_one_shot_guard_prevents_second_finding ... ok
test analyzer::arp::story_115::test_storm_window_expiry_resets_counter ... ok
test analyzer::arp::story_115::test_storm_same_second_denominator_is_1 ... ok
test analyzer::arp::story_115::test_storm_49_below_threshold_50_at_threshold ... ok
test analyzer::arp::story_115::test_storm_window_boundary_60_in_window_61_expired ... ok
test analyzer::arp::story_115::test_storm_late_burst_suppression_accepted_limitation ... ok
test analyzer::arp::story_115::test_storm_counter_cap_enforced ... ok
test analyzer::arp::story_115::test_storm_custom_rate_10 ... ok
test analyzer::arp::story_115::test_storm_detected_for_garp_flood ... ok
test analyzer::arp::story_115::test_storm_lru_no_spurious_eviction_on_existing_mac_reinit ... ok
test result: ok. 13 passed; 0 failed
```

### CLI flag tests (AC-011, AC-012)

```
cargo test --all-targets test_cli_arp_storm test_storm_rate_flag 2>&1 | grep -E 'test story_115|test result.*passed'
```

**Observed:**
```
test story_115_cli::test_cli_arp_storm_rate_parsed ... ok
test story_115_cli::test_cli_arp_storm_rate_default_50 ... ok
test story_115_cli::test_storm_rate_flag_accepted_without_arp_flag ... ok
test result: ok. 2 passed + 1 passed
```

### Integration test (AC-015)

```
cargo test --all-targets test_integration_arp_storm 2>&1 | grep -E 'test story_115_integration|test result.*passed'
```

**Observed:**
```
test story_115_integration::test_integration_arp_storm_end_to_end ... ok
test result: ok. 1 passed; 0 failed
```

### Full suite

```
cargo test --all-targets
```

**Total:** `1571 passed; 0 failed` across all test binaries.

---

## Fixture Assessment

No existing pcap in `tests/fixtures/` contains a high-rate ARP flood from a single source MAC. The closest are GARP-related fixtures used by STORY-114. For STORY-115 D3 demos and AC-015, a **synthetic libpcap file** (`/tmp/arp_storm_10fps.pcap`, 10 ARP Requests from `AA:BB:CC:DD:EE:FF` all at `ts=100`) was generated using Python's `struct` module. This is consistent with AC-015's test design (the integration test also builds its own synthetic pcap via the `write_pcap` helper in `bc_2_16_story115_arp_tests.rs`).

---

## Error Path Coverage

| Scenario | Command | Result |
|----------|---------|--------|
| Storm below default threshold | `--arp --no-color` (rate=50, 10 frames) | No finding; `storm_findings: 0` |
| Flag without `--arp` | `--arp-storm-rate 25 --no-color` | No parse error; no ARP section |

---

## Anti-Leak Verification

- All artifact files are in `.factory/demo-evidence/STORY-115/` (factory-artifacts worktree).
- No files were written to `.worktrees/STORY-115/` (develop-bound branch).
- Story worktree `git status --short` is empty (verified post-commit).

---

## Artifacts Summary

| File | Size | Purpose |
|------|------|---------|
| `AC-011-012-arp-storm-rate-flag.tape` | 1.3 KB | VHS script source |
| `AC-011-012-arp-storm-rate-flag.gif` | 228 KB | PR-embeddable recording |
| `AC-011-012-arp-storm-rate-flag.webm` | 103 KB | Archival recording |
| `AC-003-013-d3-storm-finding.tape` | 965 B | VHS script source |
| `AC-003-013-d3-storm-finding.gif` | 99 KB | PR-embeddable recording |
| `AC-003-013-d3-storm-finding.webm` | 86 KB | Archival recording |
| `AC-TEST-storm-unit-tests.tape` | 1.2 KB | VHS script source |
| `AC-TEST-storm-unit-tests.gif` | 376 KB | PR-embeddable recording |
| `AC-TEST-storm-unit-tests.webm` | 384 KB | Archival recording |
| `evidence-report.md` | this file | Coverage mapping |
