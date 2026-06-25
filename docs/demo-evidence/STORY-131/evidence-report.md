# Evidence Report — STORY-131

**Story:** STORY-131: EtherNet/IP StreamDispatcher Integration, CLI Flags, and TCP Reassembly Wiring
**Recorded:** 2026-06-25
**Tool:** VHS 0.11.0 (CLI product)
**Branch:** worktree-issue-316-story-131-enip-dispatcher

---

## AC-to-Artifact Mapping

| AC | Description | Artifact | Type | Notes |
|----|-------------|----------|------|-------|
| AC-131-001 | StreamDispatcher routes port-44818 TCP to EnipAnalyzer | `AC-001-002-dispatch-tests.gif` / `.webm` | VHS recording | 15 dispatch tests green; no pcap fixture yet (deferred to F4 HS-110..122) |
| AC-131-002 | `take_enip_analyzer()` transfers ownership | `AC-001-002-dispatch-tests.gif` / `.webm` | VHS recording | Shared with AC-001; `test_take_enip_analyzer_transfers_ownership` + `test_take_enip_analyzer_returns_none_when_not_set` in recording |
| AC-131-003 | CLI `--enip` flag enables EnipAnalyzer | `AC-003-005-006-cli-flags.gif` / `.webm` | VHS recording | `cargo run -- analyze --help` shows `--enip` flag with description |
| AC-131-004 | Missing TCP reassembly with `--enip` emits WARNING | `AC-004-reassembly-guard.gif` / `.webm` | VHS recording | Shows `--enip requires TCP reassembly; ENIP analysis disabled` on stderr |
| AC-131-005 | `--enip-write-burst-threshold` sets T0836 threshold (default 50) | `AC-003-005-006-cli-flags.gif` / `.webm` | VHS recording | Shared with AC-003/006; `--help` shows flag with `[default: 50]` |
| AC-131-006 | `--enip-error-burst-threshold` sets T0888 threshold (default 5) | `AC-003-005-006-cli-flags.gif` / `.webm` | VHS recording | Shared with AC-003/005; `--help` shows flag with `[default: 5]` |

---

## Recordings

### AC-001-002-dispatch-tests (.gif / .webm / .tape)

Demonstrates StreamDispatcher routing (AC-131-001) and `take_enip_analyzer()` ownership transfer (AC-131-002).

Command recorded:
```
cargo test --test enip_analyzer_tests dispatch 2>&1 | grep -E 'test dispatch|ok|FAILED|result'
```

Shows all 15 dispatch tests passing:
- `dispatch::test_dispatcher_routes_port_44818`
- `dispatch::test_dispatcher_does_not_route_other_ports`
- `dispatch::test_dispatcher_rule_order_dnp3_before_enip`
- `dispatch::test_take_enip_analyzer_transfers_ownership`
- `dispatch::test_take_enip_analyzer_returns_none_when_not_set`
- `dispatch::test_cli_enip_flag_constructs_analyzer`
- `dispatch::test_cli_no_enip_flag_no_analyzer`
- `dispatch::test_cli_all_flag_includes_enip`
- `dispatch::test_enip_without_reassembly_warns_and_disables`
- `dispatch::test_write_burst_threshold_custom`
- `dispatch::test_write_burst_threshold_default`
- `dispatch::test_error_burst_threshold_custom`
- `dispatch::test_error_burst_threshold_default`
- `dispatch::test_error_burst_threshold_zero_semantics`
- `dispatch::test_dispatcher_no_enip_analyzer_port_44818_is_noop`

**Deferred note:** No ENIP pcap fixture exists yet — pcap-based routing evidence is deferred to F4 fixture creation (HS-110..122). The dispatch unit tests provide full routing and ownership evidence per story instructions.

---

### AC-003-005-006-cli-flags (.gif / .webm / .tape)

Demonstrates all three CLI flags (AC-131-003 / AC-131-005 / AC-131-006).

Command recorded:
```
cargo run -- analyze --help 2>&1 | grep -A3 '\-\-enip'
```

Shows:
- `--enip` — Analyze EtherNet/IP TCP traffic (port 44818, requires stream reassembly). Default-off; included by `--all`
- `--enip-write-burst-threshold` — default 50 (`[default: 50]`)
- `--enip-error-burst-threshold` — default 5 (`[default: 5]`)

---

### AC-004-reassembly-guard (.gif / .webm / .tape)

Demonstrates the TCP reassembly guard warning (AC-131-004).

Command recorded:
```
cargo run -- analyze tests/fixtures/slammer.pcap --enip --no-reassemble 2>&1 | grep -iE 'enip|reassembl'
```

Shows stderr output:
```
--enip requires TCP reassembly; ENIP analysis disabled
```

No ENIP analysis proceeds. Mirrors the `--modbus` / `--dnp3` reassembly-guard pattern.

---

## Coverage Summary

| AC | Covered | Path | Evidence Type |
|----|---------|------|---------------|
| AC-131-001 | Yes | Success (15 tests green) | VHS + unit tests |
| AC-131-002 | Yes | Success (ownership transfer + None on 2nd call) | VHS + unit tests |
| AC-131-003 | Yes | Success (flag visible in --help) | VHS |
| AC-131-004 | Yes | Error path (warning emitted; analysis disabled) | VHS |
| AC-131-005 | Yes | Success (default 50 visible in --help) | VHS |
| AC-131-006 | Yes | Success (default 5 visible in --help) | VHS |

**Total: 6 / 6 ACs covered. 0 deferred.**

> AC-131-001 and AC-131-002 use unit tests rather than a live pcap run. This is the intended approach
> per story instructions — pcap fixture creation is deferred to F4 (HS-110..122). The unit tests
> directly exercise `StreamDispatcher::dispatch()` and `take_enip_analyzer()` with port-44818 flows,
> providing complete routing and ownership evidence.
