---
document_type: adr
adr_id: ADR-010
status: proposed
accepted_date: null
date: 2026-06-24
modified: []
subsystems_affected:
  - SS-05
  - SS-10
  - SS-17
supersedes: null
superseded_by: null
feature_cycle: feature-enip-v0.11.0
issue: "#316"
mitre_pin: ics-attack-19.1
---

# ADR-010: Binary ICS Protocol Integration (EtherNet/IP + CIP TCP/44818 + UDP/2222, ODVA)

> **One-per-file:** Each architectural decision lives in its own file.
> Filename convention: `ADR-NNN-<short-name>.md`.
> ADR IDs are sequential 3-digit (ADR-001, ADR-002, ...). Once issued, never renumber.
> Lifecycle: `proposed` -> `accepted` -> (optional) `superseded` or `deprecated`.

## Context

wirerust's `StreamDispatcher` currently classifies TCP flows through six rules: two content
rules (TLS signature, HTTP method prefix) and four port fallback rules (443/8443 → TLS,
80/8080 → HTTP, 502 → Modbus [ADR-005], 20000 → DNP3 [ADR-007]), with an implicit "no
match" arm (Rule 7). Feature cycle `feature-enip-v0.11.0` (issue #316) introduces the
EtherNet/IP + Common Industrial Protocol (CIP) analyzer as subsystem SS-17. This is the
third binary ICS protocol integration and follows the architectural pattern established by
ADR-005 (Modbus TCP) and ADR-007 (DNP3 TCP).

### Protocol Structure: Two-Level Framing

EtherNet/IP (ENIP) is an ODVA-standardized protocol that wraps the Common Industrial
Protocol (CIP) in an Ethernet/TCP transport. Unlike Modbus (single MBAP header) or DNP3
(interleaved CRC blocks), EtherNet/IP introduces a **two-level frame structure**:

1. **ENIP encapsulation header** (24 bytes, fixed, big-endian): command (2), length (2),
   session_handle (4), status (4), sender_context (8), options (4). The `length` field
   is a u16 counting the payload bytes after the header — up to 65,511 bytes total.

2. **Common Packet Format (CPF) item list** (variable, little-endian): item_count (2 LE),
   followed by `item_count` items each with type_id (2 LE), item_length (2 LE), and
   item_data[0..item_length]. The CPF list is the payload of `SendRRData` (0x006F) and
   `SendUnitData` (0x0070) commands.

3. **CIP payload** (inside CPF item data): for Connected Data Items (0x00B1) and
   Unconnected Data Items (0x00B2), the CIP message begins with a 1-byte service code
   and a request path.

This three-layer structure (ENIP → CPF → CIP) requires a multi-step parse path that is
more complex than Modbus or DNP3. The architectural choices in this ADR govern how that
complexity is managed within wirerust's pure-core / effectful-shell boundary.

### Port Assignment

EtherNet/IP uses two IANA-registered ports:
- **TCP/44818** — explicit messaging (request/response, `SendRRData`, `SendUnitData`)
- **UDP/2222** — implicit/cyclic I/O (real-time I/O data, "implicit messaging")

The approved v0.11.0 MVP scope (F1 gate D-228) covers **TCP/44818 explicit messaging**.
UDP/2222 implicit I/O is deferred. This ADR covers only the TCP/44818 path; UDP/2222
is documented as a deferred decision (see Decision 6 below).

### Relationship to Prior ADRs

This ADR is the EtherNet/IP sibling of:
- ADR-005: Modbus TCP (port 502, Rule 5). Established the binary-ICS-port-fallback pattern.
- ADR-007: DNP3 TCP (port 20000, Rule 6). Extended the pattern with CRC-block-skip and
  FIR=1-only application parse.

No existing ADR is superseded. ADR-010 adds Rule 7 (port 44818) following the same
documented exception to ADR-0001 as its predecessors.

## Decision

We integrate EtherNet/IP + CIP analysis via seven coordinated decisions:

### Decision 1: Port-44818 TCP classification (Rule 7) — documented exception to ADR-0001

`DispatchTarget::Enip` is added as a sixth enum variant (after `Dnp3`). The `classify()`
function gains a **port-44818 arm placed as Rule 7** (after the existing Rule 6
port-20000/DNP3 arm) so no existing flow can be stolen. Rules 1–6 are unchanged. The
previous implicit "no match" (formerly the tail of the match) becomes Rule 8.

**Rule ordering after this change:**

