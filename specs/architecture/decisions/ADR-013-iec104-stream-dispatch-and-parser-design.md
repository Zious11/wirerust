---
document_type: adr
adr_id: ADR-013
status: accepted
accepted_date: "2026-07-13"
date: 2026-07-13
modified:
  - date: 2026-07-13
    actor: architect
    reason: "F2 Pass-1 adversarial remediation (DF-SIBLING-SWEEP-001): (F-M9) H1 title corrected to match ADR-013 slug convention; (T0809→T0881) T0809='Data Destruction' not 'Service Stop'; feature detection intent STOPDT abuse=T0881 'Service Stop' IcsInhibitResponseFunction — all six T0809 literals replaced with T0881; (GAP-F2-002/F-H3) Decision 8 VP-044 obligation scoped to parse_apci_header ONLY; verify_classify_frame_format_totality removed from #[cfg(kani)] block and replaced with proptest! illustration (VP-046 is proptest P1, not Kani); (F-M7) T1692.002 marked NOT emitted this cycle; (TypeID fix) T1692.001 row control-command range corrected 45–52→45–51 (TypeID 52 is reserved; C_SE_ND_1 does not exist in IEC 60870-5-104); Positive Consequences text corrected: classify_frame_format is proptest-amenable (VP-046) not Kani-amenable."
  - date: 2026-07-14
    actor: architect
    reason: "F2 Pass-4 remediation (DF-SIBLING-SWEEP-001, F-P4-H2/F-P4-M1): (F-P4-H2) Decision 3 forward-progress guarantee corrected — 'at least 2 bytes per iteration' → 'at least 1 byte per iteration'; step 3 bad-start-byte updated: 'emit anomaly finding and clear carry' → 'advance 1 byte (resync scan to next 0x68 candidate)'; rationale: advancing 2 on a bad start byte skips a real 0x68 at the next offset (BC-2.19.026). (F-P4-M1) T0836 MITRE row corrected: 'C_SE set-point writes (TypeIDs 48–51)' → 'set-point + bitstring writes (C_SE 48–50, C_BO 51)' — TypeID 51 is C_BO_NA_1 (bitstring), not C_SE (per BC-2.19.019 authoritative wording)."
  - date: 2026-07-14
    actor: architect
    reason: "F2 Pass-7 remediation (M3): T0831 row status corrected — 'Pre-existing EMITTED' is an over-claim for IEC-104; NO BC in SS-19 emits T0831 and the SS-19 shard has no T0831 row. Changed status to 'NOT emitted by IEC-104 this cycle (pre-existing EMITTED via Modbus analyzer; correlated C_SE+C_SC detection deferred to a future cycle)'. Trigger wording unchanged ('C_SE + C_SC actuation on same flow') — harmonized with delta §6.1 as part of M3 fix. Both ADR copies updated identically."
  - date: 2026-07-14
    actor: architect
    reason: "F2 Pass-8 remediation (F-P8-M2): Decision 2 window-expiry clause removed — IEC-104 has no windowed detection; the sentence 'Window expiry uses saturating_sub (not wrapping_sub) to prevent spurious resets on backwards timestamps, per RULING-DNP3-SIBLING-001 §2 (ADR-007 Decision 3 pattern)' was copy-pasted from DNP3/ENIP sibling ADRs and does not apply to IEC-104; Iec104FlowState no longer has window_start_ts. Both ADR copies updated identically."
  - date: 2026-07-14
    actor: architect
    reason: "Human-mandated F2 gate follow-on (D-438): first-frame N(S) baseline guard added; last_ns fields promoted from u16 to Option<u16>. Decision 6 updated with Option<u16> tracking semantics: None = no I-frame seen yet; first I-frame sets baseline without emitting a desync finding; gap check runs only on Some(prev) state. Both ADR copies updated identically."
  - date: 2026-07-15
    actor: architect
    reason: "Decision 3 steps 3–4 reconciled with BC-2.19.026 per SR-172-03 fidelity finding; malformed-LEN detection ratified EMIT-WITH-DEDUP via research validation."
subsystems_affected:
  - SS-05
  - SS-10
  - SS-19
supersedes: null
superseded_by: null
feature_cycle: feature-iec104
mitre_pin: ics-attack-19.1
---

# ADR-013: IEC-104 Stream Dispatch and Parser Design

