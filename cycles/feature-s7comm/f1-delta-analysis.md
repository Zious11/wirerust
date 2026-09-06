---
document_type: feature-delta-analysis
phase: F1
feature_cycle: feature-s7comm
date: 2026-09-06
actor: architect
intent: feature
feature_type: backend
trivial_scope: false
severity: n/a
inputs:
  - .factory/planning/next-ot-protocol-research.md
  - docs/adr/0005-binary-ics-protocol-integration.md
  - docs/adr/0010-ethernet-ip-cip-stream-dispatch.md
  - docs/adr/0012-protocols-catalog-and-coverage-gaps.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - src/protocols.rs
  - src/dispatcher.rs
  - src/mitre.rs
  - src/analyzer/iec104.rs
  - src/analyzer/enip.rs
  - src/main.rs
  - src/cli.rs
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/stories/epics.md
input-hash: "e2ffd43"
---

# F1 Delta Analysis: Classic Siemens S7comm over TCP/102 (ISO-on-TCP)

## 0. Precedent Cycles Consulted

| Cycle / ADR | Relevance |
|---|---|
| `feature-iec104` / **ADR-013** | Closest precedent: single-file binary-ICS analyzer added to a port-fallback dispatcher rule, pure-core free-fn parse functions for Kani, directional carry buffers, licensing-constraint decision, VP-004/VP-007 atomic obligations. This F1 report follows its shape. |
| **ADR-005** (Modbus) | Established binary-ICS-port-fallback pattern as a documented exception to ADR-0001 content-first dispatch. |
| **ADR-010** (EtherNet/IP) | Established the *multi-level framing* pattern (ENIP→CPF→CIP) and the VP-007 six-part atomic MITRE-seeding obligation — directly analogous to the S7comm three-level framing (TPKT→COTP→S7comm). |
| **ADR-012** (Protocols catalog) | Already documents the **TCP/102 four-way collision** (S7comm / S7comm-plus / IEC 61850 MMS / ICCP-TASE.2) as a caveat (Decision 3b) and already carries a `PORT_102_NOTE` footnote mechanism in `main.rs` and a REGRESSION-GUARD test in `tests/protocols_tests.rs`. This is the single most load-bearing existing artifact for this feature — see §4. |

## 1. Feature Summary + Scope Boundary

**Feature:** Passive dissection of classic Siemens S7comm traffic on TCP port 102, keyed off the COTP user-data protocol-ID byte `0x32`, built on a new TPKT (RFC 1006) + COTP (ISO 8073) ISO-on-TCP framing/dispatch layer.

Grounded in `.factory/planning/next-ot-protocol-research.md` (2026-09-06 research brief): S7comm is the top-ranked next-protocol recommendation — highest threat signal in the candidate pool (Stuxnet, CISA AA26-231A active 2026 targeting), richest ATT&CK-for-ICS mapping, top-tier prevalence, and a "medium — the good kind" feasibility cost whose TPKT/COTP prerequisite is a reusable investment for IEC 61850 MMS and ICCP/TASE.2 later (all three share port 102).

### IN scope

1. **TPKT (RFC 1006) + COTP (ISO 8073) ISO-on-TCP framing/dispatch layer.** TCP reassembly → TPKT 4-byte header (version=3, reserved, length u16 BE) → COTP (CR/CC/DT TPDU types; DT is the steady-state data-transfer TPDU carrying the upper-layer payload).
2. **Classic S7comm full passive dissection**, keyed off the COTP user-data protocol-ID byte: `0x32` = classic S7comm. Job request/response function codes, PDU header (ROSCTR, PDU reference, parameter/data length), parameter and data blocks for the function codes that map to the approved MITRE technique set (§2/§3).
3. **Catalog promotion** of the `S7comm` entry in `src/protocols.rs::KNOWN_PROTOCOLS` from unsupported → supported (the entry already exists — see §4).
4. **MITRE ATT&CK-for-ICS technique attribution** for classic S7comm PDUs. Candidates from the research brief requiring validation (§7): T0843 Program Download, T0889 Modify Program, T0835 Manipulate I/O Image, T0836 Modify Parameter, T0821 Modify Controller Tasking. (T0835 and T0836 are already SEEDED and EMITTED in `src/mitre.rs` — shared across analyzers; T0843, T0889, T0821 are **not** currently in the catalog and would be new entries if validated as passively emittable.)
5. **S7comm-plus — framing-level classification only.** COTP protocol-ID `0x72` is classified/counted (S7comm-plus session observed) but the PDU payload (private integrity/anti-replay, increasingly TLS-wrapped) is NOT semantically decoded. No `S7commPlusAnalyzer`, no S7comm-plus function-code catalog.