| Rule | Type | Condition | Target |
|------|------|-----------|--------|
| 1 | Content | `data[0]==0x16 && data[1]==0x03` (TLS) | `DispatchTarget::Tls` |
| 2 | Content | HTTP method/version prefix | `DispatchTarget::Http` |
| 3 | Port fallback | ports contain 443 or 8443 | `DispatchTarget::Tls` |
| 4 | Port fallback | ports contain 80 or 8080 | `DispatchTarget::Http` |
| 5 | Port fallback | ports contain 502 (Modbus, ADR-005) | `DispatchTarget::Modbus` |
| 6 | Port fallback | ports contain 20000 (DNP3, ADR-007) | `DispatchTarget::Dnp3` |
| 7 | Port fallback | ports contain 44818 (EtherNet/IP, ADR-010) | `DispatchTarget::Enip` |
| 8 | Fallback | (no match) | `DispatchTarget::None` |

Content rules 1–2 take priority. A TLS ClientHello or HTTP request arriving on port 44818
routes correctly to Tls/Http before reaching Rule 7. Port 44818 without a content match is
classified as ENIP and routed to `EnipAnalyzer::on_data()` where the three-point
post-classification validity gate confirms or rejects the session.

**VP-004 oracle obligation:** The `classify_oracle` function in `dispatcher.rs`'s
`#[cfg(kani)] mod kani_proofs` MUST gain the port-44818 → Enip arm immediately after
the port-20000 → Dnp3 arm, with identical precedence logic. The Kani proof
`verify_content_first_precedence_exhaustive` asserts `got == want` across all port
combinations; oracle and production divergence causes this proof to fail at F6.

**`StreamDispatcher` struct delta:**

```rust
pub struct StreamDispatcher {
    http:    Option<HttpAnalyzer>,
    tls:     Option<TlsAnalyzer>,
    modbus:  Option<ModbusAnalyzer>,
    dnp3:    Option<Dnp3Analyzer>,
    enip:    Option<EnipAnalyzer>,   // NEW — SS-17
    routes:  HashMap<FlowKey, DispatchTarget>,
    classification_attempts: HashMap<FlowKey, u32>,
    max_classification_attempts: u32,
}
```

The early-exit guard that checks `is_none()` for all analyzers MUST be extended to include
`self.enip.is_none()` to prevent silent data-drop when only an ENIP analyzer is present
(mirrors ADR-007 DNP3 pattern).

### Decision 2: Two-level manual binary parser — ENIP encapsulation header + CPF item walk

`EnipAnalyzer` parses EtherNet/IP frames by directly indexing reassembled TCP byte streams.
No external ENIP or CIP crate is introduced. The project's zero-external-dependency
philosophy for analyzers is maintained.

**Rationale for rejecting ODVA crates:** The `rust-ethernet-ip` and `rseip-cip` crates are
CIP *client* libraries (encode+send+receive), not passive pcap decoders. Introducing them
would add production dependencies for code that is actively harmful in a passive-analysis
context (they attempt to establish real CIP connections). All existing ICS analyzers import
only stdlib + crate::analyzer.

**ENIP encapsulation header parse (pure-core free function):**

```
parse_enip_header(data: &[u8]) -> Option<EnipHeader>
  if data.len() < 24: return None
  command         = u16::from_be_bytes([data[0], data[1]])
  length          = u16::from_be_bytes([data[2], data[3]])
  session_handle  = u32::from_be_bytes([data[4]..data[8]])
  status          = u32::from_be_bytes([data[8]..data[12]])
  sender_context  = data[12..20]  (8-byte opaque, copy as [u8;8])
  options         = u32::from_be_bytes([data[20]..data[24]])
  return Some(EnipHeader { command, length, session_handle, status, sender_context, options })
```

All fields decoded big-endian per ODVA EtherNet/IP specification. `sender_context` is
treated as opaque bytes (not decoded further at the encapsulation layer).

**CPF item iteration (pure-core free function, called on SendRRData/SendUnitData payloads):**

```
parse_cpf_items(payload: &[u8]) -> Vec<CpfItem>
  if payload.len() < 2: return vec![]
  item_count = u16::from_le_bytes([payload[0], payload[1]])  // CPF uses little-endian
  cursor = 2
  for 0..item_count:
    if cursor + 4 > payload.len(): break
    type_id = u16::from_le_bytes([payload[cursor], payload[cursor+1]])
    length  = u16::from_le_bytes([payload[cursor+2], payload[cursor+3]])
    cursor += 4
    if cursor + length > payload.len(): break
    data = payload[cursor .. cursor + length]
    items.push(CpfItem { type_id, data })
    cursor += length
  return items
```