> **One-per-file:** Each architectural decision lives in its own file.
> Filename convention: `ADR-NNN-<short-name>.md`.
> ADR IDs are sequential 3-digit (ADR-001, ADR-002, ...). Once issued, never renumber.
> Lifecycle: `proposed` -> `accepted` -> (optional) `superseded` or `deprecated`.

## Context

wirerust's `StreamDispatcher` currently classifies TCP flows through seven rules: two content
rules (TLS signature, HTTP method prefix), four port fallback rules (443/8443 → TLS,
80/8080 → HTTP, 502 → Modbus [ADR-005], 20000 → DNP3 [ADR-007], 44818 → ENIP [ADR-010]),
and an implicit "no match" arm (Rule 8 prior to this ADR). Feature cycle `feature-iec104`
introduces IEC 60870-5-104 (IEC-104) analysis as subsystem SS-19.

IEC-104 is the dominant SCADA telecontrol protocol for substation-to-control-center
communication over TCP/IP, defined in IEC 60870-5-104:2006. It runs exclusively on TCP
port 2404 (IANA registered) and introduces a distinctive two-layer framing structure:

1. **APCI (Application Protocol Control Information)** — outer frame on every TCP segment:
   fixed start byte `0x68`, one LEN octet (4..=253), four control octets (CF1–CF4).
   Total on-wire frame = LEN + 2 bytes; maximum APDU = 255 bytes.

2. **ASDU (Application Service Data Unit)** — inner payload carried only in I-format frames:
   6-byte Data Unit Identifier (TypeID 1 byte, VSQ 1 byte, COT 2 bytes, CASDU 2 bytes),
   followed by Information Objects (IOA 3 bytes + object data).

Frame format is discriminated from the CF1 low 2 bits:
- **I-format** (bit 0 = 0): numbered data frames; carries ASDU payload; N(S) and N(R) counts
- **S-format** (bits 1:0 = 0b01): supervisory; carries only N(R); no ASDU
- **U-format** (bits 1:0 = 0b11): unnumbered connection control; CF1 distinguishes command:
  STARTDT-act (0x07), STARTDT-con (0x0B), STOPDT-act (0x13), STOPDT-con (0x23),
  TESTFR-act (0x43), TESTFR-con (0x83). Any other U-format CF1 value is non-canonical.

### Relationship to Prior ADRs

This ADR is the IEC-104 sibling of:
- ADR-005: Modbus TCP (port 502, Rule 5). Established the binary-ICS-port-fallback pattern.
- ADR-007: DNP3 TCP (port 20000, Rule 6). Established directional carry-buffer split and
  pure-core free-fn requirement for Kani.
- ADR-010: EtherNet/IP TCP (port 44818, Rule 7). Extended the pattern with two-level
  ENIP/CPF/CIP framing and VP-007 atomic obligation for new MITRE techniques.

No existing ADR is superseded. ADR-013 adds Rule 8 (port 2404) following the same documented
exception to ADR-0001 as its predecessors.

## Decision

### Decision 1: Port-2404 dispatch as Rule 8; no 0x68 content-signature primary rule

IEC-104 TCP flows are classified using a port-2404 rule appended after Rules 1–7 in the
stream dispatcher. Port 2404 is exclusively assigned to IEC-104 by IANA; no content-based
primary rule is added.

**Rationale for rejecting 0x68 as a primary content-signature rule:**

`0x68` is the ASCII character `h`. It appears in:
- HTTP response bodies and headers (Content-Type, chunk extensions, hostnames)
- UTF-8 encoded strings in arbitrary binary formats
- Little-endian integer fields in many binary protocols

Unlike TLS (two-byte discriminator `0x16 0x03` — Record Type + fixed version prefix) or
EtherNet/IP (command word patterns `0x65 0x00` / `0x63 0x00` in a 24-byte fixed header),
a single byte is not a reliable discriminator. Using it as a primary rule would misclassify
HTTP or other binary traffic on non-standard ports.

Port-only dispatch is the established pattern for DNP3 (port 20000, ADR-007 Decision 1) and
EtherNet/IP (port 44818, ADR-010 Decision 1). IEC-104 follows the same pattern.

A post-classification validity gate (`is_valid_iec104_frame`: checks start byte == 0x68
and 4 ≤ LEN ≤ 253) compensates for any false-positive classifications without polluting
the `classify()` rule table.

Cross-reference: dispatcher Rules 1–8 are documented in `src/dispatcher.rs` module comment.

