---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-15
capability: CAP-15
lifecycle_status: active
introduced: v0.6.0-feature-008
modified:
  - "v1.3: F3 story-anchor back-fill; Invariant 4 grammar fix ('must not be descend into' → 'must not be descended into') and link-FC 0x0 PRM-bit disambiguation. — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/research/dnp3-research.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
input-hash: TBD
---

# BC-2.15.008: Transport FIR=1 First-Fragment Gates Application-Layer FC Extraction

## Description

After stripping the 10-byte header from a valid DNP3 link frame, the analyzer checks whether
the first user-data byte (the transport octet) has FIR=1 (bit 6 set: `transport_octet & 0x40
!= 0`). Only FIR=1 frames carry the start of a new application fragment; only on FIR=1 frames
does the analyzer extract the Application Control octet (byte index 1 of the payload) and
Application Function Code (byte index 2 of the payload) for classification and detection.
Continuation segments (FIR=0) are counted but not re-parsed as new requests.

## Preconditions

1. The validity gate (BC-2.15.004) has returned `true` for the current frame.
2. The link layer function code (CONTROL & 0x0F) is CONFIRMED_USER_DATA (0x03) or
   UNCONFIRMED_USER_DATA (0x04) — only these link FCs carry a transport+application payload.
   [SPEC: dnp3-research.md §1.2]
3. After CRC-block stripping (ADR-007 Decision 3), `payload_buf` contains at least 3 bytes
   (transport octet + App Control + App FC).
4. `flow.is_non_dnp3 == false` (not a desync-bailed flow).

## Postconditions

**When transport_octet & 0x40 != 0 (FIR=1):**
1. The application function code is extracted from `payload_buf[2]`.
2. `classify_dnp3_fc(payload_buf[2])` is called; the result drives the detection logic
   (BC-2.15.010 through BC-2.15.013).
3. `flow.fc_counts.entry(payload_buf[2]).or_insert(0) += 1` — per-flow FC distribution updated.
4. `self.fn_code_counts.entry(payload_buf[2]).or_insert(0) += 1` — aggregate FC distribution updated.

**When transport_octet & 0x40 == 0 (FIR=0, continuation segment):**
5. No application FC is extracted.
6. No finding is emitted for this segment's content.
7. `flow.frame_count` is still incremented (frame was processed).

## Invariants

1. **FIR bit is bit 6 of the transport octet** [SPEC: dnp3-research.md §2]:
   `transport_is_fir(octet) = octet & 0x40 != 0`. Mask 0x40 = 0b01000000.
2. **FIR=0 segments are NOT re-parsed**: application bytes in a continuation segment are
   continuation data for a multi-link fragment, not fresh App Control + App FC bytes. Parsing
   them as FC would produce incorrect findings. [ADR-007 Decision 4]
3. **Single-frame fragments**: when both FIR=1 and FIN=1 in the same transport octet
   (`octet & 0xC0 == 0xC0`), the fragment is complete in one link frame. This is the common
   case for typical control commands. [SPEC]
4. **Link FC guard**: the transport+application layer is only present in CONFIRMED_USER_DATA
   (0x03) and UNCONFIRMED_USER_DATA (0x04) link frames. Other link FCs (e.g., RESET_LINK =
   primary link-FC 0x0; ACK = secondary link-FC 0x0, disambiguated by the PRM bit) carry no
   transport octet and must not be descended into. [SPEC: dnp3-research.md §1.2]

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | transport_octet = 0xC0 (FIR=1, FIN=1, SEQ=0) | FIR=1: extract App FC; single-frame fragment (common case) |
| EC-002 | transport_octet = 0x40 (FIR=1, FIN=0, SEQ=0) | FIR=1: extract App FC; multi-link fragment, first frame |
| EC-003 | transport_octet = 0x80 (FIR=0, FIN=1, SEQ=0) | FIR=0: skip App FC extraction; last segment of multi-link |
| EC-004 | transport_octet = 0x00 (FIR=0, FIN=0, SEQ=0) | FIR=0: skip App FC extraction; middle continuation segment |
| EC-005 | Link FC = 0x00 (RESET_LINK), not CONFIRMED/UNCONFIRMED_USER_DATA | No transport octet present; do NOT descend into app layer |
| EC-006 | payload_buf has fewer than 3 bytes after CRC strip (malformed payload) | No App FC extraction; increment parse_errors; skip detection |

