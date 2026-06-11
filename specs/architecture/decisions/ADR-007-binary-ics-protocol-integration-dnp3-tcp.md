---
document_type: adr
adr_id: ADR-007
status: proposed
date: 2026-06-10
modified:
  - "2026-06-10 (Pass-2 remediation, issue #8): Added 6 correlation-state fields to Dnp3FlowState sketch in Decision 4 (restart_event_count, block_event_count, pending_requests, block_finding_emitted_this_window, loss_of_control_emitted, correlation_window_start_ts) and MAX_PENDING_REQUESTS constant. These fields are required by BC-2.15.011/014/015 for T1691.001 and T0827 detection. Added correlation-window reset note consistent with architecture-delta v1.1."
subsystems_affected:
  - SS-05
  - SS-10
  - SS-15
supersedes: null
superseded_by: null
---

# ADR-007: Binary ICS Protocol Integration (DNP3 TCP, IEEE 1815)

> **One-per-file:** Each architectural decision lives in its own file.
> Filename convention: `ADR-NNN-<short-name>.md`.
> ADR IDs are sequential 3-digit (ADR-001, ADR-002, ...). Once issued, never renumber.
> Lifecycle: `proposed` -> `accepted` -> (optional) `superseded` or `deprecated`.

## Context

wirerust's `StreamDispatcher` currently classifies TCP flows via the content-first
policy of ADR-0001 (TLS and HTTP content discriminators at stream offset 0) and two
binary-ICS port fallback rules added by ADR-005 (Modbus TCP, port 502, Rule 5). Issue #8
adds DNP3 (IEEE Std 1815-2012 / ICS/OT) as the second binary ICS protocol.

DNP3 over TCP runs on IANA-registered port 20000. Its link-layer frame is a 10-byte
header (8 octets + a 2-octet CRC) followed by data blocks that each carry 16 user octets
and a 2-octet CRC. The 2-octet start word `0x0564` (START1=0x05, START2=0x64) is stable
at bytes 0–1 of every link-layer frame — this is a content-level discriminator.

**However**, wirerust's content-first classification runs at TCP stream offset 0, on the
FIRST data chunk delivered to `on_data`. There is no guarantee that the first chunk
contains at least 2 bytes, and a partial delivery (1 byte) would yield a false negative
against the start-word check. The three-point post-classification validity gate (sync
0x0564 AND LENGTH in 5..=255 AND known/plausible link FC) provides the same compensating
control as ADR-005's Modbus gate and is the preferred mitigation.

**Established ADR-0001 exception (same class as Modbus):** A content-at-bytes-0-1 check
is theoretically possible for DNP3 (unlike Modbus, which has no stable offset-0
fingerprint). However, the same architectural concern that applies to Modbus port rules
applies here: port-fallback classification is the accepted pattern for binary ICS
protocols in wirerust's dispatcher. The 0x0564 sync word is instead used as the first
element of the post-classification validity gate, not as a content rule. This keeps the
classification path uniform and avoids introducing a hybrid partial-content rule that
would need its own retry-budget analysis. Per ADR-005 Decision 1 precedent, this
exception to ADR-0001 is accepted and documented here; the validity gate is the
compensating control.

A structurally important difference from Modbus: DNP3 interleaves 2-octet CRC values
after every 16 user-data octets (and after the 8-byte header). The reassembled TCP byte
stream therefore does NOT carry contiguous protocol data — it carries a DNP3 link frame
that is a sequence of (data-block, CRC) pairs. The parser must walk the block structure
to extract contiguous transport+application data, rather than advancing a simple offset
pointer as Modbus does. This block-walk is bounded by the LENGTH field (range 5..=255)
and produces at most 250 user octets from a single link frame. Application messages that
exceed one link frame use the transport-layer FIR/FIN bits for multi-frame sequencing;
v1 scope parses only the FIR=1 first fragment.

A further dimension concerns MITRE ATT&CK-ICS technique additions. The F1 scope
(dnp3-delta-analysis.md §6) originally listed T0803 as a new catalog entry. Research
(dnp3-research.md §6–§7) confirmed T0803 is REVOKED in the project's pinned version
ics-attack-19.1: it was replaced by T1691.001 "Block Operational Technology Message:
Command Message" (Inhibit Response Function, TA0107). Separately, the locked scope
mentioned T0828 as a "Loss of Control" technique — T0828 is "Loss of Productivity and
Revenue"; the correct technique is T0827 "Loss of Control" (Impact, TA0105). This ADR
documents the corrected technique set and the VP-007 atomic update obligation.

