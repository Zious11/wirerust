---
document_type: canonical-decisions
feature_id: issue-007-modbus-analyzer
github_issue: 7
title: "F2 Canonical Decisions (v2) — Architect Directives for Product-Owner and Formal-Verifier"
status: authoritative
version: "2.0"
producer: architect
created: 2026-06-09
supersedes: "v1.0 (same file, initial burst)"
---

# F2 Canonical Decisions (v2) — Issue #7: Modbus TCP Analyzer

This file supersedes v1.0. The first ten decisions from v1.0 are carried forward without
change unless explicitly revised below. Three new decisions (D11, D12, D13) incorporate the
human-approved research-backed changes from `.factory/research/modbus-f2-design-decisions.md`.

**Target version: v0.3.0** (Decision 13 introduces a breaking JSON/CSV schema change;
see §Breaking Change Impact below.)

Do not re-litigate any item. If a downstream conflict arises, consult the architect before
deviating.

---

## Decisions Carried Forward from v1.0 (Unchanged)

Decisions 1–4, 6, 9, 10 from v1.0 are unchanged. Decision 5 (single-window threshold) is
**SUPERSEDED** by Decision 11. Decision 7 (per-PDU co-emission cap-to-most-specific) is
**SUPERSEDED** by Decision 13. Decision 8 (T0846 for recon FCs) is **SUPERSEDED** by
Decision 12.

For the full text of unchanged decisions 1–4, 6, 9, 10 see v1.0; they are not repeated
here to avoid duplication drift.

---

## Decision 11 — Dual-Window Write-Burst Detection (supersedes v1.0 Decision 5)

**Prior state (Decision 5, v1.0):** Single 1-second configurable window, `--modbus-write-threshold`
default 10, `WRITE_RATE_WINDOW_SECS = 1`.

**New state (approved):** Two independent detection windows, each CLI-configurable:

### 11.1 CLI Flags

| Flag | Default | Semantics | Validation |
|------|---------|-----------|------------|
| `--modbus-write-burst-threshold` | `20` | Max write-FCs in any single 1-second window per flow (burst detector). Fires T0806 + burst-T0855 when exceeded. | Must be ≥ 1; reject 0 with `--modbus-write-burst-threshold must be ≥ 1` |
| `--modbus-write-sustained-threshold` | `10` | Max average write-FCs per second over a ≥2-second window per flow (sustained / low-and-slow detector). Fires T0806 + burst-T0855 when exceeded. | Must be ≥ 1; reject 0 with `--modbus-write-sustained-threshold must be ≥ 1` |

The prior `--modbus-write-threshold` flag is **REMOVED**. Product-owner MUST update
BC-2.14.024 to reflect the two new flags. Product-owner MUST update BC-2.14.017 to reflect
the dual-criterion (both windows can fire; each fires at most once per their respective
window, independently).

**Rationale (evidence-backed):** Research confirms that every mature commercial ICS platform
(Dragos, Cisco Cyber Vision) and the academic SCADA-IDS literature use dual-horizon models
precisely because a single fixed window has a documented low-and-slow blind spot. A baseline
write rate of 0.1–5 writes/sec steady-state, with legitimate transitions reaching 5–15/sec
for 10–60 s, makes the default of 20 burst / 10 sustained defensible with minimal false
positives.

### 11.2 New Constants

```rust
const WRITE_BURST_WINDOW_SECS: u32 = 1;          // fixed 1-second burst window width
const DEFAULT_WRITE_BURST_THRESHOLD: u32 = 20;    // --modbus-write-burst-threshold
const WRITE_SUSTAINED_WINDOW_SECS: u32 = 2;       // minimum sustained window duration (secs)
const DEFAULT_WRITE_SUSTAINED_THRESHOLD: u32 = 10; // --modbus-write-sustained-threshold
```

The old `WRITE_RATE_WINDOW_SECS = 1` and `DEFAULT_MODBUS_WRITE_THRESHOLD = 10` are removed
and replaced by the four constants above.

### 11.3 ModbusAnalyzer Struct Changes

`ModbusAnalyzer` carries two threshold fields instead of one:

```rust
pub struct ModbusAnalyzer {
    // replaces write_threshold: u32
    write_burst_threshold: u32,    // --modbus-write-burst-threshold, default 20
    write_sustained_threshold: u32, // --modbus-write-sustained-threshold, default 10
    // ... all other fields unchanged from v1.0 Decision 2/3 ...
}

impl ModbusAnalyzer {
    pub fn new(write_burst_threshold: u32, write_sustained_threshold: u32) -> Self { ... }
}
```

### 11.4 New Sustained-Window Fields on ModbusFlowState

In addition to the existing 1-second burst fields (`window_write_count`, `window_start_ts`,
`window_burst_emitted`), add the following three fields for the sustained detector:

```rust
struct ModbusFlowState {
    // --- existing burst fields (unchanged) ---
    window_write_count: u32,
    window_start_ts: u32,
    window_burst_emitted: bool,

    // --- NEW: T0806/T0855 sustained-rate window (configurable threshold, >=2s rolling) ---
    /// Start timestamp of the current sustained-rate accumulation window (pcap-relative u32 us).
    sustained_window_start_ts: u32,
    /// Write-class FC count accumulated since sustained_window_start_ts.
    sustained_window_write_count: u32,
    /// True once the sustained-rate T0806+T0855 pair has fired for this window; reset when
    /// the window is advanced (i.e., when a new PDU arrives more than WRITE_SUSTAINED_WINDOW_SECS
    /// after sustained_window_start_ts AND the rate check is re-evaluated).
    sustained_burst_emitted: bool,

    // ... all other fields unchanged from v1.0 Decision 2/Decision 6 ...
}
```

**Complete ModbusFlowState field list (authoritative, v2):**

