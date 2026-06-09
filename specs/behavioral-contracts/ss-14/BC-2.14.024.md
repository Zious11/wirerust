---
document_type: behavioral-contract
level: L3
version: "1.0"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
input-hash: TBD
---

# BC-2.14.024: --modbus-write-threshold Configures Per-Flow Write-Burst Rate Threshold Consumed by Burst Detector

## Description

The `analyze` subcommand exposes a `--modbus-write-threshold <N>` flag that sets the
per-flow write-burst detection threshold for the Modbus analyzer. The threshold controls
how many write-class function codes per second (within the 1-second sliding window defined
by `WRITE_RATE_WINDOW_SECS = 1`) must be observed before a T0806 / T0855 burst finding is
emitted (per BC-2.14.016 and BC-2.14.017). The default value is `10` (10 writes/second
sustained within the window). The flag accepts a `u32` value; zero and values that would be
semantically invalid (none beyond zero for this type) are rejected at parse time.

## Preconditions

1. The `analyze` subcommand is configured via `src/cli.rs`.
2. `--modbus-write-threshold` is declared as:
   ```rust
   #[arg(long, default_value_t = 10)]
   modbus_write_threshold: u32,
   ```
3. `--modbus` or `--all` must also be present for this flag to have effect; the flag is
   parsed regardless (it is not conditional in clap), but if `enable_modbus == false`, the
   threshold value is simply unused.

## Postconditions

### P1: Default (flag absent)
1. `modbus_write_threshold = 10` (clap default_value_t).
2. `ModbusAnalyzer::new(10)` is constructed when `enable_modbus == true`.
3. `self.write_threshold = 10` in the `ModbusAnalyzer` instance.
4. The burst detector fires when `window_write_count > 10` within the 1-second window.

### P2: Flag present with value N > 0
1. `modbus_write_threshold = N`.
2. `ModbusAnalyzer::new(N)` is constructed when `enable_modbus == true`.
3. `self.write_threshold = N` in the `ModbusAnalyzer` instance.
4. The burst detector fires when `window_write_count > N` within the 1-second window.

### P3: Flag present with value 0
1. clap parses `--modbus-write-threshold 0` as `modbus_write_threshold: u32 = 0`.
2. This value is rejected before `ModbusAnalyzer::new` is called, with a fatal error:
   `"--modbus-write-threshold must be >= 1 (got 0)"`.
3. The process exits with code 1 (consistent with other clap validation errors).
4. **Rationale**: a threshold of 0 means "fire on every write FC", which degrades the
   signal-to-noise ratio to useless. Requiring N >= 1 ensures at least one write-class PDU
   is required before the burst fires.

### P4: Flow to ModbusFlowState
1. The threshold value flows from CLI → `main.rs` → `ModbusAnalyzer::new(write_threshold)` →
   `self.write_threshold: u32` (top-level field on `ModbusAnalyzer`).
2. Each `ModbusFlowState` uses `self.write_threshold` (from the parent `ModbusAnalyzer`) as
   the threshold for its `window_write_count` comparison.
3. The threshold is applied uniformly across all flows processed by the analyzer instance.
4. There is no per-flow threshold override; the threshold is a single instance-level parameter.

## Invariants

1. **Type is `u32`**: the threshold is an unsigned 32-bit integer. Negative values are not
   representable (rejected at the type level). The maximum value `u32::MAX` (4_294_967_295)
   is technically valid but operationally nonsensical — no finding would ever fire. It is not
   explicitly rejected.
2. **Default = 10**: `DEFAULT_MODBUS_WRITE_THRESHOLD = 10` (defined as a constant in
   `src/analyzer/modbus.rs` or `src/cli.rs`). This default represents sustained 10 writes/s
   which is anomalously high for legitimate Modbus SCADA operations (normal polling rates are
   typically 1–4 writes/s per field device).
3. **Zero rejected**: `modbus_write_threshold == 0` is rejected with a user-readable error
   before analyzer construction. The check is performed in `main.rs` after clap parsing:
   ```rust
   if *modbus_write_threshold == 0 {
       anyhow::bail!("--modbus-write-threshold must be >= 1 (got 0)");
   }
   ```
4. **Threshold semantics**: `window_write_count > write_threshold` (strict greater-than) is
   the firing condition in the burst detector. A threshold of 10 means the 11th write within
   the 1-second window triggers the burst finding.
5. **Orthogonality**: `--modbus-write-threshold` is independent of all other flags except
   `--modbus` / `--all`. It can be specified without `--modbus` (it is parsed regardless),
   but the value is only consumed when a `ModbusAnalyzer` is constructed.