## Decision

We integrate DNP3 TCP analysis via five coordinated decisions:

### Decision 1: Port-20000 classification as documented exception to ADR-0001

`DispatchTarget::Dnp3` is added as a fifth enum variant (after `Modbus`, which was the
fourth). The `classify()` function gains a **port-20000 arm placed as Rule 6** (after the
existing Rule 5 port-502/Modbus arm) so no existing flow can be stolen. The VP-004
`classify_oracle` Kani harness MUST be extended with an identical port-20000 arm in
lockstep, per the ADR-005 precedent for the VP-004 oracle obligation.

**Rule ordering after this change:**

| Rule | Type | Condition | Target |
|------|------|-----------|--------|
| 1 | Content | `data[0]==0x16 && data[1]==0x03` (TLS) | `DispatchTarget::Tls` |
| 2 | Content | HTTP method/version prefix | `DispatchTarget::Http` |
| 3 | Port fallback | ports contain 443 or 8443 | `DispatchTarget::Tls` |
| 4 | Port fallback | ports contain 80 or 8080 | `DispatchTarget::Http` |
| 5 | Port fallback | ports contain 502 (Modbus, ADR-005) | `DispatchTarget::Modbus` |
| 6 | Port fallback | ports contain 20000 (DNP3, ADR-007) | `DispatchTarget::Dnp3` |
| 7 | Fallback | (no match) | `DispatchTarget::None` |

Rules 1–2 check data content and fire before any port rule. A TLS ClientHello or
HTTP GET arriving on port 20000 matches Rule 1 or 2 before reaching Rule 6 — this
is the INV-2 / VP-004 precedence guarantee. Rule 6 is the last port fallback rule;
`DispatchTarget::None` (formerly "Rule 6") becomes Rule 7 / the fallback.

The absence of a *content-level commitment* for DNP3 is mitigated by the three-point
post-classification validity gate in `Dnp3Analyzer::on_data` (Decision 3 below).

**VP-004 oracle obligation:** The `classify_oracle` function in `dispatcher.rs`'s
`#[cfg(kani)] mod kani_proofs` MUST gain the port-20000 → Dnp3 arm immediately after
the port-502 → Modbus arm, with identical precedence logic. The Kani proof
`verify_content_first_precedence_exhaustive` asserts `got == want` across all 65 536^2
port combinations; oracle and production divergence causes this proof to fail at F6.

**`StreamDispatcher` struct delta:**

```rust
pub struct StreamDispatcher {
    http:    Option<HttpAnalyzer>,
    tls:     Option<TlsAnalyzer>,
    modbus:  Option<ModbusAnalyzer>,
    dnp3:    Option<Dnp3Analyzer>,   // NEW — SS-15
    routes:  HashMap<FlowKey, DispatchTarget>,
    classification_attempts: HashMap<FlowKey, u32>,
    max_classification_attempts: u32,
}
```

The early-exit guard `if self.http.is_none() && self.tls.is_none() && self.modbus.is_none()`
MUST be extended to `&& self.dnp3.is_none()` (or refactored as per-arm `if let` checks)
to prevent silent data-drop when only a DNP3 analyzer is present.

### Decision 2: PDU-oriented manual binary parsing with no external crate

`Dnp3Analyzer` parses DNP3 link-layer frames by directly indexing the reassembled TCP
byte stream. No external DNP3 parsing crate is introduced.

**Frame consumption algorithm:**

Given `LENGTH` byte at stream offset 2, the total on-wire frame size is:

```
num_user_octets    = LENGTH - 5        (subtract CONTROL+DEST+SOURCE)
num_data_blocks    = ceil(num_user_octets / 16)   (0 when num_user_octets == 0)
frame_len          = 5 + LENGTH + 2 * ceil((LENGTH - 5) / 16)
```

Equivalent expanded form:

