---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-06-12T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-16
capability: CAP-16
lifecycle_status: active
introduced: v0.7.0-feature-arp
modified:
  - "v1.1: Pass-4 remediation F-B4-H02: PC3 storm formula retired ('count_in_window / window_duration_secs') → cross-reference to BC-2.16.008 PC3 / Note 6 formula ('count_in_window / max(1, timestamp_secs - window_start_ts) >= storm_rate'); eliminates divide-by-zero risk from independent restatement. — 2026-06-12"
  - "v1.2: F3 story-anchor back-fill. — 2026-06-14"
  - "v1.3: F4-P1 remediation F-ARP-F4P1-001 (D-074): EC-004 and Precondition 2 resolved — `--arp-storm-rate 0` is REJECTED at CLI parse time with error `--arp-storm-rate must be >= 1 (got 0)`; replaces open 'Clamp to 1 or CLI error — F3 implementation decision' wording. Rationale: ARP storm detection uses inclusive comparison (`count/elapsed >= storm_rate`), so 0 degenerates to always-true; reject-at-CLI matches modbus precedent and latent invariant. Validated by research-agent (arp-threshold-zero-convention.md, HIGH confidence). — 2026-06-15"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/phase-f1-delta-analysis/mitre-arp-additional-detections.md
input-hash: TBD
---

# BC-2.16.013: --arp-storm-rate Overrides ARP_STORM_RATE_DEFAULT

## Description

The `--arp-storm-rate <N>` CLI flag (u32, added to `Commands::Analyze` in `src/cli.rs`)
overrides `ARP_STORM_RATE_DEFAULT = 50` (the wirerust engineering default for the per-source-MAC
frames-per-second threshold that triggers a D3 ARP storm finding). When `--arp-storm-rate N`
is provided, `ArpAnalyzer` is initialized with `storm_rate = N`. When the flag is absent,
`storm_rate = ARP_STORM_RATE_DEFAULT = 50`. OT operators analyzing ICS captures should
lower this value; ICS field devices operate at much lower ARP rates than the 50/s default.
`ARP_STORM_RATE_DEFAULT = 50` is a wirerust engineering default — not derived from any
external standard.

## Preconditions

1. The CLI command includes `--arp --arp-storm-rate <N>` where N is a valid u32.
2. `N >= 1`; passing `0` is REJECTED at CLI parse time with error
   `--arp-storm-rate must be >= 1 (got 0)` before any packet processing.
   Rationale (D-074): the storm comparison is inclusive (`count/elapsed >= storm_rate`),
   so `0` degenerates to always-true — not a coherent "alert-on-all" sentinel.
   Matches modbus precedent; consistent with latent codebase invariant (reject 0 where
   comparison is `>=`). Validated by research-agent arp-threshold-zero-convention.md
   (HIGH confidence).

## Postconditions

1. `ArpAnalyzer` is created with `storm_rate = N` when `--arp-storm-rate N` is provided.
2. `ArpAnalyzer` is created with `storm_rate = ARP_STORM_RATE_DEFAULT = 50` when absent.
3. The `storm_rate` value is used in BC-2.16.008: storm finding emitted when
   `count_in_window / max(1, timestamp_secs - window_start_ts) >= storm_rate`, per
   BC-2.16.008 Postcondition 3 / Note 6. The denominator formula is authoritative
   in BC-2.16.008 and is not restated here to avoid divergence.
4. The flag has no effect when `--arp` is absent.

## Invariants

1. **ARP_STORM_RATE_DEFAULT = 50 is a wirerust engineering default**: 50 frames/sec is a
   conservative round-number starting point. It is not derived from Snort, arpwatch, Zeek, or
   any ICS vendor standard — those tools do not use numeric rate thresholds for ARP storm
   detection (mitre-arp-additional-detections.md §4 CRITICAL CORRECTION). Documented as
   wirerust engineering choice in source.
2. **ICS operator guidance**: the default of 50/s is appropriate for enterprise networks.
   ICS/OT operators with field devices, PLCs, or RTUs should typically use values of 5–20/s.
   This guidance should appear in CLI help text.
3. **No ARP_FLAP_WINDOW_SECS exposure**: the storm window duration (60s) is not a CLI flag
   in v0.7.0. It is shared with the spoof flap window.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--arp-storm-rate 10` | Storm triggers at 10 frames/sec per source MAC |
| EC-002 | `--arp-storm-rate 50` (same as default) | Behavior identical to omitting the flag |
| EC-003 | `--arp-storm-rate 1` | Any MAC sending ≥1 frame/sec triggers storm |
| EC-004 | `--arp-storm-rate 0` | REJECTED at CLI parse time before any packet processing; error: `--arp-storm-rate must be >= 1 (got 0)`; non-zero exit code. Rationale (D-074): storm comparison is inclusive (`count/elapsed >= storm_rate`), so 0 is degenerate always-true; matches modbus precedent and latent codebase invariant. |
| EC-005 | Flag absent | storm_rate = 50 (ARP_STORM_RATE_DEFAULT) |
| EC-006 | `--arp-storm-rate` present but `--arp` absent | Flag accepted by CLI; has no effect |

## Canonical Test Vectors

| Flag | Source MAC sends 30 frames in 1s | Expected storm finding? |
|---|---|---|
| (absent, default=50) | 30/s < 50 threshold | No storm finding |
| `--arp-storm-rate 20` | 30/s >= 20 threshold | Storm Finding emitted |
| `--arp-storm-rate 100` | 30/s < 100 threshold | No storm finding |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none — CLI flag wiring verified by unit/integration tests) | storm_rate controls D3 trigger per BC-2.16.008 | unit tests: ArpAnalyzer::new(spoof=3, storm=10) vs storm=100; same frame sequence; assert different storm trigger point |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — the storm rate flag allows operators to tune D3 detection sensitivity for their specific network; this is critical for ICS environments where ARP rates differ substantially from enterprise networks |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/cli.rs, src/main.rs ArpAnalyzer initialization); ADR-008 Decision 4 |
| Stories | STORY-115 |
| Feature | arp-security-analyzer |
| MITRE Techniques | (none — CLI flag, no finding emission) |

## Related BCs

- BC-2.16.008 — depends on (storm_rate is the threshold used in D3 detection)
- BC-2.16.011 — composes with (--arp must be set for this flag to have effect)
- BC-2.16.012 — related to (both are ArpAnalyzer initialization parameters; passed together)

## Architecture Anchors

- `src/cli.rs` — `Commands::Analyze { #[arg(long, default_value_t = 50)] arp_storm_rate: u32, ... }`
- `src/main.rs` — `let arp_analyzer = ArpAnalyzer::new(args.arp_spoof_threshold, args.arp_storm_rate);`
- `src/analyzer/arp.rs` — `const ARP_STORM_RATE_DEFAULT: u32 = 50` (wirerust engineering default)
- `.factory/specs/architecture/arp-architecture-delta.md §3.2`

## Story Anchor

STORY-115

## VP Anchors

- (none)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 4 (ARP_STORM_RATE_DEFAULT documented as wirerust choice; --arp-storm-rate override); arp-architecture-delta.md §3.2; mitre-arp-additional-detections.md §4b (engineering default; fabricated industry thresholds REJECTED) |
| **Confidence** | high — flag mechanics are straightforward; threshold semantics defined by BC-2.16.008 |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | CLI argument parsing (effectful shell) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (CLI) → configures pure-core ArpAnalyzer |