### Decision 2: 255-byte directional carry buffers (MAX_IEC104_CARRY_BYTES = 255)

The maximum IEC-104 on-wire frame size is 255 bytes: LEN + 2 where LEN ≤ 253 (the APCI
LEN field upper bound per IEC 60870-5-104 §5.1). The carry buffer per direction is sized
at 255 bytes (`MAX_IEC104_CARRY_BYTES`) to hold at most one partial APDU.

Directional split follows RULING-DNP3-SIBLING-001 (ADR-007 Decision 2): two carry buffers
per flow — `carry_c2s: Vec<u8>` (client-to-server, IEC-104 controlling station to
controlled station) and `carry_s2c: Vec<u8>` (server-to-client, response direction) —
to prevent cross-direction carry-buffer splice.

Each directional carry is independently bounded at 255 bytes.

### Decision 3: APCI/ASDU two-layer framing; frame-walk loop with forward-progress guarantee

The parser processes TCP segment data with a frame-walk loop:
1. Prepend any carry bytes to the incoming data.
2. If carry + data < 2 bytes, stash in carry and return (insufficient for start byte + LEN).
3. Validate start byte == 0x68. If not, advance 1 byte (silent resync scan to next 0x68
   candidate — advancing 2 would skip a real 0x68 at the next offset and lose a valid
   frame; 1-byte advance is the correct passive-parser resync behavior per BC-2.19.026;
   per-byte findings would flood on junk traffic, so bad-start-byte is silent with no
   finding emitted).
4. If LEN < 4 or LEN > 253, emit T0814 (Anomaly/Possible/Medium) on the FIRST
   out-of-range-LEN occurrence per flow direction (per-direction dedup flag); subsequent
   malformed-LEN frames in that direction advance silently. Advance past the 2-byte APCI
   stub in all cases. Rationale: CVE-2023-5768; Snort3 IEC104_BAD_LENGTH; Zeek
   weird-length sampling — established monitors emit-then-dedup
   (`.factory/cycles/feature-iec104/research/sr-172-03-malformed-len-validation.md`).
5. If carry + data < LEN + 2, stash full carry and return (incomplete APDU).
6. Extract full APDU (LEN + 2 bytes), advance frame-walk pointer by LEN + 2, process.
7. Repeat from step 2 with remaining bytes.

**Forward progress guarantee:** every loop iteration advances at least 1 byte — valid
frame: LEN+2; malformed-LEN (step 4): 2-byte APCI stub; bad start byte (step 3): 1-byte
resync scan; insufficient data (steps 2 or 5): stash in carry and return — guarantees
termination. Per BC-2.19.026: advancing only 1 on a bad start byte (not 2) preserves any
valid 0x68 at the next offset (VP-047, cargo-fuzz).

VP-044 (Kani) proves that `parse_apci_header` — the pure-core function implementing
steps 1–5 — never panics, never produces an out-of-bounds index, and that the
`LEN + 2` arithmetic is free from integer overflow for all LEN in [4, 253].

### Decision 4: U/S/I frame format discrimination from CF1 low bits

Frame format is determined from the least-significant bits of CF1 using a pure-core
free `fn classify_frame_format(cf1: u8) -> FrameFormat`:

```
if cf1 & 0x01 == 0   → IFormat  (N(S) in CF1[7:1], CF2[7:0]; N(R) in CF3[7:1], CF4[7:0])
elif cf1 & 0x03 == 1 → SFormat  (N(R) in CF3[7:1], CF4[7:0])
else                 → UFormat  (CF1 encodes command; CF2–CF4 are reserved zeros)
```

The three cases are exhaustive (every u8 value falls into exactly one arm) and mutually
exclusive. VP-046 (proptest) proves totality over all 256 u8 values for CF1.

`classify_frame_format` is a free `fn`, not an `impl` method, making it Kani-amenable
and proptest-addressable without any mock or seam.

### Decision 5: STARTDT/STOPDT/TESTFR session state machine (ACT detection only, MVP)

For U-format frames, the analyzer detects the following connection control commands:

| CF1 value | Command name | Security relevance |
|-----------|-------------|-------------------|
| 0x07 | STARTDT-act | Normal session start; no finding unless abnormal context |
| 0x0B | STARTDT-con | Confirmation; tracked for session coherence |
| 0x13 | STOPDT-act | **Data acquisition halt; T0881 "Service Stop" finding** |
| 0x23 | STOPDT-con | Confirmation of STOPDT |
| 0x43 | TESTFR-act | Keepalive; no finding in isolation |
| 0x83 | TESTFR-con | Keepalive response; no finding in isolation |