```
frame_len = 3                           // START1 + START2 + LENGTH
          + LENGTH                      // CONTROL + DEST + SOURCE + user data
          + 2                           // header CRC
          + 2 * num_data_blocks         // one 2-byte CRC per data block
```

**Maximum frame size:** LENGTH=255 → num_user_octets=250 → num_data_blocks=16
→ frame_len = 3 + 255 + 2 + 32 = 292 bytes. [SPEC: DNP Users Group Primer Rev A]

**Minimum frame size:** LENGTH=5 → num_user_octets=0 → num_data_blocks=0
→ frame_len = 3 + 5 + 2 + 0 = 10 bytes.

The per-frame carry buffer (`flow.carry: Vec<u8>`) accumulates partial TCP segments
until a complete frame boundary is available — the same pattern as Modbus
(`ModbusFlowState.carry`). The carry buffer is bounded by the maximum frame size
(292 bytes) per flow, preventing unbounded growth.

**Three-point post-classification validity gate** (`is_valid_dnp3_frame`):
1. `data[0] == 0x05 && data[1] == 0x64` — sync word present [SPEC]
2. `data[2] >= 5 && data[2] <= 255` — LENGTH in valid range [SPEC]
3. The link function code (CONTROL & 0x0F) is plausible (known primary or secondary FC)

Frames failing the gate are skipped (`parse_errors++`). This prevents false findings
from non-DNP3 traffic on port 20000.

**is_non_dnp3 desync-safe bail:** If a flow passes the port-20000 gate but the first
16 octets contain no valid DNP3 start word and LENGTH, the analyzer sets an
`is_non_dnp3: bool` flag on the flow state and all subsequent `on_data` calls for that
flow are no-ops. This mirrors the Modbus desync-bail pattern and prevents cascading
parse errors on misclassified flows.

### Decision 3: CRC-block-skip (structure-only, no CRC validation in v1)

DNP3 intersposes 2-octet CRCs after the 8-octet header and after every 16 user octets.
v1 does NOT validate these CRCs; it strips them structurally. CRC validation is deferred
to a later cycle.

**Block-walk to extract transport+application payload:**

Starting immediately after the 10-byte header (8 header bytes + 2 header CRC):
```
for block_index in 0..num_data_blocks:
    user_start = header_size + block_index * 18   // 18 = 16 user + 2 CRC
    block_user_len = min(16, remaining_user_octets)
    copy data[user_start .. user_start + block_user_len] → payload_buf
    skip 2-byte CRC at data[user_start + block_user_len]
```

The first byte of `payload_buf` is the **transport octet** (FIR bit = 0x40, FIN bit =
0x80, SEQ bits = 0x3F). The remaining bytes are the application-layer fragment.

This block-walk is the arithmetic target of VP-023 Sub-property D (frame_len formula
correctness — must not over-read or under-read the frame boundary).

### Decision 4: FIR=1 first-fragment application-layer parse only (no multi-frame reassembly)

The transport octet determines whether the following bytes are the start of a new
application fragment (FIR=1, `transport_octet & 0x40 != 0`). v1 extracts the
Application Control octet and Application Function Code ONLY from FIR=1 frames.
Continuation segments (FIR=0) are counted but not re-parsed as a function-code.

This is sufficient for all v1 detection targets: function codes always appear in the
first application fragment, never in a continuation fragment.

**Application parse path for FIR=1 frames** (from the reassembled `payload_buf`):
- Byte 0: transport octet (FIR=1 confirmed)
- Byte 1: Application Control (UNS bit = 0x10, CON bit = 0x20, App-FIN = 0x40, App-FIR = 0x80, SEQ = 0x0F)
- Byte 2: Application Function Code (FC)
- Bytes 3+: application objects (not parsed in v1)

**`Dnp3Analyzer` struct layout:**