```rust
struct ModbusFlowState {
    // Transaction correlation
    pending: HashMap<(u16, u8), (u8, u32)>,   // bounded MAX_PENDING_TRANSACTIONS=256

    // Per-flow aggregate counters
    write_count: u64,
    exception_count: u64,
    pdu_count: u64,
    last_ts: u32,

    // T0806/T0855 burst window (1-second, configurable burst threshold)
    window_write_count: u32,
    window_start_ts: u32,
    window_burst_emitted: bool,

    // T0806/T0855 sustained window (>=2-second rolling, configurable sustained threshold)
    sustained_window_start_ts: u32,
    sustained_window_write_count: u32,
    sustained_burst_emitted: bool,

    // T0831 coordinated-write window (5-second fixed, not CLI-configurable)
    t0831_window_start_ts: u32,
    t0831_window_write_count: u32,
    t0831_burst_emitted: bool,

    // BC-2.14.019 exception-burst windows (per exception code)
    exception_window_counts: HashMap<u8, u32>,
    exception_window_start_ts: HashMap<u8, u32>,
    exception_burst_emitted: HashMap<u8, bool>,

    // Desync safety (Decision 6)
    is_non_modbus: bool,
}
```

### 11.5 Sustained-Window Detection Math (precise, truncation-free)

On each write-class FC (request direction), after the burst window update:

1. **Window initialization:** If `sustained_window_start_ts == 0` (initial state), set
   `sustained_window_start_ts = now_ts` and `sustained_window_write_count = 1`.

2. **Accumulate:** Otherwise increment `sustained_window_write_count`.

3. **Window duration check:** Compute `elapsed_us = now_ts.wrapping_sub(sustained_window_start_ts)`
   using `wrapping_sub` (u32 wrapping subtraction). This is correct for pcap-relative timestamps
   that start near 0, and correct under the Timestamp Wrap Policy (see §11.5a below).
   **Do NOT convert to seconds** — the truncation `elapsed_secs = elapsed_us / 1_000_000`
   is eliminated entirely (see §11.5a for the defect this avoids).

4. **Detection trigger** (truncation-free microsecond-scale integer math — no division):
   - If `elapsed_us >= WRITE_SUSTAINED_WINDOW_SECS * 1_000_000` AND
     `(sustained_window_write_count as u64) * 1_000_000 > (write_sustained_threshold as u64) * (elapsed_us as u64)` AND
     `!sustained_burst_emitted`:
     - Emit T0806 + T0855 (sustained variant) findings.
     - Set `sustained_burst_emitted = true`.

5. **Window reset:** When `elapsed_us >= WRITE_SUSTAINED_WINDOW_SECS * 1_000_000`, after
   detection evaluation, always reset (whether or not a finding fired): slide the window
   forward — reset `sustained_window_start_ts = now_ts`, `sustained_window_write_count = 1`,
   `sustained_burst_emitted = false`. This prevents the accumulated count from growing
   unboundedly.

6. **Emit-once semantics:** `sustained_burst_emitted = true` after the first T0806+T0855
   sustained pair fires within the window. Subsequent writes in the same window do NOT re-fire.
   The flag is reset on window advancement (step 5).

### 11.5a Defect Eliminated: Integer-Truncation False Positives (Gemini Adversarial Finding)

The prior formulation divided `elapsed_us` by `1_000_000` to get `elapsed_secs` (integer
division), then compared `sustained_window_write_count > write_sustained_threshold * elapsed_secs`.
This introduced a systematic false-positive bias: integer division truncates the elapsed time,
making the window appear shorter than it is.

**Concrete counterexample (prior formula):**
- 25 writes accumulated; `elapsed_us = 2_900_000` µs (2.9 seconds, real rate = 8.6/s)
- `elapsed_secs = 2_900_000 / 1_000_000 = 2` (truncates 0.9 s)
- Check: `25 > 10 * 2 = 20` → TRUE → fires T0806 (FALSE POSITIVE)
- Real rate (8.6/s) is below the 10/s threshold; detection should NOT fire.

**Corrected formula (cross-multiplication, no division):**
- Same inputs: `elapsed_us = 2_900_000`
- Check: `(25 as u64) * 1_000_000 = 25_000_000 > (10 as u64) * 2_900_000 = 29_000_000` → FALSE
- Correctly does NOT fire.

**Detection math summary (canonical, truncation-free):**

```
elapsed_us := now_ts.wrapping_sub(sustained_window_start_ts)   // u32 wrapping sub

trigger := elapsed_us >= WRITE_SUSTAINED_WINDOW_SECS * 1_000_000
         AND (sustained_window_write_count as u64) * 1_000_000
             > (write_sustained_threshold as u64) * (elapsed_us as u64)
         AND NOT sustained_burst_emitted
```

The u64 cast on both sides of the rate comparison prevents overflow: maximum values are
`65535 * 1_000_000 = 65_535_000_000` (fits in u64; u32 max is ~4.3 billion, so u64 is required
if `sustained_window_write_count` could approach u32::MAX — it is bounded in practice by the
MAX_FINDINGS cap and typical capture durations, but the u64 cast is mandatory for correctness).

For defaults at exactly 2 s (`elapsed_us = 2_000_000`):
- Fires if `count * 1_000_000 > 10 * 2_000_000 = 20_000_000`, i.e., count ≥ 21 writes.
- Equivalent to: strict average > 10/s over the elapsed window.

### 11.5b Timestamp Wrap Policy (u32 pcap-relative microseconds)

The pcap-relative capture timestamp is a `u32` in microseconds; it wraps at `u32::MAX ≈ 4294 s`
(~71.5 minutes). Plain subtraction (`now_ts - start`) panics in Rust debug builds
(overflow-checks = true) and produces a large garbage value in release. ALL window-duration
computations MUST use `wrapping_sub`:

```rust
let elapsed_us = now_ts.wrapping_sub(window_start_ts);
```

