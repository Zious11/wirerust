---
document_type: behavioral-contract
level: L3
version: "1.1"
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
  - "v1.1: Pass-5 remediation F-B5-M01: PC3 citation corrected from 'BC-2.16.004 Postcondition 5' (flap-window reset) to 'BC-2.16.004 Postcondition 1 (Step 3 / 1.c — escalation evaluation)'. — 2026-06-12"
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

# BC-2.16.012: --arp-spoof-threshold Overrides SPOOF_REBIND_ESCALATION_DEFAULT

## Description

The `--arp-spoof-threshold <N>` CLI flag (u32, added to `Commands::Analyze` in `src/cli.rs`)
overrides `SPOOF_REBIND_ESCALATION_DEFAULT = 3` (the wirerust engineering default for the
rebind-count threshold that escalates a D1 ARP spoof finding from MEDIUM to HIGH). When
`--arp-spoof-threshold N` is provided, `ArpAnalyzer` is initialized with `spoof_threshold = N`
instead of 3. When the flag is absent, `spoof_threshold = SPOOF_REBIND_ESCALATION_DEFAULT = 3`.
The flag is only meaningful when `--arp` is also active.

## Preconditions

1. The CLI command includes `--arp --arp-spoof-threshold <N>` where N is a valid u32.
2. `N >= 1` (a threshold of 0 would escalate immediately before any rebind; treated as invalid
   or clamped to 1 — F3 implementation detail).

## Postconditions

1. `ArpAnalyzer` is created with `spoof_threshold = N` when `--arp-spoof-threshold N` is provided.
2. `ArpAnalyzer` is created with `spoof_threshold = SPOOF_REBIND_ESCALATION_DEFAULT = 3`
   when `--arp-spoof-threshold` is absent.
3. The `spoof_threshold` value is used in BC-2.16.004 Postcondition 1 (Step 3 / 1.c —
   escalation evaluation): HIGH is emitted when `rebind_count >= spoof_threshold AND
   (timestamp_secs - first_rebind_ts <= ARP_FLAP_WINDOW_SECS) AND !spoof_high_emitted`.
4. The flag has no effect when `--arp` is absent.

## Invariants

1. **SPOOF_REBIND_ESCALATION_DEFAULT = 3 is a wirerust engineering default**: it is the
   hardcoded default used when `--arp-spoof-threshold` is not set. Not derived from any
   external standard (mitre-arp-additional-detections.md §4b). Documented as such in source.
2. **Operator flexibility**: lower values (e.g. `--arp-spoof-threshold 1`) make detection
   more sensitive (any rebind → HIGH); higher values (e.g. `--arp-spoof-threshold 10`) reduce
   HIGH-confidence alerts in environments with frequent NIC replacement or HA failover.
3. **No interaction with ARP_FLAP_WINDOW_SECS**: the flap window (60s) is not exposed as a
   CLI flag in v0.7.0 per arp-architecture-delta.md §3.2 ("Constant only in v0.7.0; expose
   as CLI flag if needed in fast-follow").

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--arp-spoof-threshold 1` | Any single rebind escalates to HIGH immediately |
| EC-002 | `--arp-spoof-threshold 3` (same as default) | Behavior identical to omitting the flag |
| EC-003 | `--arp-spoof-threshold 100` | No HIGH findings unless 100 rebinds in 60s |
| EC-004 | `--arp-spoof-threshold 0` | Clamp to 1 or return CLI error — F3 implementation decision |
| EC-005 | Flag absent | spoof_threshold = 3 (SPOOF_REBIND_ESCALATION_DEFAULT) |
| EC-006 | `--arp-spoof-threshold` present but `--arp` absent | Flag accepted by CLI; has no effect (process_arp is never called) |

## Canonical Test Vectors

| Flag | PCAP: 3 rebinds for 10.0.0.1 in 30s | Expected severity of 3rd rebind finding |
|---|---|---|
| (absent) | threshold=3 | HIGH on 3rd rebind |
| `--arp-spoof-threshold 1` | threshold=1 | HIGH on 1st rebind |
| `--arp-spoof-threshold 5` | threshold=5 | MEDIUM on 3rd rebind (3 < 5 threshold) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none — CLI flag wiring verified by integration tests) | Threshold value controls escalation severity per BC-2.16.004 | unit/integration tests: ArpAnalyzer::new(threshold=1) vs new(threshold=5); assert different escalation point |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — the spoof threshold flag allows operators to tune D1 detection sensitivity for their specific network environment, which is an essential configuration surface for the ARP Security Analysis capability in ICS contexts |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/cli.rs, src/main.rs ArpAnalyzer initialization); ADR-008 Decision 4 |
| Stories | TBD (F3 story decomposition) |
| Feature | arp-security-analyzer |
| MITRE Techniques | (none — CLI flag, no finding emission) |

## Related BCs

- BC-2.16.004 — depends on (spoof_threshold is the escalation threshold used in D1 detection)
- BC-2.16.011 — composes with (--arp must be set for this flag to have effect)

## Architecture Anchors

- `src/cli.rs` — `Commands::Analyze { #[arg(long, default_value_t = 3)] arp_spoof_threshold: u32, ... }`
- `src/main.rs` — `let arp_analyzer = ArpAnalyzer::new(args.arp_spoof_threshold, args.arp_storm_rate);`
- `src/analyzer/arp.rs` — `const SPOOF_REBIND_ESCALATION_DEFAULT: u32 = 3` (wirerust engineering default)
- `.factory/specs/architecture/arp-architecture-delta.md §3.2` — threshold constants and override mechanisms

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- (none)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 4 (SPOOF_REBIND_ESCALATION_DEFAULT documented as wirerust choice; --arp-spoof-threshold override); arp-architecture-delta.md §3.2; mitre-arp-additional-detections.md §4b (engineering default, not industry standard) |
| **Confidence** | high — flag mechanics are straightforward; threshold semantics are defined by BC-2.16.004 |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | CLI argument parsing (effectful shell) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (CLI) → configures pure-core ArpAnalyzer |