```rust
pub struct Dnp3Analyzer {
    /// Per-flow state, keyed by FlowKey.
    flows: HashMap<FlowKey, Dnp3FlowState>,

    /// Direct-operate threshold: max DIRECT_OPERATE (0x05) + DIRECT_OPERATE_NR (0x06)
    /// function codes in the detection window before T1692.001 is emitted.
    /// Exposed via CLI --dnp3-direct-operate-threshold. Default = N (to be pinned in F3).
    direct_operate_threshold: u32,

    /// Aggregate function-code distribution across all flows: FC byte → count.
    fn_code_counts: HashMap<u8, u64>,
}

pub struct Dnp3FlowState {
    /// Partial frame accumulation buffer. Max 292 bytes.
    carry: Vec<u8>,

    /// Set to true on desync; all subsequent on_data calls are no-ops.
    is_non_dnp3: bool,

    /// Counts of each application FC seen in this flow.
    fc_counts: HashMap<u8, u64>,

    /// Direct-operate count in the current detection window.
    direct_operate_count: u32,

    /// Timestamp of the first direct-operate in the current window (seconds).
    window_start_ts: u32,

    /// Guard: T1692.001 already emitted for this window.
    direct_operate_emitted: bool,

    /// Source link addresses seen claiming DIR=1 (master direction).
    master_addrs_seen: Vec<u16>,  // bounded to MAX_MASTER_ADDRS

    /// Total parse errors (invalid frames, CRC-block overruns, etc.).
    parse_errors: u64,

    /// Total frames analyzed.
    frame_count: u64,

    // ---- Correlation-window state (BC-2.15.011 / BC-2.15.014 / BC-2.15.015) ----
    // All six fields reset together at correlation-window expiry (single window,
    // default 300s [F2-GATE]).  Distinct from the direct-operate burst window
    // (window_start_ts / direct_operate_count, 60s).

    /// COLD/WARM restart event accumulator within the correlation window.
    /// Contributes to the T0827 Loss of Control derivation threshold (BC-2.15.011).
    restart_event_count: u64,

    /// Block-timeout accumulator: control requests without a RESPONSE within the
    /// T1691.001 inference timeout.  Feeds T1691.001 threshold AND T0827
    /// derivation (BC-2.15.014).
    block_event_count: u64,

    /// Outstanding control requests awaiting RESPONSE, keyed (destination_addr,
    /// app_seq) → observation_timestamp_secs.  Bounded to MAX_PENDING_REQUESTS
    /// (evicts oldest on overflow, mirrors Modbus pending-table DoS bound).
    /// Used for passive T1691.001 request/response correlation (BC-2.15.014).
    pending_requests: HashMap<(u16, u8), u32>,

    /// One-shot guard: T1691.001 finding already emitted this window (BC-2.15.014).
    /// Reset at correlation-window expiry.
    block_finding_emitted_this_window: bool,

    /// One-shot guard: T0827 finding already emitted this window (BC-2.15.015).
    /// Reset at correlation-window expiry.
    loss_of_control_emitted: bool,

    /// Start timestamp (seconds) of the current correlation window.
    /// Window default 300s [F2-GATE]; exact threshold pinned in F3 BCs.
    correlation_window_start_ts: u32,
}
```

**Additional bounded-resource constant (Pass-2, mirrors architecture-delta v1.1):**

```rust
/// Maximum outstanding pending control requests per flow for T1691.001
/// request/response correlation.  Oldest entry evicted on overflow.
const MAX_PENDING_REQUESTS: usize = 256;
```

### Decision 5: ICS-matrix MITRE technique additions — corrected technique set

**Corrected v19.1 technique set for DNP3:**

| Technique ID (v19.1) | Name | Tactic | DNP3 trigger | Legacy ID (revoked) |
|----------------------|------|--------|--------------|---------------------|
| T1692.001 | Unauthorized Message: Command Message | IcsImpairProcessControl | SELECT/OPERATE/DIRECT_OPERATE/DIRECT_OPERATE_NR from non-allowlisted source | T0855 (revoked) |
| T1691.001 | Block Operational Technology Message: Command Message | IcsInhibitResponseFunction | Request-without-response correlation (passive inference) | T0803 (revoked) |
| T0814 | Denial of Service | IcsInhibitResponseFunction | COLD_RESTART/WARM_RESTART | (unchanged) |
| T0836 | Modify Parameter | IcsImpairProcessControl | WRITE (0x02) | (unchanged) |
| T0827 | Loss of Control | IcsImpact (NEW variant) | Derived/correlated from sustained T0814 + T1691.001 conditions | (T0827 was incorrectly referenced as T0828 in F1; T0828 = "Loss of Productivity and Revenue" — different concept) |