This applies to ALL four windows:
- Burst 1s: `now_ts.wrapping_sub(window_start_ts)`
- Sustained ≥2s: `now_ts.wrapping_sub(sustained_window_start_ts)`
- T0831 5s: `now_ts.wrapping_sub(t0831_window_start_ts)`
- Exception 10s: `now_ts.wrapping_sub(exception_window_start_ts[code])`

**Wrap evasion analysis:** When `now_ts < window_start_ts` (wrapped or attacker-reordered),
`wrapping_sub` yields a value near `u32::MAX` (~4.3 billion µs). This is always larger than
any window threshold (max: 10 s × 1_000_000 = 10_000_000 µs). The result is a window reset
(`window_start_ts = now_ts`, count reset to 1, emitted flag cleared). This is evasion-resistant:
the attacker can reset the detection window by injecting an out-of-order packet, but cannot
suppress a window that has already fired (the `*_emitted` flag is already true before the reset
is evaluated in the next PDU), and cannot prevent the next window from accumulating and firing.

### 11.6 Finding Distinction: Burst vs Sustained

The T0806 finding emitted by the sustained window SHOULD include a distinguishing evidence
note: `"Sustained write rate exceeded: N writes over E seconds (>T/s average)"`. The burst
T0806 evidence is already `"Burst threshold exceeded: N write FCs in 1s window"`. This
allows analysts to distinguish the two emission paths in the finding output.

The sustained findings use the same `mitre_techniques: [T0806, T0855]` tag set as the burst
findings (see Decision 13 for multi-tag representation).

### 11.7 Architecture-Delta Updates Required

Product-owner and formal-verifier MUST:

- **Architecture-delta §2.2 (ModbusAnalyzer struct):** Replace `write_threshold: u32` with
  `write_burst_threshold: u32` and `write_sustained_threshold: u32`.
- **Architecture-delta §2.3 (ModbusFlowState):** Add the three sustained fields.
- **Architecture-delta §2.6 (emission table):** Add sustained-window row:
  `> write_sustained_threshold * elapsed_secs averaged over >=2s → T0806 + T0855 (sustained)`.
- **Architecture-delta §5.1 (CLI flags):** Replace single flag with two flags; update
  `--all` expansion and `ModbusAnalyzer::new` call.
- **Architecture-delta Appendix constants:** Replace old constants with the four new constants.
- **ADR-005 Decision 3 (§Negative):** Update: "`--modbus-write-burst-threshold` and
  `--modbus-write-sustained-threshold` implement a dual-window model (1s burst / ≥2s sustained).
  Each window fires at most once per respective window expiry."

**Product-owner obligations:**
- Revise BC-2.14.017: title and body change from "Write-Rate Burst Exceeding
  `--modbus-write-threshold`" to "Write-Rate Exceeding Either Burst or Sustained Threshold".
  Add: "Two independent detectors — burst (>N in 1s) and sustained (>M avg over ≥2s) — each
  emit T0806 + T0855 at most once per their respective window. The `window_burst_emitted` flag
  guards burst; `sustained_burst_emitted` guards sustained."
- Revise BC-2.14.024: update from single flag to dual flags with validation rules (≥1).

---

## Decision 12 — T0846 → T0888 Correctness Fix for Recon FCs (supersedes v1.0 Decision 8)

**Prior state (Decision 8, v1.0):** Recon FCs 0x11 (Report Server ID) and 0x2B/MEI 0x0E
(Read Device ID) mapped to `T0846` Remote System Discovery. T0846 was both SEEDED and
EMITTED.

**New state (approved correctness fix):**

- 0x11 and 0x2B/MEI 0x0E → **T0888 Remote System Information Discovery** (TA0102 Discovery).
  T0846 Remote System Discovery applies to network-scan behavior (enumerating that systems
  exist); T0888 applies to querying device make/model/firmware/version, which is exactly what
  these function codes return. The prior T0846 mapping was a documented common misattribution.
- 0x07 (Read Exception Status) → **excluded** as a standalone recon indicator. A single
  status byte provides insufficient signal for a Finding. May be considered as a low-weight
  corroborator in a future sequence-aware scan detector (not in-scope F2).

### 12.1 Current src/mitre.rs State (verified)

The following is the current seeded state of `src/mitre.rs` before any F2 implementation:

| ICS Technique | Seeded | Emitted (pre-F2) |
|---------------|--------|------------------|
| T0846 | YES (line 274) | NO (not in EMITTED_IDS) |
| T0855 | YES | NO |
| T0856 | YES | NO |
| T0885 | YES | NO |
| T0888 | **NO** | NO |
| T0836 | NO | NO |
| T0814 | NO | NO |
| T0806 | NO | NO |
| T0835 | NO | NO |
| T0831 | NO | NO |

Current totals: SEEDED = 15 (11 Enterprise + 4 ICS), EMITTED = 6 (Enterprise only).

### 12.2 Corrected technique_info Arms

The following arms MUST be present in `technique_info` after the Modbus feature commit.
T0888 is a NEW seeded entry. T0846 REMAINS seeded (it was already there; it is not emitted
by Modbus but stays in the catalog for completeness / future use).

```rust
// ICS — arms present after Modbus feature (ADR-005 + ADR-006):
"T0846" => ("Remote System Discovery",             MitreTactic::Discovery),     // KEPT, not Modbus-emitted
"T0855" => ("Unauthorized Command Message",        MitreTactic::IcsImpairProcessControl),
"T0856" => ("Spoof Reporting Message",             MitreTactic::IcsImpairProcessControl),
"T0885" => ("Commonly Used Port",                  MitreTactic::CommandAndControl),
"T0836" => ("Modify Parameter",                    MitreTactic::IcsImpairProcessControl), // NEW seeded
"T0814" => ("Denial of Service",                   MitreTactic::IcsInhibitResponseFunction), // NEW seeded
"T0806" => ("Brute Force I/O",                     MitreTactic::IcsImpairProcessControl), // NEW seeded
"T0835" => ("Manipulate I/O Image",                MitreTactic::IcsImpairProcessControl), // NEW seeded
"T0831" => ("Manipulation of Control",             MitreTactic::IcsImpairProcessControl), // NEW seeded
"T0888" => ("Remote System Information Discovery", MitreTactic::Discovery),     // NEW seeded (Decision 12)
```

