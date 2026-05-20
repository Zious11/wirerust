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
across three files. All 22 sites set `timestamp: None` (domain-debt O-01).

**Sources:** C-10 findings.rs. BC-FND-001..006.

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

## 22 emission sites (authoritative, from pass-2 R3 Target 1)

Grouped by file:

**analyzer/http.rs (9 sites):**

| Line | Detection | Summary type | Evidence shape | Direction |
|---|---|---|---|---|
| 187 | Path traversal | format! | vec![1 entry] | ClientToServer |
| 216 | Web shell | format! | vec![1 entry] | ClientToServer |
| 231 | Admin panel | format! | vec![1 entry] | ClientToServer |
| 246 | Unusual method | format! | vec![1 entry] | ClientToServer |
| 260 | Missing Host | string literal | vec![1 entry] | ClientToServer |
| 274 | Long URI | format! | vec![1 entry] | ClientToServer |
| 288 | Empty UA | string literal | vec![1 entry] | ClientToServer |
| 359 | Too-many-headers (request) | string literal | vec![1 entry] | ClientToServer |
| 417 | Too-many-headers (response) | string literal (byte-identical to 359) | vec![1 entry] | ServerToClient |

**analyzer/tls.rs (7 sites):**

| Line | Detection | Summary type | Evidence shape | Direction |
|---|---|---|---|---|
| 405 | SNI AsciiWithControl | format! (contains literal U+00A7 section-sign) | vec![1 entry] | ClientToServer |
| 424 | SNI NonAsciiUtf8 | format! | vec![1 entry] | ClientToServer |
| 443 | SNI NonUtf8 | format! | vec![1 entry] | ClientToServer |
| 471 | Weak ClientHello ciphers | string literal | vec![variable; 1..=~9216 cipher names] | ClientToServer |
| 492 | Deprecated ClientHello version | format! | vec![1 entry] | ClientToServer |
| 534 | Weak ServerHello cipher | format! | vec![1 entry] | ServerToClient |
| 555 | Deprecated ServerHello version | format! | vec![1 entry] | ServerToClient |

**reassembly/mod.rs (6 sites):**

| Line | Detection | Summary type | Evidence shape | Direction |
|---|---|---|---|---|
| 286 | Excessive overlaps | format! | vec![1 entry] | None |
| 305 | Excessive small segments | format! | vec![1 entry] | None |
| 329 | Out-of-window segments | format! | vec![1 entry] | None |
| 415 | Finalize segment-limit summary | format! with inline ternary (pluralization) | vec![2 entries] | None |
| 545 | Conflicting TCP overlap | format! | vec![1 entry] | None |
| 561 | Stream depth exceeded | format! | vec![1 entry] | None |

**Notable emission-site properties:**
- Sites 359 and 417 (HTTP too-many-headers) share byte-identical summary strings; differ only
  in evidence[0] ("Direction: request" vs "Direction: response") AND in direction field.
  Finding-dedup by (category+verdict+confidence+summary) would still collapse distinct events.
- Site 471 is the ONLY data-dependent-cardinality evidence vec.
- Site 415 is the ONLY 2-entry evidence vec in the engine.
- Site 405 contains a literal U+00A7 (section sign) in the source string -- the only
  non-ASCII codepoint in any emission template.

## Raw-data contract (ADR 0003; INV-4)

`Finding.summary` and `Finding.evidence` carry raw post-`from_utf8_lossy` bytes. No escape
function is applied at construction time. `escape_for_terminal` is called only by
`TerminalReporter`. `JsonReporter` delegates escaping to serde_json (RFC 8259 compliance).

This design was established after PR #49 introduced construction-site sanitization using
`{:?}` Debug formatting, which destroyed forensic data. ADR 0003 reverted and codified this.

## BC references

BC-FND-001..006: Finding schema (001-004), raw-data invariant (005), JSON schema symmetry (006).