**T1691.001 is a NEW catalog entry.** T0803 was the expected addition per F1, but T0803
is REVOKED in ics-attack-19.1; T1691.001 is the canonical replacement (created
2026-04-20, last modified 2026-05-12, tactic: Inhibit Response Function). The
`technique_info` arm must carry T1691.001, not T0803.

**T0827 requires a NEW `MitreTactic` variant: `IcsImpact`.** See §MitreTactic Enum
Delta below.

**MitreTactic enum delta — one new variant:**

T0827 "Loss of Control" belongs to tactic **Impact (TA0105)** in the ICS matrix. The
existing `MitreTactic::Impact` variant is the Enterprise Impact tactic; the ICS Impact
tactic (TA0105) is a distinct concept. To keep matrix affiliations unambiguous (per
ADR-005 Decision 4's Matrix discriminator principle), a new variant
`IcsImpact` is added to the enum:

```rust
// BEFORE (existing variants):
IcsInhibitResponseFunction,
IcsImpairProcessControl,

// AFTER (add one new variant for Issue #8 DNP3):
IcsInhibitResponseFunction,
IcsImpairProcessControl,
IcsImpact,  // NEW — ICS Impact tactic (TA0105): T0827 Loss of Control
```

The `fmt::Display` implementation gains:
```rust
MitreTactic::IcsImpact => "Impact",
```

The `all_tactics_in_report_order()` slice gains `MitreTactic::IcsImpact` appended last
(after `IcsImpairProcessControl`).

**VP-007 atomic update obligation (5-part, mirrors ADR-005 / D-032 playbook):**

The addition of T1691.001 and T0827 to `technique_info` requires these five changes in
the SAME commit:

1. **`technique_info` match arms:** Add `"T1691.001"` arm with
   `("Block Operational Technology Message: Command Message", MitreTactic::IcsInhibitResponseFunction)`
   and `"T0827"` arm with `("Loss of Control", MitreTactic::IcsImpact)`.

2. **`SEEDED_TECHNIQUE_IDS` array:** Add `"T1691.001"` and `"T0827"` to the array.

3. **`SEEDED_TECHNIQUE_ID_COUNT` constant:** Bump 21 → 23
   (adding T1691.001 + T0827; two new entries).

4. **`EMITTED_IDS` in `kani_proofs` module:** Add `"T1691.001"` and `"T0827"` to the
   emitted set (DNP3 analyzer will emit both). Current emitted count is 13 (6
   Enterprise + 7 ICS); DNP3 adds 2 new ICS emissions → 13 + 2 = **15 emitted IDs**.
   `"T1692.001"` remains in EMITTED_IDS (already emitted by Modbus; DNP3 co-emits it).

5. **Run `cargo test mitre`** before the PR merges to confirm
   `vp007_catalog_drift_guard` passes. The drift guard's exhaustive 10-million-ID
   sweep will mechanically fail if any arm is added without mirroring in
   `SEEDED_TECHNIQUE_IDS`.

**Emission summary (post-DNP3 v1):**

| ID | Emitted by | Status |
|----|-----------|--------|
| T1692.001 | Modbus (write-burst) + DNP3 (unauthorized control) | Already in EMITTED_IDS |
| T0836 | Modbus (WRITE FC) + DNP3 (WRITE 0x02) | Already in EMITTED_IDS |
| T0814 | Modbus (force-listen-only) + DNP3 (COLD/WARM_RESTART) | Already in EMITTED_IDS |
| T1691.001 | DNP3 (request-without-response inference) | **NEW — add to EMITTED_IDS** |
| T0827 | DNP3 (correlated loss-of-control impact finding) | **NEW — add to EMITTED_IDS** |

**Note on T0803 (revoked):** Per F1 delta analysis §6, T0803 was the anticipated new
catalog entry. T0803 is REVOKED in ics-attack-19.1 (replaced by T1691.001). Do NOT add
T0803 to the catalog; add T1691.001 instead. The `attack-ics-version-pin.md` document is
stale on T0855 (still listed as Active); this is a validated `DF-VALIDATION-001` finding
eligible to be filed as a GitHub issue. The DNP3 F2 burst does not update
`attack-ics-version-pin.md` — that is a separate maintenance task for the human/PO.

**Note on T0827 emission policy:** T0827 is an **Impact-tactic outcome**, not a
per-packet detection. It must be emitted as a **correlated/derived finding** — the
consequence of observing sustained T0814 (restart abuse) or T1691.001 (blocked commands)
conditions, not from a single packet. The detection logic must implement a guard (e.g.,
N restart events or M request-without-response sequences within a time window) before
emitting T0827.

### Decision 6: CLI threshold flag

`--dnp3-direct-operate-threshold` is added to `Commands::Analyze` in `cli.rs`,
mirroring `--modbus-write-burst-threshold`. Default value is pinned in F3 based on
protocol norms (F2 open item). Type: `u32`. The flag controls the `direct_operate_threshold`
field of `Dnp3Analyzer`.

## Rationale

**Port-only classification (Decision 1)** follows the ADR-005 precedent. DNP3 has a
stable byte-0-1 fingerprint (0x0564), but the content-at-bytes-0-1 check is deferred to
the validity gate to keep classification uniform across binary ICS protocols and avoid
a partial-content retry-budget problem. The 0x0564 sync word as the first gate element
is a stronger compensating control than Modbus's Protocol-ID check (which requires bytes
2–3 and still has false-positive risk on any binary data).