### 12.3 Corrected MITRE Counts (authoritative)

| Metric | Pre-F2 | Post-F2 (after this decision) |
|--------|--------|-------------------------------|
| `SEEDED_TECHNIQUE_ID_COUNT` | 15 | **21** (11 Enterprise + 10 ICS) |
| `SEEDED_TECHNIQUE_IDS` entries | 15 | **21** |
| ICS SEEDED | 4 (T0846,T0855,T0856,T0885) | **10** (prior 4 + T0836,T0814,T0806,T0835,T0831,T0888) |
| `EMITTED_IDS` entries | 6 Enterprise | **13** (6 Enterprise + 7 ICS) |
| ICS EMITTED | 0 | **7** {T0855, T0836, T0814, T0806, T0835, T0831, **T0888**} |

Note: T0846 is SEEDED but NOT EMITTED (removed from Modbus emission; may be emitted by
future address-sweep detection). T0888 is both SEEDED and EMITTED.

**Corrected `SEEDED_TECHNIQUE_IDS` (21 entries):**

```rust
#[cfg(any(kani, test))]
const SEEDED_TECHNIQUE_IDS: &[&str] = &[
    // Enterprise (11, unchanged)
    "T1027", "T1036", "T1040", "T1046", "T1071",
    "T1071.001", "T1071.004", "T1083", "T1499.002", "T1505.003", "T1573",
    // ICS (10 total — was 4, add 6 new; T0888 replaces T0846 in the emitted set)
    "T0846",                              // existing seeded, NOT emitted by Modbus
    "T0855", "T0856", "T0885",            // existing seeded
    "T0836", "T0814", "T0806", "T0835", "T0831", // new seeded (5)
    "T0888",                              // new seeded (1, replaces T0846 as Modbus recon emitter)
];

#[cfg(any(kani, test))]
const SEEDED_TECHNIQUE_ID_COUNT: usize = 21;
```

**Corrected `EMITTED_IDS` (13 entries — 6 Enterprise + 7 ICS):**

```rust
// in kani_proofs module:
const EMITTED_IDS: &[&str] = &[
    // Enterprise (6, unchanged):
    "T1027",     // TLS: SNI anomaly
    "T1036",     // Reassembly: conflicting overlap
    "T1046",     // HTTP: admin panel
    "T1083",     // HTTP: path traversal
    "T1499.002", // HTTP: header flood
    "T1505.003", // HTTP: web shell
    // ICS — 7 Modbus (ADR-005 + ADR-006 Decision 12):
    "T0855",     // Modbus: write-class FC / unauthorized command + burst companion
    "T0836",     // Modbus: 0x06/0x10/0x16 parameter writes
    "T0814",     // Modbus: 0x08 Force Listen Only / Restart Comms
    "T0806",     // Modbus: write burst or sustained rate exceeded
    "T0835",     // Modbus: I/O image manipulation writes (coil-only)
    "T0831",     // Modbus: coordinated write sequence (5-second window)
    "T0888",     // Modbus: recon FCs 0x11 (Report Server ID), 0x2B/0x0E (Read Device ID)
];
// Total: 6 + 7 = 13
```

### 12.4 MitreMatrix Discriminator Coverage

The `technique_matrix(id)` function (architecture-delta §4.1) uses the rule `b[1] == b'0'` →
ICS. T0888: b[1] = b'0' → `MitreMatrix::Ics`. Confirmed correct, no change to the
discriminator logic.

### 12.5 Architecture-Delta and ADR-005 Updates Required

- **Architecture-delta §2.6 emission table:** Replace `T0846 / Remote System Discovery`
  row with `T0888 / Remote System Information Discovery` for the recon path.
- **Architecture-delta §4.2 (new technique_info arms):** Replace T0846 arm note
  ("recon FCs now emitted") with T0888 new arm; retain T0846 arm with note "kept seeded,
  not Modbus-emitted".
- **Architecture-delta §4.3 (VP-007 atomic update set):** Change ICS SEEDED 9 → 10;
  replace T0846 in EMITTED_IDS with T0888; update EMITTED total to 13 (unchanged), SEEDED
  to 21.
- **ADR-005 Decision 4:** Update EMITTED ICS IDs from
  `{T0855, T0836, T0814, T0806, T0835, T0831, T0846}` to
  `{T0855, T0836, T0814, T0806, T0835, T0831, T0888}`. Update SEEDED count from 20 → 21.
  Update `Consequences §Positive` bullet to reference T0888.

**Product-owner obligations:**
- Revise BC-2.14.020: change recon-path `mitre_technique` (now `mitre_techniques` — see
  Decision 13) from `Some("T0846")` to `vec!["T0888"]`. Update BC body, evidence examples,
  and any cross-references to T0846 as the Modbus recon technique. Update title to name T0888.
- Update BC-2.10.005 ("technique_name Returns Some for Every Seeded ID (15 Total)"): update
  count from 15 to 21 in title and body.
- Update BC-2.10.007 ("technique_tactic Returns Correct Tactic for Every Seeded ID"): add
  T0888 → Discovery row.
- Update BC-2.10.008 ("All Emitted Technique IDs Resolve in Lookup"): add T0888 to the
  emitted set; remove T0846 from Modbus-emitted list.

**Formal-verifier obligations:**
- Update verification-delta §4 EMITTED_IDS table: replace T0846 with T0888 in the 7 ICS
  emitted IDs.
