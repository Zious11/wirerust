---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/findings.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-09
capability: CAP-09
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Feature-100 (pcap timestamps) — O-01 resolved: 21 of 22 emission sites now set timestamp: Some(...); segment-limit summary retains None. Invariant 1 updated; Refactoring Notes updated. — 2026-06-08"
  - "v1.4: ADR-006 / Decision 13 (v0.3.0 BREAKING) — mitre_technique: Option<String> renamed to mitre_techniques: Vec<String>; empty vec replaces None; singleton vec replaces Some; added EC-006 (multi-tag co-emission). Emission-site count updated from 22 to 22+ (Modbus sites added in F2). — 2026-06-09"
  - "v1.5: v19 remap: T0855 → T1692.001 per MITRE ATT&CK for ICS v19.0 revocation. All T0855 technique ID references in Description, EC-006, and Canonical Test Vectors updated to T1692.001. Tactic unchanged: IcsImpairProcessControl. Issue #222; audit: mitre-ics-v19-catalog-audit.md. — 2026-06-10"
  - "v1.6: Pass-19 B-01 re-anchor — Architecture Anchor and Source Evidence corrected: src/findings.rs:119-146 → :135-162 (struct Finding spans lines 135-162 after STORY-100 multi-tag comment block inserted). Verified against HEAD findings.rs. — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.09.001: Finding Constructed with Required Fields and Optional Fields

<!--
  PREVIOUS VERSION SUMMARY (v1.3 -> v1.4):
  Field renamed: mitre_technique: Option<String> -> mitre_techniques: Vec<String>
  EC-003: mitre_technique = Some("T1036") -> mitre_techniques = vec!["T1036"]
  EC-004: mitre_technique = None -> mitre_techniques = vec![]
  EC-006 added: mitre_techniques = vec!["T0855","T0836"] (multi-tag co-emission)
  Canonical vector updated: Some("T1027") -> vec!["T1027"]
  Description updated to reflect Vec<String> and Modbus co-emission model.
-->

## Description

The `Finding` struct is constructed directly via struct-literal syntax at each emission site.
Required fields (`category`, `verdict`, `confidence`, `summary`, `evidence`) must always be
provided. The optional fields are: `mitre_techniques: Vec<String>` (empty vec = no technique;
singleton vec = single technique; multi-element vec = co-attributed techniques per ADR-006),
`source_ip: Option<IpAddr>`, `timestamp: Option<DateTime<Utc>>`, `direction: Option<Direction>`.

After feature-100 (pcap timestamps), 21 of the original 22 emission sites set
`timestamp: Some(DateTime<Utc>)` derived from the pcap `ts_sec` value at the flush call site;
the segment-limit summary finding in `finalize` retains `timestamp: None` (it is a
post-capture aggregate, not tied to any specific packet). Feature #7 (Modbus analyzer) adds
further emission sites that directly use `mitre_techniques: vec![...]` (multi-element for
co-attributed ICS techniques). There is no builder or constructor helper -- every site provides
the full literal.

## Preconditions

1. An analyzer or engine has detected a condition warranting a Finding.
2. The caller has appropriate `category`, `verdict`, `confidence`, `summary`, and `evidence`
   values ready.

## Postconditions

1. A `Finding` value is constructed with:
   - `category`: one of `ThreatCategory` variants (Reconnaissance, Anomaly, Execution, Persistence, etc.)
   - `verdict`: one of `Verdict::Likely | Unlikely | Inconclusive`
   - `confidence`: one of `Confidence::High | Medium | Low`
   - `summary`: raw `String` (per ADR 0003; no escape applied at construction)
   - `evidence`: `Vec<String>` (raw; 0 or more entries)
   - `mitre_techniques`: `Vec<String>` — `vec![]` (no technique), `vec!["TXXXX"]` (single technique,
     migration of all pre-F2 `Some("TXXXX")` sites), or `vec!["T1692.001","T0836"]` (co-attributed;
     Modbus write-class PDUs per ADR-006 Decision 13; T1692.001 is the v19 ICS sub-technique, successor to revoked T0855)
   - `source_ip`: `Option<IpAddr>` (Some(ip) at 5 reassembly sites in mod.rs and lifecycle.rs;
     None at all HTTP/TLS and segment-limit-summary sites)
   - `timestamp`: `Option<DateTime<Utc>>` (Some(DateTime<Utc>) at 21 of 22 original emission
     sites after feature-100; None only at the segment-limit summary site in finalize)
   - `direction`: `Option<Direction>` (Some for HTTP/TLS findings; Some for reassembly mod.rs
     overlap/small-segment/out-of-window findings; None for reassembly lifecycle and
     segment-limit-summary findings)
2. No allocation beyond the struct fields themselves.
3. The constructed value is valid to pass to any reporter.

## Invariants

