# FIX-P5-002 Demo Script

**Finding:** ADV-IMPL-P04-MED-001
**Behavior change:** `--reassembly-depth 0` and `--reassembly-memcap 0` are now rejected by clap
at argument-parse time (exit 2, usage error) instead of being accepted and causing a panic or
silent misbehavior downstream.

---

## Setup

```bash
BINARY=/path/to/wirerust/.worktrees/FIX-P5-002/target/debug/wirerust
PCAP=/path/to/wirerust/.worktrees/FIX-P5-002/tests/fixtures/http.pcap
```

Build the binary:
```bash
cd /path/to/wirerust/.worktrees/FIX-P5-002
cargo build
```

---

## Step 1 — Error path: `--reassembly-depth 0`

**Command:**
```bash
$BINARY analyze --reassembly-depth 0 "$PCAP" 2>&1; echo "exit: $?"
```

**Expected stderr:**
```
error: invalid value '0' for '--reassembly-depth <REASSEMBLY_DEPTH>': 0 is not in 1..

For more information, try '--help'.
```

**Expected exit code:** `2`

**No panic.** The clap value parser rejects 0 before the binary executes any reassembly logic.

---

## Step 2 — Error path: `--reassembly-memcap 0`

**Command:**
```bash
$BINARY analyze --reassembly-memcap 0 "$PCAP" 2>&1; echo "exit: $?"
```

**Expected stderr:**
```
error: invalid value '0' for '--reassembly-memcap <REASSEMBLY_MEMCAP>': 0 is not in 1..

For more information, try '--help'.
```

**Expected exit code:** `2`

**No panic.** Same clap validation gate, same exit code.

---

## Step 3 — Contrast / success path: `--reassembly-depth 4`

**Command:**
```bash
$BINARY analyze --reassembly-depth 4 "$PCAP" 2>&1; echo "exit: $?"
```

**Expected stdout:** wirerust triage report (packets, protocols, services)

**Expected exit code:** `0`

Normal behavior is unaffected — valid values proceed through reassembly and produce output.

---

## Observed Exit Codes (confirmed on 2026-06-01)

| Command | Exit Code | Notes |
|---------|-----------|-------|
| `--reassembly-depth 0` | **2** | Clap usage error, no panic |
| `--reassembly-memcap 0` | **2** | Clap usage error, no panic |
| `--reassembly-depth 4` | **0** | Success, full report printed |

---

## VHS Recordings

All recordings were made with VHS 0.11.0 from tape scripts in this directory.
Output files: `AC-001-*.gif/.webm`, `AC-002-*.gif/.webm`, `AC-003-*.gif/.webm`.
