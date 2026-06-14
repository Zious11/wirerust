---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/cli.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: FIX-P5-002 / ADV-IMPL-P04-MED-001 — --reassembly-depth and --reassembly-memcap now require >= 1; 0 rejected at parse time via parse_nonzero_usize value_parser (exit 2, ValueValidation). Corrects implicit 0-accepted assumption. Test citations corrected to match committed names (DF-AC-TEST-NAME-SYNC-001). 2026-06-01"
  - "v1.4: DF-SIBLING-SWEEP-001 — fix stale cli.rs line anchor: reassembly flags range 61-106 → 71-122 (additional flags --overlap-threshold, --small-segment-*, --out-of-window-threshold, --flow-timeout added between old line 67 and Commands; new range is 71 to 122); verified against HEAD cfe0112a — 2026-06-01"
  - "v1.5: P20 B-04+B-05 fix: (B-04) Invariant 4 main.rs:104-117 → 147-161 (CLI override application block); Architecture Anchor main.rs:87-122 → 139-166 (full ReassemblyConfig construction+overrides+TcpReassembler::new block). (B-05) Architecture Anchor + Source Evidence cli.rs:71-122 → 73-124 (line 71 is --csv tail; --reassemble block starts at 73, ends at flow_timeout field at 124). All verified against current src. — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.005: Reassembly CLI Flags: --reassemble/--no-reassemble, depth, memcap, and five anomaly-threshold flags

## Description

Four global flags control TCP reassembly configuration: `--reassemble` (force on),
`--no-reassemble` (force off), `--reassembly-depth <MB>` (per-direction stream depth,
default 10, must be >= 1), and `--reassembly-memcap <MB>` (global memory cap, default 1024,
must be >= 1). Additionally, five anomaly-threshold override flags are available:
`--overlap-threshold`, `--small-segment-threshold`, `--small-segment-max-bytes`,
`--small-segment-ignore-ports`, and `--out-of-window-threshold`.
All default to their `ReassemblyConfig::default()` values when absent.

`--reassembly-depth` and `--reassembly-memcap` use a `parse_nonzero_usize` custom clap
`value_parser` that enforces >= 1 at parse time. Supplying 0 produces a clap
`ValueValidation` error (exit code 2, message `0 is not in 1..`) before any analysis runs.
This prevents 0 from reaching `TcpReassembler::new`'s defensive `assert!(max_depth > 0)` /
`assert!(memcap > 0)` and causing a panic (see NFR-REL-004, E-RAS-004).

## Preconditions

1. `Cli::parse()` or `Cli::try_parse_from()` is called with reassembly-related flags.

## Postconditions

1. `cli.reassemble = true` when `--reassemble` present; `false` otherwise.
2. `cli.no_reassemble = true` when `--no-reassemble` present; `false` otherwise.
3. `cli.reassembly_depth = 10` when `--reassembly-depth` absent (default).
4. `cli.reassembly_memcap = 1024` when `--reassembly-memcap` absent (default).
5. `--reassembly-depth <N>` with N >= 1 sets `cli.reassembly_depth = N`; N = 0 is
   rejected at parse time (clap `ValueValidation`, exit 2) before any analysis begins.
6. `--reassembly-memcap <N>` with N >= 1 sets `cli.reassembly_memcap = N`; N = 0 is
   rejected at parse time (clap `ValueValidation`, exit 2) before any analysis begins.
7. Threshold overrides are `None` when absent; `Some(value)` when present.
8. `--overlap-threshold` accepts values 0-255 (u32 range-checked by clap).
9. `--small-segment-threshold` accepts values 0-2048.
10. `--small-segment-max-bytes` accepts values 0-2048 (u16).

## Invariants

1. All reassembly flags are global (`global = true`).
2. `--reassemble` and `--no-reassemble` are mutually exclusive (BC-2.12.007).
3. `--small-segment-ignore-ports` is `Option<Vec<u16>>` with `value_delimiter = ','`.
4. When threshold overrides are `Some`, main.rs applies them to `ReassemblyConfig` after
   construction (main.rs:147-161).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No reassembly flags | depth=10, memcap=1024, all thresholds=None |
