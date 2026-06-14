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
  - "v1.1: F3 story-anchor back-fill. — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md
  - .factory/specs/architecture/arp-architecture-delta.md
input-hash: TBD
---

# BC-2.16.011: --arp CLI Flag Gates ARP Security Analysis

## Description

The `--arp` boolean CLI flag (added to `Commands::Analyze` in `src/cli.rs`) controls whether
`ArpAnalyzer::process_arp` is called on decoded ARP frames. When `--arp` is absent (the
default), ARP frames are decoded into `DecodedFrame::Arp` (decode-vs-analysis separation per
BC-2.16.015) but `process_arp` is NOT called, no ARP findings are emitted, and no ARP summary
is appended. When `--arp` is present, `process_arp` is called for every decoded ARP frame,
findings are emitted, and the summary is appended. This gate mirrors the `--modbus` and
`--dnp3` flag patterns established by SS-14 and SS-15.

## Preconditions

1. The CLI command is `wirerust analyze [options] <pcap>`.
2. The capture file contains at least one ARP frame (EtherType 0x0806, Ethernet/IPv4).
3. etherparse 0.20 successfully decodes the frame as `DecodedFrame::Arp(ArpFrame)`.

## Postconditions

**When `--arp` is absent (default):**
1. `process_arp` is NOT called on any decoded ARP frame.
2. No ARP-related Findings are emitted.
3. No ARP `AnalysisSummary` is appended to `analyzer_summaries`.
4. The ARP frame is silently passed over (same behavior as pre-ARP wirerust for ARP frames,
   except the frame is now decoded rather than erroring).

**When `--arp` is present:**
5. `process_arp` IS called for every `DecodedFrame::Arp(frame)` frame.
6. D1/D2/D3/D11/D12 detections are active.
7. `ArpAnalyzer::summarize()` is called at end of capture and appended to summaries.
8. ARP Findings appear in all report formats (JSON, terminal, CSV) alongside IP-pipeline Findings.

## Invariants

1. **Decode is always performed** (BC-2.16.015): `decode_packet` returns `DecodedFrame::Arp`
   regardless of the `--arp` flag. The flag gates ANALYSIS, not decoding.
2. **Flag wiring pattern**: `--arp` follows the `--modbus` / `--dnp3` precedent:
   - `src/cli.rs`: `#[arg(long)] arp: bool` on `Commands::Analyze`
   - `src/main.rs`: `if args.arp { arp_analyzer.process_arp(&frame, ts); }`
3. **Default-off rationale**: ARP analysis is a new capability. Opt-in default avoids
   unexpected performance overhead or Finding noise for existing users. Consistent with
   Modbus and DNP3 opt-in flags.
4. **No configuration file interaction in v0.7.0**: `--arp` is a CLI flag only. No
   configuration file key in v0.7.0 scope.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | PCAP with 100 ARP frames; `--arp` absent | 0 ARP findings; no ARP summary; all 100 decoded frames silently passed over |
| EC-002 | PCAP with 100 ARP frames; `--arp` present | ARP analysis active; findings emitted per detection BCs; summary appended |
| EC-003 | PCAP with zero ARP frames; `--arp` present | ARP analysis active; no ARP frames to process; empty summary (all zeros per BC-2.16.010 EC-001) |
| EC-004 | PCAP with mixed IP and ARP frames; `--arp` present | ARP frames routed to ArpAnalyzer; IP frames routed to IP pipeline (unchanged); combined findings reported |
| EC-005 | `--arp` combined with `--modbus` | Both analyzers active simultaneously; no interaction; findings from both analyzers appear in output |

## Canonical Test Vectors

| Flags | PCAP content | Expected ARP findings count | Expected ARP summary |
|---|---|---|---|
| (none) | 10 ARP Replies with IP rebinds | 0 | absent |
| `--arp` | 10 ARP Replies with IP rebinds | ≥ 1 spoof finding | present with spoof_findings ≥ 1 |
| `--arp` | Empty PCAP (no frames) | 0 | present with all-zero counts |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none — CLI flag integration verified by integration tests) | --arp absent: no ARP findings; --arp present: findings emitted | integration tests: CLI invocation with and without --arp on known PCAP |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 |
| Capability Anchor Justification | CAP-16 ("ARP Security Analysis") per ARCH-INDEX.md §SS-16 — the --arp flag is the user-facing activation switch for the ARP Security Analysis capability; without it the capability produces no output |
| L2 Domain Invariants | (none directly) |
| Architecture Module | SS-16 (src/cli.rs Commands::Analyze, src/main.rs packet loop); ADR-008 |
| Stories | STORY-113 |
| Feature | arp-security-analyzer |
| MITRE Techniques | (none — CLI flag, no finding emission) |

## Related BCs

- BC-2.16.015 — composes with (decode always occurs; this BC gates analysis only)
- BC-2.16.012 — composes with (--arp-spoof-threshold is only meaningful when --arp is set)
- BC-2.16.013 — composes with (--arp-storm-rate is only meaningful when --arp is set)

## Architecture Anchors

- `src/cli.rs` — `Commands::Analyze { #[arg(long)] arp: bool, ... }` — flag declaration
- `src/main.rs` — `if args.arp { arp_analyzer.process_arp(&frame, timestamp_secs); }` in DecodedFrame::Arp arm
- `.factory/specs/architecture/decisions/ADR-008-arp-link-layer-integration.md §Decision 1` — main.rs pattern-match; Decision 4 — ArpAnalyzer created once per run_analyze() call
- `.factory/specs/architecture/arp-architecture-delta.md §4.1`

## Story Anchor

STORY-113

## VP Anchors

- (none)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-008 Decision 1 (main.rs pattern-match and ARP routing); arp-architecture-delta.md §4.1 (--arp flag in regression surface) |
| **Confidence** | high — flag pattern mirrors existing --modbus/--dnp3 with no novelty |
| **Extraction Date** | 2026-06-12 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | CLI flag read (effectful shell); routing decision (pure logic) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (CLI flag parsing in cli.rs/main.rs) |
