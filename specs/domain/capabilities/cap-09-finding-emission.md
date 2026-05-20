---
artifact: L2-cap-09
traces_to: ../domain-spec.md
cap_id: CAP-09
title: Forensic Finding Emission
status: descriptive (brownfield) -- reconciled against develop HEAD aa2ece9
reconciled: 2026-05-20
---

# CAP-09: Forensic Finding Emission

## What the system does today

`Finding` (E-26) is the core output type. Analyzers construct Finding structs directly
using struct-literal syntax (no builder, no helper). There are 22 production emission sites
across four files. All 22 sites set `timestamp: None` (domain-debt O-01).

**Sources:** C-14 findings.rs (module-decomposition.md). BC-2.09.001..006.

## Finding schema

```rust
pub struct Finding {
    pub category:         ThreatCategory,         // always serialized
    pub verdict:          Verdict,                 // always serialized
    pub confidence:       Confidence,              // always serialized
    pub summary:          String,                  // raw bytes (ADR 0003)
    pub evidence:         Vec<String>,             // raw bytes (ADR 0003)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mitre_technique:  Option<String>,          // OMITTED from JSON if None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ip:        Option<IpAddr>,          // OMITTED from JSON if None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp:        Option<DateTime<Utc>>,   // OMITTED from JSON if None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction:        Option<Direction>,       // OMITTED from JSON if None
}
```

**JSON Option handling (fixed P1.02 / #73):** All four Option fields now use
`skip_serializing_if = "Option::is_none"`. They are omitted from JSON output when None.
This is symmetric behavior: no Option field ever serializes as `null`. The prior asymmetry
(where `mitre_technique` and `source_ip` always appeared as `null`) is now closed.

**direction field (P2.08 / #77):** All HTTP and TLS analyzer findings carry
`direction: Some(Direction::ClientToServer)` or `direction: Some(Direction::ServerToClient)`
as appropriate. Reassembly-engine findings do not set direction (left None). This field was
added as part of the direction-tagging remediation pass.

**timestamp (open item O-01):** All 22 emission sites set `timestamp: None`. The `RawPacket`
captures `timestamp_secs` and `timestamp_usecs` from the pcap record header, but these are
never threaded through to any `Finding` constructor. This is a forensic gap: findings cannot
be correlated to wall-clock time.

## 22 emission sites (authoritative, verified against develop HEAD 0082a0c)

Grouped by file:

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

BC-2.09.001..006: Finding schema (001-004), raw-data invariant (005), JSON schema symmetry (006).
