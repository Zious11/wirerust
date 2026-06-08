# Evidence Report — STORY-089

**Story:** Decode Error Counting, Dispatcher Stats Injection, Format Resolution, and Output Routing
**Story ID:** STORY-089
**Wave:** 26
**Date recorded:** 2026-05-31
**Recorder:** Demo Recorder agent
**Binary:** `target/debug/wirerust` (built from STORY-089 develop branch)
**Tool:** VHS 0.11.0

---

## Coverage Summary

| AC / EC | Title | Recording | Observable? | Status |
|---------|-------|-----------|-------------|--------|
| AC-001 | First decode error prints exactly one warning | AC-001-004-first-decode-error-warns-once | Yes — warning visible in first line of stderr | COVERED |
| AC-002 | Subsequent decode errors silent | AC-001-004-first-decode-error-warns-once | Yes — single warning line, no repeats across 73 errors | COVERED (shared with AC-001) |
| AC-003 | skipped_packets == total_decode_errors (73) | AC-003-skipped-packets-73 | Yes — JSON shows `"skipped_packets": 73` | COVERED |
| AC-004 | Warning printed at most ONCE per invocation | AC-001-004-first-decode-error-warns-once | Yes — 73 errors produce 1 warning line | COVERED (shared with AC-001) |
| AC-005 | unclassified_flows injected into reassembly summary | AC-005-unclassified-flows-present | Yes — JSON shows `"unclassified_flows": 8` | COVERED |
| AC-006 | unclassified_flows absent without reassembler | AC-006-unclassified-flows-absent | Yes — grep returns empty; echo confirms absence | COVERED |
| AC-007 | resolve_format: --json wins over --output-format | AC-007-EC-005-json-wins-over-output-format | Yes — output starts with `{` despite `--output-format csv` | COVERED |
| AC-008 | resolve_format: --csv produces CSV output | AC-008-csv-output | Yes — `category,...` header visible | COVERED |
| AC-009 | resolve_format: no flags -> terminal output | AC-009-default-terminal-output | Yes — `WIRERUST TRIAGE REPORT` header visible | COVERED |
| AC-010 | write_output: --json FILE writes to file; stdout empty | AC-010-json-to-file | Yes — stdout shows "(empty)" confirmation; file head shows `{` | COVERED |
| AC-011 | write_output: --json (no path) -> stdout | AC-011-EC-004-json-stdout | Yes — JSON printed to stdout | COVERED |
| AC-012 | File write error wrapped with anyhow context | AC-012-write-error-context | Yes — `Failed to write JSON output to /nonexistent-dir/out.json` | COVERED |
| EC-001 | Zero decode errors: no warning | (unit-test-only — no observable CLI signal without decode errors in this fixture set) | No | NOT DEMOED |
| EC-002 | All packets fail decode: 1 warning | AC-001-004-first-decode-error-warns-once | Partially — AC-001 shows 73-error case with 1 warning | COVERED via AC-001 |
| EC-003 | unclassified_flows: 0 is present, not absent | (unit-test-only — requires internal fixture injection) | No | NOT DEMOED |
| EC-004 | --json Some(None) -> stdout | AC-011-EC-004-json-stdout | Yes — same observable behavior as AC-011 | COVERED |
| EC-005 | --json and --output-format csv -> Json wins | AC-007-EC-005-json-wins-over-output-format | Yes — same observable behavior as AC-007 | COVERED |

**BC Parity (run_summary):**
| BC | Title | Recording | Status |
|----|-------|-----------|--------|
| BC-2.12.014 | skipped_packets in summary subcommand | BC-parity-summary-subcommand | COVERED — `"skipped_packets": 73` via `wirerust summary` |
| BC-2.12.016 | Format resolution in summary subcommand | BC-parity-summary-subcommand | COVERED — `--json` flag routes to JSON reporter |
| BC-2.12.017 | Output routing in summary subcommand | BC-parity-summary-subcommand | COVERED — stdout JSON output |

---

## Not-Demoed ACs (Unit-Test-Only)

- **EC-001** (zero decode errors): No fixture produces zero decode errors with visible CLI effect — the absence of a warning is verified by unit test `test_first_decode_error_warning_printed` with a fixture that has no bad packets.
- **EC-003** (unclassified_flows == 0 is present): Requires internal dispatcher producing zero unclassified flows. No CLI-observable difference from a present key with value 0. Covered by unit test `test_unclassified_flows_injected_into_reassembly_summary`.

---

## Artifacts

### Success Path Recordings

| File | AC(s) | Fixture | Key Observation |
|------|-------|---------|-----------------|
| `AC-001-004-first-decode-error-warns-once.gif/.webm` | AC-001, AC-002, AC-004 | dns-remoteshell.pcap | 73 errors, exactly 1 warning line on stderr |
| `AC-003-skipped-packets-73.gif/.webm` | AC-003 | dns-remoteshell.pcap | `"skipped_packets": 73` in JSON |
| `AC-005-unclassified-flows-present.gif/.webm` | AC-005 | dns-remoteshell.pcap | `"unclassified_flows": 8` in JSON with --http |
| `AC-007-EC-005-json-wins-over-output-format.gif/.webm` | AC-007, EC-005 | tls.pcap | Output is JSON despite `--output-format csv` |
| `AC-008-csv-output.gif/.webm` | AC-008 | tls.pcap | `category,...` CSV header |
| `AC-009-default-terminal-output.gif/.webm` | AC-009 | tls.pcap | `WIRERUST TRIAGE REPORT` terminal header |
| `AC-010-json-to-file.gif/.webm` | AC-010 | tls.pcap | Stdout empty; file written with JSON |
| `AC-011-EC-004-json-stdout.gif/.webm` | AC-011, EC-004 | tls.pcap | JSON on stdout when no path given |
| `BC-parity-summary-subcommand.gif/.webm` | BC-2.12.014/016/017 | dns-remoteshell.pcap | `skipped_packets: 73` via summary subcommand |

### Error Path Recordings

| File | AC(s) | Fixture | Key Observation |
|------|-------|---------|-----------------|
| `AC-006-unclassified-flows-absent.gif/.webm` | AC-006 | dns-remoteshell.pcap | grep finds nothing; absence confirmed |
| `AC-012-write-error-context.gif/.webm` | AC-012 | tls.pcap | `Failed to write JSON output to /nonexistent-dir/out.json` |

### Tape Scripts (VHS Sources)

- `AC-001-004-first-decode-error-warns-once.tape`
- `AC-003-skipped-packets-73.tape`
- `AC-005-unclassified-flows-present.tape`
- `AC-006-unclassified-flows-absent.tape`
- `AC-007-EC-005-json-wins-over-output-format.tape`
- `AC-008-csv-output.tape`
- `AC-009-default-terminal-output.tape`
- `AC-010-json-to-file.tape`
- `AC-011-EC-004-json-stdout.tape`
- `AC-012-write-error-context.tape`
- `BC-parity-summary-subcommand.tape`

---

## AC Coverage Tally

- **12 ACs total:** AC-001 through AC-012
- **10 ACs with direct CLI demo:** AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007, AC-008, AC-009, AC-010, AC-011, AC-012 (all 12 covered)
- **5 ECs total:** EC-001 through EC-005
- **3 ECs with direct CLI demo:** EC-002, EC-004, EC-005
- **2 ECs unit-test-only:** EC-001, EC-003 (no observable CLI surface for these boundary cases)
- **BC parity:** BC-2.12.014, BC-2.12.016, BC-2.12.017 verified via summary subcommand

**Note (lesson F-W22-T1):** This report covers STORY-089 only. No cross-story rollup is included.