6. **Immutability during run**: the threshold is set once at construction time. It is not
   updated mid-run even if the pcap spans multiple hours.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--modbus-write-threshold 0` | Fatal error: `"--modbus-write-threshold must be >= 1 (got 0)"`; exit 1 |
| EC-002 | `--modbus-write-threshold 1` | Threshold = 1; fires on 2nd write-class PDU within the 1-second window |
| EC-003 | `--modbus-write-threshold 100` | High threshold; only fires on 101st write within 1 second; typical captures with normal traffic produce zero T0806 findings |
| EC-004 | `--modbus-write-threshold 4294967295` (u32::MAX) | Parsed without error; threshold = u32::MAX; burst detector never fires in practice |
| EC-005 | `--modbus-write-threshold` absent (default) | `modbus_write_threshold = 10`; detector fires on 11th write/second |
| EC-006 | `--modbus-write-threshold 10` without `--modbus` | Flag parsed; value ignored; no analyzer constructed; no error |
| EC-007 | Two calls in the same capture that both cross the threshold | Both burst events emit T0806 + T0855 (one per window overflow); `window_burst_emitted` latch reset on window reset prevents duplicate findings within the same window |
| EC-008 | `--modbus-write-threshold -1` (negative) | clap rejects as invalid u32 value before `main.rs` validation; error message from clap; exit 2 (clap convention) |

## Canonical Test Vectors

| Setup | Expected Behavior | Category |
|-------|------------------|----------|
| `wirerust analyze test.pcap --modbus --modbus-write-threshold 5` and 6 write-class PDUs arrive within 1s window | T0806 + T0855 burst findings emitted (BC-2.14.016, BC-2.14.017); `dropped_findings = 0` | happy-path: threshold configured, burst fires |
| `wirerust analyze test.pcap --modbus --modbus-write-threshold 5` and 5 write-class PDUs arrive within 1s window | No T0806 / T0855 burst findings; individual T0855 / T0836 / T0835 findings from write FCs per BC-2.14.013-015 | happy-path: threshold not exceeded |
| `wirerust analyze test.pcap --modbus --modbus-write-threshold 0` | Exit 1 with `"--modbus-write-threshold must be >= 1 (got 0)"` | edge-case: zero rejected |
| `wirerust analyze test.pcap --modbus` (no threshold flag) | `ModbusAnalyzer::new(10)` constructed; default threshold = 10 | happy-path: default value |
| `wirerust analyze test.pcap --modbus-write-threshold 3` (no `--modbus`) | No analyzer constructed; threshold ignored; exit 0 | edge-case: threshold without analyzer |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | (indirect) Burst detector logic uses `window_write_count > write_threshold`; the threshold value flows correctly into `ModbusFlowState` comparisons | Integration test: construct `ModbusAnalyzer::new(5)`, inject 6 writes within 1s, verify T0806 emitted |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the CLI configuration contract for the write-burst detection sensitivity parameter, which is the operator-tunable control for the T0806 (Brute Force I/O) and T0855 burst-attribution detection within the ICS analysis capability |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — orthogonal; threshold only affects finding emission, not flow routing) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22: `write_threshold: u32` field); SS-12 (cli.rs: `modbus_write_threshold: u32` flag) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |
| MITRE Technique | T0806 (Brute Force I/O), T0855 (Unauthorized Command Message) — threshold controls when these findings fire |

## Related BCs

- BC-2.14.016 — depends on (this BC configures the threshold that BC-2.14.016's burst detector uses)
- BC-2.14.017 — depends on (same threshold governs T0855 burst attribution in BC-2.14.017)
- BC-2.14.023 — composes with (BC-2.14.023 enables the analyzer; this BC configures its threshold)

## Architecture Anchors

- `src/cli.rs` — `#[arg(long, default_value_t = 10)] modbus_write_threshold: u32`
- `src/main.rs` — zero-check guard: `if *modbus_write_threshold == 0 { anyhow::bail!(...) }`
- `src/main.rs` — `ModbusAnalyzer::new(*modbus_write_threshold)`
- `src/analyzer/modbus.rs` — `write_threshold: u32` field on `ModbusAnalyzer`
- `src/analyzer/modbus.rs` — `DEFAULT_MODBUS_WRITE_THRESHOLD: u32 = 10` constant
- `src/analyzer/modbus.rs` — `ModbusFlowState.window_write_count > self.write_threshold` firing condition
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.2` (write_threshold field) and §5.1 (CLI declaration)

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — (indirect) burst detection threshold flows from CLI to per-flow comparison; covered by integration tests

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §5.1 (`--modbus-write-threshold`, `default_value_t = 10`); architecture-delta.md §2.2 (`write_threshold: u32` field; `window_write_count > write_threshold`); architecture-delta.md §2.3 (window model detail); architecture-delta.md Appendix (`DEFAULT_MODBUS_WRITE_THRESHOLD = 10`) |
| **Confidence** | high |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (flag parsing is clap side effect; threshold value itself is pure data) |
| **Deterministic** | yes — same threshold value always produces same burst-detector behavior |
| **Overall classification** | effectful shell (flag parsing; zero validation emits fatal error) |
