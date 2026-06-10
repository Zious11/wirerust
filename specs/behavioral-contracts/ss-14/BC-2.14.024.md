---
document_type: behavioral-contract
level: L3
version: "2.1"
status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-14
capability: CAP-14
lifecycle_status: active
introduced: v0.3.0-feature-007
modified:
  - version: "2.0"
    date: 2026-06-09
    change: "UPDATED (v2.0 — Decision 11, f2-fix-directives.md §11): REMOVED --modbus-write-threshold (single-window flag). ADDED two new flags: --modbus-write-burst-threshold (default 20, 1-second burst window) and --modbus-write-sustained-threshold (default 10, >=2-second sustained/average window). Both flags reject 0 with separate error messages. Both flow through ModbusAnalyzer::new(burst, sustained) to separate write_burst_threshold and write_sustained_threshold fields. Targets v0.3.0."
  - version: "2.1"
    date: 2026-06-10
    change: "v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. All T0855 technique ID references in Description and Canonical Test Vectors updated to T1692.001. Tactic unchanged: IcsImpairProcessControl. Issue #222; audit: mitre-ics-v19-catalog-audit.md."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/phase-f2-spec-evolution/f2-fix-directives.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
input-hash: TBD
---

# BC-2.14.024: --modbus-write-burst-threshold and --modbus-write-sustained-threshold Configure Dual-Window Burst Detection

<!-- Previous version (v1.0): "--modbus-write-threshold Configures Per-Flow Write-Burst Rate Threshold Consumed by Burst Detector"
     v1.0 model: single --modbus-write-threshold flag (default 10), ModbusAnalyzer::new(write_threshold: u32).
     v2.0 model (Decision 11): TWO flags replace the single flag.
       --modbus-write-burst-threshold  (default 20): fires burst finding when >N writes in any 1s window.
       --modbus-write-sustained-threshold (default 10): fires sustained finding when >M avg writes/s over >=2s.
       --modbus-write-threshold is REMOVED. ModbusAnalyzer::new(burst_threshold, sustained_threshold).
       Targets v0.3.0.
-->

## Description

The `analyze` subcommand exposes two independent write-burst detection threshold flags for the
Modbus analyzer, replacing the single `--modbus-write-threshold` flag from v1.0:

1. `--modbus-write-burst-threshold <N>` — configures the per-flow burst detector. Fires T0806 +
   T1692.001 (co-tagged `["T0806","T1692.001"]` on one finding) when more than N write-class function
   codes are observed within any single 1-second window. Default: 20. Minimum: 1.

2. `--modbus-write-sustained-threshold <M>` — configures the per-flow sustained-rate detector.
   Fires T0806 + T1692.001 when the average write-FC rate exceeds M writes/second over a contiguous
   window of ≥2 seconds. Detection math: `sustained_window_write_count > M * elapsed_secs` at
   `elapsed_secs >= 2`. Default: 10. Minimum: 1.

Both flags are independent: both can be specified, either can be omitted (taking its default),
and both reject 0 with a user-readable fatal error before analyzer construction. The thresholds
flow to `ModbusAnalyzer.write_burst_threshold: u32` and `ModbusAnalyzer.write_sustained_threshold: u32`
respectively. Targets v0.3.0.

## Preconditions

1. The `analyze` subcommand is configured via `src/cli.rs`.
2. `--modbus-write-burst-threshold` is declared as:
   ```rust
   #[arg(long, default_value_t = 20)]
   modbus_write_burst_threshold: u32,
   ```
3. `--modbus-write-sustained-threshold` is declared as:
   ```rust
   #[arg(long, default_value_t = 10)]
   modbus_write_sustained_threshold: u32,
   ```
4. `--modbus` or `--all` must also be present for these flags to have effect; both flags are
   parsed regardless (clap convention), but if `enable_modbus == false`, the threshold values
   are simply unused.

## Postconditions