Detection criteria:
- **STOPDT-act without prior STARTDT-act:** anomalous session sequence → T0881 finding
  with confidence Likely.
- **Any STOPDT-act:** data acquisition halt observed → T0881 finding with confidence
  Possible.
- **Non-canonical U-format CF1 value** (any value other than the six above with
  bits 1:0 = 0b11): CVE-2026-1773 angle — implementation-defined behavior in many RTUs
  → T0814 "Denial of Service" finding with confidence Possible.

MVP scope: STARTDT/STOPDT-ACT detection only. ACT/CON four-way pairing (confirming the
full handshake completes) is deferred to a future cycle.

### Decision 6: N(S)/N(R) sequence counter tracking

IEC-104 I-frames carry 15-bit send sequence counter N(S) and receive sequence counter
N(R) (modulo 32768). The protocol mandates a maximum unacknowledged window `k = 12`
(maximum 12 unacknowledged I-frames outstanding).

N(S) desync detection tracks last-seen N(S) per direction as `Option<u16>`
(`Iec104FlowState::last_ns_c2s` and `last_ns_s2c`). On the first observed I-frame in a
direction (state `None`), the parser records the baseline `Some(ns)` and emits NO finding
— this prevents a false-positive desync on mid-session captures (the analyzer's primary
use case, where the first observed N(S) is arbitrary). On subsequent I-frames
(state `Some(prev)`), gap = `ns.wrapping_sub(prev) & 0x7FFF`; if gap > k=12, emit
T1692.001 "Unauthorized Message: Command Message" (Possible).

N(S)/N(R) extraction:
```
N(S) = ((cf1 as u16 >> 1) & 0x7F) | ((cf2 as u16) << 7)   // 15 bits, little-endian
N(R) = ((cf3 as u16 >> 1) & 0x7F) | ((cf4 as u16) << 7)   // 15 bits, little-endian
```

### Decision 7: Licensing constraint — original Rust parser only

**HARD CONSTRAINT (immutable, checked in PR reviews):**

| Source | License | Status |
|--------|---------|--------|
| `iec60870-5` crate (crates.io) | "NOT FREE for commercial/production use" (proprietary) | **BANNED** |
| Wireshark IEC-104 dissector (`epan/dissectors/packet-104.c`) | GPLv2 | **BANNED** |
| lib60870 (MZ Automation) | GPLv3 | **BANNED** |
| Erlang SCADA library | GPLv2 | **BANNED** |

The `iec60870-5` crate MUST NOT appear in `Cargo.toml` or `Cargo.lock`. GPL code from
Wireshark or lib60870 MUST NOT be copied — not even constants, lookup tables, or logic
structure. This is a blocking PR review criterion.

Permitted design references (no code copy):
- `xgbt/go-iec104` (MIT license) — design reference only
- `wendy512/iec104` (Apache-2.0) — design reference only
- IEC 60870-5-104 standard framing tables — public framing diagrams only

The `parse_apci_header`, `classify_frame_format`, and ASDU field extraction functions
are original Rust implementations derived from the IEC 60870-5-104 standard framing
diagrams. Zero lines are borrowed from any external implementation.

### Decision 8: Pure-core free-fn design for VP-044 Kani amenability

Two functions must be pure-core free `fn`s (not `impl` methods on `Iec104Analyzer` or
`Iec104FlowState`) for formal verification amenability:

1. `parse_apci_header(data: &[u8]) -> Option<ApciHeader>` — parses the 6-byte APCI prefix
   from a byte slice; returns `Some` if `data.len() >= 6`, start byte == 0x68, and
   4 ≤ LEN ≤ 253; returns `None` otherwise. No mutable state, no I/O.
   **VP-044 (Kani P0):** scoped to `parse_apci_header` arithmetic safety ONLY — no panic,
   no out-of-bounds index, and `LEN + 2` arithmetic within [6, 255]. parse_asdu field
   extraction, N(S)/N(R) counter tracking, and on_data-loop no-panic are covered by
   VP-047 (cargo-fuzz), not this Kani harness.

