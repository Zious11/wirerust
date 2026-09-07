---
document_type: adr
adr_id: ADR-014
status: proposed
date: 2026-09-06
subsystems_affected:
  - SS-05
  - SS-10
  - SS-18
  - SS-20
  - SS-21
supersedes: null
superseded_by: null
feature_cycle: feature-s7comm
mitre_pin: ics-attack-19.1
---

# ADR-014: Classic S7comm over ISO-on-TCP (TPKT/COTP) — Stream Dispatch and Parser Design

> **One-per-file:** Each architectural decision lives in its own file.
> Filename convention: `ADR-NNN-<short-name>.md`.
> ADR IDs are sequential 3-digit (ADR-001, ADR-002, ...). Once issued, never renumber.
> Lifecycle: `proposed` -> `accepted` -> (optional) `superseded` or `deprecated`.

## Context

wirerust's `StreamDispatcher` currently classifies TCP flows through nine rules: two
content rules (TLS signature, HTTP method prefix), six port-fallback rules (443/8443 →
TLS, 80/8080 → HTTP, 502 → Modbus [ADR-005], 20000 → DNP3 [ADR-007], 44818 → ENIP
[ADR-010], 2404 → IEC-104 [ADR-013]), and an implicit "no match" arm (Rule 9 prior to
this ADR). Feature cycle `feature-s7comm` introduces passive Siemens S7comm analysis as
subsystem SS-21, built on a new, reusable ISO-on-TCP (TPKT/COTP) framing layer as
subsystem SS-20.

S7comm is Siemens' proprietary PLC-programming and HMI-communication protocol,
overwhelmingly deployed on S7-300/400 (classic, protocol-ID `0x32`) and S7-1200/1500
(S7comm-plus, protocol-ID `0x72`) controllers. It is the top-ranked next-protocol
candidate per `.factory/planning/next-ot-protocol-research.md`: highest threat signal in
the candidate pool (Stuxnet; CISA AA26-231A, active 2026 targeting), richest ATT&CK-for-
ICS mapping, and top-tier prevalence. It runs on TCP port 102 (IANA-registered for
ISO-TSAP) atop a three-layer stack:

1. **TPKT (RFC 1006)** — 4-byte outer header on every TCP segment: version (1 byte, = 3),
   reserved (1 byte, = 0), length (2 bytes, big-endian, **total TPKT packet length
   including this 4-byte header**, max 65,535).
2. **COTP (ISO 8073 / ITU-T X.224)** — Connection-Oriented Transport Protocol TPDU
   inside the TPKT payload. CR (Connect Request) and CC (Connect Confirm) TPDUs perform
   session establishment and carry no upper-layer payload; DT (Data Transfer) TPDUs
   carry the steady-state upper-layer payload, prefixed by a single protocol-ID byte.
3. **S7comm** — the upper-layer payload inside a COTP DT-TPDU, keyed off the
   protocol-ID byte: `0x32` = classic S7comm (ROSCTR, PDU reference, parameter/data
   length, then parameter and data blocks); `0x72` = S7comm-plus (a distinct,
   increasingly TLS-wrapped, object/service protocol out of dissection scope — see
   Decision 6).

Critically, TCP/102 is **not** exclusive to S7comm. `ADR-012` (Protocol Coverage
Catalog) already documents and defends against a "port-102 four-way collision":
S7comm, S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 all share canonical port 102 in
`src/protocols.rs::KNOWN_PROTOCOLS`, and a REGRESSION-GUARD test in
`tests/protocols_tests.rs` currently asserts that **none** of the four are supported.
Promoting S7comm to `known-supported` while MMS/ICCP/S7comm-plus remain
`known-unsupported` breaks the pure `canonical_ports ∩ SUPPORTED_PORTS` intersection
model that `supported_protocols()` uses today (ADR-012 Decision 5) — the central design
problem this ADR resolves (Decision 3).

This ADR is grounded in the completed F1/F2 research for `feature-s7comm`:
`.factory/cycles/feature-s7comm/f1-delta-analysis.md` (scope, impact boundary,
regression risk), `.factory/cycles/feature-s7comm/f2-license-matrix.md` (clean-room
provenance gate), `.factory/research/s7comm-mitre-ics-tagging.md` (technique mappings),
and `.factory/cycles/feature-s7comm/f2-pcap-fixture-sourcing.md` (fixture strategy).

### Relationship to Prior ADRs

This ADR is the direct successor of:
- **ADR-013** (IEC-104, port 2404): the closest structural precedent — single-protocol
  port-fallback classification, pure-core free-fn parser design for Kani, directional
  carry buffers with walk-first residual-bound semantics, VP-004/VP-007 atomic
  obligations, and a licensing-constraint decision. ADR-014 follows ADR-013's shape
  decision-for-decision wherever the S7comm/ISO-on-TCP design matches it, and departs
  from it explicitly where the port-102 multi-protocol collision and the two-layer
  TPKT/COTP-then-S7comm framing require a different answer.
- **ADR-010** (EtherNet/IP, port 44818): established the *multi-level framing* pattern
  (ENIP→CPF→CIP) and the VP-007 six-part atomic MITRE-seeding obligation, directly
  analogous to S7comm's three-level TPKT→COTP→S7comm framing.
- **ADR-005** (Modbus, port 502): established the binary-ICS-port-fallback pattern as a
  documented exception to ADR-0001 content-first dispatch.
- **ADR-012** (Protocol Coverage Catalog): already documents the TCP/102 four-way
  collision as a caveat and carries the `PORT_102_NOTE` footnote mechanism in
  `main.rs`; this ADR is the first to actually promote one of the four names, which is
  why Decision 3 exists.

No existing ADR is superseded. ADR-014 adds Rule 9 (port 102) following the same
documented exception to ADR-0001 as its five binary-ICS predecessors; the prior "no
match" arm (Rule 9) is renumbered Rule 10.

## Decision

### Decision 1: Two-module split — `iso_on_tcp.rs` (SS-20) + `s7comm.rs` (SS-21); frozen interface

The ISO-on-TCP framing layer (TPKT + COTP) is implemented as a **new, standalone
module**, `src/analyzer/iso_on_tcp.rs` (SS-20), separate from the S7comm PDU dissector,
`src/analyzer/s7comm.rs` (SS-21).

**Rationale for the split (not a single file, unlike ENIP's in-file ENIP/CPF/CIP
split):** TPKT and COTP are **protocol-agnostic across three future catalog entries**
sharing port 102 — S7comm, IEC 61850 MMS, and ICCP/TASE.2 all ride on the same
TPKT/COTP substrate. ENIP's encapsulation-header/CPF split (ADR-010 Decision 2) is
ENIP-specific top-to-bottom, so one file was correct there. Here, keeping TPKT/COTP
parsing out of `s7comm.rs` entirely is what makes the "one architectural investment
unlocks three catalog entries" research rationale real: a future MMS or ICCP cycle
imports `iso_on_tcp::parse_tpkt_header`/`iso_on_tcp::parse_cotp_header` directly,
touching zero lines of `s7comm.rs`.