The CPF item iteration is bounded by the payload length; short/malformed payloads are
handled by early-break with no panic and no out-of-bounds access. The parsed item count
is bounded by the payload size (at minimum 4 bytes per item), preventing DoS via giant
declared item_count.

**CIP service extraction (from CPF item data for type_id 0x00B1/0x00B2):**

```
parse_cip_header(item_data: &[u8]) -> Option<CipHeader>
  if item_data.len() < 2: return None
  service            = item_data[0]          // raw service byte (request bit: high bit clear; response bit: high bit set)
  request_path_size  = item_data[1] as usize // in words (multiply by 2 for bytes)
  path_byte_count    = request_path_size * 2
  if item_data.len() < 2 + path_byte_count: return None
  request_path = item_data[2 .. 2 + path_byte_count]
  return Some(CipHeader { service, request_path })
```

### Decision 3: 600-byte carry buffer cap (MAX_ENIP_CARRY_BYTES = 600)

The per-flow carry buffer accumulates partial ENIP encapsulation headers until a complete
frame boundary is available, following the pattern of `ModbusFlowState.carry` (256 bytes)
and `Dnp3FlowState.carry` (292 bytes).

**Rationale for 600 bytes:**

The ENIP encapsulation header is 24 bytes fixed. The CPF overhead (item_count + first item
header) is 6 bytes. A typical explicit-messaging exchange (e.g., `SendRRData` with a single
unconnected CIP request containing a short path) fits well within 600 bytes:

- 24 bytes (ENIP header) + 2 (CPF item_count) + 4 (CPF item header) + ~50–200 bytes
  (CIP request with path and data) = 80–230 bytes total for a common CIP operation.

The ENIP `length` field is u16, theoretically allowing payloads up to 65,511 bytes. A 600-
byte cap is a deliberate MVP tradeoff:

1. **Per-flow memory bound:** Accepting unlimited ENIP payloads would require up to 65,535
   bytes per tracked flow carry buffer — a 100× increase in worst-case per-flow memory
   versus DNP3. At scale (many concurrent ENIP flows in a large pcap), this is untenable.

2. **Detection target coverage:** All detection targets in the v0.11.0 MVP (recon commands,
   CIP service codes, Reset, attribute writes, ForwardOpen) are found in the ENIP and CIP
   headers, not in large CIP data payloads. A 600-byte cap captures the full ENIP header
   (24 bytes) + CPF item header (6 bytes) + a substantial CIP payload segment, which is
   sufficient for all service-code and path-segment detections in scope.

3. **Consistency with existing pattern:** Carry buffer sizing in wirerust has always been
   bounded at the maximum meaningful frame unit, not at the theoretical maximum wire size.
   DNP3's 292-byte cap matches the maximum DNP3 link frame. For ENIP, 600 bytes matches the
   practical explicit-messaging exchange size for the services in scope.

4. **Mitigation for large payloads:** When the cumulative carry exceeds 600 bytes, the
   flow's `is_non_enip` latch is set to `true`, `parse_errors` is incremented, and all
   subsequent `on_data` calls for that flow are no-ops. This is not a false positive
   suppression — if an ENIP flow carries large CIP payloads that exceed the carry buffer,
   the flow is still analyzed up to the cap, and the parse_error counter in `summarize()`
   reflects the partial coverage.

```rust
const MAX_ENIP_CARRY_BYTES: usize = 600;
```

### Decision 4: EnipFlowState design and frame-walk algorithm

