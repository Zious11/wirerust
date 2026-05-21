---
document_type: behavioral-contract
level: L3
version: "1.2"
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
default 10), and `--reassembly-memcap <MB>` (global memory cap, default 1024). Additionally,
five anomaly-threshold override flags are available: `--overlap-threshold`, `--small-segment-threshold`,
`--small-segment-max-bytes`, `--small-segment-ignore-ports`, and `--out-of-window-threshold`.
All default to their `ReassemblyConfig::default()` values when absent.

## Preconditions

1. `Cli::parse()` or `Cli::try_parse_from()` is called with reassembly-related flags.

## Postconditions

1. `cli.reassemble = true` when `--reassemble` present; `false` otherwise.
2. `cli.no_reassemble = true` when `--no-reassemble` present; `false` otherwise.
3. `cli.reassembly_depth = 10` when `--reassembly-depth` absent (default).
4. `cli.reassembly_memcap = 1024` when `--reassembly-memcap` absent (default).
5. Threshold overrides are `None` when absent; `Some(value)` when present.
6. `--overlap-threshold` accepts values 0-255 (u32 range-checked by clap).
7. `--small-segment-threshold` accepts values 0-2048.
8. `--small-segment-max-bytes` accepts values 0-2048 (u16).

## Invariants

1. All reassembly flags are global (`global = true`).
2. `--reassemble` and `--no-reassemble` are mutually exclusive (BC-2.12.007).
3. `--small-segment-ignore-ports` is `Option<Vec<u16>>` with `value_delimiter = ','`.
4. When threshold overrides are `Some`, main.rs applies them to `ReassemblyConfig` after
   construction (main.rs:104-117).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No reassembly flags | depth=10, memcap=1024, all thresholds=None |
| EC-002 | --reassembly-depth 20 | reassembly_depth=20 |
| EC-003 | --overlap-threshold 256 | Clap error (out of 0-255 range) |
| EC-004 | --small-segment-ignore-ports 23,513 | small_segment_ignore_ports=Some([23, 513]) |
| EC-005 | --small-segment-max-bytes 0 | small_segment_max_bytes=Some(0) (disables detection) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ["wirerust", "--reassembly-depth", "20", "analyze", "x.pcap"] | reassembly_depth=20 | happy-path |
| ["wirerust", "analyze", "x.pcap"] | depth=10, memcap=1024 | happy-path |
| ["wirerust", "--overlap-threshold", "0", "analyze", "x.pcap"] | overlap_threshold=Some(0) | happy-path |
| ["wirerust", "--overlap-threshold", "256", "analyze", "x.pcap"] | Clap range error | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Default depth=10, memcap=1024 | unit: test_reassembly_flags |
| — | --no-reassemble flag sets no_reassemble=true | unit: test_no_reassemble_flag |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- the reassembly control flags (--reassemble, --no-reassemble, --reassembly-depth, etc.) are declared on the Cli struct and consumed by main.rs to wire the ReassemblyConfig; this is the entry-point orchestration concern that CAP-12 owns, not CAP-04's stream reassembly logic |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (cli.rs, C-3) |
| Stories | S-TBD |
| Origin BC | BC-CLI-005 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.12.007 -- composes with (mutually exclusive constraint on reassemble/no-reassemble)
- BC-2.12.009 -- composes with (needs_reassembly logic uses these flags)

## Architecture Anchors

- `src/cli.rs:61-106` -- reassembly flags on Cli struct
- `src/main.rs:87-122` -- reassembly configuration applied in run_analyze
- `tests/cli_tests.rs` -- test_reassembly_flags, test_no_reassemble_flag

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/cli.rs:61-106` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: clap range validators and default_value_t attributes
- **assertion**: test_reassembly_flags, test_no_reassemble_flag

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
