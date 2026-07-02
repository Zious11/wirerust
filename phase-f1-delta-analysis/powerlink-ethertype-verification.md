# Ethernet POWERLINK EtherType Verification

**Date:** 2026-07-01
**Feature:** feature-protocol-coverage (protocol-coverage catalog)
**Method:** DF-VALIDATION-001 (cite sources, verify against primary refs, flag if inconclusive)
**Question:** Is the Layer-2 EtherType for Ethernet POWERLINK equal to hex `0x88AB` (decimal 34987)?

## Answer

1. **Correct EtherType:** `0x88AB` (hex) = `34987` (decimal).
2. **Is `0x88AB` correct?** **YES — confirmed.** The `[unverified]` tag can be removed.
3. **Confidence:** **HIGH.** Corroborated by the IEEE Registration Authority EtherType
   registry plus multiple independent implementations and standards documents that
   explicitly cite the IEEE assignment. No source disagreed.

## Important disambiguation: V1 vs V2

There are two historical EtherType values associated with "Ethernet POWERLINK":

| Variant | EtherType | Status |
|---------|-----------|--------|
| POWERLINK **V1** | `0x3E3F` | Proprietary B&R (Bernecker + Rainer) precursor; obsolete. |
| POWERLINK **V2** | `0x88AB` | Current open EPSG standard (CANopen device profiles, POWERLINK Safety, EDS). **This is "Ethernet POWERLINK" as understood today.** |

The modern, IEEE-assigned, EPSG-standardized protocol — the one the wirerust catalog
means by "POWERLINK" — uses **`0x88AB`**. (Source: German Wikipedia protocol description,
cross-checked with Wireshark's constant name `ETHERTYPE_EPL_V2 = 0x88AB`.)

## Authoritative Sources

### Primary — IEEE Registration Authority (the assigning body)
- The IEEE Registration Authority publishes the public EtherType registry (`eth.txt`).
  It lists EtherType **`88AB`** as "ETHERNET Powerlink" with the Ethernet POWERLINK
  Standardization Group (EPSG) as the responsible organization. EtherType values are
  assigned exclusively by the IEEE RA.
  - IEEE EtherType public listing: https://standards-oui.ieee.org/ethertype/eth.txt
  - Registry search / RA page (referenced via Wireshark wiki): https://regauth.standards.ieee.org/standards-ra-web/pub/view.html#registries

### Corroborating standards / data models
- **IETF `ietf-ethertypes` YANG module (2019-03-04):** defines
  `enum "ethernet-powerlink" { value 34987; description "Ethernet Powerlink. Hex value of 0x88AB."; }`
  - https://www.netconfcentral.org/modules/ietf-ethertypes/2019-03-04
- **OpenDaylight / IEEE 802.1 EtherType YANG models:** encode `ethernet-powerlink` = `34987`.

### Corroborating implementations (de facto reference)
- **Wireshark `epan/etypes.h`:** `#define ETHERTYPE_EPL_V2 0x88AB /* communication profile for Real-Time Ethernet */`
  - https://www.wireshark.org/docs/wsar_html/etypes_8h.html
  - https://github.com/wireshark/wireshark/blob/master/epan/etypes.h
- **Wikipedia — EtherType** notable-values table: `0x88AB | Ethernet Powerlink`
  - https://en.wikipedia.org/wiki/EtherType
- **German Wikipedia — Ethernet Powerlink:** "Als EtherType für Ethernet Powerlink wurde
  0x88AB von der IEEE zugewiesen." (V1 = `0x3e3f`, V2 = `0x88ab`.)
  - https://de.wikipedia.org/wiki/Ethernet_Powerlink

## Limitations / caveats (per DF-VALIDATION-001)

- The full EPSG "Ethernet POWERLINK Communication Profile Specification" text (paywalled/
  members) was not directly retrieved; confirmation rests on the IEEE RA registry entry
  (the authoritative assigner) plus multiple independent implementations that all cite
  the IEEE assignment. This convergence is sufficient for HIGH confidence.
- No explicit standalone IANA registration was located; per Wikipedia, IANA's EtherType
  list is compiled from the IEEE RA list, so IANA is not an independent primary source here.
  The IEEE RA is the correct authority for EtherType assignments.
- One conflicting value exists (`0x3E3F`) but it is the obsolete **V1** protocol, not a
  contradiction of the V2 assignment.

## Recommendation

- **Remove the `[unverified]` tag** on POWERLINK in the protocol-coverage catalog; record
  the EtherType as `0x88AB` (34987).
- **F3 test obligation:** add a canonical-value assertion test asserting the POWERLINK
  EtherType constant equals `0x88AB` / `34987`, guarding against regression. Consider a
  code comment noting V1 (`0x3E3F`) is intentionally excluded as obsolete.