```rust
pub struct EnipFlowState {
    /// Partial ENIP frame accumulation buffer.
    /// Max 600 bytes (MAX_ENIP_CARRY_BYTES). Bounded DoS guard.
    carry: Vec<u8>,

    /// Set to true on desync (first 24 bytes do not form a plausible ENIP header,
    /// or carry buffer overflow). All subsequent on_data calls are no-ops.
    is_non_enip: bool,

    /// Counts of each ENIP command seen in this flow.
    command_counts: HashMap<u16, u64>,

    /// Count of CIP write-class services (SetAttribute*) in current 1-second window.
    write_count_in_window: u64,

    /// Timestamp (seconds) of the first write in the current 1-second window.
    write_window_start_ts: u32,

    /// Guard: write-burst T0836 finding already emitted for this window.
    write_burst_emitted: bool,

    /// Count of CIP error responses in current 10-second window, per general_status.
    error_counts_in_window: HashMap<u8, u64>,

    /// Timestamp of start of the 10-second error-rate window.
    error_window_start_ts: u32,

    /// Guard: CIP error-rate T0888 finding already emitted for this window.
    error_rate_emitted: bool,

    /// Count of malformed ENIP frames in current window (mirrors Dnp3FlowState).
    malformed_in_window: u64,

    /// Guard: T0814 malformed anomaly finding already emitted for this window.
    malformed_anomaly_emitted: bool,

    /// LIFETIME parse error counter. Never reset. Incremented for every frame
    /// that fails the post-classification validity gate or carry buffer overflow.
    parse_errors: u64,

    /// Total PDU count for this flow.
    pdu_count: u64,
}
```

**Frame-walk loop in `on_data()`:**

```
let buf = carry ++ new_data   (prepend carry buffer)
cursor = 0
while cursor < buf.len():
  if buf.len() - cursor < 24:
    carry = buf[cursor..]   // stash partial header
    if carry.len() > MAX_ENIP_CARRY_BYTES:
      flow.is_non_enip = true; flow.parse_errors++
    break

  header = parse_enip_header(&buf[cursor..cursor+24])
  if !is_valid_enip_frame(header):
    flow.parse_errors++; flow.malformed_in_window++; cursor += 1; continue

  total_frame_len = 24 + header.length as usize
  if buf.len() - cursor < total_frame_len:
    carry = buf[cursor..]   // stash partial frame
    if carry.len() > MAX_ENIP_CARRY_BYTES:
      flow.is_non_enip = true; flow.parse_errors++
    break

  process_pdu(&buf[cursor .. cursor + total_frame_len], &header, flow, findings)
  cursor += total_frame_len

carry = vec![]  // consumed all complete frames
```

### Decision 5: ForwardOpen connection-lifecycle tracking — IN-SCOPE for v0.11.0

The approved scope (F1 gate D-228) includes **CIP ForwardOpen connection-lifecycle tracking**
as a v0.11.0 deliverable. A ForwardOpen (CIP service 0x54) or LargeForwardOpen (0x5B)
establishes a CIP connection with specified connection parameters (O→T RPI, T→O RPI,
connection serial number, O→T connection ID, T→O connection ID). ForwardClose (0x4E)
tears down the connection.

For v0.11.0, the analyzer tracks ForwardOpen/LargeForwardOpen at the detection level:
- Detect and emit a finding when a ForwardOpen is seen from a previously unobserved
  source (connection establishment anomaly).
- Track the connection serial number for correlation with ForwardClose events.
- Do NOT attempt to maintain full connection state across the T→O and O→T channels —
  that requires UDP/2222 state which is deferred.

The MITRE technique gap for ForwardOpen (see Decision 7) governs what technique, if any,
is emitted.

### Decision 6: UDP/2222 implicit I/O — DEFERRED to post-v0.11.0

UDP/2222 implicit (cyclic) I/O is deferred from the v0.11.0 scope per F1 gate D-228.
EtherNet/IP UDP/2222 carries real-time I/O data with Assembly Object semantics: the
Controller→Device (O→T) channel carries output commands; the Device→Controller (T→O)
channel carries input status/feedback. Passive analysis of implicit messaging requires:

1. UDP flow reassembly (wirerust currently has TCP-only reassembly infrastructure).
2. Connection state correlating the ForwardOpen session (from TCP/44818) with the UDP/2222
   channel parameters (O→T/T→O connection IDs and RPIs established by ForwardOpen).
3. CIP Connection Manager object state tracking.

These are substantial additions outside the v0.11.0 MVP scope. The MITRE mappings for
UDP/2222 abuse (T1692.001 output injection, T1692.002 input spoofing) are documented here
for completeness but will not be emitted until UDP/2222 support is implemented.

**Deferred MITRE tags for UDP/2222:**
- Output injection (rogue O→T packets): T1692.001
- Input spoofing (rogue T→O status packets): T1692.002

### Decision 7: MITRE ICS technique set (v0.11.0 TCP/44818 scope, ics-attack-19.1)

All technique IDs are verified against ATT&CK for ICS v19.1 (pin: `ics-attack-19.1`,
released 2026-04-28). Source: `.factory/research/enip-mitre-ics-tagging.md` (2026-06-24).

#### Active technique set for v0.11.0