2. `classify_frame_format(cf1: u8) -> FrameFormat` — pure discriminant on CF1 low bits.
   No state, no I/O. **VP-046 (proptest P1):** proves totality over all 256 u8 values;
   proptest exhaustive-enum coverage is the correct tool for a simple discriminant function
   (Kani would be redundant here and is not used for VP-046).

These functions are placed in the module scope of `src/analyzer/iec104.rs`, not inside
`impl Iec104Analyzer`, mirroring `parse_mbap_header`/`classify_fc` (Modbus, VP-022),
`parse_dnp3_dl_header`/`classify_dnp3_fc` (DNP3, VP-023), and
`parse_enip_header`/`classify_enip_command` (EtherNet/IP, VP-032).

The Kani proof harness skeleton for VP-044 (`parse_apci_header` arithmetic safety ONLY):
```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // VP-044: parse_apci_header arithmetic safety.
    // SCOPE: this harness covers only parse_apci_header.
    // parse_asdu field extraction, N(S)/N(R) counter tracking, and on_data-loop
    // no-panic are covered by VP-047 (cargo-fuzz), not this harness.
    // classify_frame_format totality over all 256 CF1 values is covered by
    // VP-046 (proptest), not Kani.
    #[kani::proof]
    fn verify_parse_apci_header_safety() {
        let len: usize = kani::any();
        kani::assume(len <= 260); // bounded for tractability
        let mut data = vec![0u8; len];
        for i in 0..len { data[i] = kani::any(); }
        let _ = parse_apci_header(&data);
        // Property A: no panic (implicit — if fn returns, it did not panic)
        if let Some(h) = parse_apci_header(&data) {
            // Property B: total frame length is in [6, 255]
            let total = h.len as usize + 2;
            kani::assert(total >= 6, "APCI total frame >= 6");
            kani::assert(total <= 255, "APCI total frame <= 255");
            // Property C: len field in valid range
            kani::assert(h.len >= 4, "LEN >= 4");
            kani::assert(h.len <= 253, "LEN <= 253");
        }
    }
}
```

The proptest illustration for VP-046 (`classify_frame_format` totality — proptest P1):
```rust
// proptest! {
//     // VP-046: classify_frame_format is total over all 256 CF1 values.
//     // Every u8 maps to exactly one FrameFormat variant; no panic, no unreachable.
//     // proptest exhaustive u8 coverage is used here, not Kani.
//     #[test]
//     fn proptest_vp046_frame_format_totality(cf1: u8) {
//         let fmt = classify_frame_format(cf1);
//         // All three variants are reachable by construction of the CF1 bit discriminant:
//         //   bit0==0            → IFormat
//         //   bits1:0==0b01      → SFormat
//         //   bits1:0==0b11      → UFormat
//         let _ = fmt;
//     }
// }
```

### Decision 9: VP-004 oracle obligation

Adding `DispatchTarget::Iec104` and Rule 8 requires updating the `classify_oracle`
function in `src/dispatcher.rs` `#[cfg(kani)] mod kani_proofs` block in the **same
commit**. The six-step atomic obligation:

1. Add `DispatchTarget::Iec104` variant to the `DispatchTarget` enum.
2. Add the port-2404 arm to `classify()` (Rule 8, after Rule 7 ENIP).
3. Add the corresponding `DispatchTarget::Iec104` arm to `classify_oracle` so it mirrors
   production `classify()` syntactically.
4. Extend the early-exit guard to include `self.iec104.is_none()`.
5. Add `Iec104` match arms to `on_data` and `on_flow_close`.
6. Re-run `verify_content_first_precedence_exhaustive` and confirm VERIFICATION SUCCESSFUL.

Failure to update `classify_oracle` atomically invalidates the VP-004 proof.
This mirrors the obligation documented in ADR-010 Decision 1 (ENIP).

### Decision 10: VP-007 T0881 six-part atomic obligation

Adding T0881 ("Service Stop", `MitreTactic::IcsInhibitResponseFunction`) to the MITRE
catalog triggers the VP-007 six-part atomic obligation (per ADR-010 §VP-007 decision):

1. Add `"T0881"` to `SEEDED_TECHNIQUE_IDS` array (28 → 29 entries).
2. Bump `SEEDED_TECHNIQUE_ID_COUNT` constant to 29.
3. Add `technique_info("T0881")` arm returning
   `("Service Stop", MitreTactic::IcsInhibitResponseFunction)`.