**Manual binary parsing (Decision 2)** avoids introducing an external DNP3 crate
dependency. The frame format is complex (interleaved CRCs) but fully deterministic given
the LENGTH field. The block-walk algorithm is short and formally verifiable (VP-023
Sub-property D). The carry-buffer pattern is already established by Modbus and avoids
fragmented-frame mis-parses.

**CRC-skip (Decision 3)** matches the v1 scope mandate (dnp3-delta-analysis.md
Decision 2). CRC validation adds complexity and a pure-function implementation of
CRC-16/DNP (poly 0x3D65, refin=true, refout=true, xorout=0xFFFF) is a future cycle
enhancement. CRC-skip is safe for PCAP replay of real captures (corrupt CRC packets are
rare in captures of real traffic).

**FIR=1-only parse (Decision 4)** is sufficient for all v1 detections (unauthorized
commands, restarts, writes). Application function codes always appear in the first
application fragment. Full reassembly adds per-flow state complexity with no detection
benefit in v1.

**Corrected MITRE technique set (Decision 5)** uses the v19.1-canonical IDs per the
research findings in dnp3-research.md §6–§7. Using revoked IDs (T0803, T0855) in new
code would produce findings with invalid technique IDs relative to the pinned catalog
version. The new `IcsImpact` variant cleanly separates ICS Impact (T0827, TA0105) from
Enterprise Impact (T1499.002, etc.).

## Consequences

### Positive

- DNP3 TCP flows on port 20000 are correctly routed and analyzed, enabling ICS/OT threat
  detection for T1692.001, T1691.001, T0814, T0836, and T0827.
- The three-point validity gate prevents DNP3 findings from being emitted on non-DNP3
  binary traffic on port 20000.
- Carry-buffer + FIR=1 parse correctly handles cross-segment frames and single/multi-link
  application messages with consistent FC extraction.
- The `IcsImpact` MitreTactic variant makes the ICS Impact tactic first-class and
  testable, following the ADR-005 Matrix discriminator principle.
- VP-004 formal correctness is preserved: the extended `classify_oracle` (port-20000
  → Dnp3) mirrors production exactly.
- VP-007 formal correctness is preserved after the 5-part atomic update
  (SEEDED 21 → 23, EMITTED 13 → 15).

### Negative / Trade-offs

- Port-only classification for DNP3 means any non-DNP3 binary protocol on port 20000 is
  mis-routed to `Dnp3Analyzer` until the validity gate rejects its frames. This is the
  same accepted false-routing cost as Modbus (port 502). The sync-word gate is a strong
  compensating control.
- The block-walk CRC-stripping adds per-frame complexity relative to Modbus's
  simple offset advancement. The VP-023 Sub-property D Kani proof bounds this
  arithmetic to prevent over/under-read.
- T0827 "Loss of Control" is an Impact-tactic correlated finding, not a per-packet
  detector. Its emission requires a multi-event window; misconfiguring the window
  threshold produces either false positives (too low) or missed detections (too high).
