# Demo Evidence Report: STORY-144 — TLS Carry Buffer + ClientHello Fragmentation Reassembly

**Story:** STORY-144  
**Branch:** `feature/story-144-tls-carry-reassembly`  
**Binary:** `wirerust 0.11.0` (worktree release build)  
**Date:** 2026-06-29  
**Baseline reference:** `../AC-001-tls-frag-evasion-baseline.gif` (develop before fix)

---

## Headline Result

| Pcap | Baseline (develop) | STORY-144 (after fix) |
|------|-------------------|----------------------|
| `tls-clienthello-fragmented.pcap` | SNI: MISSED, JA3: MISSED, parse_errors: 2 | **SNI: ['example.com'], JA3: 6169fabc98e3e6c9..., parse_errors: 0** |
| `tls-clienthello-control.pcap` | SNI: ['example.com'], JA3: detected | SNI: ['example.com'], JA3: detected (no regression) |

**The evasion is closed.** The same fragmented ClientHello pcap that produced zero SNI/JA3 on baseline now correctly extracts both after STORY-144.

---

## AC Coverage

### AC-144-002: Handshake-message carry buffer (ClientToServer)

Behavioral contract: `BC-2.07.038`. A ClientHello fragmented across multiple TLS records must be reassembled and SNI/JA3 extracted as if it arrived in a single record.

#### Demo: Fragmented ClientHello → NOW DETECTED

| File | Description |
|------|-------------|
| `AC-144-002-reassembly-fragmented.gif` | `tls-clienthello-fragmented.pcap` against STORY-144 binary. Shows SNI=['example.com'], JA3 hash, parse_errors:0 |
| `AC-144-002-reassembly-fragmented.webm` | Same recording, archival format |
| `AC-144-002-reassembly-fragmented.tape` | VHS script source |

**Path covered:** success path (fragmented ClientHello fully reassembled).

#### Demo: Single-record Control Regression Check

| File | Description |
|------|-------------|
| `AC-144-002-control-regression.gif` | `tls-clienthello-control.pcap` against STORY-144 binary. Shows SNI still extracted (no regression). |
| `AC-144-002-control-regression.webm` | Same recording, archival format |
| `AC-144-002-control-regression.tape` | VHS script source |

**Path covered:** regression guard — single-record path unaffected by carry logic.

#### Demo: Before/After Contrast (Headline)

| File | Description |
|------|-------------|
| `AC-144-002-before-after-contrast.gif` | Side-by-side text showing BEFORE (MISSED/parse_errors:2) followed by AFTER STORY-144 (SNI+JA3 extracted, parse_errors:0) |
| `AC-144-002-before-after-contrast.webm` | Same recording, archival format |
| `AC-144-002-before-after-contrast.tape` | VHS script source |

**Path covered:** before/after contrast for PR evidence. The BEFORE values are reproduced from the baseline recording at `../AC-001-tls-frag-evasion-baseline.gif`.

---

### AC-144-003: Clear-and-recover on carry buffer overflow

Behavioral contract: `BC-2.07.039`. When a carry buffer overflow is detected (body_len > MAX_BUF = 65,536, or carry + payload would exceed MAX_BUF), the carry is cleared and `handshake_reassembly_overflows` is incremented. The counter is surfaced in `summarize()` detail.

**Recording method:** Unit test output via `cargo test --nocapture` (see note below).

| File | Description |
|------|-------------|
| `AC-144-003-overflow-clear-recover.gif` | `cargo test` run of `test_vp039_carry_overflow_clear_and_recover` + `test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key`. Both pass (ok). |
| `AC-144-003-overflow-clear-recover.webm` | Same recording, archival format |
| `AC-144-003-overflow-clear-recover.tape` | VHS script source |

**Note on recording method:** Crafting a real PCAP that triggers the overflow path and then shows `handshake_reassembly_overflows` in JSON output requires a multi-packet pcap with a spoofed body_len. The unit tests directly invoke `TlsAnalyzer::on_data` with crafted byte vectors that produce the overflow condition and then call `summarize()` to verify the counter. The terminal recording shows both tests passing under `cargo test`, which is the correct evidence for this invariant. A future improvement could craft a pcap via `gen_pcaps.py` but is out of scope for STORY-144.

**Paths covered:** overflow-clear path (Decision-5 guard), counter persistence through `summarize()`.

---

## Full Coverage Matrix

| AC | Description | Recording | Error Path | Notes |
|----|-------------|-----------|------------|-------|
| AC-144-002 | Two-record fragmented ClientHello reassembly | `AC-144-002-reassembly-fragmented.gif` | N/A (failure=baseline, see `../AC-001-tls-frag-evasion-baseline.gif`) | Success path demonstrated on real pcap fixture |
| AC-144-002 | Control regression (single-record) | `AC-144-002-control-regression.gif` | N/A | No regression confirmed |
| AC-144-002 | Before/after contrast | `AC-144-002-before-after-contrast.gif` | BEFORE = error path reproduced from baseline text | Headline PR evidence |
| AC-144-003 | Carry overflow clear-and-recover | `AC-144-003-overflow-clear-recover.gif` | Overflow condition triggered in unit test | Unit-test capture (pcap infeasible without crafted fixture) |

---

## Fixtures Reused

These fixtures from the baseline cycle are reused unchanged:

| File | Hash (sha256 prefix) |
|------|----------------------|
| `../tls-clienthello-control.pcap` | baseline fixture, single TLS record |
| `../tls-clienthello-fragmented.pcap` | baseline fixture, 2-record split |

---

## Baseline Comparison

The baseline recording (`../AC-001-tls-frag-evasion-baseline.gif`) showed:

```
EVASION (2 TLS records):
  SNI:          (empty — MISSED)
  JA3:          (empty — MISSED)
  parse_errors: 2
```

STORY-144 produces:

```
FRAGMENTED (reassembled):
  SNI:          ['example.com']
  JA3:          6169fabc98e3e6c9...
  parse_errors: 0
```

**Evasion closed: YES.**
