---
document_type: maintenance-sweep-report
sweep: risk-assumption-monitoring
sweep_number: 11
pipeline: STEADY_STATE
product: wirerust
released_version: v0.7.1
date: 2026-06-17
producer: consistency-validator
read_only: true
---

# Maintenance Sweep 11 — Risk and Assumption Monitoring

**Date:** 2026-06-17
**Latest release:** v0.7.1 (released 2026-06-17); prior releases: v0.1.0..v0.4.0, v0.5.0, v0.6.0, v0.7.0, v0.7.1 (8 releases total).
**Scope:** L2 Domain Spec shards, PRD, ADRs, NFR catalog, domain-debt register, STATE.md drift items.

---

## Executive Summary

The wirerust factory **has no formal ASM-NNN / R-NNN registry**. There is no file
named `risk-register.md`, `assumptions.md`, or equivalent; no artifact uses the
`ASM-NNN` or `R-NNN` ID scheme required by the VSDD consistency criteria 42-50.
All risk and assumption content is distributed informally across:

- Domain debt items (O-01..O-08) in `specs/domain/domain-debt.md`
- NFR violation items (NFR-VIO-001..010) in `specs/prd-supplements/nfr-catalog.md`
- ADR Consequences sections (inline caveats per ADR-005..ADR-008)
- STATE.md drift items (PG-*, DRIFT-*, O-* references)

This sweep catalogues every identifiable informal assumption and risk, assesses
their current validity against v0.7.1 product state, and recommends whether a
formal register should be backfilled.

**Summary counts:**

| Category | Total found | Still open / unvalidated | Escalation-worthy |
|----------|-------------|--------------------------|-------------------|
| Informal assumptions (treated as ASM) | 9 | 7 | 2 |
| Informal risks (treated as R-NNN) | 8 | 5 | 3 |
| Formal ASM-NNN identifiers | 0 | — | — |
| Formal R-NNN identifiers | 0 | — | — |

**Gate result:** ADVISORY. No formal registry exists; no gate can pass or fail.
Backfill is recommended before the next ICS protocol feature cycle.

---

## Section 1 — Formal ASM/R Registry Absence

### Finding RAM-001 (MEDIUM) — No formal ASM-NNN / R-NNN registry exists

**Artifact:** None — the registry file does not exist at any path under `.factory/specs/`.

**Evidence:**
- `find .factory/specs/ -name "*risk*" -o -name "*assumption*"` returns zero results.
- `grep -rn "ASM-[0-9]" .factory/specs/` returns zero results.
- `grep -rn "R-[0-9][0-9][0-9]" .factory/specs/` returns zero results.
- All holdout scenarios carry `assumption_source: null` and `risk_source: null`.
- VSDD criteria 42-50 (ASM/R traceability) cannot be satisfied without an explicit registry.

**Impact:** Consistency validator criteria 42-50 are structurally unverifiable.
Mitigation tracking is dispersed across O-NNN, NFR-VIO-NNN, and ADR Consequences sections,
with no unified view or status tracking. This is manageable at the current scale (8 minor
releases, 3 ICS analyzers), but will become a liability as the analyzer portfolio grows.