### OUT of scope (explicit non-goals)

1. **Deep S7comm-plus dissection.** Encrypted/obfuscated payload; framing-level classification only per IN-scope item 5. Do not scope a semantic S7comm-plus parser.
2. **Active polling / any transmit path.** wirerust is passive-only; this is already a project-wide invariant, restated here for completeness.
3. **IEC 61850 MMS dissection.** Port-102 traffic whose COTP payload begins with an OSI Session/Presentation/ACSE pattern (MMS-family) is classified as `known-unsupported` (already true today per ADR-012) and MUST NOT be misattributed to S7comm. The TPKT/COTP layer is built to be **shared** by a future MMS cycle, but MMS's ASN.1 BER dissection is not built in this cycle.
4. **ICCP/TASE.2 dissection.** Same treatment as MMS — classification/non-misattribution only, no dissection. (Research brief explicitly de-prioritizes ICCP: sparsest prior art, highest build cost.)

**Rationale for the boundary:** the research brief's core trade-off is "build the TPKT/COTP layer once, dissect S7comm fully, and leave S7comm-plus/MMS/ICCP as correctly-classified-but-undissected traffic on the same port." This mirrors the ADR-012 tri-state vocabulary (`known-supported` / `known-unsupported` / `unknown`) exactly — S7comm moves to `known-supported`, the other three siblings remain `known-unsupported`, and nothing on port 102 should ever fall through to `unknown` once the dispatcher recognizes the TPKT/COTP envelope.

## 2. Impact Boundary

### 2.1 New subsystems (proposed; human confirms exact numbers at F1 gate)

Highest subsystem in `ARCH-INDEX.md` today is **SS-19** (IEC-104). Proposed:

| ID | Name | Rationale |
|---|---|---|
| **SS-20** | ISO-on-TCP Framing (TPKT/COTP) | New shared framing layer, analogous in role to how `reassembly/` sits under every TCP analyzer, but scoped to TPKT/COTP only. Deliberately its own subsystem (not folded into SS-21) because it is designed to be reused by a future MMS/ICCP cycle — precedent: SS-05 (dispatch) is already factored out from per-protocol analyzer subsystems for the same reason. |
| **SS-21** | S7comm Analysis | The S7comm dissector proper (function codes, PDU parse, MITRE emission), consuming SS-20's parsed COTP payload the way `Iec104Analyzer` (SS-19) consumes reassembled TCP bytes. |

Next available CAP-NNN: **CAP-20**, **CAP-21** (SS-18/CAP-18 and SS-19/CAP-19 are the latest allocated).
Next available ADR: **ADR-014** (0013 is the latest accepted; 0008 is a withdrawn placeholder).
Proposed BC namespace: **BC-2.20.NNN** (TPKT/COTP) and **BC-2.21.NNN** (S7comm), plus the standard cross-subsystem extension rows this pattern always requires in SS-05 (dispatcher), SS-10 (MITRE), SS-12 (CLI/reporter), and SS-18 (protocol catalog) — see §3.

### 2.2 File-level impact table

