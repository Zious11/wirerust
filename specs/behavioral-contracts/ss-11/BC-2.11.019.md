---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reporter/terminal.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
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

# BC-2.11.019: TerminalReporter Renders Sections in Correct Order

## Description

`TerminalReporter::render` produces structural output in a defined section order: (1) WIRERUST
TRIAGE REPORT header with Packets/Bytes/Hosts line and optional Skipped warning, (2) optional
HOSTS breakdown (when `show_hosts_breakdown = true`), (3) PROTOCOLS breakdown, (4) SERVICES
breakdown (when non-empty), (5) FINDINGS section (when non-empty), (6) one ANALYZER: <name>
section per `AnalysisSummary`. This order is documented in the module and verified by test.

## Preconditions

1. `TerminalReporter::render` is called with valid Summary, findings, and analyzer_summaries.
2. No specific constraints on content of any input.

## Postconditions

1. The WIRERUST TRIAGE REPORT header section appears first.
2. PROTOCOLS section follows the header.
3. SERVICES section appears after PROTOCOLS, only when `summary.service_counts()` is non-empty.
4. FINDINGS section appears after SERVICES (or PROTOCOLS if SERVICES absent), only when
   `findings` is non-empty.
5. ANALYZER: <name> sections appear last, one per element of `analyzer_summaries`, in slice
   order.
6. The optional HOSTS section (when `show_hosts_breakdown = true` and hosts non-empty) appears
   immediately after the header, before PROTOCOLS.

## Invariants

1. Sections are emitted by sequential pushes to a `String`; the section order is the
   code order in `render()`.
2. FINDINGS section is entirely absent (not just empty) when `findings.is_empty()`.
3. SERVICES section is entirely absent when `service_counts()` returns an empty map.
4. If no analyzer summaries are given, no ANALYZER sections are rendered.
5. The PROTOCOLS section is ALWAYS rendered even when the protocol map is empty (the section
   header appears; the body may be empty).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No findings, no analyzer summaries | Header + PROTOCOLS only |
| EC-002 | No services | SERVICES section absent |
| EC-003 | show_hosts_breakdown=true with hosts | HOSTS section between header and PROTOCOLS |
| EC-004 | show_hosts_breakdown=false | No HOSTS section regardless of host count |
| EC-005 | Multiple analyzer summaries | Each ANALYZER: section in slice order |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Summary + findings + 1 analyzer | Header, PROTOCOLS, FINDINGS, ANALYZER:DNS in order | happy-path |
| No findings, no services | Header, PROTOCOLS only | edge-case |
| show_hosts_breakdown=true + 2 hosts | Header, HOSTS, PROTOCOLS | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Section order: header before PROTOCOLS before FINDINGS before ANALYZER | unit: reporter tests verifying section presence order (MEDIUM -- order not strictly positional-asserted) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md -- the section order of the terminal report is a documented output structure contract that downstream grep-based pipelines depend on |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-11 (reporter/terminal.rs, C-20) |
| Stories | STORY-078 |
| Origin BC | BC-RPT-019 (pass-3 ingestion corpus, MEDIUM confidence -- section presence covered; strict positional order not directly asserted in a single test) |

## Related BCs

- BC-2.11.006 -- composes with (skipped-packets line appears within the header section)
- BC-2.11.013 -- composes with (FINDINGS section content when show_mitre_grouping=true)

## Architecture Anchors

- `src/reporter/terminal.rs:83-178` -- TerminalReporter::render full body
- `src/reporter/terminal.rs:113` -- HOSTS conditional block
- `src/reporter/terminal.rs:125` -- PROTOCOLS section
- `src/reporter/terminal.rs:133` -- SERVICES conditional block (`if !services.is_empty()`)
- `src/reporter/terminal.rs:142` -- FINDINGS conditional block (`if !findings.is_empty()`)
- `src/reporter/terminal.rs:158` -- ANALYZER: sections loop

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:83-178` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: code structure at lines 83-178 clearly defines the section order
- **inferred**: reporter_tests verify section PRESENCE but do not assert positional order

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

To upgrade to HIGH: add a test that asserts the relative byte positions of section headers
(e.g., `PROTOCOLS` appears before `FINDINGS` by index in the output string). Currently only
presence is verified, not order.
