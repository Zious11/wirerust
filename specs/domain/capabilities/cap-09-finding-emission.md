---
artifact: L2-cap-09
traces_to: ../domain-spec.md
cap_id: CAP-09
title: Forensic Finding Emission
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
version: "1.1"
modified:
  - date: 2026-06-13
    actor: product-owner
    reason: "ARP-F2 Pass-14 remediation: C-01/C-02: mitre_technique Option<String> → mitre_techniques Vec<String> (skip_serializing_if Vec::is_empty; STORY-100 AC-008); four Option fields → three remaining Option fields (source_ip, timestamp, direction); stale timestamp:None universal claim updated — STORY-097/098/099 wired timestamp in http.rs/tls.rs/reassembly/lifecycle.rs, STORY-102..110 added modbus+dnp3 emission sites; site count framing updated to ≥22 (includes modbus/dnp3 analyzers); BC refs extended to BC-2.09.001..007 per STORY-100 extension."
---

# CAP-09: Forensic Finding Emission

## What the system does today

`Finding` (E-26) is the core output type. Analyzers construct Finding structs directly
using struct-literal syntax (no builder, no helper). There are at least 22 production
emission sites across six files (http.rs, tls.rs, reassembly/mod.rs, reassembly/lifecycle.rs,
analyzer/modbus.rs, analyzer/dnp3.rs). Timestamp wiring is complete across all emission sites
(STORY-097/098/099 for http/tls/reassembly; STORY-102..110 for modbus/dnp3). O-01 is closed.

**Sources:** C-14 findings.rs (module-decomposition.md). BC-2.09.001..007.

## Finding schema

```rust
pub struct Finding {
    pub category:         ThreatCategory,         // always serialized
    pub verdict:          Verdict,                 // always serialized
    pub confidence:       Confidence,              // always serialized
    pub summary:          String,                  // raw bytes (ADR 0003)
    pub evidence:         Vec<String>,             // raw bytes (ADR 0003)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mitre_techniques: Vec<String>,             // OMITTED from JSON if empty (ADR-006 Decision 13)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ip:        Option<IpAddr>,          // OMITTED from JSON if None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp:        Option<DateTime<Utc>>,   // OMITTED from JSON if None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction:        Option<Direction>,       // OMITTED from JSON if None
}
```

**mitre_techniques (STORY-100 / ADR-006 Decision 13):** `mitre_technique: Option<String>`
was replaced by `mitre_techniques: Vec<String>`. A singleton technique emits as
`vec!["TXXXX"]`; co-attributed findings (e.g., ARP spoof: `["T0830","T1557.002"]`) use a
multi-element vec. Empty vec uses `skip_serializing_if = "Vec::is_empty"` — the key is absent
from JSON when no technique is attributed. The old scalar field no longer exists (STORY-100
AC-008).

**Three remaining Option fields:** `source_ip`, `timestamp`, and `direction` each use
`skip_serializing_if = "Option::is_none"`. No Option field ever serializes as `null`.