**Frozen interface (the SS-20 → SS-21 handoff).** `iso_on_tcp.rs` exports pure functions
only — no `StreamAnalyzer` implementation of its own, and no per-flow state:

```rust
// src/analyzer/iso_on_tcp.rs — module scope, free fns only (Decision 9)

pub struct TpktHeader {
    pub version: u8,   // always 3 for a valid TPKT packet
    pub length: u16,   // total packet length INCLUDING this 4-byte header
}

pub enum CotpTpduType {
    ConnectRequest,   // CR — session establishment, no upper-layer payload
    ConnectConfirm,   // CC — session establishment, no upper-layer payload
    DataTransfer,     // DT — carries upper-layer payload prefixed by protocol_id
}

pub struct CotpHeader {
    pub tpdu_type: CotpTpduType,
    /// `Some(byte)` only for a DT-TPDU whose payload begins with a recognized
    /// protocol-ID byte (0x32 classic S7comm, 0x72 S7comm-plus, or any other
    /// value observed on the wire); `None` for CR/CC (no upper-layer payload
    /// exists yet) or when the DT payload is empty.
    pub protocol_id: Option<u8>,
    /// Byte offset into the COTP frame where the upper-layer payload begins.
    pub payload_offset: usize,
}

pub fn parse_tpkt_header(data: &[u8]) -> Option<TpktHeader>;
pub fn parse_cotp_header(tpkt_payload: &[u8]) -> Option<CotpHeader>;
```

`S7commAnalyzer` (SS-21) calls `parse_tpkt_header` then `parse_cotp_header` on every
extracted TPKT frame and branches on `CotpHeader::protocol_id` (Decision 2). No
`DispatchTarget::IsoOnTcp` variant is introduced — SS-20 is a parsing library consumed
by SS-21, not an independent dispatch target (this also avoids a dispatcher variant with
no analyzer behind it).

**Per-flow state placement (resolves F1 §2.3 open question):** the TPKT/COTP directional
carry buffers (Decision 8) live on `S7commFlowState`, not on a separate
`IsoOnTcpFlowState`. SS-20 is deliberately state-free by design (Decision 1); a shared
state struct would contradict that design and would need to be threaded through SS-21
regardless. A future MMS/ICCP cycle that also needs TPKT/COTP carry buffers defines its
own analogous flow-state field — the *parsing functions* are shared; the *per-flow
state* is not, because each consuming analyzer owns its own flow lifecycle.

### Decision 2: Port-102 dispatch — single `DispatchTarget::S7comm`; disambiguation inside the analyzer; VP-004 atomic obligation

TCP/102 traffic is classified using a single new dispatcher rule, **Rule 9** (after
existing Rule 8, port 2404/IEC-104; the former Rule 9 "no match" arm becomes Rule 10),
mapping to **one** new variant, `DispatchTarget::S7comm`. There is **no** separate
dispatcher rule for S7comm-plus, MMS, or ICCP — the dispatcher only sees raw TCP bytes
and cannot cheaply distinguish COTP protocol-IDs without doing the TPKT/COTP parse
itself, and doing that parse **is** `S7commAnalyzer`'s job (Decision 1).

**In-analyzer disambiguation.** On every parsed `CotpHeader`, `S7commAnalyzer` branches:

| `protocol_id` | Meaning | Analyzer behavior |
|---|---|---|
| `Some(0x32)` | Classic S7comm | Full S7comm PDU dissection (function codes, ROSCTR, parameter/data blocks) |
| `Some(0x72)` | S7comm-plus | Framing-level classification + session-setup metadata only (Decision 6) — **not** a full dissector |
| `None` (CR/CC TPDU) | Session establishment, no payload yet | Track connection state; defer classification until the first DT frame arrives |
| `Some(other)` or unparseable DT payload | MMS, ICCP, or unrecognized ISO-on-TCP traffic | **Left unclassified** — not counted as S7comm, not force-fit into any S7comm finding path |

The last row is the load-bearing correctness property this ADR must guarantee: **a
COTP DT-TPDU on port 102 whose protocol-ID is not `0x32` or `0x72` must never be
misattributed to S7comm.** Such traffic continues to surface through the existing
`(TransportProto, u16)` unclassified-port-count mechanism (`dispatcher.rs`
`unclassified_port_counts`) and the `PORT_102_NOTE`/`collision_note` gap-report
machinery in `main.rs` — which Decision 10 revises so it no longer claims all four
port-102 protocols are equally unattributed once S7comm is promoted.

This is the **first** dispatcher rule where post-classification disambiguation inside
the analyzer is load-bearing for correctness, not merely defense-in-depth. Contrast
with IEC-104's `is_valid_iec104_frame` (ADR-013 Decision 1), which only rejects garbage
and never re-routes to a *different* named protocol.

**VP-004 six-step atomic obligation** (mirrors ADR-013 Decision 9, ADR-010 Decision 1),
to be executed in the same commit:

1. Add `DispatchTarget::S7comm` variant to the `DispatchTarget` enum.
2. Add the port-102 arm to `classify()` (Rule 9, after Rule 8 IEC-104).
3. Add the corresponding `DispatchTarget::S7comm` arm to `classify_oracle` in
   `#[cfg(kani)] mod kani_proofs`, mirroring production `classify()` syntactically.
4. Extend the early-exit guard to include `self.s7comm.is_none()`.
5. Add `S7comm` match arms to `on_data` and `on_flow_close`.
6. Re-run `verify_content_first_precedence_exhaustive` and confirm VERIFICATION
   SUCCESSFUL.

Failure to update `classify_oracle` atomically invalidates the VP-004 proof.

### Decision 3: Port-102 catalog-model fix — RATIFIED: per-entry `Support` enum (Option d)

**RATIFIED (human, 2026-09-06).** The port-102 catalog-model problem — F1 explicitly
deferred it to F2 — is resolved by adding an explicit per-entry `Support` enum to
`KnownProtocol`, **not** the name-keyed exclusion list this ADR originally recommended
as Option (b). This ratification follows the independent validation in
`.factory/cycles/feature-s7comm/f2-port102-model-validation.md`, which was commissioned
specifically to stress-test the (b) recommendation against comparable-tool prior art
(Wireshark, Suricata, Zeek) and the extensibility axis before human sign-off. That
document's verdict — Option (d) is the evidence-optimal model; (b) is at best a
minimal-diff interim with a named migration trigger — is adopted in full.

**The model.** `KnownProtocol` gains a new field, `pub support: Support`, backed by an
explicit, exhaustively-matched enum:

```rust
/// Per-entry support state for a cataloged protocol (`src/protocols.rs`).
/// Vocabulary reused from ADR-0012 Decision 2's Suricata-derived tri-state
/// (`known-supported` / `known-unsupported` / `unknown`) for the *dynamic*
/// coverage-gap report — applied here, for the first time, to the *static*
/// catalog partition that `supported_protocols()`/`unsupported_protocols()` compute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    /// A full dissector exists for this protocol.
    Supported,
    /// No dissector exists for this protocol at all.
    KnownUnsupported,
    /// Framing-level classification / observation only — never promoted to a full
    /// dissector. (S7comm-plus, ADR-014 Decision 6: "observed, not dissected.")
    DetectionOnly,
}
```

`supported_protocols()` (`src/protocols.rs`, ADR-012 Decision 5) changes from the
`canonical_ports ∩ SUPPORTED_PORTS` derivation (plus the hand-coded `|| p.name == "ARP"`
exception) to a direct filter on the new field:

```rust
pub fn supported_protocols() -> Vec<&'static KnownProtocol> {
    KNOWN_PROTOCOLS.iter().filter(|p| p.support == Support::Supported).collect()
}
```

`unsupported_protocols()` remains the **derived complement of `supported_protocols()`**
— preserving the VP-041 partition invariant (`supported ⊎ unsupported = all ~30
entries`) — but the complement is now `support != Support::Supported`, **not**
`support == Support::KnownUnsupported`:

```rust
pub fn unsupported_protocols() -> Vec<&'static KnownProtocol> {
    KNOWN_PROTOCOLS.iter().filter(|p| p.support != Support::Supported).collect()
}
```

This distinction is load-bearing: `DetectionOnly` is a refinement of "not fully
supported," not a third partition member sitting outside both sets. Using `==
KnownUnsupported` would silently drop every `DetectionOnly` entry (S7comm-plus, this
cycle) from `unsupported_protocols()` entirely, breaking the two-set partition
`supported_protocols() ⊎ unsupported_protocols() = KNOWN_PROTOCOLS` that VP-041 asserts.

**Per-entry assignments, this cycle.** The four port-102 entries:

| Entry | `Support` value | Rationale |
|---|---|---|
| S7comm | `Supported` | Full classic-S7comm dissection (Decision 1/2) |
| S7comm-plus | `DetectionOnly` | Framing classification + unencrypted session-setup metadata only, per Decision 6 — "observed, not dissected" — a state a `bool` or an exclusion list cannot express |
| IEC 61850 MMS | `KnownUnsupported` | Out of scope this cycle (Decision 10) |
| ICCP/TASE.2 | `KnownUnsupported` | Out of scope this cycle (Decision 10) |

All other 26 `KnownProtocol` literals get their **pre-existing** supported/unsupported
status made explicit as an enum value — no behavioral change for these 26, only a
change in how the fact is expressed (explicit field vs. derived port-intersection):
Modbus, DNP3, ENIP, IEC-104, TLS, DNS, HTTP, and ARP become `Supported` (mirroring
exactly what `canonical_ports ∩ SUPPORTED_PORTS` plus the ARP name-exception already
computed for them); every remaining catalog entry becomes `KnownUnsupported`.

**Prior-art rationale (from the validation brief, §3).** Wireshark (`packet-tpkt.c` →
heuristic COTP registry → `dissect_s7comm`/`packet-mms.c`), Suricata
(`SCAppLayerProtoDetectPPRegister` + the first-class three-state `enabled: yes | no |
detection-only` app-layer option), and Zeek (`Analyzer::register_for_port` + DPD
signature confirmation) are unanimous: mature passive analyzers key "is this protocol
supported" on an **explicit per-protocol registry entry**, never on the port number —
the port is only a scoping/attachment hint, and a mature tool that shares a port across
protocols registers *multiple* candidates rather than deriving one answer from the
port. wirerust's pre-existing `canonical_ports ∩ SUPPORTED_PORTS` model is precisely the
"port ⇒ support" derivation every one of these tools rejects for an ambiguous port like
102. Suricata's three-state `enabled` option is, concretely, the `Support` enum: option
(d) is not a novel invention, it is bringing the static catalog into line with the
vocabulary ADR-0012 Decision 2 already adopted for the dynamic report — closing a
mismatch between the two layers rather than opening a new one.

Option (d) also uses **safe positive polarity**: Rust struct-expression rules require
every `KnownProtocol` literal to supply the new `support` field, so "forgot to decide
this entry's support state" is a **compile error**, not a silent behavior. This is
directly relevant to this catalog's own roadmap — Decision 1 architects `iso_on_tcp.rs`
specifically so a future IEC 61850 MMS or ICCP/TASE.2 cycle can promote one of the
*other two* remaining port-102 entries. Under (d), that promotion is a one-line change
to an existing, already-declared field. Under the previously-recommended Option (b), the
promotion would instead have depended on a human remembering to keep a *separate*
deny-list (`PORT_102_UNSUPPORTED_SIBLINGS`) in sync — an **unsafe default-allow
polarity**: any new same-port catalog entry would be silently reported as `supported`
unless someone remembered to add it to the exclusion list.

**Why (a), (b), and (c) were rejected/superseded:**

- **(a) `supported: bool` field** is **strictly dominated by (d)**: identical blast
  radius (every literal gains a field; struct changes once), but a `bool` cannot express
  `DetectionOnly` — and this cycle already needs that third state for S7comm-plus
  (Decision 6). There is no scenario in which (a) is preferable to (d) once a
  `DetectionOnly`-shaped requirement exists, which it now demonstrably does.
- **(b) name-keyed exclusion list** (this ADR's original recommendation) is
  **superseded**, not merely rejected outright — it remains a legitimate minimal-diff
  *interim* pattern in the abstract, but the validation brief's extensibility analysis
  (its decisive axis, §4) shows it is a deny-list with an unsafe default: a *new*
  catalog entry sharing an already-supported port would be silently promoted to
  `supported` unless a human remembered to add it to
  `PORT_102_UNSUPPORTED_SIBLINGS`. It also requires three coupled sources of truth
  (`SUPPORTED_PORTS`, the exclusion list, and the VP-041 oracle mirror) instead of one,
  and — decisively for this ADR — it cannot express S7comm-plus's `DetectionOnly` state
  at all, flattening it into the same bucket as fully-opaque MMS/ICCP and discarding a
  distinction Decision 6 already draws.
- **(c) `dispatch_target: Option<&'static str>` field** remains rejected for the reason
  originally stated: `protocols.rs` is a documented pure-core leaf that **must not
  depend on `dispatcher`** (module doc-comment, BC-2.05.010 PC-4), and a
  `DispatchTarget` variant name expressed as a string literal is a stringly-typed
  coupling with no compile-time check against the actual enum. Option (d)'s `Support`
  enum introduces no such coupling — it says nothing about *which* `DispatchTarget`
  promotes an entry, only *whether* wirerust dissects it.