- Update VP-007 dependency note: SEEDED count 20 → 21; EMITTED_ICS T0846 → T0888.

---

## Decision 13 — Multi-Tag Finding (supersedes v1.0 Decision 7)

This is the largest change in v2 — a breaking modification to the core `Finding` type,
every reporter, and all emission sites across all analyzers. It targets **v0.3.0**.

### 13.1 Type Change: Finding.mitre_technique → mitre_techniques

**Old field:**
```rust
pub mitre_technique: Option<String>,
```

**New field:**
```rust
pub mitre_techniques: Vec<String>,
```

Semantics:
- `Vec::new()` (empty) = no technique attributed. Replaces `None`. Used for anomaly-only
  findings (HTTP anomaly without ATT&CK ID, exception-burst findings, etc.)
- `vec!["T1027"]` = single technique. Migration path for all existing single-technique
  emission sites.
- `vec!["T0855", "T0836"]` = co-attributed techniques on one finding.

**Rationale (industry standard):** Sigma rules carry multiple `attack.tXXXX` tags per
detection. Elastic Common Schema `threat.technique` is multi-valued by design. One observable
→ one finding → N technique tags is the Sigma/Elastic/MITRE-aligned norm. Cap-to-most-specific
(Decision 7, v1.0) conflated volume control with attribution and discarded analytically useful
signal. Volume control is now handled by burst aggregation (Decision 13.5), not by tag
suppression.

### 13.2 JSON Schema Change (BREAKING — v0.3.0)

**Old JSON output for a finding with one technique:**
```json
{
  "category": "Execution",
  "verdict": "LIKELY",
  "confidence": "HIGH",
  "summary": "...",
  "evidence": ["..."],
  "mitre_technique": "T0836"
}
```

**New JSON output (same finding):**
```json
{
  "category": "Execution",
  "verdict": "LIKELY",
  "confidence": "HIGH",
  "summary": "...",
  "evidence": ["..."],
  "mitre_techniques": ["T0836"]
}
```

**For co-attributed findings:**
```json
{
  "mitre_techniques": ["T0855", "T0836"]
}
```

**For no-technique findings:**
The field is either absent (using `#[serde(skip_serializing_if = "Vec::is_empty")]`) or
emits `"mitre_techniques": []`. **Decision: omit when empty** (consistent with the existing
`skip_serializing_if = "Option::is_none"` policy on the other optional fields; empty arrays
serialize as `[]` which is noisier for consumers). Use:

```rust
#[serde(skip_serializing_if = "Vec::is_empty")]
pub mitre_techniques: Vec<String>,
```

This means: the key is absent from JSON when `mitre_techniques` is empty (no technique),
and present with a JSON array when non-empty (one or more techniques).

### 13.3 CSV Schema Change (BREAKING — v0.3.0)

**Old:** column 6 was `mitre_technique` (scalar string or empty).

**New:** column 6 is renamed `mitre_techniques` and carries a **semicolon-joined** list:

| Value | CSV column value |
|-------|-----------------|
| `vec![]` (empty) | `""` (empty string, same as prior `None`) |
| `vec!["T0836"]` | `"T0836"` (identical to prior scalar) |
| `vec!["T0855", "T0836"]` | `"T0855;T0836"` (semicolons, no space) |

The CSV column header changes from `mitre_technique` to `mitre_techniques`. Column count
remains 9. Semicolons do not trigger CSV injection (neutral characters); the existing
`neutralize_csv_injection` guard is unchanged.

**CSV field delimiter mandate (ADR-006 Sub-decision 2):** The CSV writer MUST be explicitly
configured with a **comma** (`,`) as the field delimiter. Do not rely on locale defaults or
implicit platform behavior. The semicolon is strictly an intra-cell separator within column 6
and must not conflict with the comma field delimiter. BC-2.11.024 mirroring obligation: the
product-owner MUST specify this comma-delimiter constraint when revising BC-2.11.024.

BC-2.11.020 (nine-column CSV header) MUST be revised: update column 6 name from
`mitre_technique` to `mitre_techniques` in title, body, invariants, examples, and
column-order table.

### 13.4 Migration: All Existing Emission Sites

Every existing `Finding { mitre_technique: Some("TXXXX") }` becomes
`Finding { mitre_techniques: vec!["TXXXX"] }`.

Every `Finding { mitre_technique: None }` becomes `Finding { mitre_techniques: vec![] }`.

**Blast radius — files requiring changes:**

| File | Change |
|------|--------|
| `src/findings.rs` | `mitre_technique: Option<String>` → `mitre_techniques: Vec<String>` with `#[serde(skip_serializing_if = "Vec::is_empty")]` |
| `src/analyzer/http.rs` | All `mitre_technique:` struct fields updated (10 emission sites) |
| `src/analyzer/tls.rs` | All `mitre_technique:` struct fields updated (6+ emission sites) |
| `src/analyzer/modbus.rs` (new) | Author directly with `mitre_techniques: vec![...]` |
| `src/reporter/csv.rs` | Header col-6 rename; join logic: `f.mitre_techniques.join(";")` instead of `f.mitre_technique.as_deref().unwrap_or("")` |
| `src/reporter/terminal.rs` | All `f.mitre_technique` references updated to iterate `f.mitre_techniques` |
| `src/reporter/json.rs` | Serde handles it automatically via `#[derive(Serialize)]`; update module-level comment |
| `src/reporter/mod.rs` | Update if it references `mitre_technique` |
| Test files | Any test constructing `Finding { mitre_technique: ... }` updated |

**Note on `src/reassembly/`:** grep for `mitre_technique` confirms it is emitted in
`src/reassembly/` (T1036 for conflicting overlap). That site is also updated.

### 13.5 Co-Emission BCs Revised: Aggregation, Not Tag-Suppression

