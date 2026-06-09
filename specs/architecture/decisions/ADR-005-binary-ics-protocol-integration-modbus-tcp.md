---
document_type: adr
adr_id: ADR-005
status: proposed
date: 2026-06-09
subsystems_affected:
  - SS-05
  - SS-10
  - SS-14
supersedes: null
superseded_by: null
---

# ADR-005: Binary ICS Protocol Integration (Modbus TCP)

> **Superseded in part by ADR-006 (multi-technique Finding attribution) and
> f2-fix-directives.md v2 (Decisions 11, 12, 13).**
> Decision 3 (write-threshold model) is corrected to the dual-window model.
> Decision 4 (MITRE catalog) is corrected: T0846 → T0888 as recon emitter; SEEDED 15→21.
> `mitre_technique: Option<String>` (referenced in Decision 4 and Consequences) is
> superseded by `mitre_techniques: Vec<String>` per ADR-006.
> The corrected authoritative state is documented below. See ADR-006 for the full
> rationale for the multi-tag Finding type change.

> **One-per-file:** Each architectural decision lives in its own file.
> Filename convention: `ADR-NNN-<short-name>.md` (e.g., `ADR-001-rust-dispatcher.md`)
> ADR IDs are sequential 3-digit (`ADR-001`, `ADR-002`, ...). Once issued, never renumber.
> Lifecycle: `proposed` -> `accepted` -> (optional) `superseded` or `deprecated`.
> Frontmatter `subsystems_affected` is an array of `SS-NN` identifiers from ARCH-INDEX
> Subsystem Registry. `supersedes` / `superseded_by` link to other ADR IDs (e.g., `ADR-007`).

## Context

wirerust currently dispatches TCP stream flows via a content-first policy (ADR-0001): TLS is
recognized by a `0x16 0x03` record-type-and-version prefix; HTTP is recognized by method-token
ASCII prefixes. Flows that match neither are routed by port fallback (443/8443 → TLS; 80/8080
→ HTTP). This model was designed for text-protocol and TLS, both of which expose unambiguous
byte prefixes at stream offset 0.

Issue #7 introduces Modbus TCP, a binary industrial-control-system (ICS/OT) protocol defined in
the Modbus.org MODBUS Application Protocol Specification V1.1b3 and Messaging Implementation
Guide V1.0b §3.1.3. Modbus TCP runs on IANA-registered port 502, and every ADU begins with
a 7-byte MBAP header whose layout is: Transaction ID (2 B, BE) | Protocol ID (2 B, BE, always
`0x0000`) | Length (2 B, BE) | Unit ID (1 B). The Function Code byte follows immediately at
byte 7.

The Protocol ID field (`0x0000` = Modbus) is located at bytes 2–3, not at bytes 0–1. Bytes 0–1
are the Transaction ID, a session-specific counter that takes any value from `0x0000` to
`0xFFFF`. This means there is no stable byte-prefix fingerprint at stream offset 0 that
unambiguously identifies Modbus — the MBAP header is structurally indistinguishable from
arbitrary binary data by inspecting only its first two bytes. Content-first classification
cannot be applied without reading a minimum of 4 bytes and checking bytes 2–3 against
`0x0000`, which would produce significant false positives on any binary protocol whose third
and fourth bytes happen to be zero.

In contrast, HTTP and TLS were chosen precisely because their content-discriminators are highly
distinctive at byte 0. Modbus does not share this property. Port 502 is, in practice, the sole
reliable classifier for Modbus TCP in a captured stream. The three-point post-classification
validity gate (Protocol ID == `0x0000` AND Length in 2..=253 AND plausible FC) mitigates the
risk of analyzing non-Modbus traffic that happens to land on port 502.

A second structural difference is the request/response model. Modbus TCP uses a Transaction ID
(bytes 0–1 of MBAP) that the server echoes unchanged in its response, together with the Unit ID
(byte 6), to correlate requests and responses across directions. Detecting attack patterns such
as write-burst rate (T0806), parameter modification (T0836), and denial-of-service via Force
Listen Only (T0814) requires tracking in-flight requests by (Transaction ID, Unit ID) per flow
— a per-flow transaction correlation table — so that response FC echoes and exception codes can
be attributed to the originating request. Neither HttpAnalyzer nor TlsAnalyzer maintains this
kind of cross-direction request/response correlation; this is a new pattern for the codebase.