| Behavior | Technique ID | Name | Tactic | Confidence | CIP Trigger |
|----------|-------------|------|--------|-----------|-------------|
| CIP Stop service (halt PLC program) | **T0858** | Change Operating Mode | ICS Execution / ICS Evasion | High | FC 0x07 (Stop) |
| CIP Reset service | **T0816** | Device Restart/Shutdown | ICS Inhibit Response Function | High | FC 0x05 (Reset) |
| CIP firmware update / flash download | **T1693.001** | Modify Firmware: System Firmware | ICS Inhibit Response Function | High | FC 0x4B (Download) or vendor-specific |
| CIP ListIdentity (network-wide enum) | **T0846** | Remote System Discovery | ICS Discovery | High | ENIP cmd 0x0063 |
| CIP identity attribute read (single) | **T0888** | Remote System Information Discovery | ICS Discovery | High | GetAttributeSingle/All to Identity Object |
| CIP SetAttribute write | **T0836** | Modify Parameter | ICS Impair Process Control | High | FC 0x10 / 0x02 / 0x04 |

**Already seeded in `src/mitre.rs` (no new catalog entry required):**
T0846, T0888, T0836, T1692.001, T1692.002.

**New catalog entries required by v0.11.0 implementation:**
T0858, T0816, T1693.001.

#### Revoked IDs — do NOT seed or emit