BC-2.14.013, BC-2.14.014, BC-2.14.015, and BC-2.14.016 from v1.0 Decision 7 are revised
as follows:

**Superseded rule (Decision 7, v1.0):** T0836 > T0835 priority — when T0836 fires, T0835
is suppressed for the same PDU. T0855 always emits separately. Multiple findings per PDU.

**New rule (Decision 13):** One finding per write-class PDU carrying ALL applicable technique
tags. Volume control is via burst aggregation, not tag suppression.

**Per-PDU emission rules (co-emission model):**

| PDU type | `mitre_techniques` on the single per-PDU write finding |
|----------|------------------------------------------------------|
| Holding-register write (FC 0x06, 0x10, 0x16) | `["T0855", "T0836"]` — both unauthorized command + modify parameter |
| Coil/output write (FC 0x05, 0x0F) | `["T0855", "T0835"]` — both unauthorized command + I/O image manipulation |
| Write FC not in above subsets (FC 0x15, 0x17) | `["T0855"]` — unauthorized command only |

**One finding per write-class PDU** carrying the full tag set. Prior model emitted 2–3
separate single-technique findings per PDU; new model emits 1 multi-tag finding per PDU.
This reduces finding count while preserving full technique attribution.

**BC obligation:**
- BC-2.14.013: REVISE — T0855 is no longer a standalone per-write finding. It is now
  co-included in the per-PDU multi-tag finding alongside T0836 or T0835. Remove "separately
  emitted for every write-class FC" semantics; replace with "T0855 is always included in the
  `mitre_techniques` vec of every write-class PDU finding."
- BC-2.14.014: REVISE — T0836 finding becomes a single finding `mitre_techniques: ["T0855", "T0836"]`
  for FC 0x06/0x10/0x16. Remove the "T0836 takes priority over T0835 for same PDU" suppression
  rule; this is no longer needed since both tags coexist on one finding.
- BC-2.14.015: REVISE — T0835 finding becomes `mitre_techniques: ["T0855", "T0835"]` for
  FC 0x05/0x0F. Remove the "only applies when T0836 does not apply" precondition; it remains
  true that T0835 only applies for coil-only writes, but the reason is definitional (coil
  writes vs register writes), not suppression.
- BC-2.14.016 (T0831): Revise postcondition — T0831 is an **inline co-tag** on the per-PDU
  write finding, NOT a standalone Finding object. The 2nd holding-register write within the
  5-second window emits ONE finding with `mitre_techniques: ["T0855", "T0836", "T0831"]`.
  (The canonical BC-2.14.016 body is the authoritative model; this directive matches it.)
  `t0831_burst_emitted` guards co-tagging at most once per window overflow. Subsequent writes
  in the same window emit `["T0855", "T0836"]` without the T0831 tag.

### 13.6 Burst Aggregation: Volume Control Model

To prevent write floods from exhausting `MAX_FINDINGS`, volume control is now via
**aggregation**, not tag suppression:

**Burst finding (T0806 + T0855):** Emitted once per burst event (one per 1-second window
overflow). This is unchanged from v1.0 — the `window_burst_emitted` flag already implements
this. The burst finding carries `mitre_techniques: ["T0806", "T0855"]` (co-tagged, one finding).

**Sustained finding (T0806 + T0855):** Emitted once per sustained-window overflow
(Decision 11). `sustained_burst_emitted` flag. Carries `mitre_techniques: ["T0806", "T0855"]`.

**Per-PDU write findings:** One finding per write-class PDU carrying `["T0855", "T0836"]`
or `["T0855", "T0835"]`. During a burst, these individual per-PDU write findings ARE still
emitted until MAX_FINDINGS is reached (poison-skip model). The burst finding is a SUPPLEMENT
to, not a replacement for, the per-PDU findings. The `window_burst_emitted` and
`sustained_burst_emitted` flags only gate the T0806+T0855 burst-level finding, not the
per-PDU write findings.

**Finding count per write PDU (worst case — both burst events fire on the same PDU):**

| Scenario | Findings | Tags |
|----------|----------|------|
| Register write (FC=0x06) tipping T0806 burst threshold AND T0831 window fires on same PDU | Per-PDU: `["T0855","T0836","T0831"]` (T0831 co-tagged inline); Burst: `["T0806","T0855"]`; Sustained (if applicable): `["T0806","T0855"]` | 3 findings max for this scenario (vs 5 in v1.0) |
| Register write, mid-burst | Per-PDU: `["T0855","T0836"]` | 1 finding (vs 2 in v1.0) |
| Coil write tipping burst | Per-PDU: `["T0855","T0835"]`; Burst: `["T0806","T0855"]` | 2 findings (vs 4 in v1.0) |

This is strictly fewer findings per PDU than v1.0 Decision 7, while preserving all
technique attributions. The `MAX_FINDINGS` cap risk is further reduced.

### 13.7 Terminal Reporter Update

The terminal reporter (`src/reporter/terminal.rs`) currently accesses `f.mitre_technique` as
`Option<String>` in three places:

1. `if let Some(ref t) = f.mitre_technique { ... }` → becomes iteration over
   `f.mitre_techniques` (skip block if empty).
2. `if let Some(ref id) = f.mitre_technique { ... }` → iterate, display all IDs
   (e.g., `"MITRE: T0855, T0836"` for multi-tag).
3. `f.mitre_technique.as_deref().and_then(technique_tactic)` (tactic grouping) → primary
   tactic is from `mitre_techniques[0]`; for multi-tag findings, group under the first
   technique's tactic. (This is a reasonable approximation; exact multi-tactic display is
   a future enhancement.)