A third dimension concerns the MITRE ATT&CK matrix. wirerust currently seeds Enterprise ATT&CK
technique IDs (T1xxx namespace). Modbus findings map to MITRE ATT&CK for ICS, which is a
distinct matrix with its own technique namespace (`T0xxx`, e.g., T0836 "Modify Parameter").
The two matrices use different tactic taxonomies; for example, "Impair Process Control" exists
only in the ICS matrix. `mitre.rs` already carries `MitreTactic::IcsImpairProcessControl` and
seeds T0855/T0856/T0846/T0885 in `SEEDED_TECHNIQUE_IDS`, establishing a precedent for ICS
techniques. However, the codebase has no formal representation of which matrix a technique ID
belongs to, and the Kani proof in `kani_proofs::EMITTED_IDS` does not yet include any ICS
technique as emitted. This ADR extends the catalog with T0836, T0814, T0806, T0835, T0831,
and T0888 (Decision 12 correction: T0888 replaces T0846 as the Modbus recon emitter),
and makes the matrix distinction explicit in the type system.

## Decision

We will integrate Modbus TCP analysis via four coordinated decisions:

1. **Port-only classification as documented exception to ADR-0001.** `DispatchTarget::Modbus`
   is added as a fourth enum variant. The `classify()` function gains a port-502 arm placed
   AFTER the existing 443/8443 and 80/8080 fallback arms, so no existing flow can be stolen.
   The VP-004 `classify_oracle` Kani harness is extended with an identical port-502 arm so
   formal correctness of the precedence ladder is preserved. The absence of a content-level
   fingerprint is explicitly recorded here and mitigated by the three-point post-parse validity
   gate in `ModbusAnalyzer::on_data`: Protocol ID == `0x0000` AND Length in [2, 253] AND
   Function Code is a recognized or plausible code. ADUs failing the gate are skipped without
   emitting Modbus findings, preventing misclassification of non-Modbus traffic on port 502.

2. **PDU-oriented manual binary parsing with no external crate.** `ModbusAnalyzer` parses
   complete Modbus ADUs by reading the 7-byte MBAP header directly via `u16::from_be_bytes`
   / `u8` slice indexing into the reassembled TCP byte stream. The MBAP `Length` field defines
   the PDU boundary; the parser advances an offset pointer by `6 + length` bytes per ADU and
   processes all complete ADUs in the available data, accumulating remainder bytes. No external
   Modbus parsing crate is introduced. The maximum ADU size is 260 bytes (7-byte MBAP + 253-byte
   PDU maximum per spec V1.1b3), making this approach safe and bounded without a streaming
   parser framework.

3. **Full transaction-correlation state model with dual-window write detection (corrected per
   f2-fix-directives.md v2 Decision 11).** Each flow's `ModbusFlowState` carries a
   `pending: HashMap<(u16, u8), (u8, u32)>` table keyed on `(transaction_id, unit_id)`
   mapping to `(request_fc: u8, timestamp: u32)`. On receiving a request (destination port
   502), the entry is inserted. On receiving a response (source port 502), the entry is
   looked up by `(transaction_id, unit_id)`, the FC echo is validated, and the entry is
   removed. This supports: (a) exception attribution to the originating FC, (b) orphan
   response detection, (c) FC-mismatch anomaly between request and response. The table is
   bounded to `MAX_PENDING_TRANSACTIONS = 256` entries per flow to prevent unbounded growth
   on pipelined or lossy captures.

   **Write-burst detection uses a DUAL-window model** (corrects the prior single-window
   description): `--modbus-write-burst-threshold` (default 20) and
   `--modbus-write-sustained-threshold` (default 10) implement two independent detectors:
   - *Burst detector:* fires T0806 + T0855 when a 1-second window sees >N write FCs. One
     finding per window overflow (`window_burst_emitted` guard). `WRITE_BURST_WINDOW_SECS = 1`.
   - *Sustained detector:* fires T0806 + T0855 when a ≥2-second rolling window has average
     rate > M/s. One finding per window overflow (`sustained_burst_emitted` guard).
     `WRITE_SUSTAINED_WINDOW_SECS = 2`. Detection math:
     `sustained_window_write_count > write_sustained_threshold * elapsed_secs`.
   `ModbusFlowState` carries three sustained-window fields: `sustained_window_start_ts`,
   `sustained_window_write_count`, `sustained_burst_emitted` (alongside existing burst fields).
   The prior single `write_threshold` field on `ModbusAnalyzer` is replaced by
   `write_burst_threshold` and `write_sustained_threshold`.