**JSON Option/Vec handling (fixed P1.02 / #73):** All three Option fields use
`skip_serializing_if = "Option::is_none"`. The prior asymmetry
(where `source_ip` always appeared as `null`) is closed.

**direction field (P2.08 / #77):** All HTTP and TLS analyzer findings carry
`direction: Some(Direction::ClientToServer)` or `direction: Some(Direction::ServerToClient)`
as appropriate. Reassembly-engine findings do not set direction (left None). This field was
added as part of the direction-tagging remediation pass.

**timestamp (O-01 CLOSED):** Timestamp wiring is complete. STORY-097/098/099 threaded
`timestamp_secs` from the pcap record header through `StreamHandler::on_data` and into all
http.rs, tls.rs, reassembly/mod.rs, and reassembly/lifecycle.rs emission sites.
STORY-102..110 wired timestamp in the Modbus and DNP3 analyzers.

## Emission sites (≥22 sites; includes modbus/dnp3 analyzers)

The original brownfield baseline counted 22 sites across http.rs (9), tls.rs (7),
reassembly/mod.rs (4), and reassembly/lifecycle.rs (2). STORY-102..105 (Modbus) and
STORY-106..110 (DNP3) added additional emission sites in analyzer/modbus.rs and
analyzer/dnp3.rs respectively. All sites set `timestamp` from the pcap-relative capture
clock (no longer None). Grouped by file:

**analyzer/http.rs (9 sites):**

| Line | Detection | Summary type | Evidence shape | Direction |
|---|---|---|---|---|
| 192 | Path traversal | format! | vec![1 entry] | ClientToServer |
| 219 | Web shell | format! | vec![1 entry] | ClientToServer |
| 238 | Admin panel | format! | vec![1 entry] | ClientToServer |
| 254 | Unusual method | format! | vec![1 entry] | ClientToServer |
| 290 | Missing Host | string literal | vec![1 entry] | ClientToServer |
| 306 | Long URI | format! | vec![1 entry] | ClientToServer |
| 345 | Empty UA | string literal | vec![1 entry] | ClientToServer |
| 417 | Too-many-headers (request) | string literal | vec![1 entry] | ClientToServer |
| 476 | Too-many-headers (response) | string literal (byte-identical to 417) | vec![1 entry] | ServerToClient |

**analyzer/tls.rs (7 sites):**

| Line | Detection | Summary type | Evidence shape | Direction |
|---|---|---|---|---|
| 427 | SNI AsciiWithControl | format! | vec![1 entry] | ClientToServer |
| 450 | SNI NonAsciiUtf8 | format! | vec![1 entry] | ClientToServer |
| 470 | SNI NonUtf8 | format! | vec![1 entry] | ClientToServer |
| 505 | Weak ClientHello ciphers | string literal | vec![variable; 1..=~9216 cipher names] | ClientToServer |
| 526 | Deprecated ClientHello version | format! | vec![1 entry] | ClientToServer |
| 571 | Weak ServerHello cipher | format! | vec![1 entry] | ServerToClient |
| 591 | Deprecated ServerHello version | format! | vec![1 entry] | ServerToClient |

**reassembly/mod.rs (4 sites):**

| Line | Detection | Summary type | Evidence shape | Direction |
|---|---|---|---|---|
| 433 | Excessive overlaps | format! | vec![1 entry] | Some(dir) |
| 467 | Excessive small segments | format! | vec![1 entry] | Some(dir) |
| 496 | Out-of-window segments | format! | vec![1 entry] | Some(dir) |
| 573 | Finalize segment-limit summary | format! with plural_s helper | vec![2 entries] | None |

**reassembly/lifecycle.rs (2 sites):**

| Line | Detection | Summary type | Evidence shape | Direction |
|---|---|---|---|---|
| 105 | Conflicting TCP overlap | format! | vec![1 entry] | None |
| 125 | Stream depth exceeded | format! | vec![1 entry] | None |

**analyzer/modbus.rs and analyzer/dnp3.rs:** Additional emission sites added in
STORY-102..105 (Modbus) and STORY-106..110 (DNP3). See module-decomposition.md C-22/C-23
for the per-file site inventory.

**Notable emission-site properties:**
- Sites 417 and 476 (HTTP too-many-headers) share byte-identical summary strings; differ only
  in evidence[0] ("Direction: request" vs "Direction: response") AND in direction field.
  Finding-dedup by (category+verdict+confidence+summary) would still collapse distinct events.
- Site 505 (tls.rs) is the ONLY data-dependent-cardinality evidence vec.
- Site 573 (reassembly/mod.rs finalize) is the ONLY 2-entry evidence vec in the engine.

## Raw-data contract (ADR 0003; INV-4)

`Finding.summary` and `Finding.evidence` carry raw post-`from_utf8_lossy` bytes. No escape
function is applied at construction time. `escape_for_terminal` is called only by
`TerminalReporter`. `JsonReporter` delegates escaping to serde_json (RFC 8259 compliance).

This design was established after PR #49 introduced construction-site sanitization using
`{:?}` Debug formatting, which destroyed forensic data. ADR 0003 reverted and codified this.

## BC references

BC-2.09.001..007: Finding schema (001-004), raw-data invariant (005), JSON schema symmetry (006),
timestamp wiring (007; added STORY-100 extension).