**Bucket-order determinism (ADR-006 Sub-decision 3 mandate):** The tactic-bucket grouping
using `mitre_techniques[0]` is deterministic ONLY IF the canonical construction order is
followed at all emission sites. The canonical order is defined in ADR-006 Sub-decision 3
(T0806 > T0855 > T0836 > T0835 > T0831 > T0814 > T0888). All emission sites MUST use
`vec![...]` literals in this order. The terminal reporter itself does NOT sort — it relies on
the determinism guarantee from the emission sites. BC-2.11.013 MUST specify:
"group by `mitre_techniques[0]` where non-empty; ties within a tactic group by finding order."

BC-2.11.013 (MITRE grouping tactic headers) and BC-2.11.015 (no-technique → Uncategorized)
MUST be revised to specify the multi-techniques behavior.

### 13.8 VP Impact Assessment

| VP | Impact | Action |
|----|--------|--------|
| VP-007 | Indirect — EMITTED_IDS now contains IDs that will appear in `mitre_techniques: vec![...]` calls; the grep pattern changes from `mitre_technique: Some` to `mitre_techniques: vec!`; the VP-007 comment in `mitre.rs` must be updated | Update mitre.rs comment + EMITTED_IDS |
| VP-016 (mitre-tactic-grouping-order) | Direct — proof harness constructs `Finding` with `mitre_technique` field | Harness MUST be updated to use `mitre_techniques: vec![...]`; behavior is tested for single-technique case (first-element grouping) |
| VP-020 (csv-injection-neutralization) | Direct — harness constructs Finding with `mitre_technique` field | Update to `mitre_techniques: vec!["T1036"]` etc. |
| VP-021 (timestamp-provenance) | Direct — test helper `Finding { mitre_technique: ... }` instances | Update all Finding construction in test helpers |
| VP-022 (Modbus parse) | Indirect — does not test Finding construction | No change needed |

### 13.9 Full List of Existing BCs Requiring Revision

The following BCs from EXISTING (pre-F2) converged specs are forced to revision by
Decision 13. Product-owner MUST revise all of them.

**SS-09 (Finding type — src/findings.rs):**

| BC | Current state | Required revision |
|----|---------------|-------------------|
| BC-2.09.001 | `mitre_technique: Option<String>` in postcondition field list | Change field to `mitre_techniques: Vec<String>`; update EC-003 (`mitre_technique = Some("T1036")` → `mitre_techniques = vec!["T1036"]`); update EC-004 (`mitre_technique = None` → `mitre_techniques = vec![]`); add EC-006 (multi-tag case: `vec!["T0855","T0836"]`). Update emission-site count (was 22; will increase with Modbus). |
| BC-2.09.006 | "The four affected fields are: `mitre_technique`, `source_ip`, `timestamp`, `direction`" | Change: `mitre_technique` → `mitre_techniques`; update skip_serializing_if rule: `Vec::is_empty` instead of `Option::is_none`; update EC-001 (empty → field absent), EC-002 (`Some("T1036")` → `vec!["T1036"]`); add EC-006 (multi-tag: `vec!["T0855","T0836"]` → `"mitre_techniques": ["T0855","T0836"]`). |

**SS-10 (MITRE lookup — src/mitre.rs):**

| BC | Current state | Required revision |
|----|---------------|-------------------|
| BC-2.10.005 | "technique_name Returns Some for Every Seeded ID (15 Total)" | Update count: 15 → 21. |
| BC-2.10.007 | Tactic table for seeded IDs | Add T0888 → Discovery row. |
| BC-2.10.008 | Lists emitted IDs with grep pattern `mitre_technique: Some` | Update grep pattern to `mitre_techniques: vec!`; replace T0846 with T0888 in Modbus-emitted list; add all 7 ICS emitted IDs. Update emission-site count. |

**SS-11 (Reporters — src/reporter/):**

| BC | Current state | Required revision |
|----|---------------|-------------------|
| BC-2.11.013 | MITRE grouping via `f.mitre_technique` | Specify: group by `mitre_techniques[0]`; empty vec → Uncategorized (same as prior `None`). |
| BC-2.11.015 | "No-Technique or Unknown-ID Findings Land in Uncategorized" | Update to: "empty `mitre_techniques` vec or all-unknown IDs → Uncategorized". |
| BC-2.11.017 | "Default Rendering Emits MITRE: <id> Only (No Em-Dash)" | Update to multi-ID rendering: `"MITRE: T0855, T0836"`. |
| BC-2.11.020 | "CsvReporter Emits Exactly Nine Columns in Fixed Header Order" — column 6: `mitre_technique` | Rename column 6 to `mitre_techniques` in title, body, header contract, column-order table, examples. Clarify join format (semicolons). |
| BC-2.11.024 | "CsvReporter Encodes None Optional Fields as Empty Strings" | Update: `mitre_technique = None` → `mitre_techniques = vec![]`; same serialization (empty string). Add case: `vec!["T0855","T0836"]` → `"T0855;T0836"` in column 6. |

**SS-14 (Modbus — new):**

| BC | Required revision |
|----|-------------------|
| BC-2.14.013 | REVISE: per §13.5 — T0855 is co-included in multi-tag write finding, not standalone. |
| BC-2.14.014 | REVISE: per §13.5 — T0836 finding is `["T0855","T0836"]` single finding. |
| BC-2.14.015 | REVISE: per §13.5 — T0835 finding is `["T0855","T0835"]` single finding. |
| BC-2.14.016 | REVISE: per §13.5 — T0831 is an inline co-tag on the per-PDU write finding: `["T0855","T0836","T0831"]` on the 2nd+ holding-register write in the 5s window; no standalone T0831 Finding object. |
| BC-2.14.017 | REVISE: per §11.7 — dual-window (burst+sustained); burst finding `["T0806","T0855"]`. |
| BC-2.14.020 | REVISE: per §12.5 — recon FCs emit `["T0888"]`; drop T0846. |
| BC-2.14.024 | REVISE: per §11.7 — dual flags replaces single flag. |

**Total existing BCs requiring revision: 5 (SS-09/SS-10/SS-11) + 7 (SS-14) = 12 BCs.**