### P1: Both flags absent (defaults)

1. `modbus_write_burst_threshold = 20` (clap `default_value_t`).
2. `modbus_write_sustained_threshold = 10` (clap `default_value_t`).
3. `ModbusAnalyzer::new(20, 10)` is constructed when `enable_modbus == true`.
4. `self.write_burst_threshold = 20` and `self.write_sustained_threshold = 10` in the instance.
5. Burst detector fires when `window_write_count > 20` within the 1-second window.
6. Sustained detector fires when `sustained_window_write_count > 10 * elapsed_secs` at `elapsed_secs >= 2`.

### P2: `--modbus-write-burst-threshold N` present (N > 0)

1. `modbus_write_burst_threshold = N`.
2. `ModbusAnalyzer::new(N, <sustained_default_or_given>)` constructed.
3. `self.write_burst_threshold = N`.
4. Burst detector fires when `window_write_count > N`.

### P3a: `--modbus-write-burst-threshold 0`

1. clap parses 0 as `u32`.
2. Rejected before `ModbusAnalyzer::new` is called, with a fatal error:
   `"--modbus-write-burst-threshold must be >= 1 (got 0)"`.
3. Process exits with code 1.
4. **Rationale**: a threshold of 0 causes the burst detector to fire on every write FC (0 < 1,
   trivially exceeded). This degrades signal-to-noise ratio to zero.

### P3b: `--modbus-write-sustained-threshold 0`

1. clap parses 0 as `u32`.
2. Rejected before `ModbusAnalyzer::new` is called, with a fatal error:
   `"--modbus-write-sustained-threshold must be >= 1 (got 0)"`.
3. Process exits with code 1.

### P4: `--modbus-write-sustained-threshold M` present (M > 0)

1. `modbus_write_sustained_threshold = M`.
2. `ModbusAnalyzer::new(<burst_default_or_given>, M)` constructed.
3. `self.write_sustained_threshold = M`.
4. Sustained detector fires when `sustained_window_write_count > M * elapsed_secs` at `elapsed_secs >= 2`.

### P5: Flow to ModbusFlowState

1. Both threshold values flow from CLI → `main.rs` → `ModbusAnalyzer::new(burst, sustained)` →
   `self.write_burst_threshold: u32` and `self.write_sustained_threshold: u32` (top-level
   fields on `ModbusAnalyzer`).
2. Each `ModbusFlowState` uses `self.write_burst_threshold` for burst comparison and
   `self.write_sustained_threshold` for sustained comparison.
3. Both thresholds are applied uniformly across all flows. No per-flow threshold override.
4. Thresholds are immutable during the run: set once at construction, never updated mid-run.

## Invariants

1. **Type is `u32` for both**: unsigned 32-bit integer. Negatives are type-rejected by clap.
2. **Defaults:** `DEFAULT_WRITE_BURST_THRESHOLD = 20`, `DEFAULT_WRITE_SUSTAINED_THRESHOLD = 10`.
   - Burst default 20 is defensible: legitimate SCADA baseline is 0.1–5 writes/s; legitimate
     transitions can reach 5–15/s for 10–60s; 20/s within 1s is outside normal ranges.
   - Sustained default 10 is defensible: >10/s averaged over ≥2s is anomalous for standard
     Modbus SCADA; normal polling rates are 1–4 writes/s per field device.
3. **Zero rejected (both)**: checked in `main.rs` after clap parsing:
   ```rust
   if *modbus_write_burst_threshold == 0 {
       anyhow::bail!("--modbus-write-burst-threshold must be >= 1 (got 0)");
   }
   if *modbus_write_sustained_threshold == 0 {
       anyhow::bail!("--modbus-write-sustained-threshold must be >= 1 (got 0)");
   }
   ```
4. **Burst threshold semantics**: `window_write_count > write_burst_threshold` (strict `>`).
   A burst threshold of 20 means the 21st write within the 1-second window triggers the burst.