| Revoked ID | Replacement | Action |
|------------|-------------|--------|
| T0857 System Firmware | T1693.001 | Do NOT seed T0857 — it is revoked in ics-attack-19.1 |
| T0855 Unauthorized Command Message | T1692.001 | Already replaced (issue #222) |
| T0856 Spoof Reporting Message | T1692.002 | Already replaced (issue #222) |

#### ForwardOpen anomaly — AMBIGUOUS (technique gap documented)

ATT&CK for ICS v19.1 contains **no technique specifically named for CIP connection
establishment anomalies or ForwardOpen abuse**. The closest available mappings are:

- As carrier for unauthorized commands from a rogue master: **T1692.001** (Unauthorized
  Message: Command Message — the T1692.001 detection guidance cites "new or unexpected
  connections to controllers via rogue masters").
- As reconnaissance: T0846 / T0888.

**Adopted policy for ForwardOpen findings (v0.11.0):**
- When a ForwardOpen is detected and the connection demonstrably carries an unauthorized
  CIP command in the same session, emit **T1692.001** on the command finding (not on the
  ForwardOpen itself).
- When a ForwardOpen anomaly is detected in isolation (no command payload yet observed),
  emit the finding with **no MITRE technique tag** (empty `mitre_techniques: vec![]`).
- Document the gap in each ForwardOpen finding's description: "No dedicated MITRE ICS
  technique for CIP connection establishment anomaly; T1692.001 applies only when the
  connection demonstrably carries an unauthorized command."

This is consistent with wirerust's finding attribution design (ADR-006) which supports
zero-technique findings. Do not invent a technique to fill the gap.

#### MitreTactic enum decision (VP-007 atomic obligation implication)

The existing `MitreTactic` enum in `src/mitre.rs` assigns exactly one tactic per technique
ID. The new techniques T0858 and T1693.001 present a multi-tactic challenge:

- T0858 "Change Operating Mode" — v19.1 live page lists **Execution (TA0104)** and
  **Evasion (TA0103)** as tactics.
- T1693.001 "Modify Firmware: System Firmware" — v19.1 live page lists **Persistence**,
  **Inhibit Response Function**, and **Impair Process Control** as tactics.

**Adopted decision: single primary tactic per the project's existing convention.**

The project maps each technique to one `MitreTactic` variant, choosing the most operationally
relevant tactic for wirerust's use case (passive network analysis of ICS traffic). The
multi-tactic MITRE page is authoritative for human-readable reports; wirerust's single-tactic
model is a simplification for programmatic grouping in the CLI reporter.

Recommended primary tactic assignments (to be finalized in the F4 implementation story):

| Technique | MITRE Live Tactics | Adopted Primary Tactic | MitreTactic Variant |
|-----------|-------------------|----------------------|---------------------|
| T0858 | Execution (TA0104), Evasion (TA0103) | Execution | `IcsExecution` (NEW VARIANT) |
| T0816 | Inhibit Response Function (TA0107) | Inhibit Response Function | `IcsInhibitResponseFunction` (existing) |
| T1693.001 | Persistence, Inhibit Response Function, Impair Process Control | Inhibit Response Function | `IcsInhibitResponseFunction` (existing) |

**New `MitreTactic` variant required:** `IcsExecution` for T0858. This follows the
precedent set by `IcsImpact` (added in ADR-007 for T0827) and `IcsDiscovery` (added in F5
for T0846/T0888). The new variant must carry the Display string `"Execution (ICS)"` to
distinguish it from Enterprise Execution (TA0002) per the D-069 pattern.

**VP-007 atomic obligation (5-part, mirrors ADR-007 Decision 5 playbook):**

The addition of T0858, T0816, and T1693.001 to `technique_info()` requires these five
changes in the **same commit burst** as the new technique arms:

1. **`technique_info` match arms:** Add `"T0858"` arm (T0858 name + `IcsExecution`),
   `"T0816"` arm (T0816 name + `IcsInhibitResponseFunction`), and `"T1693.001"` arm
   (T1693.001 name + `IcsInhibitResponseFunction`).

2. **`SEEDED_TECHNIQUE_IDS` array:** Add `"T0858"`, `"T0816"`, `"T1693.001"`.

3. **`SEEDED_TECHNIQUE_ID_COUNT` constant:** Bump 25 → 28 (adding 3 new entries).

4. **`EMITTED_IDS` in `kani_proofs` module:** Add `"T0858"` and `"T0816"` to the emitted
   set (the ENIP analyzer emits both). T1693.001 is seeded-only in v0.11.0 (firmware
   detection is staged; no BC in scope emits it yet). Do NOT add T1693.001 to EMITTED_IDS
   until the firmware-detection BC is implemented. Current emitted count is 17; ENIP v0.11.0
   adds T0858 + T0816 → **19 emitted IDs**. T0836/T0888/T0846 are already in EMITTED_IDS
   (Modbus / DNP3 / existing analyzers); reuse them without adding duplicates.

5. **`MitreTactic::IcsExecution` variant:** Add to the enum with `Display = "Execution (ICS)"`.
   Update `all_tactics_in_report_order()` (append after `IcsCommandAndControl`). Update
   `technique_tactic_id()` with `MitreTactic::IcsExecution => "TA0104"`. Run
   `cargo test mitre` to confirm `vp007_catalog_drift_guard` passes.

**Carried `all_tactics_in_report_order()` tail after this ADR:**

```rust
// (existing ICS variants, unchanged)
IcsInhibitResponseFunction,
IcsImpairProcessControl,
IcsImpact,
IcsDiscovery,
IcsCollection,
IcsCommandAndControl,
// NEW — ADR-010
IcsExecution,
```

### Decision 8: MVP CIP object-model scope (explicit depth limit)

The full CIP object model includes dozens of object classes (Connection Manager, Assembly,
Identity, Discrete I/O, Analog I/O, Motor Drive, etc.) with thousands of instance
attributes. v0.11.0 deliberately scopes to a minimal object-model depth:

**IN SCOPE (v0.11.0):**
- ENIP encapsulation header — all 10 fields parsed (BC-2.17.001/002)
- ENIP command classification — 9 recognized command values + Unknown (BC-2.17.004)
- CPF item iteration — item_count bounded walk, type_id recognition for 0x00B1/0x00B2
  (BC-2.17.005)
- CIP service code extraction and classification — 13 named services + Response mask +
  Unknown (BC-2.17.006/007)
- CIP error response detection — general_status extraction (BC-2.17.008)
- CIP request-path segment parse — Class and Instance segment type extraction only
  (BC-2.17.009)
- ForwardOpen connection establishment detection (BC-2.17.015, see Decision 5)

**DEFERRED (post-v0.11.0):**
- Full CIP Connection Manager state machine (ForwardOpen parameter tracking across
  T→O and O→T channels, Network Segment parse, Electronic Key Segment validation)
- Assembly Object attribute reads/writes (Attribute ID extraction)
- CIP Large Forward Open (0x5B) full parameter parse beyond service code classification
- CIP Multiple Service Packet (0x0A) recursion (nested CIP request decode)
- Any Vendor-Specific Object class traversal
- Firmware download full parameter extraction (file name, firmware version number)

### Decision 9: CLI wiring pattern (mirrors ADR-007 Decision 6)

- `--enip` boolean flag added to `Commands::Analyze` in `cli.rs` (default-off, included by
  `--all`)
- `--enip-write-burst-threshold` (u32, default: 20) — CIP write-class services per 1-second
  window threshold before T0836 write-burst finding is emitted
- When `--enip` is set without TCP reassembly, emit a WARNING and disable ENIP (same pattern
  as `--modbus` and `--dnp3`)
- `EnipAnalyzer` included in `needs_reassembly` alongside ModbusAnalyzer and Dnp3Analyzer
- `take_enip_analyzer()` on `StreamDispatcher` to collect findings and summary at the end of
  `run_analyze()`

## Rationale

**Port-only classification (Decision 1)** follows the ADR-005/ADR-007 precedent for binary
ICS protocols. EtherNet/IP has no stable content signature at TCP stream offset 0
(the ENIP encapsulation header begins with a command u16, not a fixed magic number), so
port-fallback is the only viable classification strategy. The post-classification validity
gate (is_valid_enip_frame) provides the compensating control.

**Two-level parser design (Decision 2)** is a direct consequence of ENIP's two-level
framing. The CPF item walk is architecturally separate from the ENIP header parse because
CPF uses little-endian byte order while the ENIP encapsulation header uses big-endian —
mixing these in a single function would be error-prone. Keeping them as separate pure-core
free functions is both easier to reason about and easier to verify with Kani.

**600-byte carry buffer (Decision 3)** is a conservative MVP tradeoff balancing memory
safety and detection coverage. It is not a protocol specification constraint; it is a
security policy decision. Any future version can increase the cap without breaking existing
BCs, as long as the cap remains bounded. The cap is explicitly named (`MAX_ENIP_CARRY_BYTES`)
to make this decision visible.

**ForwardOpen in-scope (Decision 5)** reflects the human-approved gate decision D-228.
Tracking ForwardOpen at the detection level (anomaly on unexpected connection establishment)
is simpler than full connection-state tracking and provides meaningful signal without
requiring UDP/2222 state correlation.

**MITRE technique gap documentation (Decision 7)** follows the VSDD principle that
architectural ambiguity must be made explicit rather than papered over with an imprecise
tag. Emitting T1692.001 on a bare ForwardOpen (with no command payload observed) would be
a speculative tag with Low confidence that could generate false positives. The adopted
policy is conservative and honest.

**Single primary tactic (Decision 7, MitreTactic decision)** is the lower-risk additive
option. Extending the enum to model multi-tactic assignments would require changes to the
reporter, the VP-007 Kani harness structure, and the BC-2.10.NNN postconditions — a
substantially larger change than adding a single new `IcsExecution` variant. The single-tactic
model is well-established in the codebase (ADR-005, ADR-007, ADR-008) and consistent with
wirerust's use of MITRE tactic as a grouping/reporting key rather than as a complete
representation of the ATT&CK matrix.

## Consequences

### Positive

- EtherNet/IP TCP flows on port 44818 are correctly routed and analyzed, enabling ICS/OT
  threat detection for T0858, T0816, T0836, T0846, T0888.
- The post-classification validity gate prevents ENIP findings from being emitted on
  non-ENIP binary traffic on port 44818.
- The 600-byte carry buffer prevents per-flow memory exhaustion while covering all MVP
  detection targets.
- The ForwardOpen technique-gap is explicitly documented — downstream consumers of ENIP
  findings can rely on accurate technique tags.
- VP-007 formal correctness is preserved after the 5-part atomic update (SEEDED 25 → 28,
  EMITTED 17 → 19).
- The `IcsExecution` MitreTactic variant makes the ICS Execution tactic (TA0104) first-class
  and testable, following the ADR-005/ADR-007 Matrix discriminator principle.

### Negative / Trade-offs

- Port-only classification means any non-ENIP binary protocol on port 44818 is mis-routed
  until the validity gate rejects its frames. This is the same accepted false-routing cost
  as Modbus and DNP3.
- The 600-byte carry buffer cap means ENIP flows with large CIP data payloads (e.g., large
  firmware download transfers) will be partially analyzed and flagged with parse_errors. For
  passive detection purposes this is acceptable; for completeness it is a known limitation.
- The two-level (ENIP→CPF) parse path is more complex than single-layer protocols. VP-032
  bounds this complexity with Kani proofs for the pure-core parse functions.
- `SEEDED_TECHNIQUE_ID_COUNT` (now 28 after this ADR) and `SEEDED_TECHNIQUE_IDS` must be
  updated atomically with each new `technique_info` arm; `vp007_catalog_drift_guard`
  enforces this mechanically.
- Adding `IcsExecution` to `MitreTactic` requires updating `fmt::Display`,
  `all_tactics_in_report_order()`, and `technique_tactic_id()`; any existing exhaustive
  match on `MitreTactic` in non-test code must be re-verified for exhaustiveness.
- UDP/2222 implicit I/O is deferred; ENIP-related detections targeting cyclic I/O abuse
  (T1692.001 output injection, T1692.002 input spoofing on the wire) are not available
  until a follow-on cycle adds UDP/2222 support.

### Open Items for F3 / Human Decision

- **`--enip-write-burst-threshold` default:** The proposed default is 20 (matching
  Modbus). CIP write commands in normal manufacturing operations may be more frequent
  than Modbus writes. F3 story decomposition should evaluate whether a higher default
  (e.g., 50) better matches the expected false-positive rate in typical ENIP traffic.
  Deferred from F2 per OQ-005 (F1 delta analysis open question).
- **T0858 `IcsExecution` enum addition:** Confirmed by this ADR as the correct design.
  The actual `enum` edit, `Display` update, `all_tactics_in_report_order()` update, and
  `technique_tactic_id()` update must be part of the VP-007 atomic burst in F4 STORY-EIP-09.
- **T1693.001 EMITTED_IDS timing:** Seeded in v0.11.0 but not added to EMITTED_IDS until
  a BC that emits firmware-detection findings is implemented. The BC for CIP firmware
  download detection is deferred from v0.11.0 scope (not in BC-2.17.001..024). Confirm
  this in F3 story decomposition.

## Alternatives Considered

**Content-at-bytes-0-1 classification:** ENIP has no stable magic bytes at TCP stream
offset 0. The ENIP command field (bytes 0–1) takes many valid values (0x0004, 0x0063,
0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075). Using a non-exhaustive
"is this one of these values?" content check at offset 0 would be unreliable and would
diverge from the established binary-ICS-port-fallback pattern. Rejected.

**Extend carry buffer to 4,096 or 65,535 bytes:** Would handle more large CIP payloads
but increases per-flow memory by 7× or 109× at worst case relative to the 600-byte cap.
The detection targets (service codes, path segments, command classifications) are all in
the first 200 bytes of a typical ENIP frame. Rejected for v0.11.0; deferred as a future
configuration option if large-payload detection is required.

**External ENIP/CIP parsing crate:** The `rust-ethernet-ip` and `rseip-cip` crates are
CIP client stacks, not passive parsers. Using them would introduce a production dependency
on code that actively tries to establish CIP network connections. Rejected per project
zero-external-dependency philosophy.

**Multi-tactic `MitreTactic` representation (enum extension vs. Vec<MitreTactic>):**
Changing `technique_info()` to return `Vec<MitreTactic>` would correctly model the
multi-tactic reality but would require cascading changes through the reporter, VP-007 harness,
all BC-2.10.NNN postconditions, and every `match` over the return type. The single-primary-
tactic approach is backwards-compatible and consistent with existing design. Deferred unless
a future cycle specifically requires multi-tactic grouping in reports.

## Source / Origin

- **EtherNet/IP wire format (ENIP encapsulation header, CPF framing, CIP service codes):**
  ODVA PUB00123R1 EtherNet/IP Specification (white paper); Wireshark dissectors
  `packet-enip.c` + `packet-cip.c` (open source, file-readable); IETF RFC 4897 (IANA
  port 44818 assignment). Confirmed in F1 delta analysis (enip-delta-analysis.md §3).
- **MITRE ATT&CK for ICS v19.1 technique set:** `.factory/research/enip-mitre-ics-tagging.md`
  (2026-06-24, verified live against attack.mitre.org); revocations T0855/T0856/T0857
  confirmed on ATT&CK updates page and consistent with issue #222 and
  `attack-ics-version-pin.md`.
- **600-byte carry buffer rationale:** enip-delta-analysis.md §10 (OQ-002) + F1 gate D-228
  human decision.
- **ForwardOpen in-scope:** F1 gate D-228 (human-approved scope).
- **ForwardOpen technique gap:** `.factory/research/enip-mitre-ics-tagging.md` §Flagged,
  behavior (7), verified 2026-06-24.
- **VP-004 oracle obligation precedent:** ADR-005 Decision 1; `.factory/STATE.md` D-032.
- **VP-007 atomic update obligation:** ADR-005 Decision 4; `.factory/STATE.md` D-033.
- **Port 44818 IANA registration:** https://www.iana.org/assignments/service-names-port-numbers