### 13.10 Breaking Change Scope Declaration

Feature #7 targets **v0.3.0** because:
- `mitre_technique: Option<String>` → `mitre_techniques: Vec<String>` is a breaking change
  to the public Rust type `Finding` in `src/findings.rs`.
- JSON output key changes from `"mitre_technique"` (string) to `"mitre_techniques"` (array).
- CSV column 6 header changes from `mitre_technique` to `mitre_techniques`; multi-value
  entries use semicolons.

Any downstream consumer (JSON parsers, CSV pipelines, code using the `Finding` struct)
MUST update to the new schema. The CHANGELOG must call this out explicitly.

**F1 impact boundary note:** F1 (Feature-100, timestamp) documented `Finding` as
DEPENDENT/unchanged. Decision 13 changes that classification: `Finding` is now MODIFIED.
Update the F2 spec delta boundary declaration accordingly.

---

## Summary: Dependency Graph Acyclicity Confirmation

Decision 11 (dual-window): adds fields to `ModbusFlowState` and `ModbusAnalyzer` only.
No new module imports. Graph remains acyclic.

Decision 12 (T0888): adds one arm to `technique_info` and one entry to `SEEDED_TECHNIQUE_IDS`.
No new module imports. Graph remains acyclic.

Decision 13 (multi-tag): modifies `Finding` in `src/findings.rs`. All analyzers import
`findings.rs`; `findings.rs` does NOT import any analyzer. All reporters import `findings.rs`;
`findings.rs` does NOT import any reporter. The dependency direction is unchanged:
`analyzers → findings ← reporters`. No new cycle is introduced.

---

## Summary: All Files Changed or Required to Change (v2 Burst)

### Architect produces in this burst

| File | Action |
|------|--------|
| `.factory/phase-f2-spec-evolution/f2-fix-directives.md` | This file (v2, supersedes v1.0) |
| `.factory/specs/architecture/decisions/ADR-006-multi-technique-finding-attribution.md` | NEW (Decision 13) |

### Product-owner (next burst)

| File | Action |
|------|--------|
| `.factory/phase-f2-spec-evolution/architecture-delta.md` | **DONE by architect (FIX C-2)** — §1 (F1 impact-boundary MODIFIED; dual-flag scope), §2.2 (dual thresholds write_burst/write_sustained), §2.3 (sustained fields + complete field list), §2.6 (T0888 replaces T0846; dual-burst rows; co-emission multi-tag; T0831 inline co-tag), §4.2 (T0888 new arm; T0846 non-emitted note), §4.3 (SEEDED 15→21, EMITTED 13, T0888 in EMITTED_IDS, T0846 NOT emitted, grep pattern updated), §5.1 (dual CLI flags), §5.2 (new() call), §11 BC summary rows, §12 (T0831 inline co-tag), Appendix constants |
| `BC-2.14.013` | Revise per §13.5 (T0855 co-included, not standalone) |
| `BC-2.14.014` | Revise per §13.5 (single finding `["T0855","T0836"]`) |
| `BC-2.14.015` | Revise per §13.5 (single finding `["T0855","T0835"]`) |
| `BC-2.14.016` | Revise per §13.5 (T0831 is inline co-tag on per-PDU finding: `["T0855","T0836","T0831"]`; no standalone T0831 Finding object) |
| `BC-2.14.017` | Revise per §11.7 (dual-window; burst finding `["T0806","T0855"]`) |
| `BC-2.14.020` | Revise per §12.5 (T0888 replaces T0846; 0x07 excluded) |
| `BC-2.14.024` | Revise per §11.7 (dual flags: `--modbus-write-burst-threshold` + `--modbus-write-sustained-threshold`) |
| `BC-2.09.001` | EXISTING — revise per §13.9 (field rename) |
| `BC-2.09.006` | EXISTING — revise per §13.9 (skip_serializing_if Vec::is_empty; multi-tag ECs) |
| `BC-2.10.005` | EXISTING — revise per §12.5 (count 15→21) |
| `BC-2.10.007` | EXISTING — revise per §12.5 (add T0888) |
| `BC-2.10.008` | EXISTING — revise per §13.9 (grep pattern + T0888 replaces T0846 in emitted list) |
| `BC-2.11.013` | EXISTING — revise per §13.7 (multi-techniques tactic grouping) |
| `BC-2.11.015` | EXISTING — revise per §13.7 (empty vec → Uncategorized) |
| `BC-2.11.017` | EXISTING — revise per §13.7 (multi-ID rendering) |
| `BC-2.11.020` | EXISTING — revise per §13.3 (column 6: mitre_technique → mitre_techniques) |
| `BC-2.11.024` | EXISTING — revise per §13.3 (semicolon-join for multi-value) |
| `BC-2.10.005` title | EXISTING — update "15 Total" → "21 Total" |
| ADR-005 | **DONE by architect (FIX I-3)** — supersession banner added; Decision 3 corrected to dual-window model; Decision 4 corrected: T0846→T0888 emitted, SEEDED 15→21, multi-tag note; Consequences §Positive updated; §Negative "single window" bullet replaced with dual-window + count-21 correction |

### Formal-verifier (next burst)

| File | Action |
|------|--------|
| `verification-delta.md` | §4 EMITTED_IDS: replace T0846 with T0888; update SEEDED 20→21; update VP-007 dependency note |
| `VP-016` proof harness | Update `Finding { mitre_technique: ... }` → `Finding { mitre_techniques: vec![...] }` per §13.8 |
| `VP-020` proof harness | Update per §13.8 |
| `VP-021` test helpers | Update per §13.8 |
| VP-INDEX | No count change (VP-022 was already planned; no new VPs from these decisions) |

### ADR-005

Update per §12.5 and §11.7: replace T0846 with T0888 in emitted IDs, SEEDED 20→21,
dual-window threshold model in Decision 3.