| EC-002 | --reassembly-depth 20 | reassembly_depth=20 |
| EC-003 | --overlap-threshold 256 | Clap error (out of 0-255 range) |
| EC-004 | --small-segment-ignore-ports 23,513 | small_segment_ignore_ports=Some([23, 513]) |
| EC-005 | --small-segment-max-bytes 0 | small_segment_max_bytes=Some(0) (disables detection; 0 is valid for this flag) |
| EC-006 | --reassembly-depth 0 | Rejected at parse time: clap ValueValidation error (exit 2), message "0 is not in 1.."; analysis never starts. Enforced by parse_nonzero_usize value_parser. |
| EC-007 | --reassembly-memcap 0 | Rejected at parse time: clap ValueValidation error (exit 2), message "0 is not in 1.."; analysis never starts. Enforced by parse_nonzero_usize value_parser. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ["wirerust", "--reassembly-depth", "20", "analyze", "x.pcap"] | reassembly_depth=20 | happy-path |
| ["wirerust", "analyze", "x.pcap"] | depth=10, memcap=1024 | happy-path |
| ["wirerust", "--overlap-threshold", "0", "analyze", "x.pcap"] | overlap_threshold=Some(0) | happy-path |
| ["wirerust", "--overlap-threshold", "256", "analyze", "x.pcap"] | Clap range error (exit 2) | error |
| ["wirerust", "--reassembly-depth", "0", "analyze", "x.pcap"] | Clap ValueValidation error (exit 2): "0 is not in 1.." | error (EC-006) |
| ["wirerust", "--reassembly-memcap", "0", "analyze", "x.pcap"] | Clap ValueValidation error (exit 2): "0 is not in 1.." | error (EC-007) |
| ["wirerust", "--reassembly-depth", "1", "analyze", "x.pcap"] | reassembly_depth=1 (minimum valid) | happy-path |
| ["wirerust", "--reassembly-memcap", "1", "analyze", "x.pcap"] | reassembly_memcap=1 (minimum valid) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Default depth=10, memcap=1024 | unit: test_reassembly_flags |
| — | --no-reassemble flag sets no_reassemble=true | unit: test_no_reassemble_flag |
| — | --reassembly-depth 0 rejected (exit 2, ValueValidation) | unit: test_EC_001_reassembly_depth_zero_rejected; integration: test_analyze_reassembly_depth_zero_exits_usage_error |
| — | --reassembly-memcap 0 rejected (exit 2, ValueValidation) | unit: test_EC_001_reassembly_memcap_zero_rejected; integration: test_analyze_reassembly_memcap_zero_exits_usage_error |
| — | --reassembly-depth 1 accepted (minimum valid) | unit: test_reassembly_depth_minimum_valid |
| — | --reassembly-memcap 1 accepted (minimum valid) | unit: test_reassembly_memcap_minimum_valid |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md -- the reassembly control flags (--reassemble, --no-reassemble, --reassembly-depth, etc.) are declared on the Cli struct and consumed by main.rs to wire the ReassemblyConfig; this is the entry-point orchestration concern that CAP-12 owns, not CAP-04's stream reassembly logic |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (cli.rs, C-3) |
| Stories | STORY-087 |
| Origin BC | BC-CLI-005 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.12.007 -- composes with (mutually exclusive constraint on reassemble/no-reassemble)
- BC-2.12.009 -- composes with (needs_reassembly logic uses these flags)

## Architecture Anchors

- `src/cli.rs:73-124` -- reassembly flags on Cli struct; `parse_nonzero_usize` value_parser enforces depth >= 1 and memcap >= 1
- `src/main.rs:139-166` -- ReassemblyConfig construction (struct 140-144), CLI override application (147-161), flow_timeout_secs wire (165), TcpReassembler::new (166)
- `src/reassembly/mod.rs:115-125` -- TcpReassembler::new defensive asserts (backstop; parse_nonzero_usize prevents 0 from reaching here in production)
- `tests/cli_story_087_tests.rs` -- test_reassembly_flags, test_no_reassemble_flag, test_EC_001_reassembly_depth_zero_rejected, test_EC_001_reassembly_memcap_zero_rejected, test_analyze_reassembly_depth_zero_exits_usage_error, test_analyze_reassembly_memcap_zero_exits_usage_error

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs:73-124` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: clap range validators and default_value_t attributes; parse_nonzero_usize value_parser enforcing depth >= 1 and memcap >= 1 (FIX-P5-002)
- **assertion**: test_reassembly_flags, test_no_reassemble_flag, test_EC_001_reassembly_depth_zero_rejected, test_EC_001_reassembly_memcap_zero_rejected, test_analyze_reassembly_depth_zero_exits_usage_error, test_analyze_reassembly_memcap_zero_exits_usage_error

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