5. **Sustained threshold semantics**: `sustained_window_write_count > write_sustained_threshold * elapsed_secs`
   (strict `>`, integer multiplication). A sustained threshold of 10 at elapsed_secs=2 fires
   when write count > 20 in that window (= >10/s average).
6. **Prior `--modbus-write-threshold` is REMOVED.** Any implementation or test that references
   this flag is stale and must be updated. The old `write_threshold: u32` field on
   `ModbusAnalyzer` is replaced by `write_burst_threshold: u32` and `write_sustained_threshold: u32`.
7. **Independence**: the two flags are independent of each other. Both can be set, or one, or
   neither (taking defaults). They are also independent of `--modbus` / `--all` (parsed always).
8. **`ModbusAnalyzer::new` signature (v2):**
   ```rust
   pub fn new(write_burst_threshold: u32, write_sustained_threshold: u32) -> Self { ... }
   ```
   This replaces the v1.0 `ModbusAnalyzer::new(write_threshold: u32)` signature.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--modbus-write-burst-threshold 0` | Fatal error: `"--modbus-write-burst-threshold must be >= 1 (got 0)"`; exit 1 |
| EC-002 | `--modbus-write-sustained-threshold 0` | Fatal error: `"--modbus-write-sustained-threshold must be >= 1 (got 0)"`; exit 1 |
| EC-003 | `--modbus-write-burst-threshold 1` | Burst threshold = 1; fires on 2nd write-class PDU within the 1-second window |
| EC-004 | `--modbus-write-sustained-threshold 1` | Sustained threshold = 1; at elapsed_secs=2: fires when count > 2 (= >1/s avg) |
| EC-005 | `--modbus-write-burst-threshold 100` | High burst threshold; typical captures with normal traffic produce zero T0806 burst findings |
| EC-006 | `--modbus-write-burst-threshold 4294967295` (u32::MAX) | Parsed without error; burst detector never fires in practice |
| EC-007 | Both flags absent | `modbus_write_burst_threshold=20`, `modbus_write_sustained_threshold=10`; default behavior |
| EC-008 | Only `--modbus-write-burst-threshold 5` specified | Burst threshold = 5; sustained threshold = 10 (default) |
| EC-009 | Only `--modbus-write-sustained-threshold 5` specified | Burst threshold = 20 (default); sustained threshold = 5 |
| EC-010 | Both flags specified: `--modbus-write-burst-threshold 5 --modbus-write-sustained-threshold 3` | Burst = 5, sustained = 3; both detectors use their respective thresholds independently |
| EC-011 | Either threshold flag without `--modbus` | Flag parsed; value ignored; no analyzer constructed; no error |
| EC-012 | `--modbus-write-threshold` (the REMOVED v1.0 flag) | clap rejects as unknown argument; exit 2 (clap unknown-flag convention). This flag no longer exists. |
| EC-013 | `--modbus-write-burst-threshold -1` (negative) | clap rejects as invalid u32 before `main.rs` validation; exit 2 |

## Canonical Test Vectors

| Setup | Expected Behavior | Category |
|-------|------------------|----------|
| `wirerust analyze test.pcap --modbus --modbus-write-burst-threshold 5` and 6 write-class PDUs in 1s | Burst Finding `["T0806","T1692.001"]` emitted (BC-2.14.017); sustained not yet evaluated | happy-path: burst threshold configured, fires |
| `wirerust analyze test.pcap --modbus --modbus-write-burst-threshold 5` and 5 write-class PDUs in 1s | No burst finding; per-PDU findings from BCs 2.14.013-015 | happy-path: burst threshold not exceeded |
| `wirerust analyze test.pcap --modbus --modbus-write-burst-threshold 0` | Exit 1: `"--modbus-write-burst-threshold must be >= 1 (got 0)"` | edge-case: burst zero rejected |
| `wirerust analyze test.pcap --modbus --modbus-write-sustained-threshold 0` | Exit 1: `"--modbus-write-sustained-threshold must be >= 1 (got 0)"` | edge-case: sustained zero rejected |
| `wirerust analyze test.pcap --modbus` (no threshold flags) | `ModbusAnalyzer::new(20, 10)` constructed; both defaults applied | happy-path: default values |
| `wirerust analyze test.pcap --modbus --modbus-write-sustained-threshold 7` and 8 writes/s for 3s (24 writes at elapsed 3s) | Sustained: 24 > 7*3=21 → FIRES sustained Finding | happy-path: sustained configured, low-and-slow fires |
| `wirerust analyze test.pcap --modbus-write-burst-threshold 3` (no `--modbus`) | No analyzer constructed; threshold ignored; exit 0 | edge-case: threshold without analyzer |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | (indirect) Both detectors use `window_write_count > write_burst_threshold` and `sustained_window_write_count > write_sustained_threshold * elapsed_secs`; threshold values flow correctly | Integration test: construct `ModbusAnalyzer::new(5, 3)`, inject writes, verify burst fires at 6 and sustained fires at correct count |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the CLI configuration contract for the dual-window write-burst detection thresholds, which are the operator-tunable controls for T0806 (Brute Force I/O) burst and sustained-rate attribution within the ICS analysis capability |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — orthogonal; thresholds only affect finding emission, not flow routing) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22: `write_burst_threshold: u32` and `write_sustained_threshold: u32` fields); SS-12 (cli.rs: dual flags) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Techniques | T0806 (Brute Force I/O), T1692.001 (Unauthorized Command Message) — thresholds control when these findings fire |

## Related BCs

- BC-2.14.017 — depends on (this BC configures the thresholds that BC-2.14.017's burst and sustained detectors use)
- BC-2.14.023 — composes with (BC-2.14.023 enables the analyzer; this BC configures its dual thresholds)

## Architecture Anchors

- `src/cli.rs` — `#[arg(long, default_value_t = 20)] modbus_write_burst_threshold: u32`
- `src/cli.rs` — `#[arg(long, default_value_t = 10)] modbus_write_sustained_threshold: u32`
- `src/main.rs` — zero-check guards: burst `anyhow::bail!("...burst-threshold must be >= 1...")` and sustained `anyhow::bail!("...sustained-threshold must be >= 1...")`
- `src/main.rs` — `ModbusAnalyzer::new(*modbus_write_burst_threshold, *modbus_write_sustained_threshold)`
- `src/analyzer/modbus.rs` — `write_burst_threshold: u32` and `write_sustained_threshold: u32` fields on `ModbusAnalyzer`
- `src/analyzer/modbus.rs` — `DEFAULT_WRITE_BURST_THRESHOLD: u32 = 20` and `DEFAULT_WRITE_SUSTAINED_THRESHOLD: u32 = 10`
- `src/analyzer/modbus.rs` — `ModbusFlowState.window_write_count > self.write_burst_threshold` (burst) and `sustained_window_write_count > self.write_sustained_threshold * elapsed_secs` (sustained) conditions
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.2` (dual threshold fields) and `§5.1` (CLI declaration)

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — (indirect) dual detection thresholds flow from CLI to per-flow comparisons; covered by integration tests

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | f2-fix-directives.md §11.1 (CLI flags table: burst default 20 / sustained default 10; rejection of 0); §11.2 (constants); §11.3 (ModbusAnalyzer struct: dual fields); §11.4 (ModbusFlowState sustained fields); §11.5 (detection math); architecture-delta.md §5.1 (CLI declaration) and §2.2 (dual threshold fields) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (flag parsing is clap side effect; threshold values are pure data) |
| **Deterministic** | yes — same threshold values always produce same detector behavior |
| **Overall classification** | effectful shell (flag parsing; zero validation emits fatal error) |