**Recommended action:** Before launching the next ICS protocol feature (S7comm, PROFINET,
or roadmap item #3), backfill a `specs/risk-register.md` file using the O-NNN and NFR-VIO-NNN
items below as seed entries, converting each to a formal R-NNN ID. Similarly create a
`specs/assumptions.md` with ASM-NNN IDs for the port-classification and
threshold-calibration assumptions. This enables VSDD criteria 42-50 to run.

---

## Section 2 — Informal Assumptions (ASM Candidates)

The following are architectural and design assumptions embedded in ADRs, domain-debt items,
and the NFR catalog. They are analyzed for validity against v0.7.1.

### ASM-CAND-001 — Port-only classification is sufficient for Modbus TCP (ADR-005 Decision 1)

**Source:** `specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md`
Context section, Decision 1.
**Assumption:** Port 502 is the sole reliable classifier for Modbus TCP. A post-classification
three-point validity gate (Protocol ID == 0x0000, Length in 2..254, plausible FC) is sufficient
to keep false-positive rates acceptable.

**Status as of v0.7.1:** VALID. The three-point gate is shipped and Kani-verified (VP-022).
No regression. The assumption was introduced before v0.4.0 and has survived 4 subsequent
releases unchanged.

**Releases since assumption recorded:** v0.4.0 (Modbus shipped), v0.5.0, v0.6.0, v0.7.0, v0.7.1 (5 releases).
**Assessment:** Stable. Recommend promoting to formal ASM-001 in a backfill register.
No escalation required.

---

### ASM-CAND-002 — Port-only classification is sufficient for DNP3 TCP (ADR-007 Decision 1)

**Source:** `specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md`
Decision 1.
**Assumption:** Port 20000 + a three-point validity gate (sync word 0x0564, length-consistent
frame, plausible FC) provides adequate discrimination. Any non-DNP3 binary on port 20000
is mis-routed but rejected at the gate.

**Status as of v0.7.1:** VALID. Sync-word gate (0x0564) is stronger than the Modbus gate
(bytes 2-3 only). VP-023 Kani proofs shipped (v0.6.0). No new analyzers since v0.6.0 conflict
with this routing.

**ADR-008 impact:** ARP analyzer (v0.7.0) uses the `DecodedFrame` enum path, completely
separate from the TCP reassembler and dispatcher. It does not affect port-dispatch logic
and cannot invalidate this assumption.

**Releases since assumption recorded:** v0.6.0, v0.7.0, v0.7.1 (3 releases).
**Assessment:** Stable. Recommend formal ASM-002.

---

### ASM-CAND-003 — Anomaly thresholds are adequate for forensic use without labelled corpus (O-03)

**Source:** `specs/domain/domain-debt.md` O-03; `specs/prd.md` Section 1.5 Out-of-Scope.
**Assumption:** Research-documented thresholds (overlap=50, small-segment=100,
small-segment-max-bytes=16, out-of-window=100) are forensically reasonable even without
empirical FP/TP calibration against labelled traffic.

**Status as of v0.7.1:** OPEN / UNVALIDATED. O-03 remains open. No labelled capture corpus
exists. The thresholds are CLI-overridable, which partially mitigates the risk.

**Releases since assumption recorded:** First recorded at greenfield spec (pre-v0.1.0).
Still unvalidated across all 8 releases.

**Assessment:** ESCALATE. This assumption is over 2 releases old (criterion for flagging
per sweep mandate). The downstream risk is false positives in ICS environments where
TCP behavior differs from standard networks. As the analyzer portfolio grows (ARP storm
threshold 50/s added in v0.7.0), more thresholds share this unvalidated-calibration status.
**Recommended action:** Create a formal ASM-003 entry recording the validation method
(labelled traffic corpus + FP/TP measurement) and target a post-v0.8.0 validation exercise.
If validation is deferred beyond the next 2 releases, consider documenting it as a
known limitation in README Section 4 (Limitations).

---

### ASM-CAND-004 — Full pcap eager-load into Vec<RawPacket> is acceptable for the target user (NFR-VIO-001)

**Source:** `specs/prd-supplements/nfr-catalog.md` NFR-PERF-002, NFR-VIO-001.
**Assumption:** Forensic analysts have sufficient RAM to hold the entire pcap in memory
(~1.5x file size). The README "multi-GB captures" claim is valid under this constraint.

**Status as of v0.7.1:** OPEN-DEBT. NFR-VIO-001 explicitly records this. Streaming
refactor is explicitly deferred. No runtime behavior change since v0.1.0.

**Assessment:** STABLE KNOWN DEBT. The assumption is correctly documented and accepted.
No ICS analyzers introduced in v0.5.0–v0.7.1 changed memory behavior (all operate on
already-loaded per-flow payload). Recommend formal ASM-004.

---

### ASM-CAND-005 — TLS byte-0/byte-1 gate (0x16 0x03) is adequate without checking minor version or length (Smell #10)

**Source:** `specs/domain/capabilities/cap-05-content-first-dispatch.md` section
"Loose TLS gate (Smell #10)".
**Assumption:** The 2-byte TLS record-type + major-version check produces negligible
false positive routing in practice.

**Status as of v0.7.1:** UNCHANGED. Smell #10 remains advisory. Zero tests exercise
the misroute path. No new analyzers introduced in v0.5.0–v0.7.1 interact with the TLS gate.
ARP frames carry EtherType 0x0806 and never reach the dispatcher's content classification.

**Assessment:** STABLE LOW. No escalation warranted. Recommend recording as formal ASM-005
with a note that the risk surface is zero in ARP/DNP3/Modbus contexts (all bypass the
content-first gate entirely).

---

### ASM-CAND-006 — ARP binding table LRU eviction at capacity 65,536 is acceptable (ADR-008 Decision 4 / Decision 5)

**Source:** `specs/architecture/decisions/ADR-008-arp-link-layer-integration.md`
Consequences section: "LRU eviction is not cryptographically safe."
**Assumption:** An offline PCAP forensics tool does not require cryptographically safe
binding table eviction; an attacker with ≥65,536 distinct source IPs in a PCAP can evict
legitimate bindings, causing missed detections, which is acceptable.

**Status as of v0.7.1:** VALID. Shipped as designed (v0.7.0, Kani-verified VP-024 Sub-D).
The E-17 cycle (v0.7.1) was test-only; no behavior change. The assumption is explicitly
documented in ADR-008 and accepted by architect.

**Assessment:** STABLE. No escalation. Recommend formal ASM-006. Note that the QinQ/MACsec
offset hardening (E-17) confirmed no regression in the binding table logic.

---

### ASM-CAND-007 — DNP3 FIR=1-only parse is sufficient for v1 detections (ADR-007 Decision 4)

**Source:** `specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md`
Decision 4 and Rationale.
**Assumption:** All detection-relevant application function codes (unauthorized commands,
restarts, writes) appear in the first application fragment (FIR=1). Multi-frame reassembly
adds state complexity with no v1 detection benefit.

**Status as of v0.7.1:** VALID for current detection scope. No new DNP3 detections were
added in v0.7.0 or v0.7.1 (both focused on ARP). The assumption was confirmed at F2 (v0.6.0)
and has not been invalidated.

**Assessment:** MONITOR. If a future feature adds DNP3 detections that require application-layer
continuity across fragments (e.g., large WRITE_SINGLE or multi-frame OPERATE), this assumption
must be revisited. Recommend formal ASM-007 with a note that reassembly is the next
capability gate.

---

### ASM-CAND-008 — DNP3 CRC skip is safe for PCAP replay of real captures (ADR-007 Decision 3)

**Source:** `specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md`
Decision 3 and Rationale. Also corroborated by `.factory/research/dnp3-f2-scope-threshold-validation.md`.
**Assumption:** Corrupt-CRC packets are rare in captures of real traffic; CRC skip does not
create false detections.

**Research note:** The validation research explicitly documents a nuance: "do NOT let
'we skip CRC' create the assumption that malformed payloads are rare — Crain/Sistrunk
frames carry correct CRCs, so CRC validation would not have caught them anyway."
The caveat is recorded in the research file but NOT propagated to ADR-007 Decision 3 text.

**Status as of v0.7.1:** VALID for shipped v1 scope. No regression from v0.7.0/v0.7.1.
However, the research caveat is a latent documentation gap.

**Assessment:** LOW. Recommend:
1. Formal ASM-008.
2. Add the Crain/Sistrunk caveat as a normative note to ADR-007 Decision 3 in the next
   spec-maintenance burst (this is a documentation-only fix, no code change).

---

### ASM-CAND-009 — ARP storm rate default (50 frames/s) is a conservative engineering choice (ADR-008 Decision 5 / v0.7.0)

**Source:** `specs/architecture/decisions/ADR-008-arp-link-layer-integration.md`
Open Items section: "Storm rate default (50/s): a conservative engineering choice."
**Assumption:** 50 frames/s is an acceptable default threshold for ARP storm detection
across typical OT/IT network environments; operators can lower it via `--arp-storm-rate`.

**Status as of v0.7.1:** UNVALIDATED. No labelled ARP storm corpus exists. This is a direct
parallel to ASM-CAND-003 (reassembly thresholds). The CLI override exists but the default
is an engineering judgment, not empirically calibrated.

**Releases since introduction:** v0.7.0, v0.7.1 (2 releases). Not yet past the 2-release
escalation threshold, but approaching it.

**Assessment:** MONITOR. Will reach escalation threshold after 1 more release. Recommend
formal ASM-009 with validation method: test against captured ICS/OT traffic exhibiting
normal high-frequency legitimate ARP (e.g., PROFINET IO frames). If no corpus is available,
document and accept explicitly at the next review.

---

## Section 3 — Informal Risks (R-NNN Candidates)

### R-CAND-001 — Unbounded weak-cipher evidence Vec (O-06 / NFR-RES-023)

**Source:** `specs/prd-supplements/nfr-catalog.md` NFR-RES-023; domain-debt O-06;
GitHub issue #102.
**Risk:** `TlsAnalyzer` ClientHello weak-cipher Finding evidence Vec has no truncation cap.
Upper bound ~9,216 entries (~270-500 KB worst-case). No per-cipher cap shipped.

**Current mitigations:** MAX_RECORD_PAYLOAD provides an upstream bound; the condition
cannot be triggered without a matching malformed/adversarial ClientHello.

**ADR-008 / etherparse 0.20 impact:** None. TLS analysis path is unchanged. etherparse
0.20 migration (STORY-111) affected only decoder.rs ARP path; tls.rs is unmodified.

**Impact:** MEDIUM. A crafted PCAP could trigger near-300 KB Finding heap allocation.
Offline forensics tool — no real-time DoS risk — but memory usage could surprise analysts
processing adversarial pcaps.

**Recommended action:** Add formal R-001. Track via issue #102 (already filed). Ship
`MAX_WEAK_CIPHER_EVIDENCE = 64` truncation cap with "+N more" entry in the next
TLS-touching story or maintenance pass. Severity: MEDIUM (P1).

---

### R-CAND-002 — README "multi-GB captures" claim vs eager Vec<RawPacket> load (NFR-VIO-001)

**Source:** `specs/prd-supplements/nfr-catalog.md` NFR-VIO-001; NFR-PERF-002.
**Risk:** Users following the README claim may attempt to analyze captures larger than
available RAM, producing OOM failures without clear error messaging.

**Current mitigations:** NFR-VIO-001 is explicitly documented as OPEN-DEBT.
No streaming refactor is planned.

**ARP/DNP3/Modbus impact:** All three new analyzers operate on per-flow reassembled payloads
and do not introduce any new memory pressure at the reader layer. The root risk is unchanged.

**Impact:** LOW-MEDIUM. The tool is documented as offline and single-pass, so sophisticated
users understand the constraint. However, the README language creates a false expectation.

**Recommended action:** Formal R-002. Fix README to say "up to available RAM, typically
1.5x pcap file size" with a concrete example. Estimated effort: trivial. Should be bundled
into issue #254 (repo-wide doc cleanup).

---

### R-CAND-003 — Single-platform CI (ubuntu-latest only) — macOS/Windows regressions silent (NFR-VIO-010 / NFR-PORT-001)

**Source:** `specs/prd-supplements/nfr-catalog.md` NFR-PORT-001, NFR-VIO-010.
**Risk:** wirerust releases binaries for 4 platforms (aarch64-apple-darwin, x86_64-apple-darwin,
x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu) but CI only tests ubuntu-latest.
Platform-specific regressions (path separators, u32 overflow on 32-bit, endianness on
cross-compiled targets) can silently land.

**New analyzer impact:** ARP analyzer (v0.7.0) uses raw byte slice access and arithmetic
with no platform-specific assumptions. DNP3 and Modbus are similarly endianness-explicit
(from_be_bytes). However, the etherparse 0.20 migration's non-exhaustive match patterns
could in principle behave differently on future etherparse API changes visible only in
non-Linux builds — no evidence this is occurring, but the risk surface is larger with
4 released platforms.

**Impact:** MEDIUM. The release CI (`release.yml`) does cross-compile and produce 4 binaries,
so build-time failures are caught. Runtime behavior regression on non-Linux platforms is
the unmitigated risk.

**Recommended action:** Formal R-003. Medium-priority. Add an explicit "tested on
ubuntu-latest only" caveat to CONTRIBUTING.md and README. As a medium-term improvement,
add macOS arm64 test job to CI matrix (cost: ~2 min additional CI time). Windows test
expansion is lower priority (cross-compiled, not natively tested by most contributors).

---

### R-CAND-004 — rayon unused dependency in production build graph (O-07 / NFR-SUP-001)

**Source:** `specs/prd-supplements/nfr-catalog.md` NFR-SUP-001, NFR-VIO-006;
domain-debt O-07.
**Risk:** `rayon = "1"` in `[dependencies]` adds rayon's full transitive closure to the
production build graph without providing any functionality. Expands supply-chain attack
surface unnecessarily.

**ADR impact:** None of ADR-005/006/007/008 depend on rayon. The ARP analyzer is
single-threaded by design (per-frame processing). The risk is entirely contained to
the unused dependency.

**Impact:** LOW. rayon is a well-maintained crate; the practical supply-chain risk increment
is small. However, the principle of minimal production dependencies is a stated NFR.

**Recommended action:** Formal R-004. Ship `rayon` removal in the next Cargo.toml touch
(trivial S-sized fix). This was previously noted in drift item O-07 and NFR-VIO-006 but
has not been actioned across 8 releases. Escalate priority to P2 (from current untracked).

---

### R-CAND-005 — Port-502 false-routing risk for non-Modbus binary protocols (ADR-005 acknowledged trade-off)

**Source:** `specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md`
Alternatives Considered section; Context section line 78.
**Risk:** Any non-Modbus binary protocol on port 502 (custom binary framing, SSH to
non-standard port, TLS extensions) is mis-routed to ModbusAnalyzer and incurs the
three-point validity gate overhead. If the gate check is wrong (e.g., bytes 2-3 happen
to be 0x0000), a false Modbus finding could be emitted.

**New analyzer context:** ARP (v0.7.0) uses DecodedFrame path, completely orthogonal.
DNP3 (v0.6.0) uses port 20000 with a stronger sync-word gate. Neither introduces new
Port-502 risk.

**Impact:** LOW. The three-point gate is Kani-verified (VP-022). Protocol ID 0x0000 +
plausible FC is a reasonably strong discriminator. However, ADR-005 acknowledges the
false-positive vector exists (bytes 2-3 can be 0x0000 in arbitrary binary protocols).

**Assessment:** ACCEPTED KNOWN RISK. Recommend formal R-005 with status ACCEPTED (mitigated
by VP-022). No action beyond documentation.

---

### R-CAND-006 — DNP3 T0827 emission threshold misconfiguration risk (ADR-007 known trade-off)

**Source:** `specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md`
Consequences: "T0827 'Loss of Control' is an Impact-tactic correlated finding... misconfiguring
the window threshold produces either false positives (too low) or missed detections (too high)."
**Risk:** The `--dnp3-direct-operate-threshold` default (pinned in F3 per ADR-007 Decision 6
open items) may not be calibrated for all OT environments.

**Status:** This threshold is explicitly documented as a "JUDGMENT" recommendation (5-20 range
per dnp3-research.md §5.1). No empirical calibration has been performed.

**Impact:** MEDIUM. Same class as ASM-CAND-003 (reassembly threshold calibration). False
T0827 emissions in environments with legitimate high-frequency Direct Operate commands
(e.g., SCADA auto-control loops) would be a usability issue.

**Recommended action:** Formal R-006. Same validation path as ASM-CAND-003: labelled ICS
traffic corpus. Document the default choice rationale in README or CLI help text for
`--dnp3-direct-operate-threshold`. Escalate to P2.

---

### R-CAND-007 — RUSTSEC-2026-0097 transitive rand 0.8.5 unsound (accepted, non-runtime)

**Source:** `specs/prd-supplements/nfr-catalog.md` STATE.md drift item RUSTSEC-2026-0097;
maintenance/dependency-audit-raw.log.
**Risk:** `rand 0.8.5` pulled transitively via `tls-parser → phf_codegen` is flagged as
unsound (RUSTSEC-2026-0097). It is a BUILD-only dependency (not linked into the runtime
binary). The rand 0.8.6 upstream fix exists.

**Status:** ACCEPTED-TRANSITIVE per current policy. No runtime exploitability confirmed.
Dependency-audit raw log confirms: "warnings (unsound): 1 (RUSTSEC-2026-0097 — rand 0.8.5,
transitive via tls-parser); notable: rand 0.8.5 -> 0.8.6 (would resolve RUSTSEC-2026-0097)."