4. Add `"T0881"` to `EMITTED_IDS` (IEC-104 STOPDT findings emit this technique).
5. Verify `SEEDED_TECHNIQUE_IDS.len() == SEEDED_TECHNIQUE_ID_COUNT` (VP-007 drift guard).
6. Verify `technique_info` resolves all SEEDED IDs (VP-007 catalog completeness harness).

All six steps MUST be executed in a single commit. The VP-007 drift guard and Kani
harnesses will fail at CI if any step is missing.

Note: T1692.002 ("Unauthorized Message: Reporting Message") is NOT promoted to EMITTED
this cycle — M_* spoofed telemetry detection is staged but out of feature-iec104 scope.
`SEEDED_TECHNIQUE_IDS` and `SEEDED_TECHNIQUE_ID_COUNT` do NOT change for T1692.002.
The SEEDED count increment (28→29) is T0881-only.

## Rationale

Port-2404-only dispatch (Decision 1) is chosen over a content-signature rule on `0x68` because a
single start byte is not a reliable discriminator — `0x68` appears in HTTP headers, UTF-8 strings,
and many binary integer fields. The precedent from DNP3 (ADR-007) and EtherNet/IP (ADR-010)
confirms port-only dispatch is the established pattern for IANA-registered binary ICS protocols.

The 255-byte directional carry buffer (Decision 2) is derived directly from the APCI LEN field
upper bound (253) plus the 2-byte start/LEN prefix. No larger buffer is needed; no smaller buffer
is safe. Directional split follows RULING-DNP3-SIBLING-001 to prevent cross-direction splice.

The licensing constraint (Decision 7) is non-negotiable: all known IEC-104 open-source
implementations are either GPL (lib60870, Wireshark dissector) or proprietary (`iec60870-5`
crate). An original Rust parser is the only compliant path.

The pure-core free-fn design (Decision 8) is required for Kani amenability of VP-044
(`parse_apci_header` arithmetic safety). Functions with mutable state or I/O cannot be
formally verified with Kani's bounded model checking. The VP-046 tool choice (proptest rather
than Kani for `classify_frame_format` totality) reflects that proptest's exhaustive u8
strategy is sufficient for a simple bit-discriminant function; Kani would add no proof value.

## Consequences

### Positive

- IEC-104 analysis adds T0881 (Service Stop) to the emitted technique catalog, filling a
  significant gap for SCADA substation traffic analysis (STOPDT abuse is a documented
  adversarial technique in ICS environments).
- The 255-byte carry cap is the smallest of all binary ICS analyzers (DNP3: 292 bytes,
  ENIP: 600 bytes), minimizing memory overhead per flow.
- Pure-core `parse_apci_header` free fn is Kani-amenable (VP-044 arithmetic safety proof);
  `classify_frame_format` free fn is proptest-amenable (VP-046 totality over all 256 CF1
  values — proptest covers this directly without Kani overhead).
- Follows the established ADR-005/007/010 pattern with zero architectural surprise.

### Constraints Preserved

- VP-004 (Kani, P0): `classify()` content-first precedence invariant is preserved. Port
  2404 is not a TLS or HTTP port; content-signature Rules 1–2 fire before Rule 8. Oracle
  must be updated atomically (Decision 9).
- VP-007 (Kani, P0): MITRE technique ID format completeness preserved. T0881 must be added
  atomically (Decision 10).
- Licensing (Decision 7): no GPL code, no proprietary crate. Original Rust parser only.

### MITRE ATT&CK for ICS Technique Set (ics-attack-19.1)

| Technique ID | Name | When Emitted | Status |
|-------------|------|-------------|--------|
| T1692.001 | Unauthorized Message: Command Message | Control command TypeIDs 45–51 observed; N(S) desync | Pre-existing EMITTED |
| T0836 | Modify Parameter | set-point + bitstring writes (C_SE 48–50, C_BO 51) | Pre-existing EMITTED |
| T0831 | Manipulation of Control | C_SE + C_SC actuation on same flow | NOT emitted by IEC-104 this cycle (pre-existing EMITTED via Modbus analyzer; correlated C_SE+C_SC detection deferred to a future cycle) |
| **T0881** | **Service Stop** | **STOPDT-act observed; STOPDT without prior STARTDT** | **NEW — add via Decision 10** |
| T0814 | Denial of Service | Malformed APCI (LEN out of [4,253]; one finding per flow direction). Bad start byte: silent resync, no finding. Non-canonical U-frame CF1. | Pre-existing EMITTED |
| T1692.002 | Unauthorized Message: Reporting Message | M_* telemetry TypeIDs 1,3,5,9,11,13 | SEEDED; NOT emitted this cycle — M_* spoofed telemetry detection staged, out of feature-iec104 scope |
| T0827 | Loss of Control | Reset process command (C_RP_NA_1, TypeID 105) | Pre-existing EMITTED |