4. **ICS-matrix MITRE representation via a `Matrix` discriminator field (corrected per
   f2-fix-directives.md v2 Decision 12).** The MITRE type design adds a `Matrix` enum
   `{ Enterprise, Ics }` inferable from the technique-ID namespace: IDs matching `T0[0-9]{3}`
   are ICS; IDs matching `T[1-9][0-9]{3}` or `T[1-9][0-9]{3}\.[0-9]{3}` are Enterprise.
   The `technique_info` match arm for each ICS technique returns the ICS tactic.
   The `EMITTED_IDS` array in the VP-007 Kani `kani_proofs` module gains seven Modbus-emitted
   ICS IDs: T0855, T0836, T0814, T0806, T0835, T0831, **T0888**. (T0846 is NOT emitted —
   see below.) `SEEDED_TECHNIQUE_IDS` gains T0836, T0814, T0806, T0835, T0831, and T0888
   (T0855 and T0846 are already seeded). `SEEDED_TECHNIQUE_ID_COUNT` advances from **15 to 21**
   (not 20 — T0888 is newly seeded; 11 Enterprise + 10 ICS total).

   **T0846 → T0888 correctness fix (Decision 12):** Recon FCs 0x11 (Report Server ID) and
   0x2B/MEI 0x0E (Read Device ID) now emit **T0888 Remote System Information Discovery**
   (TA0102 Discovery), not T0846. T0846 Remote System Discovery was a common misattribution:
   T0846 applies to network-scan behavior (enumerating systems exist); T0888 applies to
   querying device make/model/firmware/version. T0846 remains SEEDED (kept in catalog for
   future use) but is NOT in `EMITTED_IDS`.

   **Multi-tag Finding type (ADR-006):** `mitre_technique: Option<String>` is superseded by
   `mitre_techniques: Vec<String>` per ADR-006. The `EMITTED_IDS` grep pattern changes from
   `mitre_technique: Some` to `mitre_techniques: vec!`.

## Rationale

