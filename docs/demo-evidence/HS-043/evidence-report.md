# Demo Evidence Report — HS-043

**Story:** HS-043 — Idle-flow expiry wired into production + `--flow-timeout` flag
**Phase:** Phase-4 holdout-remediation fix
**Bug context (BC-2.04.013 v1.5 PC0):** `expire_flows` was defined but never called in the
production packet-processing path. As a result `flows_expired` was permanently 0 in every
JSON report regardless of how many flows had gone idle. The fix wires `expire_flows` into
`process_packet` via a gated sweep and adds a `--flow-timeout <secs>` CLI knob (default 300,
minimum 1) so callers can control the idle-reclaim threshold.

**Fixture:** `tests/fixtures/flow-expiry.pcap` — two TCP flows whose packet timestamps are
6 seconds apart (crafted to straddle any sub-10-second timeout threshold).

---

## AC-001 — Bug fix: `flows_expired` = 1 with `--flow-timeout 5`

**Path type:** success (demonstrates the fix)

**Command:**
```
wirerust analyze tests/fixtures/flow-expiry.pcap --all --flow-timeout 5 --output-format json
```

**Expected:** `"flows_expired": 1` in the JSON reassembly summary — the idle flow whose last
packet is >5 s before the next packet is reclaimed during the per-packet sweep.

**Recordings:**
- `AC-001-flows-expired-fix.gif`
- `AC-001-flows-expired-fix.webm`
- `AC-001-flows-expired-fix.tape` (VHS script)

---

## AC-002 — Contrast: default timeout (300 s) produces `flows_expired` = 0

**Path type:** contrast / negative control

**Command:**
```
wirerust analyze tests/fixtures/flow-expiry.pcap --all --output-format json
```

**Expected:** `"flows_expired": 0` — the default 300 s timeout is not reached by a 6 s gap,
so no flow is reclaimed. This proves the knob controls expiry rather than it being a
constant side-effect of the fix.

**Recordings:**
- `AC-002-default-timeout-no-expiry.gif`
- `AC-002-default-timeout-no-expiry.webm`
- `AC-002-default-timeout-no-expiry.tape` (VHS script)

---

## AC-003 — Error path: `--flow-timeout 0` rejected by clap validation

**Path type:** error path

**Command:**
```
wirerust analyze tests/fixtures/flow-expiry.pcap --all --flow-timeout 0
```

**Expected:** Clap exits with:
```
error: invalid value '0' for '--flow-timeout <FLOW_TIMEOUT>': 0 is not in 1..18446744073709551615
```

This confirms the minimum-1 guard is enforced at the CLI layer; a zero-second timeout
(which would expire every flow on every packet) is rejected before any analysis runs.

**Recordings:**
- `AC-003-timeout-zero-validation.gif`
- `AC-003-timeout-zero-validation.webm`
- `AC-003-timeout-zero-validation.tape` (VHS script)

---

## AC-004 — Help text: `--flow-timeout` flag visible in `--help` output

**Path type:** success (discoverability)

**Command:**
```
wirerust analyze --help
```

**Expected:** Help output contains `--flow-timeout <FLOW_TIMEOUT>` with the description
"Idle flow timeout in seconds ... Default: 300. Minimum: 1 (0 is rejected)".

**Recordings:**
- `AC-004-help-flag.gif`
- `AC-004-help-flag.webm`
- `AC-004-help-flag.tape` (VHS script)

---

## Coverage Summary

| Recording | AC | Path | flows_expired | Pass |
|-----------|-----|------|--------------|------|
| AC-001 | Bug fix — expire_flows wired | success | 1 | yes |
| AC-002 | Default timeout contrast | negative control | 0 | yes |
| AC-003 | Timeout=0 validation | error | clap error | yes |
| AC-004 | --help discoverability | success | n/a | yes |

All acceptance criteria have both a recording and a verified terminal output confirming
the observed value before the tape was authored. No output was fabricated.

---

## Toolchain

- VHS 0.11.0
- Font: Menlo (system default on macOS)
- Theme: Dracula
- Binary: `wirerust` installed from `.worktrees/hs043-flow-expiry` via `cargo install --path . --debug`