CWE set: CWE-306 (no authentication), CWE-319 (cleartext), CWE-294 (replay via N(S)/N(R)),
CWE-184 (incomplete disallowed-input list — non-canonical U-format), CWE-311 (no encryption).

### Verification Properties Registered

| VP | Tool | Phase | Property |
|----|------|-------|----------|
| VP-044 | Kani | P0 | `parse_apci_header` arithmetic safety — no panic, no OOB, LEN+2 in [6,255] |
| VP-045 | proptest | P1 | Directional carry-buffer isolation — carry_c2s/carry_s2c never mixed |
| VP-046 | proptest | P1 | U/S/I discrimination totality — `classify_frame_format` total over all 256 CF1 values |
| VP-047 | cargo-fuzz | P1 | APCI parser no-panic fuzz harness |

## Alternatives Considered

- **0x68 content-signature primary rule:** Single start-byte match promoted to a content rule
  ahead of port fallback. Rejected — `0x68` is the ASCII character `h` and appears in HTTP
  response bodies, UTF-8 strings, and many binary integer fields; it is not a reliable
  two-byte discriminator like TLS `0x16 0x03`. See Decision 1.

- **`iec60870-5` crate (crates.io):** Existing Rust implementation of IEC 60870-5 framing.
  Rejected — license is "NOT FREE for commercial/production use" (proprietary). See Decision 7.

- **Wireshark IEC-104 dissector (`packet-104.c`):** Mature, well-tested C implementation.
  Rejected — GPLv2 license is incompatible with this project. See Decision 7.

- **lib60870 (MZ Automation):** C library for IEC 60870-5-101/104. Rejected — GPLv3 license
  is incompatible. See Decision 7.

- **Kani for VP-046 (`classify_frame_format` totality):** Using Kani bounded model checking
  instead of proptest to verify the CF1 discriminant. Rejected — proptest's exhaustive u8
  strategy directly covers all 256 input values without the Kani compile-time overhead; Kani
  adds no proof value over proptest for a pure, branchless discriminant function. See Decision 8.

- **Single shared carry buffer (no directional split):** One carry buffer per flow rather than
  per-direction. Rejected — a single buffer cannot safely interleave C2S and S2C partial
  frames; cross-direction splice would corrupt both streams. Follows RULING-DNP3-SIBLING-001.
  See Decision 2.

## Source / Origin

- **IEC 60870-5-104:2006 standard** — APCI framing structure (§5.1), LEN field bounds [4,253],
  frame format discrimination from CF1 low bits (§5.2), N(S)/N(R) 15-bit counters, window
  parameter k=12, STARTDT/STOPDT/TESTFR CF1 values, TypeID ranges for control commands (45–51).
- **Feature cycle:** `feature-iec104` — this ADR governs the IEC-104 subsystem (SS-19)
  delivered in that cycle.
- **Predecessor ADRs:** ADR-005 (Modbus, port-fallback pattern), ADR-007 (DNP3, directional
  carry-buffer split + pure-core free-fn pattern), ADR-010 (EtherNet/IP, VP-007 atomic
  obligation pattern). ADR-013 follows all three predecessor patterns verbatim.
- **Behavioral contracts:** BC-2.19.* (SS-19 IEC-104 behavioral contracts); BC-2.05.012
  (VP-004 oracle obligation for Iec104 dispatch target); BC-2.10.010 (VP-007 T0881 atomic
  seeding obligation).
- **MITRE ATT&CK for ICS v19.1:** T0881 "Service Stop" (IcsInhibitResponseFunction TA0107);
  T1692.001 "Unauthorized Message: Command Message"; T0814 "Denial of Service"; T0836
  "Modify Parameter"; T0831 "Manipulation of Control"; T1692.002 "Unauthorized Message:
  Reporting Message" (seeded, not emitted this cycle); T0827 "Loss of Control".
- **Research:** `.factory/cycles/feature-iec104/research/f2-canonical-fact-validation.md`
  confirms TypeID 52 is RESERVED and T0881 is the correct MITRE technique for STOPDT abuse.
