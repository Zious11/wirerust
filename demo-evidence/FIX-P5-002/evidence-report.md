# FIX-P5-002 Demo Evidence Report

**Finding:** ADV-IMPL-P04-MED-001 — `--reassembly-depth 0` and `--reassembly-memcap 0` previously
accepted by clap (panicking or silently misbehaving); now rejected at argument-parse time with a
descriptive clap usage error and exit code 2.

**Binary:** `.worktrees/FIX-P5-002/target/debug/wirerust` (built from the FIX-P5-002 worktree)
**Fixture:** `tests/fixtures/http.pcap` (247 bytes, 1 TCP packet — smallest available fixture)
**Recording tool:** VHS 0.11.0
**Font:** Menlo (macOS system font)
**Date:** 2026-06-01

---

## Recordings

| AC | File | What It Shows | Exit Code |
|----|------|---------------|-----------|
| AC-001 | `AC-001-depth-zero-rejected.gif` / `.webm` | `--reassembly-depth 0` → clap error, no panic | **2** |
| AC-002 | `AC-002-memcap-zero-rejected.gif` / `.webm` | `--reassembly-memcap 0` → clap error, no panic | **2** |
| AC-003 | `AC-003-valid-depth-succeeds.gif` / `.webm` | `--reassembly-depth 4` → normal report, succeeds | **0** |

---

## Observed CLI Output

### AC-001 — `wirerust analyze --reassembly-depth 0 http.pcap`

```
error: invalid value '0' for '--reassembly-depth <REASSEMBLY_DEPTH>': 0 is not in 1..

For more information, try '--help'.
exit: 2
```

### AC-002 — `wirerust analyze --reassembly-memcap 0 http.pcap`

```
error: invalid value '0' for '--reassembly-memcap <REASSEMBLY_MEMCAP>': 0 is not in 1..

For more information, try '--help'.
exit: 2
```

### AC-003 — `wirerust analyze --reassembly-depth 4 http.pcap` (contrast / success path)

```
WIRERUST TRIAGE REPORT
────────────────────────────────────────
  Packets: 1  Bytes: 207  Hosts: 2

PROTOCOLS
────────────────────────────────────────
  Tcp: 1

SERVICES
────────────────────────────────────────
  HTTP: 1

exit: 0
```

---

## Tape Sources

- `AC-001-depth-zero-rejected.tape`
- `AC-002-memcap-zero-rejected.tape`
- `AC-003-valid-depth-succeeds.tape`

---

## Coverage Summary

- **ACs demonstrated:** 3 / 3 (100%)
- **Error paths:** 2 (AC-001, AC-002) — both confirm exit code 2, no panic
- **Success / contrast path:** 1 (AC-003) — confirms exit code 0, normal behavior unaffected
- **No panic observed:** confirmed across all three runs
