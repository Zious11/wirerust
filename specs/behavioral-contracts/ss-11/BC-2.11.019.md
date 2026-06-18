---
document_type: behavioral-contract
level: L3
version: "1.10"
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
  - "v1.6 2026-06-17: F2 adversarial pass-2 — fix dispatch block anchor terminal.rs:149-160→149-162 in Invariant 7 (F-A05)"
  - "v1.7 2026-06-17: issue-#62 F2 BC re-anchor — replace show_mitre_grouping/collapse_findings bool references with FindingsRender enum: Postcondition 9 and Invariant 7 and EC-008/EC-009 updated to use FindingsRender variant names. Rationale: illegal-state elimination (enum makes grouping && collapse unrepresentable). No behavioral change."
  - "v1.8 2026-06-18: F3 adversarial round-4 finding 2 (MEDIUM) stale dispatch anchor — Invariant 7 cited FINDINGS dispatch at terminal.rs:149-162, but line 149 is the HOSTS section (if self.show_hosts_breakdown). Verified against src/reporter/terminal.rs: actual FINDINGS dispatch if-chain is at lines 185-207 (if !findings.is_empty() block through closing brace). Re-anchored Invariant 7 to correct range 185-207."
  - "v1.9 2026-06-18: F5 post-merge re-anchor to develop a4263c7 (terminal.rs line-anchor drift fix; no normative change) — full render() body :83-186 → :129-250; HOSTS :113 → :164; PROTOCOLS :125 → :176; proto_vec sort :127-130 → :178-181; SERVICES conditional :138 → :189; svc_vec sort :141 → :192; FINDINGS dispatch :149 → :200; ANALYZER loop :165 → :229; Invariant 7 dispatch range :185-207 → :200-226; Source Evidence path updated."
  - "v1.10 2026-06-18: STORY-119 vocabulary migration — D-110 struct form: FindingsRender enum variants → struct form in PC-9, Invariant 7, EC-008/EC-009, Related BCs. PC-9 updated to reflect four-arm dispatch (match on grouping+collapse tuple). No behavioral change."
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
9. **FINDINGS dispatch (BC-2.11.025 / BC-2.11.031):** The FINDINGS section body routing depends on
   `render.grouping` and `render.collapse` (four-arm dispatch since STORY-119):
   - `{Flat, Collapsed}` (default): global flat collapse pass + ` (xN)` suffix per BC-2.11.025/026/027.
   - `{Flat, Expanded}` (`--no-collapse`, no `--mitre`): one line per finding, no collapse, pre-v0.8.0 behavior.
   - `{Grouped, Collapsed}` (`--mitre` alone, new STORY-119 default): grouped path with per-bucket
     collapse; tactic bucket headers; ` (xN)` suffix per bucket per BC-2.11.031.
   - `{Grouped, Expanded}` (`--mitre --no-collapse`): grouped path, no collapse; one finding per
     `render_finding_grouped` call; pre-STORY-119 `--mitre` behavior.
   Section presence/ordering (postconditions 1-8) is unchanged in all cases. The `FindingsRender`
   struct makes all four modes structurally orthogonal.

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
7. **FINDINGS dispatch routing (BC-2.11.025 / BC-2.11.031):** The FINDINGS dispatch at
   `terminal.rs:200-226` routes based on `(self.render.grouping, self.render.collapse)`:
   `(Grouped, Expanded)` → `render_findings_grouped` (per-finding, no collapse);
   `(Grouped, Collapsed)` → `render_findings_grouped_collapsed` (per-bucket collapse, STORY-119);
   `(Flat, Collapsed)` → `render_findings_collapsed` (global flat collapse pass);
   `(Flat, Expanded)` → per-finding flat loop. The section header and the enclosing
   `if !findings.is_empty()` guard (postcondition 4 / Invariant 2) are unchanged in all four
   arms. The `FindingsRender` struct makes all four paths mutually exclusive at the type level.

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
| EC-008 | `render = {Flat, Collapsed}` (default), findings non-empty | FINDINGS section body rendered as flat collapsed groups per BC-2.11.025/026/027; section header and section presence unchanged; section order unchanged |
| EC-009a | `render = {Grouped, Expanded}` (`--mitre --no-collapse`) | FINDINGS section body rendered via `render_findings_grouped`; no collapse pass; section order unchanged |
| EC-009b | `render = {Grouped, Collapsed}` (`--mitre` alone, STORY-119 default) | FINDINGS section body rendered via `render_findings_grouped_collapsed`; per-bucket collapse applies (BC-2.11.031); tactic bucket headers present; section order unchanged |

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
- BC-2.11.013 -- composes with (FINDINGS section content when render.grouping==Grouping::Grouped)
- BC-2.11.025 -- composes with (flat FINDINGS dispatch routes through collapse pass when render=={Flat,Collapsed})
- BC-2.11.031 -- composes with (grouped FINDINGS dispatch routes through per-bucket collapse when render=={Grouped,Collapsed})
- BC-2.11.026 -- composes with (count suffix rendering for collapsed groups within the FINDINGS body)
- BC-2.11.027 -- composes with (evidence sampling within collapsed groups in the FINDINGS body)
- BC-2.11.028 -- depends on (--no-collapse flag controls render field; when FlatExpanded, FINDINGS body unchanged from pre-v0.8.0)
- BC-2.11.029 -- composes with (JSON/CSV reporters unaffected by collapse pass; only the flat-mode FINDINGS body of the terminal reporter changes)

## Architecture Anchors

- `src/reporter/terminal.rs:129-250` -- TerminalReporter::render full body
- `src/reporter/terminal.rs:164` -- HOSTS conditional block
- `src/reporter/terminal.rs:176` -- PROTOCOLS section
- `src/reporter/terminal.rs:178-181` -- proto_vec sort: `sort_by(|a, b| b.1.cmp(a.1).then_with(|| format!("{:?}", a.0).cmp(&format!("{:?}", b.0))))` (FIX-P5-003)
- `src/reporter/terminal.rs:189` -- SERVICES conditional block (`if !services.is_empty()`)
- `src/reporter/terminal.rs:192` -- svc_vec sort: `sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)))` (FIX-P5-003)
- `src/reporter/terminal.rs:200` -- FINDINGS conditional block (`if !findings.is_empty()`)
- `src/reporter/terminal.rs:229` -- ANALYZER: sections loop
- `tests/reporter_terminal_tests.rs::test_terminal_protocols_sorted_count_then_name` -- covers postcondition 7 / invariant 6 / EC-006 (FIX-P5-003)
- `tests/reporter_terminal_tests.rs::test_terminal_services_sorted_count_then_name` -- covers postcondition 8 / invariant 6 / EC-007 (FIX-P5-003)

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reporter/terminal.rs:129-250` |
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