The port-only classification exception (decision 1) is forced by the Modbus wire format
(Modbus.org Messaging Implementation Guide V1.0b §3.1.3): bytes 0–1 are a variable session
counter. There is no alternative content discriminator for Modbus that meets ADR-0001's
requirement of a distinctive high-confidence prefix. The three-point validity gate is the
compensating control: it raises the confidence that a flow classified as Modbus actually
carries well-formed MBAP data before any findings are emitted (research/modbus-tcp-research.md
§7, false-positive consideration #4 "Port-502-but-not-Modbus"). The port-502 arm is placed
last in the fallback chain, after HTTP and TLS, so it cannot interfere with existing
classifications (INV-2 / VP-004).

Manual PDU parsing (decision 2) avoids adding an external Modbus crate dependency. The Modbus
ADU format is simple enough (fixed 7-byte header, length-delimited PDU) that a hand-rolled
parser is shorter, more readable, and more formally verifiable (VP-022 Kani harness targets
`parse_mbap_header` and `classify_fc` as pure functions) than configuring and wrapping a crate.
This follows the precedent set by the JA3/JA3S MD5 computation in `analyzer/tls.rs`, which
also uses a direct implementation rather than a crate. The bounded-offset-advancement model
(advance `6 + length` bytes per ADU) is trivially safe and avoids the slice-indexing panic risk.

The full transaction-correlation model (decision 3) is required by the approved scope (F2
mandate: FULL transaction correlation per-connection Transaction-ID+Unit-ID+FC table) and by
the six approved MITRE techniques. T0814 (Denial of Service via Force Listen Only) requires
correlating a Diagnostics request's sub-function with the flow state to distinguish a benign
diagnostic poll from a Force-Listen-Only attack. T0831 (Manipulation of Control) requires
correlating multiple write FCs across directions within a short window. A stateless per-PDU
model (which was recommended in the initial F1 delta for v1) cannot detect these patterns
reliably. The transaction table is bounded (`MAX_PENDING_TRANSACTIONS = 256`) so it does not
introduce unbounded memory growth, consistent with the bounded-resource design principle in
ARCH-INDEX §Cross-Cutting Concerns.

The `Matrix` discriminator (decision 4) prevents a category confusion defect: without an
explicit ICS marker, `technique_tactic` for T0836 returns `IcsImpairProcessControl`, but
callers cannot programmatically distinguish "this technique is from the ICS matrix" from
"this technique happens to have a tactic named after an ICS concept". The discriminator makes
the matrix affiliation a first-class field, enabling future reporters to group findings by
matrix. It is expressed as a namespace rule on the ID format (T0xxx ↔ ICS; T1xxx+ ↔
Enterprise) so no additional enum field is required on `Finding` itself — the technique ID
already encodes the matrix. The `technique_info` return type or a parallel
`technique_matrix(id: &str) -> Option<Matrix>` function makes this lookup cheap and testable.

## Consequences

What this decision causes downstream. Use sub-headings:

### Positive

- Modbus TCP flows on port 502 are correctly routed and analyzed, enabling ICS/OT threat
  detection for all seven MITRE ATT&CK for ICS techniques emitted in scope:
  T0855, T0836, T0814, T0806, T0835, T0831, **T0888** (corrected from T0846 per Decision 12).
- The three-point validity gate prevents Modbus findings from being emitted on non-Modbus
  binary traffic that happens to use port 502, keeping false-positive rates low.
- Full transaction correlation enables pattern-based detections (write-burst attribution,
  FC-mismatch anomalies, orphan responses) that stateless per-PDU analysis cannot support.
- No new external crate dependency is introduced; `ModbusAnalyzer` is self-contained.
- The ICS `Matrix` discriminator makes MITRE matrix affiliation explicit and testable,
  eliminating the ICS-vs-Enterprise category confusion risk.
- VP-004 formal correctness is preserved: the extended `classify_oracle` continues to mirror
  the production `classify` function, keeping the Kani precedence proof sound over the
  four-variant `DispatchTarget` enum.
- VP-007 formal correctness is preserved: the expanded `SEEDED_TECHNIQUE_IDS` (21 total:
  15→21, adding T0836/T0814/T0806/T0835/T0831/T0888) and `EMITTED_IDS` arrays keep the
  drift-guard test sound over the enlarged catalog.

### Negative / Trade-offs

- Port-only classification for Modbus means any non-Modbus binary protocol on port 502 is
  misrouted to `ModbusAnalyzer` until the validity gate rejects its ADUs. This is an accepted
  false-routing cost; the gate prevents false findings but the flow is still consumed by the
  Modbus parser path. This is called out as a documented exception to ADR-0001.
- Per-flow `pending` transaction table adds memory overhead: up to 256 × ~16 bytes = ~4 KB
  per Modbus flow. For ICS segments with large numbers of concurrent Modbus sessions, this
  is non-trivial but bounded.
- `StreamDispatcher::on_data` early-exit guard (currently `if self.http.is_none() &&
  self.tls.is_none()`) must be extended to also check `self.modbus.is_none()`, or rewritten
  as per-arm `if let Some(ref mut x)` guards. Missing this produces a latent path where
  Modbus data is dropped silently.
- `SEEDED_TECHNIQUE_ID_COUNT` (now **21**; corrected from the prior "20" in v1.0 — see
  Decision 4 correction above) and `SEEDED_TECHNIQUE_IDS` must be updated atomically with
  each new `technique_info` arm; the `vp007_catalog_drift_guard` test enforces this but
  requires discipline on every future ICS technique addition.
- `--modbus-write-burst-threshold` and `--modbus-write-sustained-threshold` implement a
  dual-window model (1-second burst / ≥2-second sustained, per Decision 11). Each window
  fires at most once per its respective window expiry. The prior single `--modbus-write-threshold`
  flag is removed; this is a CLI-breaking change in v0.3.0.

### Status as of 2026-06-09

Proposed. Implementation has not yet begun (Feature cycle Issue #7, F2 architecture delta
phase). The ADR decisions are design-complete and await F3 story decomposition and
implementation. VP-004 and VP-007 Kani proof updates are part of the F3 implementation
stories for `dispatcher.rs` and `mitre.rs` respectively.

## Alternatives Considered

- **Content-at-byte-2 classification (peek bytes 2–3 for `0x0000`):** Would extend the
  content-first rule to Modbus by peeking beyond byte 0. Rejected because Transaction ID bytes
  0–1 are session-specific; peeking bytes 2–3 is equivalent to checking `data[2] == 0 &&
  data[3] == 0`, which produces false positives on any binary protocol (SSH key exchange,
  custom binary framing, TLS extensions) whose bytes 2–3 happen to be zero. The false-positive
  rate is unacceptably high compared to port-502 + post-classification validity gate.

- **Stateless per-PDU parsing only (no transaction correlation):** Track only write-counts
  and exception-counts per flow without request/response correlation. Simpler state; no
  `pending` HashMap. Rejected because the approved F2 scope explicitly mandates FULL
  transaction correlation (per-connection Transaction-ID+Unit-ID+FC table), and T0814
  (Force Listen Only) and T0831 (Manipulation of Control) detections require it. A future
  downgrade to stateless-only could be made if the transaction table proves too costly.

- **Introduce a `Matrix` flag on `Finding` (not just on technique lookup):** Tag each `Finding`
  directly with `matrix: Matrix::Ics` to avoid the lookup step. Rejected because `Finding`
  already carries `mitre_technique: Option<String>` from which the matrix is derivable; adding
  a redundant field would violate the DRY principle and would require changes to every analyzer
  that constructs `Finding` instances. A `technique_matrix(id)` lookup function is cheaper and
  self-consistent.

- **Introduce a separate `IcsThreatCategory` variant on `ThreatCategory`:** Create
  `ThreatCategory::IcsImpairProcessControl` and similar variants to distinguish ICS findings
  at the category level. Rejected for v1 because `ThreatCategory::Execution` and
  `ThreatCategory::Anomaly` map adequately to the primary Modbus finding types, `ThreatCategory`
  is `#[non_exhaustive]` (future addition is non-breaking), and the six MITRE techniques in
  scope already carry the ICS tactic via `technique_tactic()`. This can be revisited when a
  reporter feature requests ICS-specific grouping.

## Source / Origin

- **Modbus wire format:** Modbus.org MODBUS Application Protocol Specification V1.1b3 (§4.2,
  §6), Modbus.org MODBUS Messaging on TCP/IP Implementation Guide V1.0b §3.1.3. Reproduced
  in `.factory/research/modbus-tcp-research.md` §1–§4.
- **MITRE ATT&CK for ICS techniques:** MITRE ATT&CK for ICS matrix (T0855, T0836, T0814,
  T0806, T0835, T0831). Verified in `.factory/research/modbus-tcp-research.md` §5–§6.
- **Port-only classification precedent:** ADR-0001 (`docs/adr/0001-content-first-stream-dispatch.md`)
  §Rationale — extensibility note; `src/dispatcher.rs` `classify()` function (~line 114) and
  `EMITTED_IDS` in `kani_proofs` (~line 191). See also `.factory/phase-f1-delta-analysis/delta-analysis.md`
  §6 "ADR-REQUIRED: ICS/OT Binary Protocol Integration Pattern".
- **Transaction-correlation mandate:** F2 scope approval (user-confirmed): "FULL transaction
  correlation (per-connection Transaction-ID+Unit-ID+FC table)".
- **ICS matrix representation gap:** `src/mitre.rs` lines 191–198 (`EMITTED_IDS` — T0855
  missing from emitted set), confirmed in `.factory/phase-f1-delta-analysis/delta-analysis.md`
  §8.
