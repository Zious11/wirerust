# Demo Evidence Report — STORY-119/B

**Story:** STORY-119/B — grouped-collapse render path + `--mitre` default-collapse CLI flip  
**Branch:** `worktree-issue-62-grouped-collapse`  
**HEAD:** `6a28bbe`  
**Binary:** `target/release/wirerust` (v0.9.0, built from worktree)  
**Input fixture:** `tests/fixtures/modbus-write.pcap`  
**Recorded:** 2026-06-19

---

## Coverage Map

| Recording | AC | What it proves |
|---|---|---|
| `AC-001-mitre-grouped-collapsed.{gif,webm}` | AC-001, AC-006, AC-011, AC-018 | `--mitre` alone → grouped + COLLAPSED (new default) |
| `AC-002-mitre-no-collapse-expanded.{gif,webm}` | AC-002 | `--mitre --no-collapse` → grouped + EXPANDED (no suffix) |
| `AC-003-flat-collapsed-vs-expanded.{gif,webm}` | (contrast) | flat collapsed vs flat expanded — unchanged behavior |

---

## AC-001: `--mitre` defaults to grouped-collapse

**Command:** `wirerust analyze tests/fixtures/modbus-write.pcap --modbus --no-color --mitre`

**Key observations in output:**
- Tactic header `## Discovery` present (grouped render path confirmed)
- Finding line: `Modbus recon: Report Server ID (FC 0x11) from unit 1 (x2)` — count suffix present
- K=3 evidence lines: two `> FC=0x11 TxnID=0x0001 UnitID=1` evidence lines shown (≤3)
- MITRE line uses em-dash: `MITRE: T0888 — Remote System Information Discovery`

**Artifacts:**
- `AC-001-mitre-grouped-collapsed.gif`
- `AC-001-mitre-grouped-collapsed.webm`
- `AC-001-mitre-grouped-collapsed.tape`
- `AC-001-output.txt` (plain-text capture)

---

## AC-002: `--mitre --no-collapse` → grouped + expanded (suffix-free)

**Command:** `wirerust analyze tests/fixtures/modbus-write.pcap --modbus --no-color --mitre --no-collapse`

**Key observations in output:**
- Tactic header `## Discovery` still present (still grouped)
- Finding `Modbus recon: Report Server ID (FC 0x11) from unit 1` appears **TWICE** — no `(x2)` suffix
- `--no-collapse` dual-scope confirmed: suppresses collapse in grouped mode (not just flat mode)
- MITRE line uses em-dash: `MITRE: T0888 — Remote System Information Discovery`

**Artifacts:**
- `AC-002-mitre-no-collapse-expanded.gif`
- `AC-002-mitre-no-collapse-expanded.webm`
- `AC-002-mitre-no-collapse-expanded.tape`
- `AC-002-output.txt` (plain-text capture)

---

## AC-006: `(xN)` count suffix

Demonstrated in AC-001 recording. The finding line in collapsed grouped output:

```
[Anomaly] INCONCLUSIVE (MEDIUM) - Modbus recon: Report Server ID (FC 0x11) from unit 1 (x2)
```

The `(x2)` suffix appears because two identical findings collapse into one representative entry with N=2.

---

## AC-011: K=3 evidence lines per group

Demonstrated in AC-001 recording. The collapsed group shows:

```
  > FC=0x11 TxnID=0x0001 UnitID=1
  > FC=0x11 TxnID=0x0001 UnitID=1
```

Two evidence lines shown (N=2 ≤ K=3), confirming the K=3 cap is respected.

---

## AC-018: MITRE em-dash format

Demonstrated in both AC-001 and AC-002 recordings. The MITRE attribution line in grouped mode:

```
  MITRE: T0888 — Remote System Information Discovery
```

Uses the em-dash (`—`) separator with full technique name, not just the technique ID.

---

## Contrast: Flat collapsed vs flat expanded (AC-003)

**Commands run back-to-back in a single recording:**

1. Flat collapsed (default, no `--mitre`):
   ```
   [Anomaly] INCONCLUSIVE (MEDIUM) - Modbus recon: Report Server ID (FC 0x11) from unit 1 (x2)
   ```
   No tactic headers. `(x2)` suffix present.

2. Flat expanded (`--no-collapse`, no `--mitre`):
   ```
   [Anomaly] INCONCLUSIVE (MEDIUM) - Modbus recon: Report Server ID (FC 0x11) from unit 1
   [Anomaly] INCONCLUSIVE (MEDIUM) - Modbus recon: Report Server ID (FC 0x11) from unit 1
   ```
   Two separate lines. No suffix. Flat behavior unchanged from pre-STORY-119.

**Artifacts:**
- `AC-003-flat-collapsed-vs-expanded.gif`
- `AC-003-flat-collapsed-vs-expanded.webm`
- `AC-003-flat-collapsed-vs-expanded.tape`

---

## Build Result

```
Finished `release` profile [optimized] target(s) in 7.88s
```

Build: PASS. No warnings.

---

## Collapsed vs Expanded Diff Summary

The headline difference between mode 1 and mode 2 on identical input (`modbus-write.pcap`):

| Mode | Command | Repeated finding output |
|------|---------|------------------------|
| Grouped + Collapsed (NEW default) | `--mitre` | `...Report Server ID (FC 0x11) from unit 1 (x2)` (one line, count suffix) |
| Grouped + Expanded | `--mitre --no-collapse` | `...Report Server ID (FC 0x11) from unit 1` x2 (two lines, no suffix) |

This is the observable proof that STORY-119/B wired `--mitre` to `collapse=true` by default and that `--no-collapse` correctly suppresses it in grouped scope.