**Critical caveat — this does not fully solve port 102.** The `Support` enum fixes the
**static catalog partition** (`supported_protocols()`/`unsupported_protocols()`) only.
`SUPPORTED_PORTS` also independently drives `main.rs::lookup_protocol_state` — the
*dynamic* coverage-gap tri-state classifier (ADR-012 Decision 2's `known-supported` /
`known-unsupported` / `unknown`), keyed on the raw `(transport, port)` pair with **no
protocol identity available**. Once `102` is added to `SUPPORTED_PORTS`, every
unclassified TCP/102 gap flow will match the *first* port-102 catalog entry by
declaration order (S7comm) and be misreported `known-supported` regardless of whether
the underlying traffic is genuine MMS, ICCP, or S7comm-plus. **No catalog-model option —
(a), (b), or (d) — fixes this**, because `lookup_protocol_state` has no per-flow
protocol name to filter on; only the analyzer's parsed COTP `protocol_id`
(Decision 2's `0x32`/`0x72`/other branch) can disambiguate a live flow, and that
information does not exist at the point `lookup_protocol_state` runs. This defect is
**correctly deferred to F4** — already flagged by Decision 10's `PORT_102_NOTE`/
`collision_note` consequence — and is unaffected by which catalog-model option is
chosen. A per-entry `Support` enum does, however, make that eventual F4 refactor
cleaner: the gap classifier can key on `(protocol identity from analyzer) →
entry.support` instead of re-deriving from ports, once the analyzer-supplied identity
is available.

### Decision 4: Clean-room provenance and licensing constraint (mirrors ADR-013 Decision 7)

**HARD CONSTRAINT (immutable, checked in PR reviews).** Per
`.factory/cycles/feature-s7comm/f2-license-matrix.md`:

| Source | License | Status |
|--------|---------|--------|
| Wireshark `packet-s7comm.c` / `packet-s7comm_plus.c` | GPL-2.0-or-later | **BANNED (code)** |
| Snap7 (upstream C++) | LGPL-3.0-or-later | **BANNED (code)** |
| libnodave | LGPL-2.0-or-later | **BANNED (code)** |
| `s7` / `s7-comm` / `s7-client` crates (crates.io) | `non-standard` (unrecoverable custom grant) | **AVOID** — not a clean permissive grant |
| `rusty-cotp` / `rusty-tpkt` / `tpkt` / `copt` crates | unclear / non-standard | **AVOID** — implement from the open specs instead |

**Primary, authoritative open specifications for the lower layers (safe to implement
from directly):**
- **RFC 1006** (IETF, STD 35) — TPKT framing. Freely downloadable/implementable.
- **ITU-T X.224 ≡ ISO/IEC 8073:1997** (official free-download PDF from ITU) — COTP.
  "Free download" is not public domain; implement the specified wire behavior, do not
  republish substantial standard text/tables.

**S7comm classic (`0x32`) has no official public Siemens specification** — it is
proprietary and reverse-engineered. Fields are derived from free-to-read *prose/
behavioral* sources only: the Wireshark **wiki** page (prose, not the dissector source),
Kleinmann & Wool 2014, and the Orange-Cyberdefense `awesome-industrial-protocols`
catalog.

**Permitted design references (no verbatim code copy, design reference only):**
- `cisagov/icsnpp-s7comm` (BSD-3-Clause)
- `kprovost/libs7comm` (BSD-2-Clause)
- `python-snap7` (MIT)

**Posture:** the implementation MUST be described as "interoperable with Siemens S7
devices," never "official" or "Siemens-certified." Reimplementing observed wire formats
for interoperability is legally distinct from copying copyrighted source expression;
this is materially lower-risk than any effort touching S7comm-plus authentication or
TLS. No external S7/COTP/TPKT crate appears in `Cargo.toml`/`Cargo.lock` — original Rust
parser only, zero lines borrowed, following the ADR-013 Decision 7 precedent exactly.

### Decision 5: MITRE ATT&CK for ICS technique set — 3 new IDs, 8 reused, tactic-variant ruling

Per `.factory/research/s7comm-mitre-ics-tagging.md` (live technique-page verification,
2026-09-06):

**Seed 3 NEW catalog entries** (`SEEDED_TECHNIQUE_ID_COUNT` 29 → 32):

| ID | Name | Live-page tactic (verified) | `MitreTactic` enum impact |
|----|------|------------------------------|---------------------------|
| **T0843** | Program Download | **Lateral Movement — TA0109** | **NEW variant required: `MitreTactic::IcsLateralMovement` (`TA0109`)** — no existing variant covers TA0109 |
| **T0889** | Modify Program | **Persistence — TA0110** | **NEW variant required: `MitreTactic::IcsPersistence` (`TA0110`)** — no existing variant covers TA0110 |
| **T0821** | Modify Controller Tasking | **Execution — TA0104** | **Reuses existing `MitreTactic::IcsExecution` (`TA0104`)** — no new variant |

**Tactic-variant ruling (resolves F1 §7.2 open question):** the codebase's
`MitreTactic` enum currently has seven ICS-specific variants (`IcsInhibitResponseFunction`
TA0107, `IcsImpairProcessControl` TA0106, `IcsImpact` TA0105, `IcsDiscovery` TA0102,
`IcsCollection` TA0100, `IcsCommandAndControl` TA0101, `IcsExecution` TA0104) — none map
to TA0109 (Lateral Movement) or TA0110 (Persistence). Two new variants are required:
`MitreTactic::IcsLateralMovement` (`tactic_id() -> "TA0109"`, `Display ->
"Lateral Movement (ICS)"`) and `MitreTactic::IcsPersistence` (`tactic_id() -> "TA0110"`,
`Display -> "Persistence (ICS)"`), added to the enum, its `Display` impl, its
`tactic_id()` impl, and `all_tactics_in_report_order()` in `src/mitre.rs`, in the same
commit as the T0843/T0889 catalog entries (part of the VP-007 obligation below). T0821
requires no enum change — it reuses `IcsExecution`.

**Reuse 8 already-seeded IDs** (add S7comm emission call-sites only — no catalog
change): T0835 (Manipulate I/O Image, `Write Var 0x05` → area `0x80`/`0x81`/`0x82`),
T0836 (Modify Parameter, `Write Var 0x05` → `0x84`/`0x83`), T0858 (Change Operating Mode,
`0x29 PLC Stop` / `0x28 P_PROGRAM`), T0816 (Device Restart/Shutdown, decoded `0x28`
restart PI-service string), T0888 (Remote System Information Discovery, Userdata
`0x07`/CPU-group `0x04`/subfn `0x01` Read SZL, or Block-group `0x03`), T0846 (Remote
System Discovery, multi-host TCP/102 sweep evidence only, not single-PDU), T0814 (Denial
of Service, connection-flood/malformed-length burst thresholds), T1692.001 (Unauthorized
Message: Command Message, successor to revoked T0855, any command from an unauthorized
source).

**Group-`0x03` block-function correction:** the Userdata (ROSCTR `0x07`) subfunction
group table must read group `0x03` = **Block functions** (`0x01` List blocks, `0x02`
List blocks of type, `0x03` Get block info) and group `0x07` = **Time functions**
(clock read/set) — the reverse of a common documentation error (some secondary sources
mis-state block enumeration as group `0x07`). This correction is load-bearing for the
T0888 (Read SZL / block-list discovery) emission call-site and MUST be reflected in the
`s7comm.rs` Userdata subfunction match arms.

**Excluded (not seeded):** T0851 Rootkit, T0873/T0873.001 Project File Infection — both
are host/file-artifact behaviors with no S7comm wire-field evidence. **Deferred:** T0813
Denial of Control — only indirectly inferable, no clean emission predicate.

**Version pin:** retain `ics-attack-19.1` (the codebase's current pin). The live release
is `ics-attack-v19.2` (2026-08-06), an Agile minor touching only Enterprise
Groups/Software with **zero** ICS technique-catalog changes — every mapping in this
decision is valid under both. No pin bump required by this feature.

**VP-007 six-part atomic obligation** (mirrors ADR-013 Decision 10, ADR-010 §VP-007
decision), executed in one commit:

1. Add `"T0843"` and `"T0889"` and `"T0821"` to `SEEDED_TECHNIQUE_IDS` (29 → 32
   entries).
2. Bump `SEEDED_TECHNIQUE_ID_COUNT` to 32.
3. Add `technique_info("T0843")`, `technique_info("T0889")`, `technique_info("T0821")`
   arms (the first two returning the two new `MitreTactic` variants; the third returning
   `MitreTactic::IcsExecution`).
4. Add `"T0843"`, `"T0889"`, `"T0821"`, and the 8 reused IDs' S7comm emission
   call-sites to `EMITTED_IDS` (reused IDs may already be present from Modbus/ENIP —
   add only if not already listed).
5. Verify `SEEDED_TECHNIQUE_IDS.len() == SEEDED_TECHNIQUE_ID_COUNT` (VP-007 drift
   guard).
6. Verify `technique_info` resolves all SEEDED IDs (VP-007 catalog completeness
   harness).

### Decision 6: S7comm-plus scope — framing classification plus unencrypted session-setup metadata only

S7comm-plus (protocol-ID `0x72`) support is bounded explicitly, per human decision at
the F1 gate:

**IN scope:**
- Framing-level classification: a COTP DT-TPDU with `protocol_id == Some(0x72)` is
  counted and reported as an observed S7comm-plus session (contributes to gap-report
  visibility, does not itself register as `known-supported` in `protocols.rs` — see
  Decision 3; S7comm-plus stays in `PORT_102_UNSUPPORTED_SIBLINGS`).
- Unencrypted session-setup handshake metadata observation: the S7comm-plus session-
  setup exchange (prior to any TLS upgrade) may expose limited, unencrypted framing
  metadata (message-type/opcode byte, sequence markers) at the object/service-protocol
  envelope level. This metadata MAY be surfaced as a forensic observation finding.

**OUT of scope (explicit non-goal):**
- No `S7commPlusAnalyzer`, no S7comm-plus function-code catalog, no object/service
  dissection.
- No attempt to decrypt or interpret TLS-wrapped S7comm-plus (S7-1500 firmware ≥ 2.9,
  S7-1200 firmware ≥ 4.5 increasingly wrap the session in TLS) — an offline analyzer
  without keys sees only endpoints, sizes, and timing for TLS-protected flows, which the
  existing TLS analyzer (SS-07) already covers generically.
- No integrity/anti-replay material interpretation.

This boundary is deliberately asymmetric with classic S7comm: S7comm-plus is
*observed*, not *dissected*. The catalog-model fix in Decision 3 reflects this — only
classic S7comm (`0x32`) is promoted to `known-supported`.

### Decision 7: Test-fixture provenance — synthetic committed, real captures fetch-only

Per `.factory/cycles/feature-s7comm/f2-pcap-fixture-sourcing.md`: no public S7comm PCAP
was found that is simultaneously real, small, cleanly labeled, AND under a clearly
permissive license with no positive evidence of third-party GPL/LGPL origin. Every
small, well-labeled real capture traces to one GPLv2 lineage (the Wireshark S7comm
dissector's own test traces), re-hosted under a CC-BY-4.0 wrapper by
ITI/ICS-Security-Tools — a CC-BY wrapper does not launder GPLv2-origin content, mirroring
this project's existing `iec104-iti-dissect.pcap` fetch-only ruling (F-009/D-524).

**Decision:**
1. **Committed AC/unit and small E2E fixtures are SYNTHETIC.** A new generator,
   `tests/fixtures/mk_s7comm_pcap.py`, hand-crafts minimal TPKT + COTP + S7comm-PDU byte
   sequences (Setup Communication `0xF0`, Read/Write Var `0x04`/`0x05`, PLC STOP/START,
   Request Download/Download Block/Download Ended, Upload, Userdata/SZL read, and one
   S7comm-plus `0x72` framing skeleton for dispatch coverage), dedicated CC0/MIT,
   following the `mk_modbus_large_pcap.py` precedent.
2. **Real-world E2E validation captures are FETCH-ONLY, gitignored, SHA-256-pinned**
   under `tests/fixtures/local-samples/`, wired into `bin/fetch-e2e-pcaps`, never
   committed — the same treatment as `iec104-iti-dissect.pcap` and
   `dnp3dataset_capture.pcap`. Best candidates: the ITI `pcaps/s7/` set (GPLv2-origin)
   and cisagov `testing/traces/*.pcap` (BSD-3-Clause repo, but no trace-file-specific
   license grant — treated as fetch-only pending written INL/CISA confirmation).
3. **Never commit** any Wireshark-wiki S7comm capture, SourceForge s7commwireshark
   sample, or ITI `pcaps/s7/` file.

### Decision 8: TPKT/COTP reassembly — directional carry buffers sized to the TPKT length-field ceiling; walk-first residual-bound semantics

TPKT frames can span TCP segment boundaries; the TPKT `length` field (Decision 1) bounds
a frame that may arrive fragmented, exactly as IEC-104's APCI `LEN` octet does. This ADR
adopts the same **directional carry-buffer split** (RULING-DNP3-SIBLING-001, ADR-007
Decision 2 / ADR-013 Decision 2) — `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>` on
`S7commFlowState` — and the same **walk-first, residual-bound semantics**
(WALK-FIRST-RESIDUAL-BOUND, ADR-013 Decision 2/3) rather than an aggregate
carry-plus-delivery pre-check: the frame-walk loop runs unconditionally on carry +
incoming data, extracting all complete TPKT frames first; the byte bound is applied only
to the leftover partial-frame residual stashed back into carry. Aggregate pre-check is
rejected for the identical reason ADR-013 rejected it for IEC-104: it is a
Ptacek/Newsham-class evasion channel (an attacker pads a burst to push the total over
the bound, causing the monitor to drop an already-complete malicious frame at the head
while the endpoint, which reassembles at the TCP layer, processes it normally).

**Carry-buffer sizing: `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535`.** This bound derives
from the TPKT `length` field itself (RFC 1006 §5: 16-bit unsigned, "length of entire
packet in octets, including packet header" — max 65,535), **not** from COTP's
single-byte Length Indicator (ISO 8073 §13.2, max 254), which bounds only the COTP
header's own variable part. The S7comm user-data trailing the fixed+variable COTP header
can occupy the remainder of the TPKT packet up to the 65,535 ceiling — classic
block-download PDUs (Request Download/Download Block/Download Ended) are the traffic
class most likely to approach this size. This is dramatically larger than every prior
binary-ICS carry cap (IEC-104: 255 bytes; DNP3: 292 bytes; ENIP: 600 bytes) because it
reflects the actual protocol's on-wire ceiling, not an artifact of insufficient care —
see Consequences/Negative for the resulting per-flow memory trade-off.

**Resync anchor:** the TPKT `version` byte (always `0x03` for a valid TPKT packet) is
the resync candidate on a bad-start-byte condition, mirroring IEC-104's `0x68` resync
anchor (ADR-013 Decision 3 step 3) — advance 1 byte at a time on invalid version bytes,
never 2, to avoid skipping a real `0x03` at the next offset.

**Carry-overflow reaction** (bound-trip on malformed or adversarial carry, mirrors
ADR-013 Decision 2): clear the offending direction's residual carry, byte-walk forward
to the next `0x03` candidate (drop-and-rescan, not a permanent desync latch), and emit
**one T0814 (Anomaly/Possible/Medium) per flow direction** via a dedicated per-direction
carry-overflow dedup flag, distinct from the malformed-length dedup flag used for
in-range TPKT-length validation failures.

### Decision 9: Pure-core free-fn design for verification amenability

Three functions are pure-core free `fn`s (module scope, not `impl` methods), following
the `parse_apci_header`/`classify_frame_format` (IEC-104), `parse_mbap_header`/
`classify_fc` (Modbus), and `parse_enip_header`/`classify_enip_command` (EtherNet/IP)
precedent:

1. `iso_on_tcp::parse_tpkt_header(data: &[u8]) -> Option<TpktHeader>` (SS-20) — Kani
   candidate: no panic, no out-of-bounds index, `length` field bounds respected.
2. `iso_on_tcp::parse_cotp_header(tpkt_payload: &[u8]) -> Option<CotpHeader>` (SS-20) —
   Kani candidate: no panic, no OOB, correct TPDU-type/protocol-ID-byte extraction.
3. `s7comm::parse_s7comm_header(data: &[u8]) -> Option<S7commHeader>` (SS-21) — parses
   ROSCTR, PDU reference, parameter/data length from a classified `0x32` payload;
   cargo-fuzz candidate (combined with the on_data frame-walk loop) rather than Kani,
   mirroring VP-047's IEC-104 treatment of `parse_asdu`.

**Tool selection:** Kani P0 for the two SS-20 header-parse functions' arithmetic safety
(bounds, no overflow) — the smallest, most tractable pure functions in the new surface,
exactly the profile Kani is suited to (mirrors VP-044). proptest P1 for the protocol-ID
branch totality (Decision 2's four-way match must be exhaustive over all `u8` values)
and directional carry-buffer isolation (mirrors VP-045/VP-046). cargo-fuzz P1 for the
combined TPKT→COTP→S7comm parse chain's no-panic property under arbitrary byte input
(mirrors VP-047).

**VP numbering is explicitly deferred to product-owner** at F2 BC/VP authoring (this ADR
does not register new VP-NNN IDs; VP-004 and VP-007 are pre-existing obligations being
extended, not new VPs). The last currently-registered VP is VP-047 (IEC-104); the F1
delta analysis estimates 4-6 new VPs for this feature, so product-owner should expect to
allocate in the VP-048 range.

### Decision 10: MMS/ICCP remain explicitly out of scope; `PORT_102_NOTE` consequence

IEC 61850 MMS and ICCP/TASE.2 traffic on port 102 is classified only to the extent
Decision 2's disambiguation table allows (an observed COTP DT-TPDU with a non-`0x32`/
`0x72` protocol-ID, or an unparseable payload) — **no dissection of either protocol is
in scope for this feature.** Both remain in `PORT_102_UNSUPPORTED_SIBLINGS`
(Decision 3) and therefore `known-unsupported` in the coverage-gap catalog, unchanged
from ADR-012's original ruling.

**Consequence for `main.rs`'s existing `PORT_102_NOTE`/`collision_note` machinery:**
today this logic unconditionally treats **all four** port-102 catalog names as an
undifferentiated collision (name omitted, generic footnote naming all four). Once
S7comm is promoted (Decision 3), this becomes semantically wrong on two counts: (a) the
tri-state's `known-supported` sanity-check branch (ADR-012 Decision 2 — "should never
appear in a gap report") becomes reachable for TCP/102 for the **first time** in the
project's history, since S7comm traffic is no longer a coverage gap; (b) the footnote
text must be revised to name S7comm-plus/MMS/ICCP specifically as the remaining
port-102 gap, rather than implying all four are equally unattributed. This is a
**consequence to be implemented at F4** (not a new architectural decision beyond what
Decisions 2/3 already establish) — flagged here so it is not rediscovered as a surprise
regression during implementation.

## Rationale

Port-102-only dispatch (Decision 2) follows the established pattern from Modbus
(ADR-005), DNP3 (ADR-007), EtherNet/IP (ADR-010), and IEC-104 (ADR-013): no reliable
content-signature exists for ISO-on-TCP traffic at the dispatcher layer (a TPKT version
byte `0x03` is even less discriminating than IEC-104's already-rejected single-byte
`0x68`), so port-fallback is correct and consistent. What is genuinely new is that this
port's post-classification disambiguation is load-bearing, not merely defensive
(Decision 2) — a departure the ADR makes explicit rather than silently inheriting the
IEC-104 framing.

The two-module split (Decision 1) is chosen over ENIP's in-file precedent because the
outer two layers are reusable across three future catalog entries, not one — the
"build once, benefit three times" argument from the research brief is only real if the
module boundary enforces it, so the boundary is drawn at the file level, not just
conceptually.

The port-102 catalog fix (Decision 3) is the one decision that changes a previously
load-bearing, tested invariant (ADR-012 Decision 5's pure-intersection model). The
ratified fix, a per-entry `Support` enum (Option d), is chosen over this ADR's original
minimal-diff recommendation (Option b, a name-keyed exclusion list) because the
independent validation brief
(`.factory/cycles/feature-s7comm/f2-port102-model-validation.md`) showed that (b)
optimizes for this diff's size at the cost of the model: it keeps a "port ⇒ support"
derivation every comparable mature analyzer (Wireshark, Suricata, Zeek) rejects, carries
an unsafe allow-by-default polarity for future same-port entries, and — decisively —
cannot express the `DetectionOnly` state Decision 6 already requires for S7comm-plus.
The `Support` enum costs more diff now (all ~30 `KnownProtocol` literals gain an
explicit field) but is a single, compiler-checked source of truth going forward, and
brings the static catalog into line with the Suricata-derived tri-state vocabulary
ADR-0012 Decision 2 already adopted for the dynamic report.

The MITRE technique set (Decision 5) is grounded in live-page verification (not
inference from technique names), following the project's standing revocation-diligence
discipline; the two new `MitreTactic` variants are added only because live-page
verification showed no existing variant covers TA0109/TA0110 — T0821 deliberately reuses
`IcsExecution` rather than inventing a redundant variant.

The licensing constraint (Decision 4) and fixture provenance (Decision 7) are both
non-negotiable: S7comm's most complete, best-labeled prior art is uniformly GPL/LGPL-
tainted at the source level (Wireshark, Snap7, libnodave) or of unrecoverable license at
the crate level, so an original clean-room parser plus synthetic committed fixtures is
the only compliant path — exactly the posture ADR-013 established for IEC-104.

## Consequences

### Positive

- S7comm analysis adds three new MITRE techniques (T0843, T0889, T0821) covering
  program-download and controller-tasking manipulation — a significant gap given
  Stuxnet's continued relevance and CISA AA26-231A's 2026 targeting of S7 environments.
- The TPKT/COTP framing layer (SS-20) is architected for reuse; a future IEC 61850 MMS
  or ICCP/TASE.2 cycle inherits `parse_tpkt_header`/`parse_cotp_header` with zero
  changes to `iso_on_tcp.rs`.
- The port-102 catalog fix (Decision 3) resolves a documented architectural gap from
  ADR-012 with a per-entry `Support` enum that matches prior art (Wireshark/Suricata/
  Zeek all key "supported" on an explicit per-protocol registration, never a port) and
  reuses the Suricata-derived vocabulary ADR-0012 Decision 2 already established for the
  dynamic report — closing a static-vs-dynamic modeling mismatch, not opening a new one.
  It also natively expresses S7comm-plus's `DetectionOnly` state (Decision 6), which no
  narrower fix (bool or exclusion list) could represent.
- Follows the established ADR-005/007/010/013 pattern with no dispatcher-level
  architectural surprise; the one genuine novelty (in-analyzer disambiguation being
  load-bearing) is explicitly called out rather than silently inherited.

### Negative / Trade-offs

- The 65,535-byte-per-direction carry buffer (Decision 8) is two orders of magnitude
  larger than any prior binary-ICS analyzer's cap (IEC-104: 255 bytes). Total memory
  exposure remains bounded by the existing `max_flows`/`memcap` configuration on
  `ReassemblyConfig` (SS-04), but per-flow worst case is materially higher than any
  sibling analyzer; this should be flagged to performance-engineer for benchmark
  attention at F6 hardening.
- S7comm-plus's "framing classification plus unencrypted metadata" scope (Decision 6)
  is deliberately incomplete — it will not satisfy an operator wanting full S7-1200/1500
  visibility. This is a conscious, human-ratified boundary (F1 §7.5), not an oversight.
- The port-102 catalog fix (Decision 3), now a per-entry `Support` enum, touches all
  ~30 `KnownProtocol` literals (each must supply an explicit `support:` value) plus the
  named REGRESSION-GUARD test in `tests/protocols_tests.rs` and the VP-041 proptest
  oracle (`proptest_vp041_oracle_cross_check`, `proptest_vp041_partition_invariant`),
  both of which must be updated — not deleted — to assert against `entry.support`
  instead of `SUPPORTED_PORTS`. This is a larger one-time diff than the originally-
  recommended exclusion list (Option b), traded deliberately for a single,
  compiler-checked source of truth instead of coupled lists. It does **not**, by itself,
  touch `main.rs`'s `PORT_102_NOTE`/`collision_note` logic — that consequence remains
  scoped to F4 per Decision 10, and is unaffected by which catalog-model option was
  chosen (see Decision 3's critical caveat: the dynamic gap classifier,
  `main.rs::lookup_protocol_state`, is a separate, unsolved axis requiring the
  analyzer's `protocol_id`, not a catalog property).
- Zero official Siemens specification for classic S7comm means residual reverse-
  engineering risk persists regardless of clean-room discipline; the "interoperable, not
  certified" posture (Decision 4) is a mitigation, not an elimination, of that risk.

### Status as of 2026-09-06

**Proposed** (ADR-level status unchanged — no behavioral contracts, verification
properties, or source code exist yet for `feature-s7comm`). **Decision 3 is now
human-RATIFIED**: the port-102 catalog-model fix is the per-entry `Support` enum
(Option d), per the independent validation in
`.factory/cycles/feature-s7comm/f2-port102-model-validation.md`, superseding this ADR's
original Option (b) recommendation. Product-owner should now author BC-2.18.NNN
contracts (SS-18) encoding the `Support` enum, its four port-102 assignments, and
`supported_protocols()`/`unsupported_protocols()`'s new derivation — and BC-2.20.*/
BC-2.21.* contracts encoding the rest of this ADR — with the explicit understanding that
the dynamic gap-classifier defect (`main.rs::lookup_protocol_state`, Decision 3's
critical caveat) remains out of scope until F4 (Decision 10).

### MITRE ATT&CK for ICS Technique Set (ics-attack-19.1)

| Technique ID | Name | When Emitted | Status |
|-------------|------|---------------|--------|
| **T0843** | **Program Download** | Complete `0x1A→0x1B→0x1C` block-download sequence; optional `0x28 _INSE` activate | **NEW — add via Decision 5** |
| **T0889** | **Modify Program** | Same download sequence, or `0x28 _INSE`/`_DELE` block activate/delete | **NEW — co-tag with T0843** |
| **T0821** | **Modify Controller Tasking** | Program-download traffic involving organization blocks (OB1); reuses `IcsExecution` | **NEW — low-confidence co-tag** |
| T0835 | Manipulate I/O Image | `Write Var 0x05` → area `0x80`/`0x81`/`0x82` | Pre-existing EMITTED (Modbus); add S7comm call-site |
| T0836 | Modify Parameter | `Write Var 0x05` → `0x84`/`0x83` | Pre-existing EMITTED (Modbus); add S7comm call-site |
| T0858 | Change Operating Mode | `0x29 PLC Stop`; `0x28 P_PROGRAM` start | Pre-existing EMITTED (ENIP); add S7comm call-site |
| T0816 | Device Restart/Shutdown | Decoded `0x28` restart PI-service | Pre-existing EMITTED (ENIP); add S7comm call-site |
| T0888 | Remote System Information Discovery | Userdata `0x07`/`0x04`/`0x01` Read SZL; `0x07`/`0x03`/* block-list | Pre-existing EMITTED (Modbus); add S7comm call-site |
| T0846 | Remote System Discovery | Multi-host TCP/102 sweep evidence only | Pre-existing EMITTED (ENIP); emit only on sweep evidence |
| T0814 | Denial of Service | Connection flood; malformed-length burst threshold | Pre-existing EMITTED; add S7comm call-site |
| T1692.001 | Unauthorized Message: Command Message | Any command from a source outside an allowlist | Pre-existing EMITTED; co-tag only with positive unauthorized-source evidence |

CWE set: CWE-306 (no authentication — classic S7comm has none), CWE-319 (cleartext
transmission), CWE-311 (missing encryption for sensitive data), CWE-294 (replay via
absent session tokens), CWE-693 (protection-mechanism failure — reliance on network
segmentation only).

### Verification Properties Registered

No new VP-NNN IDs are registered by this ADR (Decision 9). VP-004 and VP-007 are
pre-existing obligations extended per Decisions 2 and 5. Product-owner registers new VPs
for `parse_tpkt_header`/`parse_cotp_header`/`parse_s7comm_header` at F2 BC/VP authoring
(anticipated VP-048 range).

## Alternatives Considered

- **Single-file design (TPKT/COTP inline inside `s7comm.rs`), mirroring ENIP's ENIP/CPF/
  CIP split:** Rejected — TPKT/COTP are shared across three future catalog entries
  (S7comm, MMS, ICCP), unlike ENIP's CPF layer which is ENIP-specific end-to-end. See
  Decision 1.

- **Separate dispatcher rules per port-102 protocol identity:** Rejected — the
  dispatcher cannot cheaply distinguish COTP protocol-IDs from raw TCP bytes without
  performing the TPKT/COTP parse itself, which is the analyzer's responsibility, not the
  dispatcher's. See Decision 2.

- **`supported: bool` field on `KnownProtocol` (port-102 fix option (a)):** Rejected —
  strictly dominated by the ratified `Support` enum (option d): identical blast radius
  (all ~30 literals gain a field), but a `bool` cannot express the `DetectionOnly` state
  Decision 6 already requires for S7comm-plus. See Decision 3.

- **Name-keyed exclusion list, `PORT_102_UNSUPPORTED_SIBLINGS` (port-102 fix option
  (b)):** This ADR's *original* recommendation; superseded by human ratification of
  option (d) after independent validation
  (`.factory/cycles/feature-s7comm/f2-port102-model-validation.md`) showed it keeps a
  "port ⇒ support" derivation every comparable mature analyzer (Wireshark, Suricata,
  Zeek) rejects, has an unsafe allow-by-default polarity for future same-port catalog
  entries, requires three coupled sources of truth instead of one, and cannot express
  `DetectionOnly`. See Decision 3.

- **`dispatch_target: Option<&'static str>` field (port-102 fix option (c)):** Rejected
  — same struct-wide blast radius as (a)/(d), plus a stringly-typed, uncompiled-checked
  coupling to `dispatcher::DispatchTarget` from a module documented as forbidden to
  depend on `dispatcher`. See Decision 3.

- **Full S7comm-plus function-code dissection:** Rejected for this cycle — increasingly
  TLS-wrapped, requires a distinct object/service protocol dissector with materially
  higher build cost and lower research maturity than classic S7comm. See Decision 6.

- **Aggregate carry-buffer pre-check (carry+delivery bound before frame extraction):**
  Rejected — identical Ptacek/Newsham-class evasion channel already ruled out for
  IEC-104 (ADR-013 Decision 2, F-172-001) and DNP3 (F-B-002); adopting it here would
  reopen a hole this project has already closed twice. See Decision 8.

- **Wireshark `packet-s7comm.c`, Snap7, libnodave as implementation templates:**
  Rejected — GPL-2.0-or-later / LGPL-3.0-or-later / LGPL-2.0-or-later, all incompatible
  with wirerust's MIT OR Apache-2.0 dual license. See Decision 4.

- **Real public S7comm PCAP captures as committed fixtures:** Rejected — every small,
  well-labeled real capture traces to GPLv2-origin Wireshark test traces, even when
  re-hosted under a CC-BY-4.0 wrapper; a permissive wrapper license does not launder
  copyleft-origin content. See Decision 7.

## Source / Origin

- **RFC 1006** (IETF, STD 35) — TPKT framing structure and length-field semantics.
- **ITU-T X.224 ≡ ISO/IEC 8073:1997** — COTP TPDU types (CR/CC/DT), Length Indicator
  field.
- **S7comm classic protocol structure** — free-to-read prose/behavioral sources only
  (Wireshark S7comm wiki page, Kleinmann & Wool 2014, Orange-Cyberdefense
  `awesome-industrial-protocols`); no GPL/LGPL source consulted as an implementation
  template (Decision 4).
- **Feature cycle:** `feature-s7comm` — this ADR governs the S7comm/ISO-on-TCP
  subsystems (SS-20, SS-21) delivered in that cycle.
- **F1/F2 research (this cycle):**
  `.factory/cycles/feature-s7comm/f1-delta-analysis.md` (scope, impact boundary,
  regression risk, port-102 catalog-model problem statement);
  `.factory/cycles/feature-s7comm/f2-license-matrix.md` (clean-room provenance gate,
  per-source license matrix);
  `.factory/research/s7comm-mitre-ics-tagging.md` (MITRE technique validation,
  live-page tactic verification for T0843/T0889/T0821);
  `.factory/cycles/feature-s7comm/f2-pcap-fixture-sourcing.md` (fixture provenance
  ranking and synthesis recommendation);
  `.factory/cycles/feature-s7comm/f2-port102-model-validation.md` (independent
  validation of the Decision 3 port-102 catalog-model fix against Wireshark/Suricata/
  Zeek prior art and the extensibility axis — the grounding for the human ratification
  of option (d), the `Support` enum, superseding this ADR's original option (b)
  recommendation).
- **Predecessor ADRs:** ADR-005 (Modbus, port-fallback pattern); ADR-007 (DNP3,
  directional carry-buffer split + pure-core free-fn pattern); ADR-010 (EtherNet/IP,
  multi-level framing + VP-007 atomic obligation pattern); ADR-012 (Protocol Coverage
  Catalog, the pure-intersection model this ADR amends — Decision 2's Suricata-derived
  tri-state vocabulary is reused by Decision 3's `Support` enum); ADR-013 (IEC-104, the
  direct structural template for this ADR — walk-first residual-bound carry semantics,
  licensing-constraint format, pure-core free-fn Kani design).
- **Behavioral contracts:** BC-2.18.NNN (SS-18 `protocols.rs` `Support` enum and its
  port-102 assignments, per Decision 3 — not yet authored), BC-2.20.* (SS-20 ISO-on-TCP
  framing, not yet authored), BC-2.21.* (SS-21 S7comm analysis, not yet authored) — to
  be authored by product-owner following this ADR.
- **MITRE ATT&CK for ICS v19.1** (currency-checked against live v19.2, no ICS catalog
  delta): T0843 "Program Download" (Lateral Movement, TA0109); T0889 "Modify Program"
  (Persistence, TA0110); T0821 "Modify Controller Tasking" (Execution, TA0104); T0835,
  T0836, T0858, T0816, T0888, T0846, T0814, T1692.001 (all pre-existing, reused).