| File | Change | Notes |
|---|---|---|
| `src/analyzer/iso_on_tcp.rs` **(NEW)** | New module | Pure-core TPKT header parse (`parse_tpkt_header`) + COTP TPDU parse (`parse_cotp_header`, CR/CC/DT discrimination + protocol-ID byte extraction for DT). Free `fn`s, no `impl`, mirroring `parse_apci_header`/`parse_enip_header` for Kani amenability. Deliberately placed under `analyzer/` (not top-level) since it is dissection logic, not generic reassembly infra — but written with zero dependency on `s7comm.rs` so a future `mms.rs`/`iccp.rs` can import it directly. |
| `src/analyzer/s7comm.rs` **(NEW)** | New module | `S7commAnalyzer` implementing the same per-flow-state / `summarize()` / `findings()` pattern as `Iec104Analyzer`. Consumes `iso_on_tcp::parse_cotp_header` output; branches on protocol-ID (`0x32` classic → full S7comm PDU parse; `0x72` S7comm-plus → framing-only classification; anything else COTP-DT-shaped → left unclassified for the MMS/ICCP gap path, not force-fit into S7comm). |
| `src/dispatcher.rs` | MODIFIED | New `DispatchTarget::S7comm` variant (single target for ALL of port 102 — the S7comm/S7comm-plus/MMS/ICCP disambiguation happens *inside* the analyzer after TPKT/COTP parse, not as separate dispatcher rules, because the dispatcher only sees TCP bytes and cannot cheaply distinguish COTP protocol-IDs without doing the TPKT/COTP parse itself — doing that parse IS the analyzer's job). New Rule 9 (port 102, appended after Rule 8/IEC-104) is a **documented exception layered on the existing ADR-005 exception**: it is port-only, like all prior binary-ICS rules, but it is the **first port rule that does not map 1:1 to a single protocol identity** — the post-classification disambiguation inside `S7commAnalyzer` is now load-bearing for correctness, not just defense-in-depth (contrast with IEC-104's `is_valid_iec104_frame`, which only rejects garbage, never re-routes to a *different* named protocol). New `Option<S7commAnalyzer>` field, `on_data`/`on_flow_close` match arms, early-exit guard extension, `classify_oracle` atomic update (VP-004 obligation, ADR-013 Decision 9 pattern), rule-order doc-comment update (Rules 1–9 + fallback). |
| `src/protocols.rs` | MODIFIED | `S7comm` catalog entry already exists (lines 176–185) — **no new entry needed**, only its supported/unsupported classification changes. This is the file with the highest-risk change in the whole delta — see §4.1 (the port-102 four-way collision breaks the existing pure-intersection `supported_protocols()` derivation model). `SUPPORTED_PORTS` doc-comment and possibly its type/derivation function need to change, not just its value. |
| `src/mitre.rs` | MODIFIED | New `technique_info()` arms for whichever of T0843/T0889/T0821 (and possibly T0873.001) survive the passive-detectability validation in §7. `SEEDED_TECHNIQUE_IDS`/`SEEDED_TECHNIQUE_ID_COUNT` bump. `EMITTED_IDS` extension for newly-emitting techniques. T0835/T0836 need NO catalog change (already seeded+emitted) — only new emission call-sites in `s7comm.rs`. Six-part VP-007 atomic obligation per ADR-010/ADR-013 precedent. |
| `src/cli.rs` | MODIFIED | New `--s7comm` boolean flag on `Commands::Analyze`, default-off, included by `--all` — mirrors `--iec104`/`--enip`/`--dnp3`. Possibly a burst-threshold or similar tunable flag if a rate-based detection (e.g., repeated Program Download / Stop CPU) is scoped in F2. |
| `src/main.rs` | MODIFIED | `enable_s7comm` threading through the same 15-touch-point pattern already used for `enable_iec104` (needs_reassembly aggregation, reassembly-guard `eprintln!`, `S7commAnalyzer::new()` construction gated on reassembly, dispatcher field wiring, `take_s7comm_analyzer()` collection into `all_findings`/`analyzer_summaries`). The port-102 `PORT_102_NOTE` collision-footnote logic (lines ~1016–1270) needs revision — see §4.1: once S7comm is `known-supported`, the "all four names collapse to a collision note" logic must special-case S7comm out (S7comm gaps, if any occur due to a routing edge case, are a dissector bug, not an attribution ambiguity — the tri-state's `known-supported` sanity-check branch, ADR-012 Decision 2, becomes reachable for port 102 for the first time). |
| `docs/adr/0014-*.md` **(NEW)** | New ADR | S7comm + TPKT/COTP dispatch design, following ADR-013's shape (subsystems_affected: SS-05, SS-10, SS-20, SS-21; licensing-constraint decision per §2.3; port-102 disambiguation decision). |
| `.factory/specs/architecture/ARCH-INDEX.md` | MODIFIED | Subsystem Registry rows for SS-20/SS-21; ADR-014 row; resource-bound table rows (carry-buffer sizes for TPKT/COTP + S7comm). |
| `.factory/specs/architecture/ss-20-iso-on-tcp-framing.md`, `ss-21-s7comm-analysis.md` **(NEW)** | New architecture shards | Following the `ss-18-*.md`/`ss-19-*.md` sharding convention. |
| `.factory/stories/epics.md` | MODIFIED | New epic **E-23** (S7comm), following the E-22/IEC-104 template. |
| Reporter (`src/reporter/{csv,json,json_dto,terminal,mod}.rs`) | **DEPENDENT, likely unmodified** | Per the IEC-104 precedent (0 hits for `iec104`/`Iec104` across all `reporter/*.rs`), the reporter is generic over `AnalysisSummary`/`Finding` — new analyzers plug in without reporter code changes. Confirm this holds for S7comm during F2 (no reason to expect otherwise; flagged as DEPENDENT not MODIFIED). |
| `src/reassembly/*` | **DEPENDENT, unmodified** | TCP reassembly is protocol-agnostic; S7comm/TPKT/COTP needs `needs_reassembly` inclusion (a `main.rs` config concern) but no reassembly-engine code change, mirroring every prior binary-ICS analyzer. |
| `src/decoder.rs`, `src/findings.rs`, `src/summary.rs` | **DEPENDENT, unmodified** | Consumed as-is (Finding/Confidence/Verdict types, decode loop). No precedent binary-ICS cycle has touched these. |

### 2.3 TPKT/COTP framing layer — design recommendation

**Recommendation: new module, not an extension of an existing file.** Rationale:

- It is a genuinely new *layer* (TPKT ⊃ COTP ⊃ upper-layer payload) sitting between TCP reassembly and the S7comm PDU parse — structurally analogous to how ENIP's encapsulation-header/CPF-item split (ADR-010 Decision 2) is two pure-core free-fn layers in one file, except here the outer layers (TPKT, COTP) are **protocol-agnostic across three future catalog entries** (S7comm, MMS, ICCP), whereas ENIP's CPF layer is ENIP-specific.
- Placing it in its own file (`src/analyzer/iso_on_tcp.rs`) rather than inline in `s7comm.rs` is what makes the "one architectural investment unlocks three catalog entries" dividend from the research brief real — a future MMS cycle imports `iso_on_tcp::parse_tpkt_header`/`parse_cotp_header` without touching `s7comm.rs` at all.
- It should export pure functions only (`parse_tpkt_header(&[u8]) -> Option<TpktHeader>`, `parse_cotp_header(&[u8]) -> Option<CotpHeader>` with a `protocol_id: Option<u8>` field for DT-TPDUs), no `StreamAnalyzer` impl of its own — it is a parsing library consumed by `S7commAnalyzer`, not an independent dispatch target. This keeps the dispatcher's `DispatchTarget` enum from needing an `IsoOnTcp` variant that would have no analyzer behind it.

**Reassembly interaction:** TPKT frames can span TCP segments (TPKT length field, like IEC-104's APCI LEN, bounds a frame that may arrive fragmented). This requires the same directional-carry-buffer pattern as every prior binary-ICS analyzer (ADR-007 Decision 2 / ADR-013 Decision 2: `carry_c2s`/`carry_s2c`, walk-first residual-bound semantics per the IEC-104 F-172-001 ruling — the aggregate-pre-check evasion channel finding applies here too and should be inherited, not rediscovered). The carry buffer belongs on `S7commFlowState` (or a shared `IsoOnTcpFlowState` if SS-20 tracks its own per-flow state independently of SS-21) — this placement decision is an F2 architecture question, not resolved here.

### 2.4 Licensing constraint (carry forward ADR-013 Decision 7 pattern)

S7comm has extensive prior art with restrictive licenses: Wireshark's `packet-s7comm.c` (GPLv2, BANNED per existing project policy), the CISA/INL **ICSNPP-S7Comm** Zeek analyzer (license TBD — verify), `libnodave`/`libs7comm` (C, license TBD — verify), and possibly a `snap7`-derived crate (license TBD — the research brief flags `snap7`/`python-snap7` as the tool used in the CISA AA26-231A advisory's *attack* tooling, not as a dissection reference — do not conflate). **F2 architecture work must run the same license-matrix check ADR-013 Decision 7 ran for IEC-104** before any design-reference code is consulted. This is a blocking gate, not a nice-to-have — flagged as an open question in §7.

## 3. Affected Specs (F2 will author; not authored here)

| Artifact type | Count (estimate) | Notes |
|---|---|---|
| New ADR | 1 | ADR-014, following ADR-013's shape (Context / Decisions / Rationale / Consequences / Alternatives / Source). Likely 8–10 decisions given the extra framing layer versus IEC-104's single-layer APCI/ASDU split (compare to ENIP's 9 decisions for its two-level ENIP/CPF/CIP framing). |
| New PRD behavioral contracts | ~35–45, split across two new subsystem shards | Estimate by precedent: IEC-104 (single-layer, simpler protocol) had 30 BCs (BC-2.19.001–030) plus 3 cross-subsystem extension rows. S7comm's extra framing layer (TPKT+COTP, distinct from the S7comm PDU layer itself) plus the three-way protocol-ID branch (classic/plus/other) argues for a modest increase: rough split ~10–15 BCs for BC-2.20.NNN (TPKT/COTP: header parse, COTP TPDU-type discrimination, protocol-ID branch, carry-buffer/frame-walk) + ~20–25 BCs for BC-2.21.NNN (S7comm: PDU header parse, function-code classification, per-technique detections, S7comm-plus framing-only path, dispatcher/CLI/catalog/MITRE cross-subsystem extension rows in SS-05/SS-10/SS-12/SS-18 — expect 4–5 such rows, matching the IEC-104 precedent's 3). |
| New verification properties (VP) | 4–6 | By direct analogy to VP-044..047 (IEC-104: 1 Kani P0 + 2 proptest P1 + 1 fuzz P1): expect (a) Kani P0 for `parse_tpkt_header`/`parse_cotp_header` arithmetic safety (length-field bounds, no OOB, no overflow — likely 1–2 VPs since two pure functions), (b) proptest P1 for directional carry-buffer isolation (mirrors VP-045) and protocol-ID branch totality (0x32/0x72/other — mirrors VP-046), (c) cargo-fuzz P1 no-panic harness for the combined TPKT/COTP/S7comm parse chain (mirrors VP-047). |
| Domain-spec capability addition | 1 new CAP (or 2, if TPKT/COTP is split into its own CAP like SS-18/SS-19 have distinct CAP-18/CAP-19) | CAP-20 (ISO-on-TCP framing) + CAP-21 (S7comm analysis), tracing to the domain-spec capabilities.md the way CAP-19 traces to IEC-104. |
| Story count | See §6 | |

**Not authored in F1:** none of the above documents are written in this phase. This table exists to size F2's workload and give the human a basis for the F1 approval-gate decision.

## 4. Regression Risk

### 4.1 HIGH — the port-102 catalog-model collision (the central regression risk of this feature)

ADR-012 already anticipated and partially defended against this feature. Three artifacts currently assert, as a **regression guard**, that S7comm is *unsupported*:

1. **`src/protocols.rs`** doc-comment (lines 78–85): "Total supported: 8; total unsupported: 22" and the explicit statement that IEC-104 is "promoted-in-place" via **port-filter membership** — i.e., `supported_protocols()` returns any `KnownProtocol` whose `canonical_ports` intersects `SUPPORTED_PORTS`.
2. **`tests/protocols_tests.rs`** (lines 463–510): a REGRESSION-GUARD test explicitly named for this exact collision — "Verifies the port-102 four-way collision — S7comm, S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 all exist in `KNOWN_PROTOCOLS` with canonical port 102... **None of these are in `SUPPORTED_PORTS`** (port 102 is absent), so all four appear in [unsupported]." This test's premise — that port 102 is entirely absent from `SUPPORTED_PORTS` — becomes false the moment S7comm is promoted.
3. **`src/main.rs`** `PORT_102_NOTE` machinery (lines ~1016–1270) and its integration tests (`tests/integration_tests.rs` lines 278–360, 1434–1500, 1807–1875): the collision footnote/JSON `collision_note` field currently assumes port 102 is **entirely** `known-unsupported` (all four names collapse into one footnote, `name` field omitted for the whole port). Once S7comm is `known-supported`, this becomes semantically wrong on two counts: (a) the tri-state's `known-supported` sanity-check branch (ADR-012 Decision 2 — "should never appear in gap report") becomes reachable for TCP/102 for the first time in the project's history, and (b) the collision note must be rewritten to say something like "TCP/102 also hosts S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 (unsupported)" rather than blanket-naming all four as equally unattributed.

**Root architectural cause:** `supported_protocols()`'s derivation (ADR-012 Decision 5) is a pure set-intersection between `canonical_ports` and `SUPPORTED_PORTS` — a model that implicitly assumes a port maps to *at most one* protocol identity (true for every port used so far: 502→Modbus, 20000→DNP3, 44818→ENIP, 2404→IEC-104, 443/8443→TLS, 80/8080→HTTP, 53→DNS). Port 102 breaks that assumption: **one port, four catalog entries, only one of which becomes dissected.** Naively adding `102` to `SUPPORTED_PORTS` would make `supported_protocols()` incorrectly report S7comm-plus, MMS, and ICCP as "supported" too (since they all share `canonical_ports: &[102]`), inflating the supported count to 11 instead of 9 and producing three false "known-supported but never actually dissected" entries. This is worse than the status quo, not better.

**This is a genuine design problem F2 must resolve, not a mechanical catalog edit.** The existing ARP precedent (ADR-012 Decision 5: "ARP handling... a special case in `supported_protocols()`") shows the catalog module already tolerates one hand-coded exception to the pure-intersection rule; S7comm needs a second, structurally different one (ARP's special case adds a supported entry with *no* port; S7comm's needs to correctly attribute *one of several entries sharing a port*). Candidate resolutions for F2 to evaluate (not decided here): (a) a `supported: bool` field directly on `KnownProtocol` replacing derived-from-ports logic entirely (cleanest, but is a small breaking change to the catalog's design philosophy); (b) a name-keyed exception list mirroring the ARP special case, in the opposite direction (exclude S7comm-plus/MMS/ICCP from the intersection despite the port match); (c) a `dispatch_target: Option<&'static str>` field asserting exactly which catalog entry a given `DispatchTarget` variant actually promotes. **Recommendation for the human: flag this as a Decision Point at the F1 gate** (see §8) since it changes a documented, tested architectural invariant (ADR-012 Decision 5), not just a value.

**Regression-test surface (must all be reviewed/updated in F2/F4, in this order of blast radius):**
- `tests/protocols_tests.rs` — the port-102-collision REGRESSION-GUARD test (rewrite its assertion, don't delete it: it should now assert "3 of the 4 port-102 entries are unsupported; S7comm is supported").
- `src/protocols.rs` doc-comments — supported/unsupported counts (8→9, 22→21) and the "promoted-in-place" prose pattern, reused for S7comm.
- `tests/integration_tests.rs` — every `PORT_102_NOTE`/`collision_note`/`TCP/102` assertion (≈15 call sites found via grep), especially the ones asserting `--supported` output never mentions "TCP/102" (line 302–326) and the ones asserting the JSON `collision_note` names all four protocols (line 1495+) — these need updated expected text, not just re-running.
- Any demo-evidence file referencing the "9 protocols" / catalog totals (`docs/demo-evidence/STORY-173/AC-004-supported-ports-9-protocols-8.md` and siblings) — these are STORY-173 evidence for the *previous* count (9 catalog-detectable ports / 8 supported protocols) and will need a successor evidence artifact for S7comm's story, not an edit to the frozen STORY-173 evidence.
- `SUPPORTED_PORTS` constant and its doc-comment (currently enumerates each port's dissection path 1:1 — needs a caveat sentence for port 102's multi-entry nature if approach (a)/(c) above is chosen, since a bare `102` in the list would no longer read as "the one protocol on this port").

**Severity: HIGH.** This is the only regression risk in this feature that touches a *documented architectural invariant with an existing named regression-guard test*, as opposed to ordinary "add a rule, update the doc comment" mechanical drift.

### 4.2 MEDIUM — dispatcher rule-order and VP-004 oracle

Standard risk, same shape as every prior binary-ICS cycle (ADR-005/007/010/013 Decision 9 pattern): adding `DispatchTarget::S7comm` and Rule 9 requires the atomic `classify_oracle` update in `dispatcher.rs`'s `#[cfg(kani)] mod kani_proofs` block. Precedent shows this is mechanical but load-bearing — VP-004 (`verify_content_first_precedence_exhaustive`) fails hard if the oracle and production `classify()` diverge. No new risk beyond the established pattern; flagged MEDIUM only because it is a new rule, not because it is novel.

### 4.3 MEDIUM — MITRE catalog drift guard (VP-007)

Adding new `technique_info()` arms (whichever of T0843/T0889/T0821 survive §7 validation) requires the six-part atomic obligation (ADR-010 Decision 7 / ADR-013 Decision 10 pattern): `SEEDED_TECHNIQUE_IDS` + `SEEDED_TECHNIQUE_ID_COUNT` (currently 29) bump, `EMITTED_IDS` extension, `vp007_catalog_drift_guard` re-verification. Mechanical, well-precedented, but any single missed step fails CI (`cargo test mitre`) — flagged MEDIUM for visibility, not because it's architecturally uncertain.

### 4.4 LOW — reassembly / reporter / decode-loop

Per §2.2, these are DEPENDENT-only. No prior binary-ICS cycle has needed to modify `reassembly/*`, `reporter/*.rs`, `decoder.rs`, `findings.rs`, or `summary.rs`. Risk of silent breakage here is LOW and covered entirely by the full existing regression suite (Rule 1 of the feature-mode scoping rules: regression is never scoped down, regardless of this assessment).

### 4.5 LOW — new module addition itself

`src/analyzer/iso_on_tcp.rs` and `src/analyzer/s7comm.rs` are NEW files with no existing behavior to regress. Standard TDD/Red-Gate discipline covers correctness; there is no legacy-breakage vector here by construction.

## 5. DTU Assessment

**`dtu_required: false`** for this feature, consistent with the project-wide determination.

wirerust is a passive, offline pcap/pcapng dissection tool. This feature adds parsing logic for bytes that arrive from a locally-supplied capture file (or live capture the tool listens to passively) — there is no live external S7 PLC, no snap7-style client connection, no third-party API call, and no service wirerust calls out to. All "external dependency" categories from the standard DTU assessment protocol are structurally inapplicable:

| Category | Determination |
|---|---|
| Inbound data sources | None — pcap/pcapng file or passive capture input only, same as every existing analyzer. |
| Outbound operations | None — wirerust never transmits; passive-only is a project-wide invariant. |
| Identity & access | None. |
| Persistence & state | None — no database, no external cache. |
| Observability & export | None. |
| Enrichment & lookup | None — MITRE technique lookup is the static, compile-time `technique_info()` catalog (no external ATT&CK API call). |

No Digital Twin Universe clone is needed. Test fixtures are captured/synthetic pcap bytes (see §7 for fixture-availability open question), not a live-service clone.

## 6. Story Estimate + Wave Placement

### Comparison baseline: IEC-104 (feature-iec104, E-22)

- F3 initial decomposition: **8 stories** (STORY-167..174), strictly linear/serial due to single-file contention on `src/analyzer/iec104.rs` — pure-core header parse → frame-format discrimination/session state machine → ASDU extraction → control-command detection → sequence tracking → carry-buffer/frame-walk → dispatcher integration → formal hardening.
- Post-wave-gate addition: STORY-180 (timed control-command extension), for **9 stories total**, 33 BCs, waves 76–85 (10 waves, one story per wave due to strict serialization).

### S7comm estimate

The extra TPKT/COTP framing layer as a **separate file** (`iso_on_tcp.rs`, distinct from `s7comm.rs`) is the key difference from IEC-104: it removes some of IEC-104's single-file serialization constraint (TPKT/COTP parse work and S7comm PDU-parse work can, in principle, be split into stories on two different files with a clean interface between them, similar to how ENIP's two-level ENIP/CPF/CIP framing still lived in one file but ADR-010 still decomposed it into ~12 stories/26 BCs because of its extra complexity tier).

**Rough estimate: 10–13 stories, ~50–70 points, 35–45 BCs (per §3), spanning ~10–13 waves** if serialized the way IEC-104 and DNP3 were (single-file-contention pattern), or potentially fewer waves if `iso_on_tcp.rs` and `s7comm.rs` can be developed with limited parallelism once the interface (parsed `CotpHeader`) is frozen. Suggested decomposition shape (F3 will finalize):

1. Pure-core TPKT header parse + Kani skeleton (mirrors STORY-167).
2. COTP TPDU-type discrimination (CR/CC/DT) + protocol-ID byte extraction (mirrors STORY-168's frame-format discrimination role).
3. S7comm PDU header parse (ROSCTR, PDU reference, param/data length) — depends on (2).
4. S7comm function-code classification + per-technique detection, split across 2–3 stories if the function-code space is large (compare DNP3's function-code classification breadth).
5. S7comm-plus framing-only classification path (protocol-ID `0x72`) — likely small, could combine with (2) or (3).
6. Carry-buffer / frame-walk loop + flow lifecycle (mirrors STORY-172).
7. Dispatcher integration — `DispatchTarget::S7comm`, VP-004 atomic, `--s7comm` flag, catalog promotion + the port-102 collision-model fix from §4.1 (this may itself deserve its own story given its regression footprint), `SUPPORTED_PORTS` (mirrors STORY-173, but larger given §4.1).
8. Formal hardening — Kani/proptest/fuzz/mutation (mirrors STORY-174).

**Epic:** New epic **E-23** (not E-22, which is IEC-104's frozen epic; a *new protocol epic* per the task's framing, following the E-21 protocol-coverage-catalog / E-22 IEC-104 numbering sequence).

**Wave placement:** last completed wave in `.factory/STATE.md` is **wave-086**. This feature would begin at **wave-087** and run serially for the estimated story count, i.e., roughly wave-087 through wave-097/100 depending on final F3 decomposition and how much of the TPKT/COTP-vs-S7comm split parallelizes.

## 7. Open Questions / Risks for the Human

1. **License-matrix check for S7comm prior art (BLOCKING for F2, mirrors ADR-013 Decision 7).** Wireshark `packet-s7comm.c` is GPLv2 → BANNED under existing project policy. The CISA/INL **ICSNPP-S7Comm** Zeek analyzer's license, `libnodave`/`libs7comm`'s license, and any Rust `s7` crates on crates.io need explicit verification before F2 architecture work begins (same blocking gate ADR-013 Decision 7 enforced for IEC-104's `iec60870-5`/lib60870/Wireshark). Do NOT assume MIT/Apache — verify.
2. **Which MITRE techniques are defensibly passively detectable from S7comm PDUs alone?** The research brief lists T0843 (Program Download), T0889 (Modify Program), T0835 (Manipulate I/O), T0836 (Modify Parameter), T0821 (Modify Controller Tasking), plus mentions T0888/T0846 (discovery), T0851 (Rootkit — almost certainly NOT passively detectable from network traffic alone), and T0873.001 (Siemens Project File Infection — likely NOT network-visible, it's a file-infection technique). **T0835 and T0836 are already SEEDED+EMITTED** (shared catalog entries — only need new S7comm emission call-sites, no catalog change). **T0843, T0889, T0821 are NOT currently in `src/mitre.rs`** — each needs (a) a defensibility argument for passive-only detection (what specific S7comm function code / parameter pattern maps to it, with what confidence), and (b) MITRE ATT&CK-for-ICS version-pin verification against whatever ics-attack version is current at F2 time (the codebase's most recent pin, per ADR-013, is `ics-attack-19.1` — confirm this is still current and that none of the candidate IDs have been revoked/renamed, following the T0855/T0856/T0857/T0809 revocation lessons baked into ADR-007/010/013). Recommend an F2 research-agent pass mirroring `.factory/research/enip-mitre-ics-tagging.md`.
3. **Port-102 catalog-model fix (§4.1) — which resolution approach?** The three candidate fixes ((a) explicit `supported: bool` field, (b) name-keyed exclusion list, (c) `dispatch_target` field) have different blast radii on `protocols.rs`'s existing pure-intersection design and its VP-041 proptest oracle-cross-check harness. This should be decided explicitly at the F1/F2 boundary, not left implicit in a story.
4. **Test-fixture availability.** Public S7comm pcap captures exist (e.g., ICS-focused pcap repositories, Wireshark sample captures, academic ICS-security datasets) but must be independently sourced and license-checked for redistribution as committed test fixtures — mirrors the `tests/fixtures/iec104-iti-diverse.pcap` precedent. Flag for F3/F4: identify concrete candidate pcap sources before story-level AC authoring locks in specific byte-level expectations.
5. **S7comm-plus protocol-ID disambiguation depth.** Is a bare "COTP protocol-ID `0x72` observed" sufficient for a framing-level classification finding, or should the analyzer also attempt to read the (unencrypted) S7comm-plus session-setup handshake metadata the research brief mentions ("session-setup observation")? This affects whether S7comm-plus gets 1 BC or several.
6. **Rate/burst detection thresholds.** Every prior binary-ICS analyzer (Modbus, DNP3, ENIP) ships at least one CLI-configurable burst/rate threshold (e.g., `--enip-write-burst-threshold`). Does S7comm need an analogous configurable threshold for, e.g., repeated Stop-CPU or Program-Download commands? Not addressed by the research brief; F2 should decide.

## 8. F1 Exit Recommendation

**Recommendation: PROCEED TO F2**, conditional on the human confirming the three decisions below. The scope is well-bounded (classic S7comm dissection + reusable TPKT/COTP layer + explicit S7comm-plus/MMS/ICCP non-goals), the impact boundary is fully enumerated with a strong existing precedent (ADR-013/IEC-104) to follow, and DTU is definitively not required. The one genuine architectural risk (§4.1, the port-102 catalog model) is well-understood and has multiple viable resolutions — it does not block proceeding, but it should not be decided implicitly inside F2 story text; it deserves an explicit human call now.

### Top 3 decisions for the human to confirm at the F1 gate

1. **Port-102 catalog-model fix approach (§4.1, §7.3).** Which of (a) `supported: bool` field, (b) name-keyed exclusion list mirroring the ARP special case, or (c) `dispatch_target` field should `src/protocols.rs` adopt to correctly promote S7comm to `known-supported` while keeping S7comm-plus/IEC 61850 MMS/ICCP-TASE.2 correctly `known-unsupported` despite sharing canonical port 102? This changes a documented ADR-012 invariant and its regression-guard test.
2. **MITRE technique scope (§7.2).** Confirm which of T0843, T0889, T0821 (and whether T0873.001 or T0851 belong at all, given they read as file-infection/host-artifact techniques rather than network-observable ones) should be seeded and targeted for emission this cycle, versus seeded-but-staged (T1692.002-style precedent from ADR-013) versus excluded entirely.
3. **S7comm-plus depth (§1, §7.5).** Confirm the scope boundary at "COTP protocol-ID `0x72` framing-level classification only, no semantic decode" is correct, or whether limited session-setup metadata observation should be pulled in-scope now rather than deferred.