1. After feature-100: 21 of 22 original emission sites set `timestamp: Some(DateTime<Utc>)` derived
   from the pcap capture-relative `ts_sec` value. The segment-limit summary finding in
   `finalize` (the 22nd site) retains `timestamp: None` — this is correct behavior, not a
   gap. See BC-2.09.007 for the full provenance and conversion invariants.
2. Reassembly anomaly findings in `reassembly/mod.rs` (overlap, small-segment, out-of-window)
   set `source_ip: Some(packet.src_ip)`. Reassembly lifecycle findings in
   `reassembly/lifecycle.rs` (conflicting-overlap, stream-depth-exceeded) also set
   `source_ip: Some(src_ip)`. HTTP and TLS analyzer findings set `source_ip: None`.
   The segment-limit summary finding in `reassembly/mod.rs` sets `source_ip: None`.
   (Domain-debt O-01 mischaracterized source_ip as always None; 5 of 22 original sites set Some.)
3. HTTP and TLS analyzer findings set `direction: Some(...)`.
4. Reassembly anomaly findings in `reassembly/mod.rs` (overlap, small-segment,
   out-of-window) set `direction: Some(dir)`. Reassembly lifecycle findings and the
   segment-limit summary finding set `direction: None`.
5. `summary` and `evidence` carry raw bytes (ADR 0003 / INV-4).
6. `mitre_techniques` is NEVER `None` — it is always a `Vec<String>`, possibly empty. An
   empty vec is the canonical replacement for the former `Option::None`. No emission site
   may use `Option<String>` for technique attribution after v0.3.0 (ADR-006).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | evidence vec is empty | Valid Finding; reporters omit evidence section |
| EC-002 | summary is empty string | Valid Finding; unusual but not rejected |
| EC-003 | mitre_techniques = vec!["T1036"] | Singleton vec; flows through to JSON ("mitre_techniques":["T1036"]) and MITRE grouping (groups under T1036 tactic) |
| EC-004 | mitre_techniques = vec![] (empty) | No "mitre_techniques" key in JSON (skip_serializing_if = Vec::is_empty); finding lands in Uncategorized bucket |
| EC-005 | Finding with direction = Some(ServerToClient) | direction field set; JSON emits "ServerToClient" |
| EC-006 | mitre_techniques = vec!["T1692.001","T0836"] | Multi-tag co-emission (Modbus register write per ADR-006 §13.5; T1692.001 is ICS sub-technique, v19 successor to revoked T0855); JSON: "mitre_techniques":["T1692.001","T0836"]; groups under first technique's tactic (IcsImpairProcessControl) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| HttpAnalyzer path-traversal detection | Finding { category: Anomaly, verdict: Likely, confidence: High, direction: Some(ClientToServer), timestamp: Some(DateTime<Utc>) } | happy-path |
| Reassembly conflicting overlap | Finding { category: Anomaly, direction: None, timestamp: Some(DateTime<Utc>) } | happy-path |
| TLS SNI control-byte detection | Finding { category: Anomaly, mitre_techniques: vec!["T1027"] } | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-021 | 21 of 22 emission sites produce Finding with timestamp=Some(...); segment-limit summary has None | integration test + proptest |
| — | HTTP and TLS findings carry direction=Some | unit: assert direction is Some after HTTP/TLS analysis |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-09 ("Forensic finding emission") per domain/capabilities/cap-09-finding-emission.md |
| Capability Anchor Justification | CAP-09 ("Forensic finding emission") per domain/capabilities/cap-09-finding-emission.md -- this BC defines the full schema of the Finding struct which is the core output type of CAP-09 |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-09 (findings.rs, C-14) |
| Stories | STORY-069 |
| Origin BC | BC-FND-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.09.005 -- composes with (raw-bytes contract for summary/evidence)
- BC-2.09.006 -- composes with (JSON serialization of Option fields; timestamp now appears when Some)
- BC-2.09.007 -- composes with (timestamp provenance and value invariants; extends this BC's timestamp postcondition)
- BC-2.09.002 -- composes with (Display rendering of this struct)

## Architecture Anchors

- `src/findings.rs:135-162` -- Finding struct definition with all fields and serde attributes

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/findings.rs:135-162` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: struct-literal syntax at emission sites; compiler enforces all fields
- **documentation**: O-01 doc comment on timestamp; INV-4 doc comment

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (Finding is an owned value) |
| **Overall classification** | pure |

## Refactoring Notes

O-01 (timestamp always None) is resolved by feature-100 (issue #100). `RawPacket.timestamp_secs`
is now threaded through `StreamHandler::on_data` (BC-2.04.055) to 21 of 22 emission sites.
The `chrono` crate dependency now carries active forensic value. The one remaining `None` site
(segment-limit summary in `finalize`) is correct behavior, not a gap — it is documented as
an invariant in BC-2.09.007.
