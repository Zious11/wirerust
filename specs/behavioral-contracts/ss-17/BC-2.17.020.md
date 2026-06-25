---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.020: CLI --enip Flag Enables Analyzer; --enip-write-burst-threshold and --enip-error-burst-threshold Configure Detection

## Description

The CLI `Commands::Analyze` gains three new flags for EtherNet/IP analysis: `--enip` (boolean,
default-off, included by `--all`) enables the `EnipAnalyzer`; `--enip-write-burst-threshold`
(u32, default 50) sets the write-burst detection threshold for T0836 (BC-2.17.012);
`--enip-error-burst-threshold` (u32, default 5) sets the error-burst detection threshold for
T0888 Pattern B (BC-2.17.014, configured via BC-2.17.026). When `--enip` is set without TCP
reassembly (`--tcp-reassembly` / `--all`), the analyzer emits a WARNING and disables ENIP
silently — mirroring the `--modbus` and `--dnp3` pattern. The `EnipAnalyzer` is included in
the `needs_reassembly` check.

## Preconditions

1. `wirerust analyze [pcap-file] --enip [--enip-write-burst-threshold N] [--enip-error-burst-threshold M]` is invoked.
2. OR `wirerust analyze [pcap-file] --all` (includes --enip by default).

## Postconditions

1. When `--enip` is set:
   - `EnipAnalyzer` is constructed and wired to the `StreamDispatcher`.
   - `EnipAnalyzer.enip_write_burst_threshold = args.enip_write_burst_threshold` (default 50).
   - `EnipAnalyzer.enip_error_burst_threshold = args.enip_error_burst_threshold` (default 5).
   - `needs_reassembly.push(EnipAnalyzer)` (TCP reassembly required).
2. When `--enip` is set AND TCP reassembly is not enabled:
   - A WARNING is emitted: `"--enip requires TCP reassembly; ENIP analysis disabled"`.
   - `EnipAnalyzer` is NOT constructed; no ENIP findings are emitted.
3. When `--enip` is NOT set (and `--all` is not passed): no ENIP analysis; no ENIP findings.
4. At end of analysis: `dispatcher.take_enip_analyzer()` collects findings and summary.

## Invariants

1. **Default-off**: `--enip` is disabled by default. Enabling requires explicit opt-in or
   `--all`.
2. **Reassembly dependency**: ENIP analysis requires TCP reassembly (same as Modbus and DNP3).
   The WARNING-and-disable pattern prevents silent empty results when reassembly is missing.
3. **Threshold validation**: `--enip-write-burst-threshold` (u32, default 50) and
   `--enip-error-burst-threshold` (u32, default 5) each accept 0..u32::MAX. A value of 0
   triggers detection on the first event (write or error, respectively) in the window.
   (OA-001 RESOLVED=50; OA-005 default=5; F2 gate confirmation pending for both)
4. **`--all` includes `--enip`**: the `--all` flag expansion must include `--enip` in the
   same set as `--modbus`, `--dnp3`, etc.
5. **`take_enip_analyzer()`**: mirrors `take_dnp3_analyzer()` on StreamDispatcher; transfers
   ownership of findings + summary to the main reporter.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--enip` without `--reassembly` | WARNING emitted; no ENIP analysis; run completes |
| EC-002 | `--enip --enip-write-burst-threshold 50` | write-burst threshold=50 used for T0836 detection |
| EC-003 | `--all` (includes --enip) with reassembly | Full ENIP analysis enabled; write-burst threshold=50, error-burst threshold=5 |
| EC-004 | No `--enip` flag | No ENIP analyzer constructed; no port-44818 routing |
| EC-005 | `--enip-write-burst-threshold 0` | 0 would trigger on first write; (OA-001 RESOLVED=50; F2 gate confirmation pending) |
| EC-006 | `--enip --enip-error-burst-threshold 10` | error-burst threshold=10 used for T0888 Pattern B detection |
| EC-007 | `--enip-error-burst-threshold 0` | 0 would trigger on first error; (OA-005 default=5; F2 gate confirmation pending) |

## Canonical Test Vectors

| CLI invocation | ENIP enabled? | Write-burst threshold | Error-burst threshold |
|----------------|--------------|----------------------|----------------------|
| `analyze pcap.pcap --enip` (with reassembly) | Yes | 50 (default) | 5 (default) |
| `analyze pcap.pcap --enip --enip-write-burst-threshold 50` | Yes | 50 | 5 (default) |
| `analyze pcap.pcap --enip --enip-error-burst-threshold 10` | Yes | 50 (default) | 10 |
| `analyze pcap.pcap --all` (with reassembly) | Yes | 50 | 5 |
| `analyze pcap.pcap --enip` (without reassembly) | No (WARNING) | — | — |
| `analyze pcap.pcap` | No | — | — |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | CLI flag parsing, warning emission, reassembly guard: integration test | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — the CLI flags are the user-facing control surface for enabling EtherNet/IP analysis; without `--enip`, no ENIP traffic is analyzed regardless of pcap content |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-12 (cli.rs, main.rs), SS-17 (analyzer/enip.rs); ADR-010 Decision 9 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — CLI wiring; no finding emission) |

## Related BCs

- BC-2.17.012 — depends on (enip_write_burst_threshold value used for T0836 threshold)
- BC-2.17.014 — depends on (enip_error_burst_threshold value used for T0888 Pattern B threshold)
- BC-2.17.019 — depends on (StreamDispatcher receives EnipAnalyzer only when --enip is set)
- BC-2.17.026 — composes with (--enip-error-burst-threshold is one of the ENIP CLI flags documented here)

## Architecture Anchors

- `src/cli.rs` — `Commands::Analyze { enip: bool, enip_write_burst_threshold: u32, enip_error_burst_threshold: u32 }` — new fields
- `src/main.rs` — ENIP analyzer construction, needs_reassembly push, take_enip_analyzer() call
- `src/dispatcher.rs` — `enip_analyzer()` and `take_enip_analyzer()` accessors
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 9`

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(none — CLI wiring; integration test)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 9 (CLI wiring pattern mirrors ADR-007 Decision 6); architecture-delta.md §6 (existing file touch list for cli.rs and main.rs) |
| **Confidence** | high — mirrors established Modbus and DNP3 CLI patterns |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads CLI args; emits WARNING to stderr when reassembly missing |
| **Global state access** | constructs and wires EnipAnalyzer to dispatcher |
| **Deterministic** | yes — same flags produce same analyzer configuration |
| **Thread safety** | single-threaded (main setup) |
| **Overall classification** | effectful shell (CLI wiring) |