- `SEEDED_TECHNIQUE_ID_COUNT` (now 23 after this ADR) and `SEEDED_TECHNIQUE_IDS`
  must be updated atomically with each new `technique_info` arm;
  `vp007_catalog_drift_guard` enforces this but requires discipline.
- Adding `IcsImpact` to `MitreTactic` requires updating `fmt::Display` and
  `all_tactics_in_report_order()`; any existing `match` on `MitreTactic` in non-test
  code must be checked for exhaustiveness (the `#[non_exhaustive]` attribute prevents
  external crates from hitting a compile error but warns on internal incomplete matches).

### Open Items for F3 / Human Decision

- **Default for `--dnp3-direct-operate-threshold`**: pinned in F3. No spec-defined rate;
  recommend a value in the 5–20 range per JUDGMENT (dnp3-research.md §5.1).
- **T1691.001 request-without-response timeout**: the passive inference for "blocked
  command" requires a (flow_key + app_seq + dest_addr) correlation key and a timeout
  window. F2 open item; pinned in F3 BCs.
- **T0827 emission guard thresholds**: number of T0814 or T1691.001 events required to
  derive T0827. F2 open item; pinned in F3 BCs.

### Status as of 2026-06-10

Proposed. Implementation has not yet begun (Feature cycle Issue #8, F2 architecture
delta phase). VP-004 and VP-007 Kani proof updates are part of the F3 implementation
stories for `dispatcher.rs` and `mitre.rs` respectively. VP-023 Kani proof is an F3/F4
story for `src/analyzer/dnp3.rs`.

## Alternatives Considered

- **Content-at-bytes-0-1 classification (0x0564 as content rule):** Would add a Rule 1.5
  or Rule 3.5 checking the DNP3 sync word before the port fallback. Rejected because it
  requires a minimum 2-byte chunk at stream offset 0 (not guaranteed), introduces a
  partial-content edge case the retry budget does not address for successful-but-wrong
  classifications, and diverges from the binary-ICS-port-fallback pattern established
  by Modbus. The sync word is more usefully placed as the validity gate's first arm.

- **Full application-layer reassembly (multi-link fragment reassembly):** Buffer all
  transport segments by (src_addr, dst_addr, transport_seq) and deliver the complete
  application message. Rejected for v1: the detection targets (FC-based: control
  commands, restarts, writes) all fire on the first fragment's FC. Full reassembly
  adds per-flow state proportional to message size with no v1 detection benefit.

- **Retain T0803 (revoked) as back-compat alias:** Emit both T0803 and T1691.001.
  Rejected: T0803 is revoked in the pinned version (ics-attack-19.1). Emitting a
  revoked ID in new code would produce findings that MITRE tools/consumers may flag as
  invalid. Emit T1691.001 only; document the lineage in comments.

- **Introduce T0827 as a per-packet rule (single COLD_RESTART → IcsImpact):**
  Rejected: T0827 is an Impact-tactic outcome, not a method. Single-packet mapping
  would conflate the detection signal (T0814 DoS) with the resulting impact (T0827
  loss of control), producing misleading findings. T0827 must be a derived/correlated
  finding from a sustained pattern.

## Source / Origin

- **DNP3 wire format (LENGTH, frame_len, CRC blocks, transport/app layers):**
  IEEE Std 1815-2012, DNP Users Group Primer Rev A, RACOM DNP3 reference, Chipkin
  AN2013-004b. All confirmed in `.factory/research/dnp3-research.md` §1–§3.
- **Maximum frame size 292 bytes:** DNP Users Group Primer Rev A verbatim; confirmed in
  `.factory/research/dnp3-research.md` §1.4.
- **MITRE ATT&CK-ICS v19.1 technique table (T1691.001, T1692.001, T0827, T0814, T0836):**
  `.factory/research/dnp3-research.md` §5–§7 (confirmed against attack.mitre.org +
  v18.1→v19.0 detailed changelog).
- **VP-004 oracle obligation precedent:** ADR-005 Decision 1; `.factory/STATE.md` D-032.
- **VP-007 atomic update obligation:** ADR-005 Decision 4; `.factory/STATE.md` D-033.
- **StreamDispatcher struct delta and carry-buffer pattern:** `src/dispatcher.rs`
  (Modbus arm); `.factory/phase-f2-spec-evolution/architecture-delta.md` §2.