## Canonical Test Vectors

Byte layout for a valid FIR=1 DIRECT_OPERATE frame (after header):
```
DNP3 link frame:
  05 64 0E C4 03 00 01 00  [header CRC]  ← 10-byte header
  C0 81  05  [app objects...]  [data CRC]  ← data block 1

transport_octet = 0xC0 → FIR=1 (bit 6 set), FIN=1 (bit 7 set)
app_control     = 0x81 → App Control (FIR=1, FIN=1, CON=0, UNS=0, SEQ=1)
app_fc          = 0x05 → DIRECT_OPERATE → classify_dnp3_fc(0x05) = Control
```

| Scenario | transport_octet | Payload[2] (App FC) | Expected behavior |
|----------|----------------|-------------------|-------------------|
| DIRECT_OPERATE, single frame | 0xC0 | 0x05 | FIR=1: extract FC=0x05, classify→Control, detection fires |
| COLD_RESTART, single frame | 0xC0 | 0x0D | FIR=1: extract FC=0x0D, classify→Restart, detection fires |
| Continuation segment | 0x80 | (data, not FC) | FIR=0: no extraction, no detection |
| WRITE, single frame | 0xC0 | 0x02 | FIR=1: extract FC=0x02, classify→Write, detection fires |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | FIR extraction is a 1-liner (`octet & 0x40 != 0`); no Kani target. Covered by unit tests. | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — FIR=1 gating is the mechanism by which the DNP3/ICS analyzer extracts application function codes from reassembled TCP streams; without this gate, the analyzer would incorrectly parse continuation data as new FC bytes, producing false-positive detections |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — FIR=1 gating ensures application-layer parsing only fires on structurally valid application fragment starts) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24 `on_data`); ADR-007 Decision 4 |
| Stories | STORY-106 |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — gating logic; detection BCs BC-2.15.010–013 are the emitters) |

## Related BCs

- BC-2.15.004 — depends on (validity gate is a precondition for entering the transport-layer parse path)
- BC-2.15.007 — depends on (frame_len determines carry-buffer boundary; transport octet is first byte after CRC-stripped header)
- BC-2.15.010 — depends on (FIR=1 gate enables FC extraction that drives Control detection)
- BC-2.15.011 — depends on (FIR=1 gate enables Restart detection)
- BC-2.15.012 — depends on (FIR=1 gate enables Write detection)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `fn transport_is_fir(transport_octet: u8) -> bool { transport_octet & 0x40 != 0 }` (trivial 1-liner, pure)
- `src/analyzer/dnp3.rs` — `fn has_user_data(control: u8) -> bool { let link_fc = control & 0x0F; link_fc == 0x03 || link_fc == 0x04 }` (trivial, pure)
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §6` — transport octet helpers
- `.factory/research/dnp3-research.md §2` — FIR=1 (bit 6, mask 0x40) [SPEC]
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 4` — "FIR=1 first-fragment application-layer parse only"

## Story Anchor

STORY-106

## VP Anchors

(none — unit test coverage only; transport_is_fir is a trivial 1-liner per dnp3-architecture-delta.md §11)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-research.md §2 (transport octet: bit 6 = FIR, mask 0x40); ADR-007 Decision 4; dnp3-architecture-delta.md §6 |
| **Confidence** | high — FIR bit position confirmed [SPEC] by Wireshark dissector, CISA icsnpp-dnp3, Triangle MicroWorks, and dnp3-research.md §2 |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (FIR check is pure; effects are in on_data caller) |
| **Global state access** | none |
| **Deterministic** | yes — same transport octet always produces same FIR result |
| **Thread safety** | Send + Sync (gate function pure; on_data effectful shell) |
| **Overall classification** | gate: pure 1-liner; caller (on_data): effectful shell |