**Impact:** LOW. Build-only dep; binary does not link rand. However, with each new release
the advisory remains visible in `cargo audit` output, which can create noise for security
auditors.

**Recommended action:** Formal R-007. Track as ACCEPTED-TRANSITIVE pending tls-parser
update (upstream fix at rand 0.8.6). Check if tls-parser has released a version that bumps
its rand dep; if so, schedule a Dependabot sweep. Otherwise, document explicitly in
`deny.toml` or equivalent when cargo-deny is added.

---

### R-CAND-008 — VLAN/QinQ/MACsec ARP offset detection limitation (E-17 documented limitation, v0.7.1)

**Source:** `STORY-117.md` (E-17 documented-limitation story); STATE.md E-17 F4/F5/F6
status entries.
**Risk:** For MACsec-encapsulated ARP frames, wirerust cannot inspect the encrypted inner
payload to extract ARP fields. The tool correctly reports the limitation (STORY-117 scope)
but a forensic analyst may not realize that ARP detections (D1-D11) do not fire on MACsec
ARP traffic.

**ARP/etherparse 0.20 context:** The E-17 cycle specifically examined this limitation.
F5 scoped adversarial confirmed the CWE-693 ciphertext-opacity property is correct-polarity
and non-vacuous (etherparse lax_macsec_slice.rs — Layer::Arp is structurally unreachable
for Modified payloads). VP-024 Kani proofs verified no regression.

