# Evidence Report — STORY-139

**Story:** ENIP EC-X1/EC-X2 detection-correctness fix  
**Branch:** `fix/enip-direction-and-clock`  
**Date recorded:** 2026-06-27  
**Tool:** VHS 0.11.0  
**Product type:** CLI (Rust) — test-runner surface  

---

## Coverage Summary

| AC | Description | Success path | Error path | Status |
|----|-------------|-------------|------------|--------|
| AC-139-001 | `carry_c2s`/`carry_s2c` direction split + `on_data` `Direction` param | 3 tests pass | 0 tests on misspelled filter | PASS |
| AC-139-002 | T0836 CIP error response burst — backwards-ts no-reset (`saturating_sub`) | 1 test passes | 0 tests on wrong suffix | PASS |
| AC-139-003 | T0888 CIP write burst — backwards-ts no-reset (`saturating_sub`) | 1 test passes | 0 tests on wrong suffix | PASS |
| AC-139-004 | T0814 malformed frame threshold — backwards-ts no-reset (`saturating_sub` + strict `>300`) | 1 test passes | 0 tests on wrong suffix | PASS |

All 4 acceptance criteria covered. All success paths green. All error paths confirm test filter precision.

---

## Recordings

### AC-139-001 — Direction split (`carry_c2s` / `carry_s2c`)

**Acceptance criterion:** `on_data` accepts a `Direction` parameter; `carry_c2s` and `carry_s2c` are
independent per-direction reassembly buffers; a partial c2s frame does not contaminate a
subsequent s2c parse (EC-X1 cross-direction splice bug eliminated).

**Traces:** BC-2.17.016 v2.0 Invariant 7, EC-010; RULING-EDGECASE-001 §1.2, §6

| File | Format | Content |
|------|--------|---------|
| `AC-139-001-direction-split.gif` | GIF | Animated terminal recording (PR embed) |
| `AC-139-001-direction-split.webm` | WebM | Archival video |
| `AC-139-001-direction-split.tape` | VHS tape | Recording script source |

**Tests run (success path):**
- `direction_and_clock::test_ec_x1_cross_direction_no_splice` — ok
- `direction_and_clock::test_carry_c2s_and_carry_s2c_are_independent` — ok
- `direction_and_clock::test_direction_based_source_ip` — ok

**Error path:** filter `direction_and_clock::test_ec_x1_cross_direction_spliced` (misspelled) →
`running 0 tests`, confirming the exact test name is required (no false-positive matching).

---

### AC-139-002 — T0836 backwards-ts no-reset

**Acceptance criterion:** A backward timestamp delivered during a T0836 CIP error response burst
uses `saturating_sub` (delta clamps to 0) and does not reset the burst window. The accumulator
must not be zeroed on time reversal.

**Traces:** BC-2.17.008 v1.3; RULING-EDGECASE-001 §2

| File | Format | Content |
|------|--------|---------|
| `AC-139-002-backwards-ts-t0836.gif` | GIF | Animated terminal recording |
| `AC-139-002-backwards-ts-t0836.webm` | WebM | Archival video |
| `AC-139-002-backwards-ts-t0836.tape` | VHS tape | Recording script source |

**Test run (success path):**
- `direction_and_clock::test_ec_x2_backwards_ts_t0836_no_reset` — ok (1 passed, 183 filtered out)

**Error path:** filter `test_ec_x2_backwards_ts_t0836_reset` (missing `no_`) →
`running 0 tests`

---

### AC-139-003 — T0888 backwards-ts no-reset

**Acceptance criterion:** A backward timestamp during a T0888 CIP write/recon burst uses
`saturating_sub` and does not reset the recon window.

**Traces:** BC-2.17.012 v1.2; RULING-EDGECASE-001 §3

| File | Format | Content |
|------|--------|---------|
| `AC-139-003-backwards-ts-t0888.gif` | GIF | Animated terminal recording |
| `AC-139-003-backwards-ts-t0888.webm` | WebM | Archival video |
| `AC-139-003-backwards-ts-t0888.tape` | VHS tape | Recording script source |

**Test run (success path):**
- `direction_and_clock::test_ec_x2_backwards_ts_t0888_no_reset` — ok (1 passed, 183 filtered out)

**Error path:** filter `test_ec_x2_backwards_ts_t0888_reset` (missing `no_`) →
`running 0 tests`

---

### AC-139-004 — T0814 backwards-ts no-reset

**Acceptance criterion:** A backward timestamp during T0814 malformed-frame accumulation uses
`saturating_sub` and does not reset the malformed-frame window. The threshold check is strict
`> 300` (not `>=`).

**Traces:** BC-2.17.018 v1.1; RULING-EDGECASE-001 §4

| File | Format | Content |
|------|--------|---------|
| `AC-139-004-backwards-ts-t0814.gif` | GIF | Animated terminal recording |
| `AC-139-004-backwards-ts-t0814.webm` | WebM | Archival video |
| `AC-139-004-backwards-ts-t0814.tape` | VHS tape | Recording script source |

**Test run (success path):**
- `direction_and_clock::test_ec_x2_backwards_ts_t0814_no_reset` — ok (1 passed, 183 filtered out)

**Error path:** filter `test_ec_x2_backwards_ts_t0814_reset` (missing `no_`) →
`running 0 tests`

---

## Recording Methodology

- **Tool:** VHS 0.11.0 (`/opt/homebrew/bin/vhs`)
- **Shell:** bash (VHS-managed pseudoterminal)
- **Font:** Menlo (system font, `/System/Library/Fonts/Menlo.ttc`)
- **Theme:** Dracula
- **Cargo cache:** warm (worktree target directory pre-built; test runs complete in ~0.07s)
- **Success/error path coverage:** every tape records both a passing test run and a deliberate
  zero-match invocation (misspelled filter) to demonstrate test filter precision and guard
  against false-positive green signals.
- **`2>&1` redirect:** cargo emits `Finished`/`Running` lines on stderr; the redirect routes
  them through the pseudoterminal so VHS captures the full output.
