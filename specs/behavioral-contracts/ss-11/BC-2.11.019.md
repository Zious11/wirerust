---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - "v1.3: FIX-P5-003 / ADV-IMPL-P06-MED-001 — add postconditions 7-8 for deterministic PROTOCOLS/SERVICES body ordering (count desc, name asc); replace qualitative Deterministic claim with mechanistic one; add EC-006/EC-007; add VP/anchor for test_terminal_protocols_sorted_count_then_name and test_terminal_services_sorted_count_then_name — 2026-06-01"
  - "v1.4: DF-SIBLING-SWEEP-001 — fix stale terminal.rs line anchors: SERVICES conditional :133 → :138, svc_vec sort :140-141 → :141, FINDINGS conditional :142 → :149, ANALYZER loop :158 → :165; full body range :83-178 → :83-186; path row updated to :83-186; verified against HEAD cfe0112a — 2026-06-01"
  - "v1.5: issue-#259 F2 integrate (v0.8.0 collapse feature) — add Postcondition 9 and Invariant 7 and EC-008/EC-009 for FINDINGS dispatch collapse interaction: when collapse_findings=true (default in v0.8.0), the flat-mode FINDINGS body routes through the collapse pass (BC-2.11.025) before calling render_finding_flat per group; when collapse_findings=false (--no-collapse) or show_mitre_grouping=true, the FINDINGS body is unchanged from pre-v0.8.0. Section presence/ordering (postconditions 1-8) is unchanged. Cross-references BC-2.11.025/026/027/028/029. ADR-0003 (display-layer aggregation subsection) cited. — 2026-06-17"
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
7. The body lines of the PROTOCOLS section are rendered in count-descending order; ties among
   protocols are broken by the protocol's Debug representation string ascending
   (lexicographic). The section body order is deterministic across all runs regardless of the
   underlying HashMap iteration order of `protocol_counts()`. Sort key:
   `b.count.cmp(a.count).then_with(|| format!("{:?}", a.proto).cmp(&format!("{:?}", b.proto)))`.
8. The body lines of the SERVICES section are rendered in count-descending order; ties among
   service names are broken by service name string ascending (lexicographic). The section body
   order is deterministic across all runs regardless of the underlying HashMap iteration order
   of `service_counts()`. Sort key:
   `b.count.cmp(a.count).then_with(|| a.name.cmp(b.name))`.
9. **v0.8.0 flat-mode FINDINGS dispatch (BC-2.11.025):** When `collapse_findings = true`
   (the v0.8.0 default) AND `show_mitre_grouping = false`, the FINDINGS section body routes
   through the collapse pass before rendering. The collapse pass groups the `findings` slice
   by `(category, verdict, confidence, summary)` key (BC-2.11.025) and renders one display
   group per unique key with a ` (xN)` count suffix for N≥2 (BC-2.11.026) and at most K=3
   evidence lines per group (BC-2.11.027). When `collapse_findings = false` (`--no-collapse`)
   OR when `show_mitre_grouping = true`, the FINDINGS section body is rendered identically to
   the pre-v0.8.0 behavior. Section presence/ordering (postconditions 1-8) is unchanged in
   all cases.

## Invariants

1. Sections are emitted by sequential pushes to a `String`; the section order is the
   code order in `render()`.
2. FINDINGS section is entirely absent (not just empty) when `findings.is_empty()`.
3. SERVICES section is entirely absent when `service_counts()` returns an empty map.
4. If no analyzer summaries are given, no ANALYZER sections are rendered.
5. The PROTOCOLS section is ALWAYS rendered even when the protocol map is empty (the section
   header appears; the body may be empty).
6. The within-section body order for PROTOCOLS and SERVICES is determined by an explicit sort
   (not HashMap iteration order); the output is therefore fully reproducible given the same
   input regardless of Rust runtime HashMap randomization.
7. **v0.8.0 collapse routing (BC-2.11.025):** The flat-mode FINDINGS dispatch at
   `terminal.rs:149-160` (the `else` branch of `if self.show_mitre_grouping`) is extended in
   v0.8.0 to check `self.collapse_findings`. When true, it invokes the collapse pass to
   produce collapsed groups and renders one display group per unique key. When false, it
   iterates findings as before. The section header and the enclosing `if !findings.is_empty()`
   guard (postcondition 4 / Invariant 2) are unchanged.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No findings, no analyzer summaries | Header + PROTOCOLS only |