**Impact:** LOW. This is a documented hardware-layer constraint, not a bug. The tool
correctly processes unencrypted ARP frames (inner Ethernet ARP after QinQ tags is handled
via the link_exts offset calculation from D-F1 fix, PR #249).

**Assessment:** ACCEPTED KNOWN LIMITATION. Recommend formal R-008 with status ACCEPTED
(documented by design in STORY-117, tested in E-17). No action beyond ensuring README
documents the MACsec ARP limitation in the Known Limitations section.

---

## Section 4 — Mitigation Validity Check Against Current Architecture

### Check: ADR-008 (ARP / etherparse 0.20) impact on existing mitigations

| Pre-existing risk / mitigation | Impacted by ADR-008? | Verdict |
|-------------------------------|----------------------|---------|
| R-CAND-001 (weak-cipher heap) | No — tls.rs unchanged | NOT INVALIDATED |
| R-CAND-002 (eager Vec load) | No — reader.rs unchanged | NOT INVALIDATED |
| R-CAND-005 (Port-502 Modbus routing) | No — dispatch.rs port-502 arm unchanged | NOT INVALIDATED |
| VP-022 Modbus Kani proof | No — ModbusAnalyzer unchanged | NOT INVALIDATED |
| VP-023 DNP3 Kani proof | No — Dnp3Analyzer unchanged | NOT INVALIDATED |
| TLS loose gate (Smell #10) | No — ARP frames exit at decode_packet, never reach dispatcher | NOT INVALIDATED |
| MAX_FINDINGS cap (10,000) | No — ARP findings flow through same cap | NOT INVALIDATED |
| RUSTSEC-2026-0097 (rand transitive) | No change | NOT INVALIDATED |

### Check: etherparse 0.20 migration impact

etherparse 0.20 adds `NetSlice::Arp` / `LaxNetSlice::Arp` variants. The migration
(STORY-111, sub-delta A) updated `decoder.rs` to handle these as `DecodedFrame::Arp`.
The non-exhaustive match guard at `src/decoder.rs:210,232` (DRIFT-ETHERPARSE-0.20-MIGRATION-001)
is **RESOLVED** — it was folded into the ARP feature cycle (D-066) and shipped in v0.7.0.

Any future etherparse version bump would require re-checking `decoder.rs` match arms,
but this is covered by the standard dependency-audit / Dependabot path.

**Verdict:** etherparse 0.20 migration does NOT invalidate any prior mitigation.

### Check: DNP3 analyzer (v0.6.0) impact on Modbus mitigations

DNP3 uses `DispatchTarget::Dnp3` (port 20000), entirely distinct from
`DispatchTarget::Modbus` (port 502). VP-004 `classify_oracle` Kani harness was updated
atomically with DNP3 addition. No cross-contamination possible.

**Verdict:** DNP3 analyzer does NOT invalidate any Modbus mitigation.

---

## Section 5 — Invalidated Assumption / Missing Risk Escalation Check

This section checks criterion 49: every invalidated assumption must have a corresponding
risk escalation.

One assumption was explicitly invalidated during the Modbus feature cycle:

**Invalidated:** The f2-spec microsecond-scale window assumption in BC-2.14.016 and
BC-2.14.017 (T0831/T0806 burst detection). The specs originally assumed `on_data` delivered
timestamps in microseconds (`T0831_WINDOW_SECS * 1_000_000`). This was wrong; `on_data`
delivers seconds.

**Escalation:** This was corrected in-flight as an F5 spec defect fix (BC-2.14.016 v2.2,
BC-2.14.017 v2.2). The fix was shipped in v0.6.0. No separate R-NNN risk entry was created
because the invalidation was discovered and remediated within the same feature cycle, before
release.

**Assessment:** The remediation is complete and correct. No open risk results from this
invalidation. However, if a formal ASM registry had existed, the invalidation + correction
would have been properly recorded as ASM-CAND-010 (status: invalidated) with a reference
to the F5 fix. This is another argument for backfilling the register.

---

## Section 6 — Prioritized Recommendations

| Priority | ID | Action | Effort |
|----------|----|--------|--------|
| HIGH | REC-001 | Backfill formal `specs/risk-register.md` with R-001..R-008 derived from R-CAND items above. Add `specs/assumptions.md` with ASM-001..ASM-009 derived from ASM-CAND items above. | M (1 session) |
| MEDIUM | REC-002 | ASM-CAND-003 (anomaly threshold calibration) and ASM-CAND-009 (ARP storm rate) are unvalidated assumptions entering their 3rd+ release cycle. Document a validation plan (labelled ICS traffic) before the next ICS feature or accept formally with a written rationale. | S |
| MEDIUM | REC-003 | R-CAND-001 (weak-cipher heap, GitHub #102) and R-CAND-004 (rayon unused dep) are actionable with small effort. Ship both in the next maintenance PR (bundle with issue #254 doc cleanup). | S |
| LOW | REC-004 | Add Crain/Sistrunk CRC-caveat (ASM-CAND-008) as a normative note to ADR-007 Decision 3. Documentation-only, no code change. | XS |
| LOW | REC-005 | R-CAND-007 (RUSTSEC-2026-0097): check if tls-parser has updated its rand dep to 0.8.6; if so, schedule a Dependabot bump to close this advisory. | XS |
| LOW | REC-006 | R-CAND-008 (MACsec ARP limitation): add one sentence to README Known Limitations section referencing STORY-117. Bundle with issue #254 doc cleanup. | XS |
| INFORMATIONAL | REC-007 | All 80 consistency criteria cannot be fully evaluated until a formal ASM-NNN/R-NNN registry exists. Criteria 42-50 are currently structurally unverifiable. Record this in STATE.md as a new DRIFT item. | XS |

---

## Section 7 — Conclusion

wirerust does not maintain a formal ASM-NNN / R-NNN risk register. The product is
well-maintained and the risks identified above are tracked informally across domain-debt,
NFR-VIO, and ADR Consequences sections — a pragmatic approach for the current scale.

No existing mitigation has been invalidated by ADR-008 (ARP), etherparse 0.20, or the
new ARP/DNP3/Modbus analyzer portfolio. The etherparse migration (DRIFT-ETHERPARSE-0.20-MIGRATION-001)
is fully resolved as of v0.7.0.

Two assumptions (anomaly threshold calibration, ARP storm rate default) and one risk
(weak-cipher heap bound) are the highest-priority open items. The single most impactful
structural improvement is backfilling a formal risk register (REC-001) before the next
ICS protocol feature cycle.