| EC-002 | No services | SERVICES section absent |
| EC-003 | show_hosts_breakdown=true with hosts | HOSTS section between header and PROTOCOLS |
| EC-004 | show_hosts_breakdown=false | No HOSTS section regardless of host count |
| EC-005 | Multiple analyzer summaries | Each ANALYZER: section in slice order |
| EC-006 | Multiple protocols with equal counts | Within the tied group, protocols appear in ascending order of Debug string (e.g., "Icmp" < "Other(10)" < "Tcp"); result is deterministic regardless of HashMap internal order |
| EC-007 | Multiple services with equal counts | Within the tied group, services appear in ascending alphabetical order by service name string; result is deterministic regardless of HashMap internal order |
| EC-008 | collapse_findings=true (default), findings non-empty, show_mitre_grouping=false | FINDINGS section body rendered as collapsed groups per BC-2.11.025/026/027; section header and section presence unchanged; section order unchanged |
| EC-009 | collapse_findings=true, show_mitre_grouping=true | FINDINGS section body rendered via grouped path (render_findings_grouped); collapse pass NOT applied (BC-2.11.013 Invariant 4); section order unchanged |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Summary + findings + 1 analyzer | Header, PROTOCOLS, FINDINGS, ANALYZER:DNS in order | happy-path |
| No findings, no services | Header, PROTOCOLS only | edge-case |
| show_hosts_breakdown=true + 2 hosts | Header, HOSTS, PROTOCOLS | edge-case |
| 9 distinct protocols (Tcp=20, Udp=10, 5x Other(N)=5 each inserted in reverse order, Icmp=2, Other(255)=1) | PROTOCOLS body: Tcp, Udp, Other(10), Other(20), Other(30), Other(40), Other(50), Icmp, Other(255) — count desc then Debug-string asc for ties | tiebreaker / EC-006 |
| 7 services (TLS=30, SMB=10, DNS/HTTP/Modbus/SSH=5 each inserted in reverse order, DNP3=1) | SERVICES body: TLS, SMB, DNS, HTTP, Modbus, SSH, DNP3 — count desc then name asc for ties | tiebreaker / EC-007 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Section order: header before PROTOCOLS before FINDINGS before ANALYZER | unit: reporter tests verifying section presence order (MEDIUM -- order not strictly positional-asserted) |
| — | PROTOCOLS body lines sorted count desc then Protocol Debug-string asc; deterministic under reverse-insertion | unit: test_terminal_protocols_sorted_count_then_name (postcondition 7 / invariant 6 / EC-006) (FIX-P5-003) |
| — | SERVICES body lines sorted count desc then service name asc; deterministic under reverse-insertion | unit: test_terminal_services_sorted_count_then_name (postcondition 8 / invariant 6 / EC-007) (FIX-P5-003) |

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
- BC-2.11.025 -- composes with (v0.8.0 collapse: flat FINDINGS dispatch routes through collapse pass when collapse_findings=true)
- BC-2.11.026 -- composes with (count suffix rendering for collapsed groups within the FINDINGS body)
- BC-2.11.027 -- composes with (evidence sampling within collapsed groups in the FINDINGS body)
- BC-2.11.028 -- depends on (--no-collapse flag controls collapse_findings field; when false, FINDINGS body unchanged from pre-v0.8.0)
- BC-2.11.029 -- composes with (JSON/CSV reporters unaffected by collapse pass; only the flat-mode FINDINGS body of the terminal reporter changes)

## Architecture Anchors

- `src/reporter/terminal.rs:83-186` -- TerminalReporter::render full body
- `src/reporter/terminal.rs:113` -- HOSTS conditional block
- `src/reporter/terminal.rs:125` -- PROTOCOLS section
- `src/reporter/terminal.rs:127-130` -- proto_vec sort: `sort_by(|a, b| b.1.cmp(a.1).then_with(|| format!("{:?}", a.0).cmp(&format!("{:?}", b.0))))` (FIX-P5-003)
- `src/reporter/terminal.rs:138` -- SERVICES conditional block (`if !services.is_empty()`)
- `src/reporter/terminal.rs:141` -- svc_vec sort: `sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)))` (FIX-P5-003)
- `src/reporter/terminal.rs:149` -- FINDINGS conditional block (`if !findings.is_empty()`)
- `src/reporter/terminal.rs:165` -- ANALYZER: sections loop
- `tests/reporter_terminal_tests.rs::test_terminal_protocols_sorted_count_then_name` -- covers postcondition 7 / invariant 6 / EC-006 (FIX-P5-003)
- `tests/reporter_terminal_tests.rs::test_terminal_services_sorted_count_then_name` -- covers postcondition 8 / invariant 6 / EC-007 (FIX-P5-003)

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:83-186` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: code structure at lines 83-178 clearly defines the section order
- **inferred**: reporter_tests verify section PRESENCE but do not assert positional order
- **assertion**: test_terminal_protocols_sorted_count_then_name (FIX-P5-003)
- **assertion**: test_terminal_services_sorted_count_then_name (FIX-P5-003)

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — PROTOCOLS and SERVICES body lines are explicitly sorted (count desc, name/Debug-key asc) before rendering; HashMap iteration order of protocol_counts()/service_counts() has no effect on output (FIX-P5-003). Previously this claim was inaccurate: the body order was HashMap-iteration order and therefore non-deterministic. |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

To upgrade section-order confidence to HIGH: add a test that asserts the relative byte
positions of section headers (e.g., `PROTOCOLS` appears before `FINDINGS` by index in the
output string). Currently only presence is verified, not order.

Within-section body ordering (postconditions 7-8) is now HIGH-confidence: covered by explicit
sort tests `test_terminal_protocols_sorted_count_then_name` and
`test_terminal_services_sorted_count_then_name` (FIX-P5-003).
