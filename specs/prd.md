---
document_type: prd
level: L3
version: "1.36"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: 1a
origin: brownfield
inputs:
  - .factory/specs/domain/domain-spec.md
  - .factory/specs/domain/domain-debt.md
  - .factory/specs/domain/invariants/inv-01-core-invariants.md
  - .factory/specs/domain/capabilities/cap-01-pcap-ingestion.md
  - .factory/specs/domain/capabilities/cap-02-link-type-gating.md
  - .factory/specs/domain/capabilities/cap-03-packet-decoding.md
  - .factory/specs/domain/capabilities/cap-04-tcp-reassembly.md
  - .factory/specs/domain/capabilities/cap-05-content-first-dispatch.md
  - .factory/specs/domain/capabilities/cap-06-http-analysis.md
  - .factory/specs/domain/capabilities/cap-07-tls-analysis.md
  - .factory/specs/domain/capabilities/cap-08-dns-analysis.md
  - .factory/specs/domain/capabilities/cap-09-finding-emission.md
  - .factory/specs/domain/capabilities/cap-10-mitre-mapping.md
  - .factory/specs/domain/capabilities/cap-11-reporting-output.md
  - .factory/semport/wirerust/wirerust-pass-3-behavioral-contracts.md
  - .factory/semport/wirerust/wirerust-pass-3-deep-behavioral-contracts-r4.md
input-hash: "ff3462e"
traces_to: .factory/specs/domain/domain-spec.md
supplements:
  - prd-supplements/interface-definitions.md
  - prd-supplements/error-taxonomy.md
  - prd-supplements/test-vectors.md
  - prd-supplements/nfr-catalog.md
---

# Product Requirements Document: wirerust

> **Brownfield Mode:** This PRD is DESCRIPTIVE of the shipped system as of develop HEAD (post
> remediation-cycle PRs #69-#98, reconciled against 0082a0c). Every requirement is grounded in
> verified source evidence. Known gaps are recorded as debt (O-01..O-08), not silently omitted.
> Do NOT treat this document as aspirational -- it specifies what the system does today.

> **BC Index Model:** This PRD is an index document. Each Behavioral Contract (BC) lives in its
> own file under `behavioral-contracts/ss-NN/`. The tables below provide one-line summaries
> linking to individual BC files. Full contract details are NOT inlined here.
>
> **Version 1.1 delta:** Added Section 2.14 (SS-14 Modbus/ICS Analysis, 25 BCs, Feature #7,
> ADR-005). Updated Section 1.5 Out of Scope (T0855/T1692.001 and 5 other ICS techniques now emitted).
> Updated Section 6 KD-005 and KD-003 with Modbus-specific BC references. Added SS-14 rows to
> Section 7 RTM. Total BC count: 244 (was 219).
> **→ Current total after all deltas: 288 BCs.**
>
> **Version 1.2 delta (2026-06-09 — F2 Modbus revision):** Adopts three approved decisions from
> `f2-fix-directives.md` v2 (Decisions 11, 12, 13). **BREAKING CHANGE targeting v0.3.0:**
> Decision 13 changes `Finding.mitre_technique: Option<String>` to
> `Finding.mitre_techniques: Vec<String>` — JSON key renames to `"mitre_techniques"` (array),
> CSV column-6 header renames to `mitre_techniques` with semicolon-join encoding. Existing BCs
> revised: BC-2.09.001 v1.4, BC-2.09.006 v1.5, BC-2.10.005 v1.4, BC-2.10.007 v1.3,
> BC-2.10.008 v1.5, BC-2.11.013 v1.6, BC-2.11.015 v1.6, BC-2.11.017 v1.5, BC-2.11.020 v1.5,
> BC-2.11.024 v1.4. SS-14 revised BCs: BC-2.14.013/014/015/016/017/020/022/024 (all v2.0).
> ADR-006 registered. See `spec-changelog.md` §[1.2] for full entry.
>
> **Version 1.3 delta (2026-06-09 — F2 schema add-ons + release split):** Two research-backed
> schema add-ons (f2-multitag-schema.md) and release sequencing decision (f2-bundle-vs-split.md).
> ADD-ON 1: BC-2.11.001 v1.5 — JSON report envelope adds `mitre_domain: "ics-attack"` +
> `mitre_attack_version: "ics-attack-v15"` (F4 must pin). ADD-ON 2: BC-2.11.024 v1.5 — empty
> CSV cell clarification: EMPTY STRING not null; EC-015 added for consumer split guard.
> Release split: v0.3.0 = schema-only break (SS-09/10/11 + add-ons); v0.4.0 = Modbus additive
> (SS-14). See RELEASE SEQUENCING box in Section 2 and `spec-changelog.md` §[1.3].
>
> **Version 1.4 delta (2026-06-10 — MITRE ATT&CK for ICS v19 remap, issue #222):** 1:1 technique-ID
> remap driven by DF-VALIDATION-001-validated defect. T0855 "Unauthorized Command Message"
> (revoked v19) → T1692.001 "Unauthorized Message: Command Message" (ICS sub-technique, v19).
> T0856 "Spoof Reporting Message" (revoked v19) → T1692.002 "Unauthorized Message: Reporting
> Message" (ICS sub-technique, v19). Tactic unchanged: IcsImpairProcessControl for both.
> All T0855/T0856 references in live spec body updated. Audited via
> `mitre-ics-v19-catalog-audit.md` and `dnp3-mitre-verification.md`. Updated BCs: SS-14
> BC-2.14.006/007/008/011/013/014/015/016/017/018/019/020/022/024; SS-11
> BC-2.11.001/013/017/020/024; SS-10 BC-2.10.008; SS-09 BC-2.09.001/006.
> See `spec-changelog.md` §[v19-remap-2026-06-10].
>
> **Version 1.5 delta (2026-06-10 — Feature #8 DNP3/ICS analyzer, issue #8):** Added Section
> 2.15 (SS-15 DNP3/ICS Analysis, 22 BCs, ADR-007). Updated Section 2.10 O-04 domain debt
> note: SEEDED 21→23 (added T1691.001 + T0827), EMITTED 13→15. New ICS-unique MitreTactic
> variant `IcsImpact` (Display "Impact (ICS)", ICS TA0105) added; `all_tactics_in_report_order`
> grows 16→17 elements. Updated BCs: BC-2.10.002/003/004/005/007/008 (v1.3–v1.7 per BC).
> Added SS-15 rows to Section 7 RTM. KD-005 and KD-007 extended with DNP3 BCs.
> Total BC count: 266 (was 244). See `spec-changelog.md` §[dnp3-f2-2026-06-10].
>
> **Version 1.6 delta (2026-06-10 — Feature #8 DNP3 research must-adds, issue #8 post-gate):**
> Added 2 research-validated must-add detections from `dnp3-f2-scope-threshold-validation.md`:
> BC-2.15.023 (DISABLE_UNSOLICITED/ENABLE_UNSOLICITED abuse → T0814) and BC-2.15.024
> (malformed/structural DNP3 anomaly from parse_errors threshold → T0814, Crain-Sistrunk
> coverage). Both map to existing T0814 — MITRE catalog counts unchanged (23/15/8). Applied
> threshold clarifications: BC-2.15.010 v1.2 (10/60s is flood guard; unauthorized-source
> fires at count=1; ~5/60s option for quiet profiles); BC-2.15.014 v1.4 (DIRECT_OPERATE_NR
> exclusion research-confirmed); BC-2.15.015 v1.4 (≥3 must be distinct impact events, not
> double-counted). SS-15 now 24 BCs. Total BC count: 268 (was 266).
> See `spec-changelog.md` §[dnp3-f2-mustadds-2026-06-10].

> **Version 1.7 delta (2026-06-10 — Adversarial finding C-2 fix, issue #8 blocking):**
> Fixed BC-2.15.024 (v1.1): replaced the erroneous windowed use of `parse_errors` with a
> separate windowed counter `malformed_in_window`. `parse_errors` is now correctly specified
> as a LIFETIME/monotonic counter (NEVER reset at window expiry; consumed by BC-2.15.020
> summarize()). `malformed_in_window` is the new windowed counter used for all threshold
> checks; resets to 0 at 300s window expiry. Extended BC-2.15.015 (v1.5) to reset the two
> new BC-2.15.024 windowed fields at window expiry (malformed_in_window, malformed_anomaly_emitted);
> Invariant 6 updated from "four fields" to six. PRD prose updated from "BC-2.15.001..022"
> to "BC-2.15.001..024", "22 BCs" to "24 BCs", and RTM entry for BC-2.15.024 corrected to
> name `malformed_in_window`. No new BCs; no MITRE catalog change; counts 23/15/8 unchanged.
>
> **Version 1.8 delta (2026-06-10 — PRD version alignment bump, no new BCs):** Version bump
> to align with BC-INDEX v1.6 and SS-15 must-add additions tracked in v1.6 delta above.
> No new BCs; spec-changelog §[dnp3-f2-mustadds-c2fix-2026-06-10] is the authoritative
> record. → Current total after all deltas: 268 BCs.
>
> **Version 1.9 delta (2026-06-12 — Feature #9 ARP security analyzer, issue #9):** Added
> Section 2.16 (SS-16 ARP Security Analysis, 15 BCs, ADR-008). Revised BC-2.02.009 v1.4→v1.5
> (ADR-008 Decision 1: three-way ARP/non-Ethernet-ARP/non-IP postcondition; `decode_packet`
> return type changes from `Result<ParsedPacket>` to `Result<DecodedFrame>`). New decoder
> variant `DecodedFrame::Arp(ArpFrame)` introduced. New error code E-DEC-004 ("Non-Ethernet/
> IPv4 ARP frame") and ARP section (E-ARP-001..003) added to error-taxonomy supplement (v1.3).
> MITRE techniques added to catalog: T0830 (Adversary-in-the-Middle, LateralMovement),
> T1557.002 (ARP Cache Poisoning, CredentialAccess) — SEEDED count grows
> 23→25; EMITTED grows 15→17; CATALOGUE-ONLY remains 8. Added SS-16 rows to Section 7 RTM.
> Total BC count: 283 (was 268). See `spec-changelog.md` §[arp-f2-2026-06-12].
> **F3 implementation ambiguities flagged (record only — not spec defects; F3 story-writer
> must resolve as implementation choices):**
> - ARP-AMB-001: LRU substrate for binding table — HashMap-ordered LRU (indexmap-based) vs
>   BTreeMap-ordered LRU vs custom doubly-linked list; BC-2.16.006 specifies cap invariant
>   only, not substrate. F3 story must pick and pin in story body.
> - ARP-AMB-002: Malformed-frame integration mechanism — whether D11 finding is emitted inside
>   `decode_packet` (decoder layer), inside `process_arp` (analyzer layer), or via a separate
>   hook; BC-2.16.009 and BC-2.02.009 are silent on call site. F3 story STORY-111 must pick.
> - ARP-AMB-003: **RESOLVED in F2.** Storm-rate denominator is integer-seconds based
>   (`u32` timestamps). The sound formula is `rate = count_in_window / max(1, ts -
>   window_start_ts)`. When all frames arrive in the same second (`ts == window_start_ts`),
>   `max(1, 0) = 1`, so rate = count_in_window (no division-by-zero). EC-002 of BC-2.16.008
>   is consistent with this formula. There is no sub-second ambiguity because timestamps
>   are integer seconds. BC-2.16.008 updated accordingly. (Was incorrectly deferred as F3
>   ambiguity; the formula is fully determined by u32 integer-seconds semantics.)
> - ARP-AMB-004: **RESOLVED in F2.** Malformed ARP frames (extract_arp_frame → None,
>   E-DEC-004) do NOT count toward `frames_analyzed`. They are tracked by a separate
>   `malformed_frames` counter (distinct from `malformed_findings`). This makes
>   BC-2.16.010 Invariant 3 (`request_count + reply_count <= frames_analyzed`) trivially
>   consistent: only well-formed Ethernet/IPv4 ARP frames increment `frames_analyzed`.
>   BC-2.16.010 updated to add `malformed_frames` as a 10th summary key and to state the
>   exclusion explicitly. (Was incorrectly deferred as F3 ambiguity.)
> - ARP-AMB-005: Stale line-number anchors in BC-2.02.009 Architecture Anchors post-STORY-111
>   — the Architecture Anchors section of BC-2.02.009 cites decoder.rs line references that
>   will be invalidated by STORY-111's DecodedFrame enum addition. F3 story-writer must update
>   BC-2.02.009 Architecture Anchors after STORY-111 implementation.
> - ARP-AMB-006: Affected stories STORY-111..STORY-115 (estimated wave assignments TBD) must
>   be created by F3 story decomposition. BC-2.16.001..015 all have `Story Anchor: TBD`.

> **Version 1.10 delta (2026-06-12 — F2 adversarial Pass 1 remediation + architect propagation):**
> This version propagates architect decisions from arp-architecture-delta.md §6 and remediates
> F2 adversarial Pass 1 findings routed to product-owner (F-ARP-C2, F-ARP-C3, F-ARP-H5,
> F-ARP-H6, F-ARP-H7, F-ARP-H8, F-ARP-O1, F-ARP-O4, F-ARP-O5). Key changes:
> - **A.1** Binding-table substrate clarified: `HashMap<[u8;4], BindingEntry>` (production);
>   BTreeMap is Kani-surrogate only (VP-024 Sub-D). PRD §2.16 and BC-2.16.005/006 updated.
> - **A.2** BC-2.16.006 eviction claim downgraded: evicts entry with minimum `last_seen_ts`
>   (heuristic LRU approximation). VP-024 Sub-D proves only `len <= cap`, not a proven LRU order.
> - **A.3** MITRE tactic corrections: T0830 → `LateralMovement` (not IcsImpairProcessControl);
>   T1557.002 → `CredentialAccess`. All occurrences updated in PRD, HS-INDEX, spec-changelog.
> - **A.4** HS-INDEX waves 40-44 rewritten to match arch-delta §6 canonical story decomposition.
>   BC-2.16.016 (arch-delta mis-cite in STORY-115 row) reconciled: no such BC exists; maps to
>   BC-2.16.010 (storm_findings already a required summarize() key).
> - **A.5** BC-2.16.003 GARP preconditions confirmed opcode-agnostic (no `operation == 2`
>   restriction present — no change needed; confirmed clean).
> - **F-ARP-C2** PRD §2.16 reference "GARP-that-conflicts D14 paths" corrected to
>   "GARP-that-conflicts (BC-2.16.014) paths". There is no detection "D14".
> - **F-ARP-C3** VP-024 sub-property labels in PRD §2.16 corrected to match VP-024 exactly:
>   Sub-A=extraction; Sub-B=GARP biconditional; Sub-C=binding last-write-wins (proptest);
>   Sub-D=MAX_ARP_BINDINGS cap (scaled Kani).
> - **F-ARP-H5** BC-2.16.008 storm-rate formula corrected:
>   `rate = count_in_window / max(1, ts - window_start_ts)`. EC-002 and canonical test vectors
>   made arithmetically consistent. ARP-AMB-003 reclassified: RESOLVED in F2.
> - **F-ARP-H6** error-taxonomy.md updated: added E-ARP-004 (D1 spoof finding) and E-ARP-005
>   (D2 GARP finding). E-ARP-001 (D11) verdict triple aligned: Anomaly/LOW (per BC-2.16.009).
> - **F-ARP-H7** BC-2.16.010 updated: malformed frames explicitly excluded from
>   `frames_analyzed`; `malformed_frames` added as 10th summary key (separate from
>   `malformed_findings`). ARP-AMB-004 reclassified: RESOLVED in F2.
> - **F-ARP-H8** BC-2.16.004 severity logic clarified: a rebind emits exactly one D1 finding.
>   Severity = HIGH iff `rebind_count >= spoof_threshold && !spoof_high_emitted`, else MEDIUM.
>   BC-2.16.014 EC-004 aligned. Unconditional "first rebind = MEDIUM" language removed.
> - **F-ARP-O1** ARP-AMB-003 and ARP-AMB-004 reclassified RESOLVED in F2 (see above).
> - **F-ARP-O4** RTM verification-method for BC-2.16.004 and BC-2.16.005 updated to
>   unit+proptest. BC-2.16.005 is the primary VP-024 Sub-C anchor; BC-2.16.004 is
>   indirectly supported (primary-owned by STORY-114, verified by unit+proptest) but
>   is NOT in VP-024's formal Verified-BCs scope — see VP-INDEX footnote.
> - **F-ARP-O5** HS-INDEX P1 count corrected: 2 seeds are P1 — HS-W44-001 and HS-W44-003
>   (both in wave 44: D3 storm and --arp-storm-rate override). HS-W42-002 and HS-W43-003 are
>   P0, not P1; they were previously mislabeled. Total ARP seeds = 26 (24 P0 + 2 P1).
> Total BC count: 283 (unchanged). See `spec-changelog.md` §[arp-f2-pass1-remediation-2026-06-12].

> **Version 1.11 delta (2026-06-12 — F2 adversarial Pass 2 remediation + ADR-008 Decision 7 propagation):**
> Propagates canonical 11-key summarize() set from ADR-008 Decision 7 (adds `other_opcode_count`
> as key 4; reconciliation invariant `request_count + reply_count + other_opcode_count ==
> frames_analyzed` stated explicitly). Remediates all PO-routed Pass 2 findings. Key changes:
> - **F-B-001/F-B-006/F-D-M2** BC-2.16.010 updated nine→ten→eleven; `other_opcode_count` added
>   as key 4; reconciliation invariant stated; malformed_frames exclusion documented. v1.1→v1.2.
> - **F-B-003** BC-2.16.014 Postcondition 2 repaired: D1 severity now cites all three conditions
>   per BC-2.16.004 Postcondition 1.b (rebind_count >= threshold AND elapsed <= window AND
>   !spoof_high_emitted). v1.1→v1.2.
> - **F-B-004** BC-2.16.004 explicit intra-event ordering added (Step 1 increment, Step 2 set
>   first_rebind_ts, Step 3 evaluate). EC-008 updated to reflect ordering. v1.1→v1.2.
> - **F-B-005** BC-2.16.008 "rate is evaluated after each frame increment" statement added;
>   2-second burst vector annotated with unambiguous elapsed denominator. v1.1→v1.2.
> - **F-B-007** BC-2.16.010 contradictory vector row 2 repaired (inputs now consistent with outputs).
> - **F-B-008** BC-2.16.003 EC-003 reworded to drop "RFC 5227 probe" label for both-zero case;
>   EC-009 added for real RFC 5227 probe (sender_ip=0, target_ip=192.0.2.1 → is_gratuitous_arp=false).
>   v1.0→v1.1.
> - **F-B-009** BC-2.16.005 pins zero/broadcast sender IP admissibility rule (filtered, not
>   inserted); Invariant 5 added; EC-006/007 updated. BC-2.16.004 EC-010 cross-references BC-2.16.005.
>   v1.1→v1.2.
> - **C-CRIT-001/F-D-H1** HS-INDEX ARP summary table corrected: 26 total (24 P0, 2 P1);
>   frontmatter `arp_waves_40_44` updated 20→26; STORY-113 row updated (11 keys). v1.2→v1.3.
> - **C-IMP-002** HS-W43-004: "after STORY-114 merges" qualifier added.
> - **F-D-C1** PRD §2.10 O-04 updated SEEDED=23→25, EMITTED=15→17; BC-2.10.005 row updated
>   (23 Total → 25 Total); RTM entry updated; §6.5 KD-005 updated.
> - **F-D-C2** PRD F-ARP-O5 note corrected: P1 count=2 (HS-W44-001, HS-W44-003); HS-W42-002
>   and HS-W43-003 were mislabeled as P1 (both are P0).
> - **F-D-H2** spec-changelog ARP-AMB-003/004 entries annotated RESOLVED.
> - **F-D-H3** test-vectors.md ARP-AMB-004 note updated to RESOLVED.
> - **F-D-H4** spec-changelog arp-f2-pass1-remediation entry: "Documents updated" table added
>   with test-vectors 1.1→1.2 version bump.
> - **F-D-M1** PRD §2.16 "5 MITRE ATT&CK techniques" corrected to "5 detection types (D1, D2,
>   D3, D11, D12) and emits 2 MITRE techniques (T0830, T1557.002)".
> - **O-D1** PRD §2.16 Detection surface GARP bullet labeled "D2: GARP".
> - **O-D3** error-taxonomy E-ARP-002 "exceeds" → "meets or exceeds".
> Total BC count: 283 (unchanged). See `spec-changelog.md` §[arp-f2-pass2-remediation-2026-06-12].

> **Version 1.12 delta (2026-06-12 — F2 adversarial Pass 4 propagation sweep):**
> Completes propagation of pass-3 Enterprise/ICS split corrections into all consuming documents.
> Key changes:
> - **F-D4-C1** §2.10 O-04: corrected "11 Enterprise + 14 ICS seeded; 6 Enterprise + 11 ICS emitted"
>   → "12 Enterprise + 13 ICS seeded (25 total); 7 Enterprise + 10 ICS emitted (17 total);
>   CATALOGUE-ONLY=8". Authoritative split from BC-2.10.005 v1.9 / BC-2.10.008 v1.10 (pass-3).
>   T1557.002 is Enterprise; T0830 is ICS.
> - **F-D4-C1** §6.5 KD-005 BC-2.10.005 row: "11 Enterprise + 14 ICS" → "12 Enterprise + 13 ICS;
>   T0830 [ICS] + T1557.002 [Enterprise] new ARP F2".
> Total BC count: 283 (unchanged). See `spec-changelog.md` §[arp-f2-pass4-remediation-2026-06-12].

> **Version 1.13 delta (2026-06-12 — F2 adversarial Pass 8 remediation):**
> F-D8-M01: §2.2 BC-2.02.009 row title updated from stale v1.4 title to canonical v1.5 H1/BC-INDEX
> title ("decode_packet routes lax ARP to extract_arp_frame"). No new BCs; no BC count change.
> See `spec-changelog.md` §[arp-f2-pass8-remediation-2026-06-12].

> **Version 1.14 delta (2026-06-12 — F2 adversarial Pass 10 remediation):**
> F-D10-M01: §2.10 O-04 note corrected T0885/T1692.002 label from "(Enterprise)" to "(ICS)";
> arithmetic 12E+13I unaffected. No new BCs; no BC count change.
> See `spec-changelog.md` §[arp-f2-pass10-remediation-2026-06-12].

> **Version 1.15 delta (2026-06-12 — F2 adversarial Pass 11 remediation):**
> F-D11-H01: BC-2.04.055 and BC-2.09.007 rows added to §2.4, §2.9, and §7 RTM (issue-#100 F2
> additions already counted in the 283-total derivation but missing from the index tables).
> F-D11-M01: §2.9 range note corrected. O-D11-02: T0846 added to §1 technique enumeration.
> No new BCs; total remains 283. See `spec-changelog.md` §[arp-f2-pass11-remediation-2026-06-12].

> **Version 1.16 delta (2026-06-13 — ARP-F2 Pass-13 slice-D fix):**
> Slice-D: BC-2.16.008 citation EC-008 → EC-002 (same-second storm denominator edge case).
> No new BCs; no BC count change. See `spec-changelog.md` §[pass-13-corpus-cleanup-2026-06-13].

> **Version 1.17 delta (2026-06-13 — ARP-F2 Pass-14 PO Burst 2 remediation):**
> D-01 (HIGH): BC-2.14.004 row §2.14.A corrected reject range from "[2, 253]" to "[2, 254]".
> Canonical range per BC-2.14.004 H1, ECs, VP-022:117, and BC-INDEX:344. Length field=254 is
> valid (unit-id byte + 253-byte PDU); len=255 is the first invalid value. No BC count change.
> See `spec-changelog.md` §[arp-f2-pass-14-po-burst-2-2026-06-13].

> **Version 1.18 delta (2026-06-13 — ARP-F2 Pass-14 PO Burst 9, O-01 CLOSED):**
> Three residual O-01 stale current-state claims removed from prd.md: §1.5 Out of Scope
> timestamp note, §2.9 ss-09 "timestamp always None" note, §8 Domain Debt Index O-01 row
> struck through. Domain-debt O-01 (Finding.timestamp always None) is CLOSED — timestamp
> wired by STORY-097/098/099 across all 21/22 applicable emission sites; BC-2.04.054 retains
> timestamp:None by design as the sole exception. No new BCs; no BC count change.
> See `spec-changelog.md` §[arp-f2-pass-14-po-burst-9-2026-06-13].

> **Version 1.19 delta (2026-06-13 — Pass-21 ledger hygiene sync):**
> B-01: Added concise body delta notes for versions 1.13/1.14/1.15/1.16/1.18 (previously absent
> from the inline version history). Version history is now contiguous from 1.1 through 1.19.
> No behavioral changes; no BC count change.
> See `spec-changelog.md` §[pass-21-fixes-2026-06-13].

> **Version 1.20 delta (2026-06-13 — Pass-24 DNP3 component mislabel sweep):**
> D-01 (HIGH): All ss-15 (DNP3) BCs updated C-23 → C-24 (Dnp3Analyzer; C-23 was previously
> assigned to SS-15/DNP3, which was renumbered to C-24 when the ARP analyzer claimed C-23).
> §2.15 group header corrected C-26 → C-24. No new BCs; no BC count change.
> [Prose corrected in v1.22 per DRIFT-PRD-V120-MBAPFRAMER-001: original text erroneously stated
> "C-23 was MbapFramer, a Modbus component" — no MbapFramer component ever existed.]
> See `spec-changelog.md` §[pass-24-fixes-2026-06-13].

> **Version 1.21 delta (2026-06-13 — Pass-29 PRD findings D-01 + D-02):**
> D-01 (MED): FC 0x17 added to holding-register write set in 4 locations: §2 v2 co-emission box
> (0x06/0x10/0x16 → 0x06/0x10/0x16/0x17), §2.14.D group header, §2.14.D BC-2.14.014 index row,
> §6.5 KD-005 BC-2.14.014 row. Canonical write-set {0x06, 0x10, 0x16, 0x17} per BC-2.14.014 v2.1
> (BC-DISCREPANCY-001 reconciliation). D-02 (LOW): v1.16 delta changelog anchor corrected from
> §[pass-13-2026-06-13] (non-existent) to §[pass-13-corpus-cleanup-2026-06-13] (resolving).
> Architect P29 A-01 architecture doc bumps: module-decomposition, system-overview,
> purity-boundary-map, module-criticality (per architect P29 A-01 burst).
> No new BCs; no BC count change.
> See `spec-changelog.md` §[pass-29-fixes-2026-06-13].

> **Version 1.22 delta (2026-06-14 — Pass-22 F3-convergence PRD reconciliation):**
> Three defects remediated (F3 Pass-22):
>
> - **FIX-1 (HIGH) — ARP holdout seed count reconciliation (26→28):** The v1.10 and v1.11 delta
>   notes recorded the ARP seed count as 26 (24 P0, 2 P1) and HS-W44-001 as P1. These notes were
>   accurate for the HS-INDEX state at that time (v1.2–v1.3). The HS-INDEX was subsequently
>   expanded to v1.6, adding HS-W44-004 through HS-W44-007 (7 seeds in wave 44 vs. the prior 3)
>   and reclassifying HS-W44-001 from P1 to P0. The canonical HS-INDEX v1.6 values are:
>   **Total ARP feature holdout seeds = 28 (27 P0, 1 P1)**; the single P1 seed is
>   **HS-W44-003** (--arp-storm-rate override) ONLY; **HS-W44-001 is P0** (D3 storm detection).
>   frontmatter `arp_waves_40_44 = 28`. Wave breakdown: W40=4, W41=4, W42=8, W43=5, W44=7.
>   The v1.10/v1.11 historical notes are preserved as-is (immutable history); this note is the
>   authoritative reconciliation record. DRIFT-PRD-ARP-SEED-COUNT-001 CLOSED.
>
> - **FIX-2 (LOW) — BC-2.02.009 version annotation:** The v1.13 delta note cites
>   "canonical v1.5 H1/BC-INDEX title" for BC-2.02.009. That title was subsequently superseded:
>   BC-2.02.009 was further revised to v1.6 (per BC-INDEX.md:28/:63, ARCH-INDEX ADR-008, and
>   spec-changelog). The v1.13 historical note is preserved; this annotation records that
>   BC-2.02.009 was subsequently bumped to v1.6 after the v1.13 pass.
>   The §2.2 live-body row title (line ~454) already reflects the current BC-INDEX H1 — no
>   live-body change required.
>
> - **FIX-3 (LOW) — v1.20 MbapFramer prose corrected:** The v1.20 delta note (Pass-24) stated
>   "C-23 was MbapFramer, a Modbus component." This is factually incorrect — no MbapFramer
>   component ever existed in the architecture. The correct history is that C-23 was previously
>   assigned to SS-15/DNP3 (Dnp3Analyzer), and SS-15/DNP3 was renumbered from C-23 to C-24 when
>   the ARP analyzer (SS-16/ArpAnalyzer) claimed C-23. The v1.20 prose error is corrected in the
>   v1.20 delta text below. DRIFT-PRD-V120-MBAPFRAMER-001 CLOSED.
> No new BCs; no BC count change. See `spec-changelog.md` §[pass-22-f3-convergence-2026-06-14].

> **Version 1.23 delta (2026-06-14 — Pass-23 F3-convergence PRD defect remediation):**
> Two defects remediated (F3 Pass-23 convergence):
>
> - **FIX-1 (HIGH) — Dangling changelog anchor resolved:** The v1.22 delta note referenced
>   `spec-changelog.md §[pass-22-f3-convergence-2026-06-14]`, but that anchor did not exist.
>   The missing entry has been added to spec-changelog.md (inserted at the top of the entry
>   list, above [pass-5-propagation-gap-fixes-2026-06-14]) recording: ARP holdout seed-count
>   26→28 reconciliation (27 P0 + 1 P1=HS-W44-003; HS-W44-001=P0; arp_waves_40_44=28);
>   DRIFT-PRD-ARP-SEED-COUNT-001 CLOSED; v1.20 MbapFramer prose corrected →
>   DRIFT-PRD-V120-MBAPFRAMER-001 CLOSED; BC-2.02.009 v1.6 annotation. The anchor now
>   resolves correctly.
>
> - **FIX-2 (MEDIUM) — BC-2.16.004 mis-listed as VP-024 Sub-C formal anchor corrected:**
>   §2.16 formal verification description (Sub-property C) previously stated "Anchors
>   BC-2.16.004/BC-2.16.005." Per VP-INDEX, VP-024 Verified BCs are BC-2.16.001, .002,
>   .003, .005, .006 ONLY; BC-2.16.004 is explicitly excluded from VP-024's formal
>   Verified-BCs scope (primary-owned by STORY-114, verified by unit+proptest, indirectly
>   supported). Sub-C primary anchor is BC-2.16.005. Both the §2.16 Sub-C description and
>   the F-ARP-O4 delta note (v1.10) have been corrected to reflect this. RTM row for
>   BC-2.16.004 (unit+proptest) was already correct; no RTM change required.
> No new BCs; no BC count change. See `spec-changelog.md` §[pass-22-f3-convergence-2026-06-14].

> **Version 1.24 delta (2026-06-14 — Pass-24 F3-convergence two-defect remediation):**
> Two defects remediated (F3 Pass-24 convergence):
>
> - **FIX-1 (CRITICAL) — BC-2.15.017 spec<->code mis-anchor reverted:** The Pass-22 rename of
>   `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT` → `DNP3_DIRECT_OPERATE_THRESHOLD_DEFAULT` in
>   BC-2.15.017 was erroneous. `DNPXX_` is the actual shipped constant name
>   (src/analyzer/dnp3.rs:169, src/cli.rs:16+183, STORY-110). All three live occurrences in
>   BC-2.15.017 (Precondition 2, Architecture Anchor cli.rs ref, Architecture Anchor
>   dnp3-architecture-delta.md ref) have been restored to `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT`.
>   BC-2.15.017 bumped to v1.4. The sealed historical v1.3 changelog entry is preserved as-is.
>   Note for orchestrator: the `DNPXX_` source symbol name is a code-quality tech-debt candidate
>   for a future code-cleanup pass; it is NOT an F3 fix target.
>
> - **FIX-2 (LOW) — §2.16.F BC-2.16.010 title-sync "(11 Keys)" enrichment:** The §2.16.F index
>   row title has been updated from "ArpAnalyzer::summarize() returns AnalysisSummary with
>   required keys" to "ArpAnalyzer::summarize() returns AnalysisSummary with required keys
>   (11 Keys)" to match the canonical BC H1 (BC-2.16.010.md) and BC-INDEX per Criterion-75.
>
> No new BCs; no BC count change. See `spec-changelog.md` §[pass-24-f3-convergence-2026-06-14].

> **Version 1.26–1.28 delta (2026-06-17 — Feature #259 terminal finding collapse, issue #259, v0.8.0; v1.27 adv-pass-9 remediation; v1.28 adv-passes-12-14 remediation):**
> Added 5 new BCs (BC-2.11.025..029) for the terminal finding collapse feature. Extended 4 existing
> BCs (BC-2.11.010 v1.4→v1.8, BC-2.11.013 v1.8→v1.11, BC-2.11.017 v1.7→v1.11, BC-2.11.019 v1.4→v1.6)
> with collapse-interaction clauses; further bumped by adversarial pass-1/2/3/4/5 remediation:
> BC-2.11.010 v1.5 (F2-integrate) → v1.6 (adv-pass-3) → v1.7 (adv-pass-4 anchor) → v1.8 (adv-pass-5: remove residual 'path-(b)' label from Invariant 4);
> BC-2.11.013 v1.9 (F2-integrate) → v1.10 (adv-pass-2) → v1.11 (adv-pass-4 EC-007 observable-behavior);
> BC-2.11.017 v1.8 (F2-integrate) → v1.9 (adv-pass-1) → v1.10 (adv-pass-2) → v1.11 (adv-pass-4 observable-behavior + PC-6) → v1.12 (adv-pass-9 F-PA-01: cross-ref BC-2.11.026 PC-6 in Invariant 5) → v1.13 (adv-passes-12-14 F-PA-A01: representative = group_members[0]; PC-6 + EC-007 updated; divergent-mitre test vector);
> BC-2.11.019 v1.5 (F2-integrate) → v1.6 (adv-pass-2 anchor fix);
> New greenfield BCs: BC-2.11.025 v1.0→v1.3, BC-2.11.026 v1.0→v1.6, BC-2.11.027 v1.0→v1.3, BC-2.11.028 v1.0→v1.2, BC-2.11.029 v1.0→v1.2;
> BC-2.11.025 v1.2 → v1.3 (adv-pass-4: Invariant 6 observable-behavior; anchor; flood vector timestamp fix) → v1.4 (adv-pass-9 F-PA-02: soften timestamp claim to MAY differ) → v1.5 (adv-passes-12-14 F-PA-A01: generalize representative definition to all N≥1 = group_members[0]);
> BC-2.11.026 v1.3 → v1.4 (adv-pass-4) → v1.5 (adv-pass-5: remove 'path-(b) separation') → v1.6 (adv-passes-6-8: red-bold test vector LOW-1) → v1.7 (adv-pass-9 F-PA-01: normative PC-6 color-ladder) → v1.8 (adv-passes-12-14 F-PA-A01: normative PC-7 representative = group_members[0]; divergent-mitre test vector);
> BC-2.11.028 v1.2 → v1.3 (adv-pass-9 F-PA-03: add EC-010 default-output default-on) → v1.4 (adv-passes-12-14 F-PB-01: drop global-flag convention citation; fix cli.rs anchor; correct subcommand-scoped precedent);
> with collapse-interaction clauses. §2.11 index table updated with 5 new rows and a group note.
> Total BC count: 288 (was 283).
> Key design decisions (F1-gated, non-negotiable):
> - OQ-1: DEFAULT-ON collapse; `--no-collapse` opt-out flag (BC-2.11.028).
> - OQ-2: ALWAYS collapse, no threshold; N=1 singletons render without suffix (BC-2.11.026).
> - OQ-3: FLAT-MODE ONLY for v0.8.0; grouped/`--mitre` mode deferred to STORY-119 (BC-2.11.013 v1.9 Invariant 4).
> - OQ-4: K=3 evidence lines per collapsed group; hardcoded constant (BC-2.11.027).
> No new VP: collapse feature is test-sufficient per F1 analysis; VP-012 (`escape_for_terminal`) unchanged.
> ADR-0003 extended by architect (display-layer aggregation subsection); cited in all new/extended BCs.
> See `spec-changelog.md` §[issue-259-collapse-f2-2026-06-17].
>
> **Version 1.25 delta (2026-06-14 — Pass-26 post-consistency-flush §2.15 title-sync):**
> Two §2.15 BC index rows synced to their canonical H1 headings (part of the post-Pass-26
> full-corpus consistency flush; same burst also covered VP-006 Must→Should table,
> src-citation symbol-anchoring, and line-pin de-pins):
>
> - **§2.15.C BC-2.15.009 row title synced:** Updated subtitle to match H1 "Initial-Delivery
>   No-Sync (One-Shot, First Delivery Only)"; removed stale "first 16 bytes" framing that had
>   drifted from the canonical H1 wording.
>
> - **§2.15.F BC-2.15.016 row title synced:** Added "master_addrs ≤64, pending_requests ≤256"
>   bounds to the row title to match the canonical BC H1 (which carried these bounds
>   post-feature-008-F2).
>
> No new BCs; no BC count change (283). See `spec-changelog.md` §[prd-v1.25-ss15-titlesync-2026-06-14].

> **Version 1.36 delta (2026-06-24 — F2 EtherNet/IP + CIP analyzer, feature-enip-v0.11.0, issue #316):**
> Added Section 2.17 (SS-17 EtherNet/IP + CIP Analysis, 25 BCs, ADR-010, VP-032). New MITRE
> techniques entering catalog: T0858 "Change Operating Mode" (IcsExecution TA0104 — CIP Stop,
> new `MitreTactic::IcsExecution` variant required) and T0816 "Device Restart/Shutdown"
> (IcsInhibitResponseFunction TA0107 — CIP Reset). Both require `technique_info()` arms in
> src/mitre.rs. Already-seeded techniques used: T0836/T0846/T0888/T0814; T0846 is NOW emitted
> (BC-2.17.010 ListIdentity). T1693.001 staged but not emitted in v0.11.0 (GetAndClear firmware
> detection deferred). SEEDED grows 25→28; EMITTED grows 17→20 (T0858+T0816+T0846 move from
> catalogue-only/not-emitted to emitted); CATALOGUE-ONLY changes 8→8 (T0846 leaves catalogue-only (now emitted); T1693.001 enters catalogue-only;
> T0858/T0816 are new seeds, immediately emitted; net change = 0). Open item OA-001: --enip-write-burst-threshold
> default (50/1s) — changed from 20, MEDIUM-confidence, human confirmation at F2 gate. See `.factory/phase-f2-spec-evolution/enip-prd-delta.md`
> for full delta record. Added SS-17 rows to Section 7 RTM. Total BCs: 304 on disk → 329;
> active: 304 → 328. BC-INDEX v1.73→v1.74.

> **Version 1.35 delta (2026-06-23 — F5 ICS tactic-ID correctness fix, DF-SIBLING-SWEEP-001):**
> §2.10 BC-2.10.004 index row updated: "(17 total)" → "(20 total)" per MitreTactic enum growing
> from 17 to 20 variants (14 Enterprise + 6 ICS). Three new ICS variants added in F5 D-209:
> IcsDiscovery (TA0102), IcsCollection (TA0100), IcsCommandAndControl (TA0101).
> No new BCs; active BC count unchanged at 303.

> **Version 1.34 delta (2026-06-22 — F2 issue #64 mitre_attack JSON enrichment, v0.11.0):**
> 1 new BC (BC-2.11.035) added for per-finding `mitre_attack` JSON array (ECS/OCSF alignment).
> Adds resolved technique objects (id, name, tactic_id, tactic_name, reference) per entry in
> `mitre_techniques`, in order. Unknown IDs emit partial objects (id + reference only; agent-safety).
> Empty `mitre_techniques` omits `mitre_attack` (skip_serializing_if, additive non-breaking).
> Catalog extension required: `technique_tactic_id()` in src/mitre.rs maps MitreTactic variants
> to canonical TA-prefix IDs. No new error codes; no new VP (test sufficient). BC-2.11.001 v1.6→v1.7
> (advisory pointer to BC-2.11.035). interface-definitions.md v1.2→v1.3 (mitre_attack field in
> per-finding JSON schema). §2.11 BC index table: BC-2.11.035 row added; footer note updated.
> §7 RTM: BC-2.11.035 row added. SS-11: 34→35 BCs. Total active BCs: 302→303 (304 on disk).
> See BC-2.11.035.md.

> **Version 1.33 delta (2026-06-19 — F2 re-audit PRD-BC2-1 remediation):**
> PRD-BC2-1 (MEDIUM): §2.1 BC-2.12.011 index row description updated from stale pre-v1.5 wording
> ("Directory target expands to all *.pcap files sorted; *.pcapng excluded from glob") to match
> BC-2.12.011 v1.5: "Directory target expands to capture files detected by magic bytes (content
> detection), not extension." No normative BC content changed; no BCs added or retired.

> **Version 1.32 delta (2026-06-19 — §7 RTM sync to F2 remediation state):**
> §7 RTM rows for BC-2.01.009–018 updated: Test Type column now carries VP assignments
> (VP-025→BC-2.01.014, VP-026→BC-2.01.010, VP-027→BC-2.01.012, VP-028→BC-2.01.017,
> VP-029→BC-2.01.015, VP-030→BC-2.01.018), corrected error-code routing (E-INP-008/009/010/011/012
> per BC), and provisional story anchors (STORY-123..126 F3-planned). BC-2.12.011 row updated
> with STORY-127 anchor. No normative BC content changed; no BCs added or retired.

> **Version 1.31 delta (2026-06-19 — pcapng completeness deltas F-06/F-07/F-11):**
> AC-level additions to BC-2.01.010 (F-06: multi-section SHB reject, E-INP-012), BC-2.01.015
> (F-07: enumerate all skip-arm variants — NRB, ISB, DSB, SystemdJournalExport, OPB 0x2,
> Unknown), and BC-2.01.018 (F-11: per-file error isolation in directory mode, actionable
> E-INP-011 hint). E-INP-012 added to error-taxonomy.md (multi-section SHB reject);
> E-INP-011 Notes revised (tcpdump -i any actionable hint). No new BCs; active BC count
> unchanged at 302. See `spec-changelog.md` §[pcapng-completeness-f06-f07-f11-2026-06-19].
>
> **Version 1.30 delta (2026-06-19 — F2 audit FINDING-003 remediation):** §7 RTM corrected:
> BC-2.01.004 RTM row struck through [RETIRED → BC-2.01.009]; 10 new RTM rows added for
> BC-2.01.009–018 (CAP-01, SS-01, priorities per §2.1, test type: integration — not yet
> delivered; F3 stories STORY-123..127). No normative BC content changed.

> **Version 1.29 delta (2026-06-19 — F2 pcapng-reader-support, ADR-009, FE-001):** pcapng is
> now a SUPPORTED input format. 10 new BCs (BC-2.01.009–018) added to §2.1 for pcapng block-walk
> reader (magic-byte probe, SHB, IDB, EPB, SPB, unknown-block skip, timestamp normalization,
> link-type gating, error surfacing, multi-IDB agreement policy). BC-2.01.004 RETIRED (behavioral
> inversion: pcapng was rejected, now accepted). §1.5 Out-of-Scope: pcapng item struck out and
> marked REMOVED from out-of-scope. BC-2.01.001 v1.6→v1.7 (EC-005 scope note). BC-2.01.002
> v1.5→v1.6 (classic-pcap-branch scope note). error-taxonomy.md v2.2→v2.3 (E-INP-008..011 added;
> E-INP-002 notes revised). nfr-catalog.md v2.1→v2.2 (NFR-COMPAT-001 revised). Total active
> BCs: 293→302 (303 on disk − 1 retired). See `spec-changelog.md` §[pcapng-f2-2026-06-19].

> **Supplement Model:** Sections 3-5 reference extracted supplement files under
> `prd-supplements/`. These supplements are produced in a SEPARATE burst (Phase 1b).
> Entries in those sections are summary stubs until the supplement burst completes.


## 1. Product Overview

### 1.1 Problem Statement

Network security analysts and incident responders must triage captured network traffic for
indicators of compromise. Existing tools (Wireshark, Zeek, Suricata) require network
connectivity, complex configuration, or ongoing daemon processes. Analysts working on isolated
forensic workstations need a single-binary tool that produces structured, machine-readable
findings from pcap captures without any runtime infrastructure.

Additionally, existing tools often sanitize or alter attacker-controlled data during analysis,
destroying forensic fidelity. A raw HTTP URI containing C0 control bytes looks different after
being processed by a display-layer renderer -- yet the raw bytes are the evidence.

### 1.2 Solution Vision

wirerust is an offline, single-binary, single-pass forensic triage CLI that ingests classic-pcap
captures and emits structured findings about HTTP, TLS, and DNS traffic plus TCP stream-reassembly
anomalies. It has no network I/O, no async runtime, no unsafe blocks, and no process-to-process
state. The binary is the complete deployment unit.

The core design principle is "trustworthy forensic data preservation plus display-layer safety":
raw attacker-controlled bytes survive intact through every layer to JSON output; the terminal
renderer is the sole owner of escape logic. This ensures SIEM consumers see unaltered forensic
data while terminal operators are protected from terminal injection attacks.

Architecture: 5-layer synchronous pipeline (Entry -> Ingest -> Stream -> Domain -> Output), 24
Rust source files, 3,868 source LOC, 282 tests, single crate, Rust 2024 edition, MSRV 1.91.

### 1.3 Key Differentiators

| ID | Differentiator | Description |
|----|---------------|-------------|
| KD-001 | Offline single-binary deployment | No daemon, no network I/O, no runtime dependencies. Suitable for air-gapped forensic workstations. |
| KD-002 | Forensic-fidelity raw-data contract | Attacker-controlled bytes (URIs, SNI hostnames, payloads) pass through unmodified to JSON output; escape runs only at terminal display (ADR 0003). |
| KD-003 | Content-first protocol identification | Protocol dispatch inspects TCP payload bytes before port numbers, defeating port-obfuscation attacks (ADR 0001). |
| KD-004 | First-wins TCP overlap forensics | Conflicting retransmissions are detected and emitted as findings; attackers cannot silently insert alternate bytes (INV-3). |
| KD-005 | MITRE ATT&CK tactic-grouped output | Findings carry structured MITRE technique IDs; terminal output can group by tactic for kill-chain analysis. |
| KD-006 | SNI anomaly detection with 4-way classification | TLS SNI hostnames are classified into four categories (clean ASCII, C0/DEL-containing, non-ASCII UTF-8, non-UTF-8 bytes) each triggering distinct findings. |
| KD-007 | Bounded-resource design | MAX_FINDINGS cap (10,000), per-direction buffer caps (65 KB), configurable reassembly thresholds with CLI override, no unbounded accumulation paths (except O-06). |

### 1.4 Target Users

| Persona | Description | Volume | Pain Level |
|---------|-------------|--------|------------|
| Forensic analyst | Processes pcap captures from incident response collections on isolated workstations | Low volume, high frequency during IR | High -- needs structured output fast, cannot install complex tooling |
| SOC operator | Bulk-processes pcap archives for indicator extraction, feeds output into SIEM | Medium volume, batch mode | High -- JSON output must be machine-parseable, not display-oriented |
| Malware researcher | Analyzes C2 traffic patterns, TLS fingerprinting, HTTP evasion techniques | Low volume, deep analysis | Medium -- needs JA3/JA3S and SNI anomaly details |
| Security toolchain integrator | Uses wirerust as a preprocessing stage in a pipeline (jq, grep, awk on JSON output) | High volume, automated | Medium -- needs deterministic JSON key order, stable exit codes |

### 1.5 Out of Scope

> Machine-consumed constraint list. The adversary and consistency-validator check that no story
> AC implements any feature listed here. Be explicit and unambiguous.

- ~~pcapng format support (wirerust reads classic pcap ONLY; pcapng files are rejected at the reader boundary)~~ **REMOVED from out-of-scope (F2 pcapng-reader-support, ADR-009, 2026-06-19): pcapng is now a SUPPORTED input format via BC-2.01.009–018 magic-byte probe and block-walk reader.**
- Live network capture / sniffing (no network I/O of any kind; offline pcap files only)
- HTTP/2 or HTTP/3 analysis (HTTP/1.x only; H2 frames will be parsed as unknown bytes)
- DNS-based detection findings (DNS is statistics-only: query/response counts only; no NXDOMAIN flood, no tunneling detection)
- TLS decryption or certificate validation (SNI and cipher fingerprinting only; no key material involved)
- BPF filtering (--filter flag removed by PR #74; clap rejects --filter as unknown argument; out of scope for current release)
- C2 beacon detection (--beacon flag removed by PR #74; clap rejects --beacon as unknown argument; no beacon analyzer exists)
- --threats flag behavior (flag removed by PR #74; clap rejects --threats as unknown argument; no corresponding analyzer)
- --verbose flag (removed by PR #74 alongside --filter/--beacon/--threats; clap rejects --verbose as unknown argument; no verbosity levels defined)
- --services flag on summary subcommand (removed by PR #74; clap rejects --services as unknown argument; per-service breakdown is out of scope for current release)
- Parallel file processing (rayon = "1" is a declared production dependency but is entirely unused in src/; single-threaded only)
- Streaming / lazy-read pcap processing (entire file loaded into RAM before processing)
- Per-packet timestamp in findings: RESOLVED — BC-2.09.007 (F2) wired timestamp from the pcap record header at all applicable emission sites (STORY-097/098/099); domain-debt O-01 CLOSED. Exception: segment-limit summary finding (BC-2.04.054) retains timestamp:None by design.
- Empirically-calibrated anomaly thresholds (defaults are research-documented but not validated against labelled traffic; O-03)
- MITRE techniques T1040, T1071, T1071.001, T1071.004, T1573, T1692.002, T0885 (catalogued but never emitted; O-04; note: T1692.001, T0836, T0814, T0806, T0835, T0831, T0888 are now emitted by the Modbus/ICS analyzer — see Section 2.14; T0846 is NOW emitted by the EtherNet/IP analyzer (BC-2.17.010 ListIdentity) — removed from not-emitted list; T1692.002 replaces revoked T0856 per ATT&CK-ICS v19 remap)


## 2. Behavioral Contracts Index

> BCs are organized by L2 domain capability (CAP-NN). BC numbering: BC-2.NN.NNN where
> 2 = PRD section, NN = capability number, NNN = sequential within capability.
> Files live in `behavioral-contracts/ss-NN/BC-2.NN.NNN.md`.

> **BREAKING OUTPUT SCHEMA CHANGE — v0.3.0 (ADR-006):**
> `Finding.mitre_technique: Option<String>` is renamed and retyped to
> `Finding.mitre_techniques: Vec<String>`. This affects ALL analyzers and ALL reporters:
> - **JSON:** key `"mitre_technique"` (scalar string) → `"mitre_techniques"` (array);
>   field absent when empty (same policy as prior `None` via `skip_serializing_if`).
> - **JSON envelope:** two new top-level fields added: `mitre_domain: "ics-attack"` and
>   `mitre_attack_version: "ics-attack-v15"` (placeholder; F4 must pin). See BC-2.11.001 v1.5.
> - **CSV:** column-6 header renamed `mitre_technique` → `mitre_techniques`; multiple
>   values semicolon-joined (`"T1692.001;T0836"`); single value unchanged; empty cell is `""`
>   (not `"null"`, not `"[]"`); consumers splitting on `;` must guard the empty-cell case
>   (see BC-2.11.024 v1.5 EC-015). CSV carries no envelope fields.
> - **Rust type:** `Option<String>` → `Vec<String>`; all emission sites updated.
>   All downstream JSON consumers, CSV pipelines, and Rust code using `Finding` must update.
> See ADR-006, BC-2.09.001, BC-2.09.006, BC-2.11.001, BC-2.11.020, BC-2.11.024.
> Affected stories: STORY-069, STORY-070, STORY-071, STORY-078, STORY-079, STORY-080.

> **RELEASE SEQUENCING — Feature #7 split: v0.3.0 (schema) + v0.4.0 (Modbus) (f2-bundle-vs-split.md B2):**
> Feature #7 is split into two releases per research recommendation (B2 — Trivy/Zeek pattern):
>
> **v0.3.0 — "Multi-technique findings" (schema migration only; breaking):**
> All existing analyzers (HTTP/TLS/DNS/lifecycle) migrated to `mitre_techniques: Vec<String>`.
> JSON envelope fields added. No new protocol analyzer. This is a **semver-honest breaking
> release**: one signal, one break, focused migration note.
> BCs in scope for v0.3.0:
> - SS-09 (findings.rs): BC-2.09.001, BC-2.09.006
> - SS-10 (mitre.rs): BC-2.10.005, BC-2.10.007, BC-2.10.008
> - SS-11 (reporters): BC-2.11.013, BC-2.11.015, BC-2.11.017, BC-2.11.020, BC-2.11.024
>   (+ BC-2.11.001 for envelope ADD-ON 1)
> - Existing stories: STORY-069, STORY-070, STORY-071, STORY-078, STORY-079, STORY-080
>
> **v0.4.0 — "Modbus TCP analyzer" (purely additive; no schema break):**
> Adds the Modbus TCP protocol analyzer on top of the stabilized multi-tag contract.
> Multi-tag type ships in v0.3.0; Modbus emits multi-tag findings natively but the *type
> itself* is already stable. No `**Breaking:**` entry in v0.4.0 changelog.
> BCs in scope for v0.4.0: all SS-14 BCs (BC-2.14.001 through BC-2.14.025).
> T0888/dual-window/co-emission detection are v0.4.0 (Modbus analyzer emits these;
> the multi-tag Vec<String> type that enables them ships in v0.3.0).
>
> Rationale: f2-bundle-vs-split.md establishes that multi-tag is independent of Modbus
> (shared `Finding` struct in `findings.rs`), bundling couples a cross-cutting refactor
> with a new stateful analyzer (worst pairing for bisection/rollback), and the Trivy
> two-phase flag model is the closest OSS precedent. Compat softening: `--compat-mitre-scalar`
> flag (default on in v0.3.x) emits the old scalar `mitre_technique` key alongside the new
> array for a deprecation window, following the Zeek dual-field approach.

### 2.1 PCAP File Ingestion / pcapng Reader Support (CAP-01)

> **F2 pcapng-reader-support delta (2026-06-19, ADR-009, FE-001):** BC-2.01.004 RETIRED (behavioral inversion). 10 new BCs (BC-2.01.009–018) added for pcapng support. pcapng is now a SUPPORTED input format.

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.01.001 | Accept Supported Link Types and Reject Unsupported at File Open | P0 | BC-RDR-001 |
| BC-2.01.002 | Read All Packets from PCAP as Vec<RawPacket> Preserving Timestamps (classic-pcap branch) | P0 | BC-RDR-002 |
| BC-2.01.003 | Accept pcap with zero packets (header-only) without error | P1 | BC-RDR-003 |
| ~~BC-2.01.004~~ | ~~Reject pcapng-format input at reader level~~ [RETIRED — superseded by BC-2.01.009] | ~~P0~~ | BC-RDR-004 |
| BC-2.01.005 | Convert pcap record timestamp to (timestamp_secs: u32, timestamp_usecs: u32) | P1 | BC-RDR-005 |
| BC-2.01.006 | Surface pcap header parse errors with anyhow context | P1 | BC-RDR-006 |
| BC-2.01.007 | Surface per-packet read errors with anyhow context | P1 | BC-RDR-007 |
| BC-2.01.008 | from_file opens via BufReader and delegates to from_pcap_reader | P2 | BC-RDR-008 |
| BC-2.01.009 | Accept pcapng Format: Transparent Detection via Magic-Byte Probe | P0 | feature-pcapng-F2 |
| BC-2.01.010 | Parse pcapng Section Header Block (SHB): Byte-Order Detection and Version | P0 | feature-pcapng-F2 |
| BC-2.01.011 | Parse pcapng Interface Description Block (IDB): Link Type and Timestamp Resolution | P0 | feature-pcapng-F2 |
| BC-2.01.012 | Parse pcapng Enhanced Packet Block (EPB): Packet Data and Timestamp | P0 | feature-pcapng-F2 |
| BC-2.01.013 | Parse pcapng Simple Packet Block (SPB): Packet Data Without Timestamp | P1 | feature-pcapng-F2 |
| BC-2.01.014 | Pure-Core 64-bit pcapng Timestamp Normalization to (ts_sec, ts_usecs) | P0 | feature-pcapng-F2 |
| BC-2.01.015 | Unknown pcapng Block Types Are Silently Skipped via block-total-length | P1 | feature-pcapng-F2 |
| BC-2.01.016 | Reject pcapng with Unsupported Link Type in IDB (Mirrors BC-2.01.001) | P0 | feature-pcapng-F2 |
| BC-2.01.017 | pcapng Block-Level Parse Errors Surface via anyhow Context Chain | P1 | feature-pcapng-F2 |
| BC-2.01.018 | Multi-IDB Link-Type Agreement Policy: Conflict Returns Error (Fail-Closed) | P0 | feature-pcapng-F2 |

> Full contracts: `behavioral-contracts/ss-01/BC-2.01.001.md` through `BC-2.01.018.md` (BC-2.01.004 retired)

### 2.2 Link-Type Gating (CAP-02)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.02.001 | Decode Ethernet-framed IPv4 TCP packet to ParsedPacket | P0 | BC-DEC-001 |
| BC-2.02.002 | Decode Ethernet-framed IPv4 UDP packet with DNS hint | P0 | BC-DEC-002 |
| BC-2.02.003 | Decode RAW link-layer IPv4 TCP packet via from_ip | P0 | BC-DEC-003 |
| BC-2.02.004 | DataLink::IPV4 decodes identically to DataLink::RAW | P1 | BC-DEC-004 |
| BC-2.02.005 | Decode RAW IPv6 TCP packet surfacing IPv6 addresses | P0 | BC-DEC-005 |
| BC-2.02.006 | Decode Linux SLL (cooked) TCP packets | P0 | BC-DEC-006 |
| BC-2.02.007 | Reject malformed input bytes with anyhow error (no panic) | P0 | BC-DEC-007 |
| BC-2.02.008 | Reject unsupported link types in decode_packet | P1 | BC-DEC-008 |
| BC-2.02.009 | Non-IP Non-ARP Frames Return No-IP-Layer Error; ARP Frames Return DecodedFrame::Arp | P1 | BC-DEC-009 |
| BC-2.02.010 | Classify ICMP as Protocol::Icmp with TransportInfo::None | P1 | BC-DEC-010 |
| BC-2.02.011 | Classify other IP protocols as Protocol::Other(byte) | P1 | BC-DEC-011 |
| BC-2.02.012 | app_protocol_hint returns service strings from port number | P1 | BC-DEC-012 |
| BC-2.02.013 | app_protocol_hint returns None when TransportInfo is None | P2 | BC-DEC-013 |
| BC-2.02.014 | packet_len is set to total frame length not just payload | P1 | BC-DEC-014 |
| BC-2.02.015 | Extract TCP control flags and sequence number into TransportInfo::Tcp | P0 | BC-DEC-015 |

> Full contracts: `behavioral-contracts/ss-02/BC-2.02.001.md` through `BC-2.02.015.md`

### 2.3 Packet Decoding (CAP-03)

> CAP-03 BCs are co-located with CAP-02 in ss-02 because the decoder is the single component
> (C-5) implementing both capabilities. The BC-DEC-NNN ingestion IDs map to BC-2.02.NNN above.
> No separate ss-03 directory is required for this capability.

### 2.4 TCP Stream Reassembly (CAP-04)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.04.001 | TcpReassembler::new panics on invalid config (defensive assert) | P1 | BC-RAS-001 |
| BC-2.04.002 | Non-TCP packets are skipped and packets_skipped_non_tcp increments | P1 | BC-RAS-002 |
| BC-2.04.003 | Canonical FlowKey ordering ensures A->B and B->A produce identical key | P0 | BC-RAS-003 |
| BC-2.04.004 | First SYN sets client ISN and initiator | P0 | BC-RAS-004 |
| BC-2.04.005 | SYN+ACK marks server as responder and transitions state to Established | P0 | BC-RAS-005 |
| BC-2.04.006 | Bidirectional data delivered with correct Direction tag | P0 | BC-RAS-006 |
| BC-2.04.007 | In-order data flushes contiguously to handler in segment order | P0 | BC-RAS-007 |
| BC-2.04.008 | Out-of-order segments buffer until gap filled then flush contiguously | P0 | BC-RAS-008 |
| BC-2.04.009 | Mid-stream join infers ISN from first-data seq-1 and marks flow partial | P0 | BC-RAS-009 |
| BC-2.04.010 | RST closes flow immediately with CloseReason::Rst and zeroes total_memory | P0 | BC-RAS-010 |
| BC-2.04.011 | Both FINs close flow with CloseReason::Fin and remove from table | P0 | BC-RAS-011 |
| BC-2.04.012 | finalize flushes all remaining flows with Timeout and is idempotent | P0 | BC-RAS-012 |
| BC-2.04.013 | expire_idle_by_timeout / expire_flows closes flows idle past flow_timeout_secs | P1 | BC-RAS-013 |
| BC-2.04.014 | total_memory tracks buffered bytes and decrements on flush and close | P1 | BC-RAS-014 |
| BC-2.04.015 | Flow eviction on max_flows hit uses LRU non-established-first policy | P1 | BC-RAS-015 |
| BC-2.04.016 | Memory pressure eviction when total_memory exceeds memcap | P1 | BC-RAS-016 |
| BC-2.04.017 | Eviction sort: non-established first, then oldest-last-seen within band | P1 | BC-RAS-017 |
| BC-2.04.018 | Conflicting overlap emits Anomaly/Likely/High finding with MITRE T1036 | P0 | BC-RAS-018 |
| BC-2.04.019 | Excessive overlaps (>threshold) emit one-shot T1036 finding | P0 | BC-RAS-019 |
| BC-2.04.020 | Excessive small segments (>threshold) emit one-shot finding | P1 | BC-RAS-020 |
| BC-2.04.021 | Excessive out-of-window segments (>threshold) emit one-shot Low finding | P1 | BC-RAS-021 |
| BC-2.04.022 | Per-direction alert fires at most once per flow (sticky latch) | P0 | BC-RAS-022 |
| BC-2.04.023 | Truncated segment emits Anomaly/Inconclusive/Low finding (no MITRE) | P1 | BC-RAS-023 |
| BC-2.04.024 | Total findings capped at MAX_FINDINGS=10000; excess silently dropped | P0 | BC-RAS-024 |
| BC-2.04.025 | finalize emits segment-limit summary finding when segments dropped (with pluralization) | P0 | BC-RAS-025 |
| BC-2.04.026 | finalize does NOT emit segment-limit finding when counter is zero | P0 | BC-RAS-026 |
| BC-2.04.027 | segments_depth_exceeded tracks fully-rejected segments after depth hit | P1 | BC-RAS-027 |
| BC-2.04.028 | summarize returns AnalysisSummary with reassembly stats detail map | P1 | BC-RAS-028 |
| BC-2.04.029 | close_flow for missing key logs one-shot process-wide warning | P2 | BC-RAS-029 |
| BC-2.04.030 | bytes_reassembled equals total bytes delivered to handler at end | P1 | BC-RAS-030 |
| BC-2.04.031 | ISN set on first SYN; inferred as seq-1 on data-without-SYN | P0 | BC-RAS-031 |
| BC-2.04.032 | insert_segment with no ISN returns IsnMissing and inserts nothing | P0 | BC-RAS-032 |
| BC-2.04.033 | Single segment insertion returns Inserted and stores under offset key | P0 | BC-RAS-033 |
| BC-2.04.034 | flush_contiguous consumes segments from base_offset in order | P0 | BC-RAS-034 |
| BC-2.04.035 | Identical retransmission returns Duplicate and does not double-count bytes | P0 | BC-RAS-035 |
| BC-2.04.036 | First-wins overlap: gap bytes added, existing bytes preserved | P0 | BC-RAS-036 |
| BC-2.04.037 | Same-range conflicting overlap returns ConflictingOverlap, preserves original | P0 | BC-RAS-037 |
| BC-2.04.038 | Multi-segment full coverage returns Duplicate or ConflictingOverlap as appropriate | P0 | BC-RAS-038 |
| BC-2.04.039 | TCP sequence wraparound across 32-bit boundary reassembles correctly | P0 | BC-RAS-039 |
| BC-2.04.040 | Small-segment counter increments per direction for segments under threshold | P1 | BC-RAS-040 |
| BC-2.04.041 | Depth truncation: segment crossing max_depth is truncated to remaining capacity | P0 | BC-RAS-041 |
| BC-2.04.042 | Segment beyond max_receive_window returns OutOfWindow; boundary segment accepted | P1 | BC-RAS-042 |
| BC-2.04.043 | Adjacent segments at exact boundary do not count as overlap | P0 | BC-RAS-043 |
| BC-2.04.044 | Segments map full: non-overlapping insert returns SegmentLimitReached | P0 | BC-RAS-044 |
| BC-2.04.045 | Segments map full: overlapping insert needing gap insertion returns SegmentLimitReached | P0 | BC-RAS-045 |
| BC-2.04.046 | Segments map fills mid-loop: partial insertion with later gaps dropped | P0 | BC-RAS-046 |
| BC-2.04.047 | buffered_bytes mirrors segment size sum after all insert/overlap/flush ops | P0 | BC-RAS-047 |
| BC-2.04.048 | ISN_MISSING_WARNED atomic prevents repeated eprintln on missing-ISN errors | P2 | BC-RAS-048 |
| BC-2.04.049 | FlowKey::Display formats as lower_ip:lower_port -> upper_ip:upper_port with U+2192 | P1 | BC-RAS-049 |
| BC-2.04.050 | Flow state machine: New->SynSent->Established->Closing->Closed transitions | P0 | BC-RAS-050 |
| BC-2.04.051 | RST transitions state to Closed from any prior state | P0 | BC-RAS-051 |
| BC-2.04.052 | on_data_without_syn transitions New->Established and sets partial=true | P0 | BC-RAS-052 |
| BC-2.04.053 | TcpFlow::direction returns ClientToServer when src matches initiator | P0 | BC-RAS-053 |
| BC-2.04.054 | finalize unconditionally bypasses MAX_FINDINGS cap for segment-limit finding | P0 | BC-RAS-054 |
| BC-2.04.055 | StreamHandler::on_data Carries Capture-Relative Timestamp Parameter | P1 | BC-RAS-055 |

> Full contracts: `behavioral-contracts/ss-04/BC-2.04.001.md` through `BC-2.04.055.md`
> (BC-2.04.055 added Feature Mode F2 issue #100: StreamHandler::on_data timestamp parameter)

### 2.5 Content-First Protocol Dispatch (CAP-05)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.05.001 | TLS content signature routes flow to TLS regardless of port | P0 | BC-DSP-001 |
| BC-2.05.002 | HTTP method prefix routes flow to HTTP | P0 | BC-DSP-002 |
| BC-2.05.003 | Port fallback: 443/8443->TLS, 80/8080->HTTP when content insufficient | P0 | BC-DSP-003 |
| BC-2.05.004 | Unknown content and unknown port returns DispatchTarget::None | P1 | BC-DSP-004 |
| BC-2.05.005 | Classification cached per FlowKey after first non-None result | P0 | BC-DSP-005 |
| BC-2.05.006 | DispatchTarget::None NOT cached until retry cap (default 8); reclassification retried per on_data until cap, then None cached permanently | P0 | BC-DSP-006 |
| BC-2.05.007 | unclassified_flows increments only at on_flow_close for never-classified flows | P1 | BC-DSP-007 |
| BC-2.05.008 | No analyzer configured: dispatcher early-returns from on_data | P1 | BC-DSP-008 |
| BC-2.05.009 | on_flow_close removes route entry and forwards close to analyzer | P0 | BC-DSP-009 |

> Full contracts: `behavioral-contracts/ss-05/BC-2.05.001.md` through `BC-2.05.009.md`

### 2.6 HTTP Traffic Analysis (CAP-06)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.06.001 | Parse complete HTTP/1.1 request extracting method, URI, version, Host, User-Agent | P0 | BC-HTTP-001 |
| BC-2.06.002 | Parse pipelined requests with independent per-request method/uri counting | P0 | BC-HTTP-002 |
| BC-2.06.003 | Partial requests buffered until complete; not counted until full | P0 | BC-HTTP-003 |
| BC-2.06.004 | Parse HTTP/1.1 responses with status code counting and transaction advance | P0 | BC-HTTP-004 |
| BC-2.06.005 | Path traversal in URI emits Reconnaissance/Likely/High finding mapped to T1083 | P0 | BC-HTTP-005 |
| BC-2.06.006 | Web-shell URI patterns emit Execution/Likely/Medium finding mapped to T1505.003 | P0 | BC-HTTP-006 |
| BC-2.06.007 | Admin panel paths emit Reconnaissance/Inconclusive/Low finding mapped to T1046 | P1 | BC-HTTP-007 |
| BC-2.06.008 | Unusual HTTP methods emit Reconnaissance/Inconclusive/Medium finding (no MITRE) | P1 | BC-HTTP-008 |
| BC-2.06.009 | HTTP/1.1 request without Host header emits Anomaly/Inconclusive/Medium finding | P0 | BC-HTTP-009 |
| BC-2.06.010 | URI longer than 2048 chars emits Execution/Likely/Medium finding with char count | P1 | BC-HTTP-010 |
| BC-2.06.011 | Empty (present-but-blank) User-Agent emits Anomaly/Inconclusive/Low finding; absent UA does NOT | P1 | BC-HTTP-011 |
| BC-2.06.012 | Well-formed HTTP request produces zero findings | P0 | BC-HTTP-012 |
| BC-2.06.013 | Non-HTTP bytes increment parse_errors but do not emit Token-error findings | P0 | BC-HTTP-013 |
| BC-2.06.014 | Too many headers (>96) emits Anomaly/Inconclusive/Medium finding mapped to T1499.002 | P0 | BC-HTTP-014 |
| BC-2.06.015 | After 3 consecutive parse errors a direction is poisoned; subsequent bytes skipped | P0 | BC-HTTP-015 |
| BC-2.06.016 | Single parse error does not poison; next valid request parses normally | P0 | BC-HTTP-016 |
| BC-2.06.017 | Poisoning is per-direction: poisoned request does not affect response | P0 | BC-HTTP-017 |
| BC-2.06.018 | non_http_flows counts a flow once even if both directions get poisoned | P1 | BC-HTTP-018 |
| BC-2.06.019 | on_flow_close removes per-flow state; reopening same FlowKey starts fresh | P0 | BC-HTTP-019 |
| BC-2.06.020 | HTTP body bytes after header completion do not inflate parse_errors | P1 | BC-HTTP-020 |
| BC-2.06.021 | Cross-flow isolation: parse errors and poisoning in one flow do not leak | P0 | BC-HTTP-021 |
| BC-2.06.022 | Per-direction header buffer capped at MAX_HEADER_BUF (65536 bytes) | P1 | BC-HTTP-022 |
| BC-2.06.023 | summarize emits AnalysisSummary with HTTP stats detail map | P1 | BC-HTTP-023 |
| BC-2.06.024 | Per-map cardinality cap: new keys dropped past MAX_MAP_ENTRIES (50000) | P2 | BC-HTTP-024 |
| BC-2.06.025 | uris list capped at MAX_URIS=10000; further URIs silently dropped | P2 | BC-HTTP-025 |
| BC-2.06.026 | Header value extraction uses from_utf8_lossy.trim(); raw bytes preserved per ADR 0003 | P0 | BC-HTTP-026 |

> Full contracts: `behavioral-contracts/ss-06/BC-2.06.001.md` through `BC-2.06.026.md`

### 2.7 TLS Traffic Analysis (CAP-07)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.07.001 | Parse complete TLS ClientHello: version, ciphers, extensions, SNI, JA3 | P0 | BC-TLS-001 |
| BC-2.07.002 | Parse complete TLS ServerHello: JA3S fingerprint computed | P0 | BC-TLS-002 |
| BC-2.07.003 | After both hellos seen, subsequent records silently skipped | P0 | BC-TLS-003 |
| BC-2.07.004 | TLS record payload > MAX_RECORD_PAYLOAD (18432) increments parse_errors and truncated_records | P0 | BC-TLS-004 |
| BC-2.07.005 | Per-direction buffer capped at MAX_BUF=65536 bytes | P1 | BC-TLS-005 |
| BC-2.07.006 | JA3 computation filters GREASE values per RFC 8701 | P0 | BC-TLS-006 |
| BC-2.07.007 | JA3 string format: version,ciphers,extensions,curves,pointfmts hyphen-joined; MD5 hex | P0 | BC-TLS-007 |
| BC-2.07.008 | JA3S string format: version,cipher,extensions hyphen-joined; MD5 hex | P0 | BC-TLS-008 |
| BC-2.07.009 | Weak client cipher (NULL/ANON/EXPORT in ClientHello) emits Anomaly/Likely/High finding | P0 | BC-TLS-009 |
| BC-2.07.010 | Weak server cipher selected (NULL/ANON/EXPORT/RC4) emits Anomaly/Likely/Medium finding | P0 | BC-TLS-010 |
| BC-2.07.011 | Deprecated client protocol (<=SSLv3) emits Anomaly/Likely/High finding citing RFC 7568 | P0 | BC-TLS-011 |
| BC-2.07.012 | Deprecated server protocol (<=SSLv3) emits Anomaly/Likely/High finding | P0 | BC-TLS-012 |
| BC-2.07.013 | Clean ASCII SNI without C0/DEL bytes produces no SNI-related finding | P0 | BC-TLS-013 |
| BC-2.07.014 | SNI containing C0/DEL byte emits Anomaly/Inconclusive/Low finding mapped to T1027 | P0 | BC-TLS-014 |
| BC-2.07.015 | Multiple control bytes in one SNI produce exactly ONE finding | P0 | BC-TLS-015 |
| BC-2.07.016 | C0 boundary: 0x1F trips the finding; 0x20 (space) does not | P0 | BC-TLS-016 |
| BC-2.07.017 | Non-ASCII but valid UTF-8 SNI emits Anomaly/Inconclusive/Low finding mapped to T1027 | P0 | BC-TLS-017 |
| BC-2.07.018 | Punycode A-label (xn--...) is pure ASCII and emits no SNI finding | P1 | BC-TLS-018 |
| BC-2.07.019 | Non-UTF-8 SNI bytes emit Anomaly/Inconclusive/Low finding mapped to T1027; count key tagged | P0 | BC-TLS-019 |
| BC-2.07.020 | Non-UTF-8 SNI summary preserves raw bytes (no Debug-format escaping per ADR 0003) | P0 | BC-TLS-020 |
| BC-2.07.021 | Non-ASCII UTF-8 SNI summary preserves raw bytes per ADR 0003 | P0 | BC-TLS-021 |
| BC-2.07.022 | SNI extension with empty ServerNameList: no count, no finding, handshake still counted | P1 | BC-TLS-022 |
| BC-2.07.023 | SNI with empty hostname bytes counts under "" key; no non-UTF-8 finding | P2 | BC-TLS-023 |
| BC-2.07.024 | Only FIRST ServerName entry in multi-name SNI list is processed | P1 | BC-TLS-024 |
| BC-2.07.025 | Non-zero NameType entries are passed through as hostnames (current tls-parser behavior) | P2 | BC-TLS-025 |
| BC-2.07.026 | Trailing bytes in ServerNameList tolerated; first hostname still extracted | P2 | BC-TLS-026 |
| BC-2.07.027 | Large SNI (16 KB) under MAX_RECORD_PAYLOAD parses successfully | P1 | BC-TLS-027 |
| BC-2.07.028 | sni_counts cap at MAX_MAP_ENTRIES silently drops keys; SNI anomaly finding still fires | P0 | BC-TLS-028 |
| BC-2.07.029 | Bad TLS record body increments parse_errors and does not panic | P0 | BC-TLS-029 |
| BC-2.07.030 | Normal handshake (strong cipher) produces zero findings | P0 | BC-TLS-030 |
| BC-2.07.031 | summarize emits AnalysisSummary with TLS stats detail map | P1 | BC-TLS-031 |
| BC-2.07.032 | TLS 1.3 ClientHello legacy_version recorded as 0x0303 per JA3 spec | P1 | BC-TLS-032 |
| BC-2.07.033 | TLS analyzer ignores non-handshake records (record_type != 0x16) | P1 | BC-TLS-033 |
| BC-2.07.034 | After both hellos seen for flow, on_data short-circuits without buffering | P0 | BC-TLS-034 |
| BC-2.07.035 | on_flow_close drops per-flow TlsFlowState | P1 | BC-TLS-035 |
| BC-2.07.036 | Unknown cipher IDs render as hex 0xNNNN lowercase | P2 | BC-TLS-036 |
| BC-2.07.037 | SNI with both non-ASCII and C0 control bytes fires arm 3 (NonAsciiUtf8), not arm 2 | P0 | BC-TLS-037 |

> Full contracts: `behavioral-contracts/ss-07/BC-2.07.001.md` through `BC-2.07.037.md`

### 2.8 DNS Traffic Analysis (CAP-08)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.08.001 | DnsAnalyzer matches packets where src or dst port == 53 (TCP or UDP) | P0 | BC-DNS-001 |
| BC-2.08.002 | DNS QR-bit dispatch: response_count++ if bit set; query_count++ otherwise; returns empty findings | P0 | BC-DNS-002 |
| BC-2.08.003 | summarize emits AnalysisSummary with dns_queries and dns_responses counts | P1 | BC-DNS-003 |
| BC-2.08.004 | DnsAnalyzer NEVER emits findings (statistics-only by design) | P0 | BC-DNS-004 |

> Full contracts: `behavioral-contracts/ss-08/BC-2.08.001.md` through `BC-2.08.004.md`

### 2.9 Forensic Finding Emission (CAP-09)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.09.001 | Finding is constructed with required and optional fields as specified | P0 | BC-FND-001 |
| BC-2.09.002 | Finding Display renders [Category] VERDICT (CONFIDENCE) -- summary (raw text) | P1 | BC-FND-002 |
| BC-2.09.003 | Verdict Display: Likely/Unlikely/Inconclusive render as uppercase tokens | P1 | BC-FND-003 |
| BC-2.09.004 | Confidence Display: High/Medium/Low render as uppercase tokens | P1 | BC-FND-004 |
| BC-2.09.005 | Finding.summary and evidence store RAW post-from_utf8_lossy bytes per ADR 0003 | P0 | BC-FND-005 |
| BC-2.09.006 | Finding JSON serialization: empty Vec fields omitted (skip_serializing_if Vec::is_empty); mitre_techniques serialized as array | P0 | BC-FND-006 |
| BC-2.09.007 | Finding.timestamp Carries Capture-Relative Pcap Timestamp from on_data Call Site | P1 | BC-FND-007 |

> Full contracts: `behavioral-contracts/ss-09/BC-2.09.001.md` through `BC-2.09.007.md` (BC-2.09.007 added Feature Mode F2 issue #100)
>
> BC-2.09.007 (F2) wired timestamp from the pcap record header at all applicable emission sites
> (STORY-097/098/099); domain-debt O-01 CLOSED. The segment-limit summary finding (BC-2.04.054)
> retains timestamp:None by design as the sole exception.

### 2.10 MITRE ATT&CK Mapping (CAP-10)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.10.001 | MitreTactic Display renders Enterprise tactics with canonical spacing | P0 | BC-MIT-001 |
| BC-2.10.002 | ICS tactics render unprefixed (no ICS: prefix) | P1 | BC-MIT-002 |
| BC-2.10.003 | all_tactics_in_report_order returns kill-chain order first then ICS-unique | P0 | BC-MIT-003 |
| BC-2.10.004 | all_tactics_in_report_order contains every variant exactly once (20 total) | P0 | BC-MIT-004 |
| BC-2.10.005 | technique_name returns Some for every seeded ID (28 Total) | P0 | BC-MIT-005 |
| BC-2.10.006 | technique_name returns None for unknown IDs | P0 | BC-MIT-006 |
| BC-2.10.007 | technique_tactic returns correct tactic for every seeded ID | P0 | BC-MIT-007 |
| BC-2.10.008 | All technique IDs currently emitted by analyzers resolve in lookup | P0 | BC-MIT-008 |
| BC-2.10.009 | MitreTactic is #[non_exhaustive] (adding variants is non-breaking) | P2 | BC-MIT-009 |

> Full contracts: `behavioral-contracts/ss-10/BC-2.10.001.md` through `BC-2.10.009.md`
>
> Domain debt O-04 (revised v1.36 / F2 EtherNet/IP): 28 techniques seeded (12 Enterprise + 16 ICS); 20 emitted
> (7 Enterprise + 13 ICS). Catalogued-but-never-emitted (8): T1040, T1071, T1071.001, T1071.004,
> T1573 (Enterprise); T1692.002 (ICS — IcsImpairProcessControl; replaces revoked T0856 per ATT&CK-ICS v19 remap),
> T0885 (ICS — CommandAndControl), T1693.001 (ICS — IcsInhibitResponseFunction; staged firmware detection, seeded-not-emitted v0.11.0).
> T0846 NOW emitted by EtherNet/IP analyzer (BC-2.17.010).
> T1692.001, T0836, T0814, T0806, T0835, T0831, T0888 are emitted by the Modbus analyzer.
> T1691.001, T0827 are emitted by the DNP3 analyzer (Feature #8).
> T0830, T1557.002 are emitted by the ARP analyzer (Feature #9) — added in v1.9.
> T0858, T0816, T0836, T0846, T0888, T0814 are emitted by the EtherNet/IP analyzer (Feature #316, v0.11.0).
> Arithmetic: SEEDED=28, EMITTED=20, CATALOGUE-ONLY=28−20=8.
> BC-2.10.005 documents all 28 seeded IDs; BC-2.10.008 documents 20 emitted IDs. (BC-2.10.005/BC-2.10.008 version-bump pending.)

### 2.11 Reporting and Output (CAP-11)

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.11.001 | JsonReporter renders JSON object with summary, findings, analyzers keys | P0 | BC-RPT-001 |
| BC-2.11.002 | JsonReporter includes skipped_packets in summary (zero when unset) | P1 | BC-RPT-002 |
| BC-2.11.003 | JsonReporter escapes C0 control bytes per RFC 8259 via serde | P0 | BC-RPT-003 |
| BC-2.11.004 | JsonReporter preserves non-ASCII Unicode in readable form (no unnecessary \uNNNN) | P1 | BC-RPT-004 |
| BC-2.11.005 | JsonReporter passes C1 codepoints through as raw UTF-8 (serde_json does not escape them) | P1 | BC-RPT-005 |
| BC-2.11.006 | TerminalReporter shows Skipped: N packets only when N > 0 | P1 | BC-RPT-006 |
| BC-2.11.007 | TerminalReporter escapes C0+DEL+C1+backslash in finding summary and evidence | P0 | BC-RPT-007 |
| BC-2.11.008 | TerminalReporter escape preserves printable ASCII, Cyrillic, emoji, mixed Unicode | P0 | BC-RPT-008 |
| BC-2.11.009 | TerminalReporter escapes C1 codepoints U+0080-U+009F; U+00A0 is preserved | P0 | BC-RPT-009 |
| BC-2.11.010 | TerminalReporter escapes both Finding.summary AND each evidence line | P0 | BC-RPT-010 |
| BC-2.11.011 | TerminalReporter escapes analyzer-summary detail values (closes C1 gap) | P0 | BC-RPT-011 |
| BC-2.11.012 | TerminalReporter end-to-end: C1 CSI in path-traversal finding is escaped | P0 | BC-RPT-012 |
| BC-2.11.013 | MITRE grouping emits tactic headers in all_tactics_in_report_order; Uncategorized last | P0 | BC-RPT-013 |
| BC-2.11.014 | Within tactic bucket findings sort by verdict then confidence then emission order | P1 | BC-RPT-014 |
| BC-2.11.015 | No-technique or unknown-ID findings land in Uncategorized; unknown IDs get (unknown) label | P0 | BC-RPT-015 |
| BC-2.11.016 | MITRE grouping expands per-finding line with em-dash and technique name for known IDs | P1 | BC-RPT-016 |
| BC-2.11.017 | Default (flag-off) rendering emits MITRE: <id(s)> only with no em-dash; multi-ID rendered "MITRE: T1692.001, T0836" | P1 | BC-RPT-017 |
| BC-2.11.018 | TerminalReporter colorization: Likely/High=red bold, Likely/other=yellow, Inconclusive=cyan, Unlikely=dimmed | P2 | BC-RPT-018 |
| BC-2.11.019 | TerminalReporter renders sections in order: header, PROTOCOLS, SERVICES, FINDINGS, ANALYZER summaries | P1 | BC-RPT-019 |
| BC-2.11.020 | CsvReporter Emits Exactly Nine Columns in Fixed Header Order | P0 | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |
| BC-2.11.021 | CsvReporter Neutralizes CSV-Injection Trigger Characters with a Leading Single Quote | P0 | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |
| BC-2.11.022 | CsvReporter Joins Evidence Vec Elements with "; " into a Single Cell | P1 | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |
| BC-2.11.023 | CsvReporter Implements Reporter Trait and Emits One Row per Finding; Summary and AnalysisSummary Are Ignored | P0 | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |
| BC-2.11.024 | CsvReporter Encodes Optional Fields as Empty Strings and mitre_techniques as Semicolon-Joined String | P1 | BC-RPT (brownfield extraction, adversarial-review pass-4 finding H-1) |
| BC-2.11.025 | Flat-Mode Collapse Groups Findings by (category, verdict, confidence, summary) Key; First-Occurrence Order; Deterministic | P0 | issue-#259 greenfield (v0.8.0) |
| BC-2.11.026 | Collapsed Group of N≥2 Renders Header with (xN) Suffix; Singleton (N=1) Renders Without Suffix | P0 | issue-#259 greenfield (v0.8.0) |
| BC-2.11.027 | Collapsed Group Retains at Most K=3 Representative Evidence Lines; Remainder Elided from Terminal Display | P1 | issue-#259 greenfield (v0.8.0) |
| BC-2.11.028 | --no-collapse Opt-Out Flag Disables Terminal Collapse and Restores One-Line-Per-Finding Rendering; JSON/CSV Unaffected | P0 | issue-#259 greenfield (v0.8.0) |
| BC-2.11.029 | Collapse is Display-Layer Only; JSON/CSV Reporters Receive Unmodified findings Slice; Non-Repeated Findings Individually Visible in All Outputs | P0 | issue-#259 greenfield (v0.8.0) |
| BC-2.11.035 | Per-Finding `mitre_attack` Array Enriches JSON Output with Resolved Technique Objects; Order-Preserving; Unknown IDs Emit Partial Objects; Empty Vec Omits Field | P1 | issue-#64 greenfield (v0.11.0) |

> Full contracts: `behavioral-contracts/ss-11/BC-2.11.001.md` through `BC-2.11.029.md`, `BC-2.11.030.md` through `BC-2.11.034.md` (STORY-119 grouped-collapse, v0.9.0), `BC-2.11.035.md` (issue #64 mitre_attack enrichment, v0.11.0).
> (BC-2.11.020–024 added adversarial-review pass-4: CsvReporter coverage gap H-1;
> BC-2.11.025–029 added Feature Mode F2 issue #259: terminal finding collapse, v0.8.0;
> BC-2.11.030–034 added Feature Mode F2 STORY-119: grouped-collapse, v0.9.0;
> BC-2.11.035 added Feature Mode F2 issue #64: mitre_attack JSON enrichment, v0.11.0)

### 2.12 CLI and Entry Point (CAP-12 / CLI Orchestration)

> CLI BCs are cross-cutting: they describe the entry point (C-1..C-3) that wires all capabilities
> together. Numbered under ss-12 for organizational clarity.

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.12.001 | analyze subcommand parses positional targets and all analysis flags | P0 | BC-CLI-001 |
| BC-2.12.002 | summary subcommand parses positional targets and --hosts flag | P1 | BC-CLI-002 |
| BC-2.12.003 | Global flag --no-color is parsed and stored | P1 | BC-CLI-003 |
| BC-2.12.004 | Global flag --output-format json parses to Some(OutputFormat::Json); default is None | P0 | BC-CLI-004 |
| BC-2.12.005 | Reassembly CLI flags: --reassemble/--no-reassemble, depth, memcap, and five anomaly-threshold flags | P0 | BC-CLI-005 |
| BC-2.12.006 | Multiple positional targets accepted in analyze | P1 | BC-CLI-006 |
| BC-2.12.007 | --reassemble and --no-reassemble are mutually exclusive (clap conflicts_with) | P0 | BC-CLI-007 |
| BC-2.12.008 | --all enables dns/http/tls together (boolean OR semantics) | P1 | BC-CLI-008 |
| BC-2.12.009 | needs_reassembly = (--reassemble OR --http OR --tls); --no-reassemble forces off with warning | P0 | BC-CLI-009 |
| BC-2.12.010 | NO_COLOR env var disables color even without --no-color flag | P2 | BC-CLI-010 |
| BC-2.12.011 | Directory target expands to capture files detected by magic bytes (content detection), not extension | P1 | BC-CLI-011 |
| BC-2.12.012 | Non-existent target yields bail! with Target not found message | P1 | BC-CLI-012 |
| BC-2.12.013 | Per-target progress bar on stderr using indicatif | P2 | BC-CLI-013 |
| BC-2.12.014 | Per-target decode errors counted into skipped_packets; only first error printed to stderr | P1 | BC-CLI-014 |
| BC-2.12.015 | dispatcher.unclassified_flows() injected into reassembly AnalysisSummary detail | P1 | BC-CLI-015 |
| BC-2.12.016 | --output-format json picks JsonReporter; --output-format csv picks CsvReporter; default terminal | P0 | BC-CLI-016 |
| BC-2.12.017 | Output routed: file path if --json <FILE> or --csv <FILE> given; stdout otherwise | P0 | BC-CLI-017 |
| BC-2.12.018 | Summary::ingest increments total_packets, total_bytes, hosts, protocol counters | P0 | BC-SUM-001 |
| BC-2.12.019 | Summary::ingest derives service name from app_protocol_hint and increments service counter | P1 | BC-SUM-002 |
| BC-2.12.020 | Summary::unique_hosts returns sorted deduplicated Vec<IpAddr> | P1 | BC-SUM-003 |
| BC-2.12.021 | Summary serializes with total_packets, total_bytes, skipped_packets fields | P1 | BC-SUM-004 |

> Full contracts: `behavioral-contracts/ss-12/BC-2.12.001.md` through `BC-2.12.021.md`

### 2.13 Absent / Unwired Feature Contracts (Documented Current Behavior)

> These BCs document flags or behaviors that do not exist in the current codebase (removed by
> PR #74). clap rejects all four as unknown arguments; there is no runtime behavior for any of
> them. They are HIGH-confidence absent contracts verified against src/cli.rs.

| BC ID | Title | Priority | Origin BC |
|-------|-------|----------|-----------|
| BC-2.13.001 | --threats flag does not exist; clap rejects it as unknown argument | P0 (absent) | BC-ABS-001 |
| BC-2.13.002 | --beacon flag does not exist; no C2 beacon analyzer exists | P0 (absent) | BC-ABS-002 |
| BC-2.13.003 | --filter <BPF> flag does not exist; no BPF filter applied | P0 (absent) | BC-ABS-003 |
| BC-2.13.004 | --verbose flag does not exist; no verbose logging mode | P2 (absent) | BC-ABS-010 |

> Full contracts: `behavioral-contracts/ss-13/BC-2.13.001.md` through `BC-2.13.004.md`

### 2.14 Modbus/ICS Analysis (CAP-14) [Feature #7 — ADR-005, ADR-006]

> **Release target: v0.4.0 (additive — no schema break).**
> All SS-14 BCs (BC-2.14.001..025) ship in v0.4.0. The `mitre_techniques: Vec<String>` type
> they depend on ships in **v0.3.0** (schema migration of existing analyzers). Modbus is built
> on top of the stable v0.3.0 contract and is purely additive at v0.4.0. See RELEASE SEQUENCING
> box in Section 2 for the full v0.3.0/v0.4.0 split rationale (f2-bundle-vs-split.md).

> **Feature Mode F2 addition (v1.1) + v2 revision (v1.2).** 25 BCs covering the Modbus TCP
> protocol analyzer (SS-14, C-22 ModbusAnalyzer). Analyzer detects 7 MITRE ATT&CK for ICS
> techniques: T1692.001, T0836, T0814, T0806, T0835, T0831, T0888 (Remote System Information
> Discovery — recon FCs 0x11/0x2B/0x0E; **T0888 replaces prior T0846 per Decision 12**).
> Matrix discriminator: ICS technique IDs use T0xxx namespace (second char '0'),
> Enterprise use T1xxx-T9xxx. See ADR-005 for binary ICS protocol integration rationale;
> ADR-006 for multi-technique Finding attribution.
>
> **v2 co-emission model (Decision 13, ADR-006):** One finding per write-class PDU carrying
> ALL applicable technique tags (`mitre_techniques: Vec<String>`). No tag-suppression.
> Write FCs 0x06/0x10/0x16/0x17 → `["T1692.001","T0836"]`; coil FCs 0x05/0x0F → `["T1692.001","T0835"]`;
> burst/sustained rate findings → `["T0806","T1692.001"]`; T0831 co-tagged inline on per-PDU write finding → `["T1692.001","T0836","T0831"]` (no separate T0831 Finding object).
>
> **v2 dual-window burst detection (Decision 11):** Two independent CLI-configurable windows:
> `--modbus-write-burst-threshold` (default 20, 1-second burst) and
> `--modbus-write-sustained-threshold` (default 10, >=2-second sustained rolling window).
> Old `--modbus-write-threshold` flag is **REMOVED**.
>
> **CLI flags added:** `--modbus` (enable analyzer), `--modbus-write-burst-threshold N`
> (default 20; zero rejected), `--modbus-write-sustained-threshold N` (default 10; zero
> rejected). `--all` includes Modbus. Modbus analysis requires stream reassembly
> (`--no-reassemble` disables it with a warning). Dispatcher Rule 5: port-502 flows →
> `DispatchTarget::Modbus`, checked AFTER content rules (Rules 1-2) and TLS/HTTP port
> fallbacks (Rules 3-4).
>
> **Formal verification:** VP-022 covers `parse_mbap_header` (None for < 8 bytes),
> `classify_fc` (total over all 256 values), and the exception biconditional (fc >= 0x80).
> VP-004 extended: `classify_oracle` must mirror Rule 5 for port 502.

#### 2.14.A MBAP Parse and Validity Gate

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.001 | MBAP header accepted for well-formed 8-byte-minimum ADU | P0 | feature-007-F2 |
| BC-2.14.002 | MBAP header rejected for ADU shorter than 8 bytes | P0 | feature-007-F2 |
| BC-2.14.003 | MBAP header rejected when Protocol ID is not 0x0000 | P0 | feature-007-F2 |
| BC-2.14.004 | MBAP header rejected when Length is outside [2, 254] | P0 | feature-007-F2 |

#### 2.14.B Function-Code Classification

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.005 | classify_fc is total over all 256 FC values — covers Read, Write, Diagnostic, Exception, and Unknown classes | P0 | feature-007-F2 |
| BC-2.14.006 | Exception response detection — FC high bit set identifies exception and recovers original FC | P0 | feature-007-F2 |
| BC-2.14.007 | Write-class FC classification — state-changing function codes identified as elevated-risk | P0 | feature-007-F2 |
| BC-2.14.008 | Diagnostic-class FC classification and sub-function dispatch (0x08 and 0x2B) | P1 | feature-007-F2 |

#### 2.14.C Transaction Correlation

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.009 | Request PDU (client-to-server) inserted into per-flow pending table keyed on (Transaction ID, Unit ID) | P0 | feature-007-F2 |
| BC-2.14.010 | Response PDU (server-to-client) matched against pending table; entry removed on FC echo match | P0 | feature-007-F2 |
| BC-2.14.011 | Exception response PDU attributed to originating request FC via pending table lookup | P0 | feature-007-F2 |
| BC-2.14.012 | Pending table bounded to MAX_PENDING_TRANSACTIONS=256; new requests dropped (not evicting) when full | P0 | feature-007-F2 |

#### 2.14.D Finding Emission: Write-Class Events

> **v2 co-emission model (ADR-006, Decision 13):** One finding per write-class PDU carrying
> ALL applicable technique tags. No tag-suppression. Holding-register FCs (0x06/0x10/0x16/0x17) →
> `["T1692.001","T0836"]`; coil FCs (0x05/0x0F) → `["T1692.001","T0835"]`; other write FCs →
> `["T1692.001"]`. Volume control via burst aggregation (BC-2.14.017), not tag-suppression.

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.013 | Write-class FC in request direction emits multi-tag finding carrying T1692.001 and applicable technique tags; one finding per write PDU | P0 | feature-007-F2 |
| BC-2.14.014 | Write FC 0x06/0x10/0x16/0x17 in request direction emits finding tagged ["T1692.001","T0836"]; single multi-tag finding per PDU | P0 | feature-007-F2 |
| BC-2.14.015 | Write FC to coil output only (0x05/0x0F) emits finding tagged ["T1692.001","T0835"]; single multi-tag finding per PDU | P0 | feature-007-F2 |

#### 2.14.E Finding Emission: Coordinated Write (T0831) and Dual-Window Write-Burst Detection (T0806/T1692.001)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.016 | Coordinated write sequence to holding registers within 5-second window co-tags the per-PDU finding with T0831 inline (`["T1692.001","T0836","T0831"]`); no separate T0831 Finding object | P0 | feature-007-F2 |
| BC-2.14.017 | Write-rate exceeding either burst threshold (>N in 1s) or sustained threshold (>M avg over >=2s) emits `["T0806","T1692.001"]` finding; each window fires at most once per overflow | P0 | feature-007-F2 |

#### 2.14.F Finding Emission: Diagnostic/DoS (T0814) and Exception Burst Anomaly

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.018 | Diagnostics FC 0x08 sub-function 0x0004 or 0x0001 emits T0814 (Denial of Service) finding; sub-func guard h.length >= 4 | P0 | feature-007-F2 |
| BC-2.14.019 | Exception response anomaly — burst of exception codes (> 5 in 10s) emits Anomaly finding for recon/scanning | P0 | feature-007-F2 |

#### 2.14.G Anomaly/Recon, Summary, Statistics, and Bounded Resource

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.020 | Reconnaissance FCs (0x11, 0x2B/0x0E) emit T0888 (Remote System Information Discovery) finding; 0x07 not a standalone finding; unusual unknown FCs emit generic Anomaly | P1 | feature-007-F2 |
| BC-2.14.021 | summarize() returns AnalysisSummary with SIX keys: pdu_count, write_count, exception_count, function_code_distribution, parse_errors, dropped_findings (always present) | P1 | feature-007-F2 |
| BC-2.14.022 | MAX_FINDINGS cap (10,000) and poison-skip behavior for ModbusAnalyzer | P0 | feature-007-F2 |

#### 2.14.H Dispatcher and CLI Integration

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.14.023 | --modbus CLI flag enables ModbusAnalyzer; --all includes Modbus; default-off; requires stream reassembly | P0 | feature-007-F2 |
| BC-2.14.024 | --modbus-write-burst-threshold (default 20) and --modbus-write-sustained-threshold (default 10) configure dual-window burst detection; old --modbus-write-threshold removed | P0 | feature-007-F2 |
| BC-2.14.025 | StreamDispatcher classifies port-502 flows to DispatchTarget::Modbus as Rule 5 (after content and TLS/HTTP port rules); routes on_data and on_flow_close to ModbusAnalyzer; VP-004 oracle must mirror this rule | P0 | feature-007-F2 |

> Full contracts: `behavioral-contracts/ss-14/BC-2.14.001.md` through `BC-2.14.025.md`


### 2.15 DNP3/ICS Analysis (CAP-15) [Feature #8 — ADR-007]

> **Release target: v0.6.0 (additive — no schema break).**
> All SS-15 BCs (BC-2.15.001..024) ship in v0.6.0. The `mitre_techniques: Vec<String>` type
> and multi-tag finding model established by v0.3.0 are reused without modification. DNP3 is
> purely additive at v0.6.0.

> **Feature Mode F2 addition (v1.5).** 24 BCs covering the DNP3 TCP protocol analyzer (SS-15,
> C-24 Dnp3Analyzer). Analyzer detects 5 MITRE ATT&CK for ICS techniques directly and 2 via
> correlation: T1692.001 (unauthorized control command — direct), T0814 (restart/DoS — direct),
> T0836 (write FC — direct), T1691.001 (inferred block-command, ICS sub-technique — per-flow
> inference), T0827 (derived loss-of-control — correlated across events).
>
> **New ICS tactic variant:** `IcsImpact` (Display "Impact (ICS)", TA0105) added to `MitreTactic`
> enum for T0827. `all_tactics_in_report_order` grows from 16 to 17 elements (element [16]).
> The "(ICS)" qualifier disambiguates from Enterprise `Impact` (TA0040, bare "Impact") per D-069
> adjudication (WCAG 2.4.6; mitre-impact-tactic-disambiguation.md). src/mitre.rs:91 = "Impact (ICS)"
> is correct; the prior spec assertion "Impact" (bare) was wrong. See BC-2.10.002 v1.5.
>
> **DNP3 frame model:** Link-layer header (10 bytes minimum: 8 header + 2 CRC). Validity gate:
> sync==0x0564 and LENGTH>=5. DEST/SOURCE addresses little-endian at offsets 4–7. Maximum
> frame size 292 bytes (BC-2.15.007). Carry buffer per-flow bounded to 292 bytes.
>
> **FC classification:** `classify_dnp3_fc` is total over all 256 values — Control class
> {0x03,0x04,0x05,0x06}, Restart class {0x0D,0x0E}, Write class {0x02}, Read class {0x01},
> Unknown otherwise. Transport FIR=1 gates application-layer FC extraction (BC-2.15.008).
>
> **Desync safety:** `is_non_dnp3` check — if no valid sync bytes in first 16 bytes, flow is
> silenced permanently (BC-2.15.009). Prevents false-positive finding spam on non-DNP3 flows.
>
> **Correlated findings (F2 novel):** T1691.001 (BC-2.15.014) requires a control request
> without response within a configurable window — per-flow request/response correlation.
> T0827 (BC-2.15.015) requires N restart/block events within a detection window — cross-event
> aggregation producing a single derived impact finding.
>
> **CLI flags added:** `--dnp3` (enable analyzer), `--dnp3-direct-operate-threshold N`
> (default 5; zero rejected). `--all` includes DNP3. DNP3 analysis requires stream reassembly
> (`--no-reassemble` disables it with a warning). Dispatcher Rule 6: port-20000 flows →
> `DispatchTarget::Dnp3`, checked AFTER content rules (Rules 1-2), TLS/HTTP port fallbacks
> (Rules 3-4), and Modbus Rule 5.
>
> **Formal verification:** VP-023 covers `parse_dnp3_dl_header` (None for < 10 bytes),
> `classify_dnp3_fc` (total over all 256 values), `is_valid_dnp3_frame_header` (biconditional),
> and `compute_dnp3_frame_len` (arithmetic correctness, result in [10,292]).

#### 2.15.A DL Header Parse and Validity Gate

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.001 | DNP3 DL header accepted for well-formed 10-byte-minimum frame | P0 | feature-008-F2 |
| BC-2.15.002 | DNP3 DL header rejected for frame shorter than 10 bytes (truncation safety) | P0 | feature-008-F2 |
| BC-2.15.003 | DEST/SOURCE addresses decoded little-endian from fixed offsets 4–7 | P0 | feature-008-F2 |
| BC-2.15.004 | Three-point validity gate returns true iff sync==0x0564 and LENGTH>=5 | P0 | feature-008-F2 |

#### 2.15.B Function-Code Classification

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.005 | classify_dnp3_fc is total over all 256 FC values (no gap, no panic) | P0 | feature-008-F2 |
| BC-2.15.006 | FC classification correctness — Control {0x03,0x04,0x05,0x06}, Restart {0x0D,0x0E}, Write {0x02}, Read {0x01} | P0 | feature-008-F2 |
| BC-2.15.007 | compute_dnp3_frame_len arithmetic correct; result in [10,292]; no overflow | P0 | feature-008-F2 |

#### 2.15.C Transport Layer and Desync Safety

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.008 | Transport FIR=1 first-fragment gates application-layer FC extraction | P0 | feature-008-F2 |
| BC-2.15.009 | is_non_dnp3 Desync-Safe Bail — Flow Silenced on Initial-Delivery No-Sync (One-Shot, First Delivery Only) | P0 | feature-008-F2 |

#### 2.15.D Finding Emission: Detection (Direct Techniques)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.010 | Unauthorized control command — Unexpected source (count=1) or Control-class FC exceeding threshold emits T1692.001 | P0 | feature-008-F2 |
| BC-2.15.011 | COLD_RESTART/WARM_RESTART observed — emits T0814 per-occurrence finding | P0 | feature-008-F2 |
| BC-2.15.012 | WRITE FC observed — emits T0836 Modify-Parameter finding per-occurrence | P0 | feature-008-F2 |
| BC-2.15.013 | Co-emission ordering — direct finding (T0814/T1692.001) precedes derived T0827 | P0 | feature-008-F2 |

#### 2.15.E Finding Emission: Inferred and Correlated (T1691.001 and T0827)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.014 | Inferred block-command — control request without response within window emits T1691.001 | P0 | feature-008-F2 |
| BC-2.15.015 | Derived loss-of-control — N restart/block events in window emits T0827 as correlated finding | P0 | feature-008-F2 |

#### 2.15.F Bounded Resource and CLI Integration

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.016 | Per-Flow State Bounds — Carry Buffer ≤292 B, master_addrs ≤64, pending_requests ≤256 | P0 | feature-008-F2 |
| BC-2.15.017 | --dnp3-direct-operate-threshold CLI flag controls control-command detection window | P0 | feature-008-F2 |

#### 2.15.G Anomaly Detection

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.018 | Broadcast destination anomaly — DEST in 0xFFFD/0xFFFE/0xFFFF emits anomaly finding | P1 | feature-008-F2 |
| BC-2.15.019 | Unsolicited response anomaly — UNS bit set or FC 0x82 from unexpected pattern | P1 | feature-008-F2 |

#### 2.15.H Summary, Dispatcher, and DoS Bound

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.020 | summarize() emits function-code distribution and control-operation counts | P1 | feature-008-F2 |
| BC-2.15.021 | Port-20000 flow dispatched to Dnp3Analyzer (DispatchTarget::Dnp3, Rule 6) | P0 | feature-008-F2 |
| BC-2.15.022 | MAX_FINDINGS DoS bound — finding cap prevents unbounded all_findings growth | P0 | feature-008-F2 |

#### 2.15.I Research Must-Add Detections (Post-Gate F2, issue #8)

> Added 2026-06-10 based on `dnp3-f2-scope-threshold-validation.md` scope validation.
> Both detections map to existing T0814 — no MITRE catalog change; counts remain 23/15/8.

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.15.023 | Unsolicited-response enable/disable abuse — FC 0x15/0x14 observed emits T0814 | P1 | feature-008-F2 |
| BC-2.15.024 | Malformed/structural DNP3 anomaly — malformed_in_window threshold emits T0814 | P1 | feature-008-F2 |

> Full contracts: `behavioral-contracts/ss-15/BC-2.15.001.md` through `BC-2.15.024.md`


### 2.16 ARP Security Analysis (CAP-16) [Feature #9 — ADR-008]

> **Release target: v0.7.0 (additive — existing schema unchanged).**
> All SS-16 BCs (BC-2.16.001..015) ship in v0.7.0. ARP analysis is purely additive; no
> existing analyzer, struct, or serialization key changes, except the `decode_packet` return
> type change (Result<ParsedPacket> → Result<DecodedFrame>) mandated by ADR-008 Decision 1,
> which is a BREAKING CHANGE targeted at STORY-111.

> **Feature Mode F2 addition (v1.9).** 15 BCs covering the link-layer ARP security analyzer
> (SS-16, C-23 ArpAnalyzer). Analyzer has 5 detection types (D1, D2, D3, D11, D12) and emits
> 2 MITRE techniques (T0830, T1557.002): T0830 (Adversary-in-the-Middle — spoof D1 and D12
> paths), T1557.002 (ARP Cache Poisoning — spoof D1 and GARP-that-conflicts (BC-2.16.014)
> paths). Two new techniques enter the seeded catalog.

> **ARP frame model:** Standard Ethernet/IPv4 ARP (28-byte minimum payload): hardware type
> 0x0001 (Ethernet), protocol type 0x0800 (IPv4), hw_addr_size=6, proto_addr_size=4.
> Non-Ethernet/IPv4 ARP frames (different hw_type, proto_type, or address sizes) are rejected
> by `extract_arp_frame` → `None` → E-DEC-004 degraded skip + optional D11 finding.

> **Binding table:** `HashMap<[u8; 4], BindingEntry>` (production substrate; BTreeMap used only
> as Kani surrogate in VP-024 Sub-D scaled proof) bounded
> to MAX_ARP_BINDINGS=65,536 via LRU eviction (BC-2.16.006). The VP-024 Kani proof uses a
> scaled model (TEST_MAX_ARP_BINDINGS=8) to prove the cap invariant holds for all inputs.

> **Detection surface (5 detections):**
> - D1: ARP Spoof — IP→MAC rebind emits MEDIUM then HIGH finding (BC-2.16.004); GARP-that-
>   conflicts upgrades to MEDIUM + D1 MEDIUM (BC-2.16.014) or HIGH if threshold reached.
> - D3: ARP Storm — rate detection per source MAC, one-shot per 60s window (BC-2.16.008).
> - D11: Malformed ARP — non-Ethernet/IPv4 sizes emit LOW finding (BC-2.16.009).
> - D12: L2/L3 Mismatch — Ethernet outer src MAC ≠ ARP sender HW addr (BC-2.16.007).
> - D2: GARP (Gratuitous ARP): sender_ip == target_ip, LOW when no conflict, MEDIUM when conflict
>   (BC-2.16.003; escalation via BC-2.16.014).

> **CLI flags added:** `--arp` (enable analyzer, default off), `--arp-spoof-threshold N`
> (default 3 rebinds within 60s before HIGH; override via BC-2.16.012), `--arp-storm-rate N`
> (default 50 frames/sec; override via BC-2.16.013). `--all` does NOT include `--arp` by
> default (ARP is opt-in; cross-layer integration note: ARP frames are link-layer only, not
> IP-layer, so they bypass the stream dispatcher). ARP analysis does NOT require stream
> reassembly.

> **Decode-vs-analysis separation:** `decode_packet` always produces `DecodedFrame::Arp` for
> valid Ethernet/IPv4 ARP frames — regardless of whether `--arp` is active. The ArpAnalyzer
> only processes the frame when `--arp` is set (BC-2.16.015). This preserves the existing
> skipped-packet counting behavior when `--arp` is absent.

> **Formal verification:** VP-024 covers four sub-properties:
> - Sub-property A: `extract_arp_frame` parse safety — no-panic, field correctness (Request
>   and Reply extraction); `None` for non-Ethernet/IPv4 inputs. Anchors BC-2.16.001/BC-2.16.002.
> - Sub-property B: GARP detection totality — `is_gratuitous_arp` biconditional
>   (`sender_ip == target_ip`), opcode-agnostic over all 65,536 u16 operation values.
>   Anchors BC-2.16.003. Kani: symbolic ArpFrame.
> - Sub-property C: Binding-table last-write-wins determinism — proptest over arbitrary
>   Vec<ArpFrame> sequences; `bindings[ip].mac` equals MAC from last frame; no duplicate
>   keys. Anchors BC-2.16.005 (BC-2.16.004 indirectly supported only; not in VP-024's
>   formal Verified-BCs scope — see VP-INDEX footnote).
> - Sub-property D: MAX_ARP_BINDINGS cap — `bindings.len()` never exceeds cap; LRU evicts
>   exactly one entry on overflow. Scaled Kani proof (TEST_MAX_ARP_BINDINGS=8). Anchors
>   BC-2.16.006.

#### 2.16.A ARP Frame Extraction (Group A)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.16.001 | ARP Request frame correctly parsed from ArpPacketSlice | P0 | feature-009-F2 |
| BC-2.16.002 | ARP Reply frame correctly parsed from ArpPacketSlice | P0 | feature-009-F2 |

#### 2.16.B Binding Table and Core Detection (Group B)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.16.003 | Gratuitous ARP detection — sender_ip == target_ip classified as GARP | P0 | feature-009-F2 |
| BC-2.16.004 | ARP Spoof detection — IP→MAC rebind emits MEDIUM then HIGH finding | P0 | feature-009-F2 |
| BC-2.16.005 | Binding-table update — last-seen MAC wins for a given IP | P0 | feature-009-F2 |
| BC-2.16.006 | Binding-table cap — table never exceeds MAX_ARP_BINDINGS via LRU eviction | P0 | feature-009-F2 |

#### 2.16.C L2/L3 Mismatch Detection (Group C)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.16.007 | D12 L2/L3 sender mismatch — Ethernet src MAC != ARP sender HW addr | P0 | feature-009-F2 |

#### 2.16.D ARP Storm Rate Detection (Group D)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.16.008 | D3 ARP storm rate detection — source MAC exceeds ARP_STORM_RATE_DEFAULT frames/sec | P1 | feature-009-F2 |

#### 2.16.E Malformed ARP Detection (Group E)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.16.009 | D11 malformed ARP — non-Ethernet/IPv4 HW/proto address sizes emit LOW finding | P1 | feature-009-F2 |

#### 2.16.F Summary Statistics (Group F)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.16.010 | ArpAnalyzer::summarize() returns AnalysisSummary with required keys (11 Keys) | P1 | feature-009-F2 |

#### 2.16.G CLI Integration (Group G)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.16.011 | --arp CLI flag gates ARP security analysis | P0 | feature-009-F2 |
| BC-2.16.012 | --arp-spoof-threshold overrides SPOOF_REBIND_ESCALATION_DEFAULT | P1 | feature-009-F2 |
| BC-2.16.013 | --arp-storm-rate overrides ARP_STORM_RATE_DEFAULT | P1 | feature-009-F2 |

#### 2.16.H GARP Escalation (Group H)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.16.014 | GARP-that-conflicts upgrades to MEDIUM and triggers D1 spoof finding | P0 | feature-009-F2 |

#### 2.16.I Decode-vs-Analysis Separation (Group I)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.16.015 | Decode-vs-analysis separation — DecodedFrame::Arp always produced; analysis gated on --arp | P0 | feature-009-F2 |

> Full contracts: `behavioral-contracts/ss-16/BC-2.16.001.md` through `BC-2.16.015.md`


### 2.17 EtherNet/IP + CIP Analysis (CAP-17) [Feature — ADR-010, issue #316]

> **Release target: v0.11.0 (additive — port-44818 TCP explicit messaging MVP).**
> All SS-17 BCs (BC-2.17.001..025) ship in v0.11.0. EtherNet/IP analysis is purely additive;
> no existing analyzer, struct, or serialization key changes except: (1) new
> `DispatchTarget::Enip` variant in the stream dispatcher, (2) new `MitreTactic::IcsExecution`
> variant in src/mitre.rs, (3) two new `technique_info()` arms (T0858, T0816). UDP/2222
> implicit I/O is deferred to a future release.

> **Feature Mode F2 addition (v1.36).** 25 BCs covering the EtherNet/IP + CIP TCP analyzer
> (SS-17, C-25 EnipAnalyzer). Analyzer has 6 detection paths and emits 6 MITRE techniques:
> T0858 (CIP Stop — Change Operating Mode), T0816 (CIP Reset — Device Restart/Shutdown),
> T0836 (CIP write-class burst — Modify Parameter), T0846 (ListIdentity — Remote System
> Discovery), T0888 (Identity Object read / error burst — Remote System Information Discovery),
> T0814 (malformed ENIP threshold — Denial of Service). T1693.001 is staged (GetAndClear
> firmware service) but not emitted in v0.11.0.

> **Protocol stack:** ENIP encapsulation (24-byte fixed header, little-endian) → CPF item layer
> (little-endian item_count + variable-length items) → CIP service header (service_code u8 +
> request_path). Both ENIP encapsulation and CPF layers use little-endian byte order per ODVA.
> The carry-buffer frame-walk loop stashes partial frames into
> `EnipFlowState.carry` (bounded to `MAX_ENIP_CARRY_BYTES = 600`).

> **Detection surface (6 detections + 1 lifecycle anomaly):**
> - ListIdentity (0x0063): T0846 Remote System Discovery per-occurrence (BC-2.17.010).
> - CIP Stop (0x07): T0858 Change Operating Mode per-occurrence, Likely/High (BC-2.17.011).
> - CIP write-class burst (SetAttribute*/etc.): T0836 Modify Parameter one-shot/window (BC-2.17.012). [OA-001: threshold default 50/1s — RESOLVED = 50 (MEDIUM-confidence, pending human confirm at F2 gate)]
> - CIP Reset (0x05): T0816 Device Restart/Shutdown per-occurrence, Likely/High (BC-2.17.013).
> - CIP Identity Object read / error burst: T0888 Remote System Information Discovery (BC-2.17.014).
> - ForwardOpen (0x54/0x5B): connection-lifecycle anomaly, no MITRE technique, Possible/Low (BC-2.17.015).
> - Malformed ENIP threshold (3/300s window): T0814 Denial of Service one-shot/window (BC-2.17.018).

> **CLI flags added:** `--enip` (enable analyzer, default off), `--enip-write-burst-threshold N`
> (default 50 writes/1s; overrides T0836 detection threshold via BC-2.17.023). `--all` INCLUDES
> `--enip` (same expansion as `--modbus`, `--dnp3`; EtherNet/IP is default-off standalone but
> enabled by `--all`; port-44818 TCP only).

> **Formal verification:** VP-032 covers four Kani sub-properties:
> - Sub-A: `parse_enip_header` never panics; returns None for len<24; Some with correct field layout.
>   Anchors BC-2.17.001, BC-2.17.002.
> - Sub-B: `classify_enip_command` total over all 65,536 u16 values; Unknown arm reachable.
>   Anchors BC-2.17.004.
> - Sub-C: `is_valid_enip_frame` biconditional iff command in known-command set.
>   Anchors BC-2.17.003.
> - Sub-D: `classify_cip_service` total over all 256 u8 values; response-bit mask (0x80) correct.
>   Anchors BC-2.17.007.

> **Open item OA-001 [RESOLVED = 50, MEDIUM-confidence]:** The --enip-write-burst-threshold
> default has been updated to 50 writes/1s (was 20) based on research calibration for high-write
> CIP environments (servo drives, motion control). This value carries MEDIUM confidence; human
> confirmation required at F2 gate before shipping.

#### 2.17.A ENIP Header Parse Safety (Group A)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.001 | parse_enip_header Returns None for Input Shorter Than 24 Bytes | P0 | feature-enip-v0.11.0 |
| BC-2.17.002 | EnipHeader Field Contracts — Fixed Little-Endian Offsets for 24-Byte Input | P0 | feature-enip-v0.11.0 |

#### 2.17.B ENIP Validity Gate and Command Classification (Group B)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.003 | is_valid_enip_frame Validity Gate Biconditional — Known-Command Set | P0 | feature-enip-v0.11.0 |
| BC-2.17.004 | classify_enip_command Total Classification with Unknown Arm Over All u16 Values | P0 | feature-enip-v0.11.0 |

#### 2.17.C CPF Item Walk and CIP Header Extraction (Group C)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.005 | CPF Item-Layer Walk — Bounded Little-Endian Item Iteration | P0 | feature-enip-v0.11.0 |
| BC-2.17.006 | parse_cip_header Extracts Service Code and Request Path from Item Data | P0 | feature-enip-v0.11.0 |

#### 2.17.D CIP Service Classification (Group D)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.007 | classify_cip_service Total Classification with Response-Bit Mask — 13 Named Request Services + Response + Unknown = 15 Variants | P0 | feature-enip-v0.11.0 |

#### 2.17.E CIP State Extraction (Group E)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.008 | CIP Error Response Detection — general_status Extraction from Unconnected (0x00B2) Response Frames | P1 | feature-enip-v0.11.0 |
| BC-2.17.009 | parse_cip_request_path Class and Instance Segment Extraction | P1 | feature-enip-v0.11.0 |

#### 2.17.F Detection — Finding Emission (Group F)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.010 | ListIdentity Command Observed Emits T0846 Network Enumeration Finding | P0 | feature-enip-v0.11.0 |
| BC-2.17.011 | CIP Stop Service Observed Emits T0858 Change Operating Mode Finding | P0 | feature-enip-v0.11.0 |
| BC-2.17.012 | CIP Write-Class Service Burst Exceeding Threshold Emits T0836 Modify Parameter Finding | P1 | feature-enip-v0.11.0 |
| BC-2.17.013 | CIP Reset Service Observed Emits T0816 Device Restart/Shutdown Finding | P0 | feature-enip-v0.11.0 |
| BC-2.17.014 | CIP Identity-Read to Identity Object or Error Burst Emits T0888 Remote System Information Discovery | P0 | feature-enip-v0.11.0 |
| BC-2.17.015 | ForwardOpen and ForwardClose Connection-Lifecycle Anomaly Detected with Empty MITRE Technique Set | P1 | feature-enip-v0.11.0 |

#### 2.17.G Bounded Resource — Carry Buffer (Group G)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.016 | Carry-Buffer Frame-Walk Loop — Partial Frame Stash and MAX_ENIP_CARRY_BYTES Cap | P0 | feature-enip-v0.11.0 |

#### 2.17.H Flow Lifecycle (Group H)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.017 | on_flow_close Removes Flow State and Updates Aggregate Counters | P1 | feature-enip-v0.11.0 |

#### 2.17.I Malformed Detection (Group I)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.018 | Malformed ENIP Frame Threshold Emits T0814 Structural Anomaly Finding | P1 | feature-enip-v0.11.0 |

#### 2.17.J Dispatcher Integration (Group J)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.019 | StreamDispatcher Rule 7 — Port 44818 TCP Classified as DispatchTarget::Enip | P0 | feature-enip-v0.11.0 |

#### 2.17.K CLI Integration and Summary (Group K)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.020 | CLI --enip Flag Enables Analyzer; --enip-write-burst-threshold Configures Write Detection | P0 | feature-enip-v0.11.0 |
| BC-2.17.021 | summarize() Emits ENIP Command Distribution and Aggregate Statistics | P1 | feature-enip-v0.11.0 |

#### 2.17.L DoS Bound (Group L)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.022 | MAX_FINDINGS DoS Bound — Finding Cap Prevents Unbounded all_findings Growth | P0 | feature-enip-v0.11.0 |

#### 2.17.M CLI Threshold Tuning and Accounting (Group M)

| BC ID | Title | Priority | Origin |
|-------|-------|----------|--------|
| BC-2.17.023 | --enip-write-burst-threshold CLI Flag Configures T0836 Write Detection Sensitivity | P1 | feature-enip-v0.11.0 |
| BC-2.17.024 | pdu_count Incremented Per Processed Frame and Reflected in summarize() | P1 | feature-enip-v0.11.0 |
| BC-2.17.025 | RegisterSession (0x0065) and UnRegisterSession (0x0066) Classified and PDU-Counted; No Finding Emitted | P1 | feature-enip-v0.11.0 |

> Full contracts: `behavioral-contracts/ss-17/BC-2.17.001.md` through `BC-2.17.025.md`


## 3. Interface Definition

> **Supplement:** Full interface definitions are in `prd-supplements/interface-definitions.md`.
> This section is a stub until the supplement burst (Phase 1b) completes.

Summary: wirerust exposes a single CLI binary. Subcommands: `analyze` (produces findings),
`summary` (produces protocol/host overview). Global flags include `--output-format`,
`--no-color`, `--reassemble`, `--no-reassemble`, reassembly threshold overrides, and file
output paths (`--json <FILE>`, `--csv <FILE>`). Exit codes: 0=success, 1=fatal error.
See `prd-supplements/interface-definitions.md` for the complete flag reference, exit code
semantics, JSON output schema, and flag interaction rules.


## 4. Non-Functional Requirements

> **Supplement:** Full NFR catalog is in `prd-supplements/nfr-catalog.md`.
> This section is a stub until the supplement burst (Phase 1b) completes.

The NFR catalog (79 entries from pass-4) covers categories: PERF (throughput and latency),
SEC (memory safety, no unsafe, injection prevention), REL (overflow checks, saturating
arithmetic), OBS (counters for dropped findings, truncated records, poisoned bytes),
RES (MAX_FINDINGS cap, buffer caps, map cardinality caps), MNT (MSRV, test coverage),
PORT (Rust 2024 edition), SUP (MITRE version), COMPAT (pcap classic only).
See `prd-supplements/nfr-catalog.md` for NFR-NNN entries with numerical targets.

Known NFR violation: NFR-VIO-001 -- README's "multi-GB captures" claim is only accurate
under matching RAM constraints (eager full-file load).


## 5. Error Taxonomy

> **Supplement:** Full error taxonomy is in `prd-supplements/error-taxonomy.md`.
> This section is a stub until the supplement burst (Phase 1b) completes.

Errors follow anyhow chaining patterns. Key categories:
- E-INP-NNN: Input / File errors (header parse failure, unsupported link type, file open failure, packet read failure)
- E-DEC-NNN: Decoder errors (unsupported link type, no IP layer, etherparse parse failure)
- E-RAS-NNN: Reassembly errors (lifecycle state-machine edge cases and resource limits)
- E-ANA-NNN: Analyzer errors (HTTP, TLS, DNS protocol-level parse failures)
- E-OUT-NNN: Output errors (file write failures for --json/--csv paths)
- E-CFG-NNN: Configuration errors (mutually exclusive flag combinations rejected by clap)
See `prd-supplements/error-taxonomy.md` for the complete E-xxx-NNN catalog.


## 6. Competitive Differentiator Traceability

> Maps each key differentiator (Section 1.3) to the behavioral contracts that implement it.

### 6.1 KD-001: Offline Single-Binary Deployment

| BC ID | Contribution |
|-------|-------------|
| BC-2.01.001 | Link-type gating at read time: no network call needed |
| BC-2.01.002 | Eager full-file load into memory: no streaming or daemon state |
| BC-2.12.016 | All three output reporters (terminal, JSON, CSV) are self-contained |

### 6.2 KD-002: Forensic-Fidelity Raw-Data Contract

| BC ID | Contribution |
|-------|-------------|
| BC-2.09.005 | Finding.summary and evidence carry RAW post-from_utf8_lossy bytes (ADR 0003) |
| BC-2.11.003 | JsonReporter uses serde RFC 8259 escaping; does NOT call escape_for_terminal |
| BC-2.11.007 | TerminalReporter is the SOLE caller of escape_for_terminal |
| BC-2.07.020 | TLS SNI non-UTF-8 bytes preserved raw in Finding.summary |
| BC-2.07.021 | TLS SNI non-ASCII UTF-8 bytes preserved raw in Finding.summary |
| BC-2.06.026 | HTTP header bytes preserved raw at analyzer layer |

### 6.3 KD-003: Content-First Protocol Identification

| BC ID | Contribution |
|-------|-------------|
| BC-2.05.001 | 0x16 0x03 content signature routes to TLS regardless of port |
| BC-2.05.002 | HTTP method prefix routes to HTTP regardless of port |
| BC-2.05.003 | Port fallback only when content is insufficient (5 bytes minimum) |
| BC-2.05.005 | Classification cached per flow for efficiency |
| BC-2.05.006 | DispatchTarget::None not cached until retry cap (default 8); late protocol identification retried until cap, then permanently cached as None |
| BC-2.14.025 | Modbus port-502 Rule 5 checked AFTER content rules (1-2) and TLS/HTTP port fallbacks (3-4); TLS/HTTP traffic on port 502 is never stolen by Modbus rule |
| BC-2.15.021 | DNP3 port-20000 Rule 6 checked AFTER all prior rules (1-5); TLS/HTTP/Modbus traffic on port 20000 is never stolen by DNP3 rule |

### 6.4 KD-004: First-Wins TCP Overlap Forensics

| BC ID | Contribution |
|-------|-------------|
| BC-2.04.036 | First-wins: gap bytes added; existing bytes preserved on partial overlap |
| BC-2.04.037 | Same-range conflicting overlap returns ConflictingOverlap; original data wins |
| BC-2.04.018 | ConflictingOverlap emits Anomaly/Likely/High finding with T1036 (Masquerading) |
| BC-2.04.019 | Excessive overlap threshold emits one-shot T1036 alert finding |

### 6.5 KD-005: MITRE ATT&CK Tactic-Grouped Output

| BC ID | Contribution |
|-------|-------------|
| BC-2.10.003 | all_tactics_in_report_order returns kill-chain order for deterministic grouping |
| BC-2.10.005 | technique_name lookup for all 28 seeded IDs (12 Enterprise + 16 ICS: **T0846 now emitted (BC-2.17.010 ListIdentity)**; T1692.001/T1692.002/T0885 existing; T0836/T0814/T0806/T0835/T0831/T0888 new Modbus; T1691.001/T0827 new DNP3 F2; T0830 [ICS] + T1557.002 [Enterprise] new ARP F2; T0858, T0816 [ICS] + T1693.001 [ICS staged] new EtherNet/IP F2) |
| BC-2.11.013 | TerminalReporter MITRE grouping with tactic headers in canonical order; groups by `mitre_techniques[0]`; multi-tag findings display all IDs |
| BC-2.11.015 | Uncategorized bucket for empty `mitre_techniques` vec or all-unknown IDs |
| BC-2.11.016 | Per-finding MITRE expansion with em-dash and name |
| BC-2.14.013 | T1692.001 co-included in multi-tag finding vec for every write-class FC (ADR-006); not standalone |
| BC-2.14.014 | Holding-register writes (0x06/0x10/0x16/0x17) emit `["T1692.001","T0836"]` single multi-tag finding |
| BC-2.14.015 | Coil-only writes (0x05/0x0F) emit `["T1692.001","T0835"]` single multi-tag finding |
| BC-2.14.016 | T0831 co-tagged inline on per-PDU write finding as `["T1692.001","T0836","T0831"]`; no separate T0831 Finding object (per-PDU write finding already carries T1692.001+T0836) |
| BC-2.14.017 | Burst/sustained rate detection emits `["T0806","T1692.001"]` — dual-window model (1s burst + >=2s sustained) |
| BC-2.14.018 | T0814 (Denial of Service) emitted for Force-Listen-Only (0x0004) and Restart-Comms (0x0001) Diagnostics sub-functions |
| BC-2.14.020 | T0888 (Remote System Information Discovery) emitted for recon FCs 0x11 and 0x2B/0x0E (correctness fix; T0846 not emitted) |
| BC-2.15.010 | T1692.001 emitted for unexpected source (count=1) or Control-class FC exceeding threshold per flow (DNP3) |
| BC-2.15.011 | T0814 (Denial of Service) emitted for COLD_RESTART/WARM_RESTART FCs (DNP3) |
| BC-2.15.012 | T0836 (Modify Parameter) emitted for WRITE FC (DNP3) |
| BC-2.15.013 | Co-emission ordering — direct finding (T0814/T1692.001) precedes derived T0827; broadcast-anomaly (018↔010) dedup rule |
| BC-2.15.014 | T1691.001 (Block Operational Technology Message: Command Message) emitted via per-flow request/response correlation — control request without response within window |
| BC-2.15.015 | T0827 (Loss of Control) emitted as derived correlated finding — N restart/block events in detection window |
| BC-2.15.023 | T0814 emitted per-occurrence for DISABLE_UNSOLICITED (0x15, Likely/Medium) and ENABLE_UNSOLICITED (0x14, Possible/Low) — alarm-suppression / event-blinding primitive detection |
| BC-2.15.024 | T0814 emitted as low-confidence anomaly when malformed_in_window ≥ MALFORMED_ANOMALY_THRESHOLD [F2-GATE-DEFAULT: 3] in 300s window — Crain-Sistrunk malformed-frame crash-class coverage (parse_errors is lifetime/monotonic; malformed_in_window is the windowed threshold counter) |

### 6.6 KD-006: SNI Anomaly Detection with 4-Way Classification

| BC ID | Contribution |
|-------|-------------|
| BC-2.07.013 | Clean ASCII SNI: silent, no finding |
| BC-2.07.014 | AsciiWithControl SNI: C0/DEL bytes detected, T1027 finding |
| BC-2.07.017 | NonAsciiUtf8 SNI: non-ASCII chars detected, T1027 finding |
| BC-2.07.019 | NonUtf8 SNI: invalid UTF-8 bytes detected, T1027 finding |
| BC-2.07.037 | Disambiguation: mixed non-ASCII+control fires arm 3 (NonAsciiUtf8) not arm 2 |

### 6.7 KD-007: Bounded-Resource Design

| BC ID | Contribution |
|-------|-------------|
| BC-2.04.024 | MAX_FINDINGS=10000 cap on reassembly engine findings |
| BC-2.04.025 | finalize bypass is the ONLY unconditional push past MAX_FINDINGS |
| BC-2.07.004 | MAX_RECORD_PAYLOAD=18432 cap on TLS record parsing |
| BC-2.07.005 | MAX_BUF=65536 per-direction buffer cap in TLS |
| BC-2.06.022 | MAX_HEADER_BUF=65536 per-direction buffer cap in HTTP |
| BC-2.04.041 | max_depth truncation prevents unbounded stream accumulation |
| BC-2.04.042 | max_receive_window rejects out-of-window segments |
| BC-2.15.016 | Per-flow DNP3 carry buffer bounded to MAX_DNP3_FRAME_LEN=292 bytes; master_addrs_seen bounded to 64 entries |
| BC-2.15.022 | MAX_FINDINGS cap prevents unbounded all_findings growth in Dnp3Analyzer |


## 7. Requirements Traceability Matrix

> Module column reflects subsystem IDs from ARCH-INDEX (ARCH-INDEX.md Subsystem Registry, Phase 1c). Priority is from Section 2.
> Test type is from BC source evidence (HIGH confidence = test exists; MEDIUM = code-only;
> LOW = ADR/comment-only).

| BC ID | Source (L2 CAP) | Module(s) | Priority | Test Type |
|-------|----------------|-----------|----------|-----------|
| BC-2.01.001 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.002 | CAP-01 | SS-01 (reader.rs) | P0 | unit |
| BC-2.01.003 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| ~~BC-2.01.004~~ | ~~CAP-01~~ | ~~SS-01 (reader.rs)~~ | ~~P0~~ | ~~unit~~ [RETIRED → BC-2.01.009] |
| BC-2.01.005 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.006 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.007 | CAP-01 | SS-01 (reader.rs) | P1 | unit |
| BC-2.01.008 | CAP-01 | SS-01 (reader.rs) | P2 | inferred |
| BC-2.01.009 | CAP-01 | SS-01 (reader.rs) | P0 | integration (STORY-123) |
| BC-2.01.010 | CAP-01 | SS-01 (reader.rs) | P0 | integration+VP-026 (E-INP-008/012; STORY-123) |
| BC-2.01.011 | CAP-01 | SS-01 (reader.rs) | P0 | integration (E-INP-008; STORY-124) |
| BC-2.01.012 | CAP-01 | SS-01 (reader.rs) | P0 | integration+VP-027 (E-INP-009/010; STORY-125) |
| BC-2.01.013 | CAP-01 | SS-01 (reader.rs) | P1 | integration (E-INP-009; STORY-126) |
| BC-2.01.014 | CAP-01 | SS-01 (reader.rs) | P0 | integration+VP-025 (STORY-125) |
| BC-2.01.015 | CAP-01 | SS-01 (reader.rs) | P1 | integration+VP-029 (E-INP-010; STORY-126) |
| BC-2.01.016 | CAP-01 | SS-01 (reader.rs) | P0 | integration (E-INP-010; STORY-124) |
| BC-2.01.017 | CAP-01 | SS-01 (reader.rs) | P1 | integration+VP-028/cargo-fuzz (E-INP-008/010; STORY-126) |
| BC-2.01.018 | CAP-01 | SS-01 (reader.rs) | P0 | integration+VP-030 (E-INP-011; STORY-124) |
| BC-2.02.001 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.002 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.003 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.004 | CAP-02 | SS-02 (decoder.rs) | P1 | unit |
| BC-2.02.005 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.006 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.007 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.02.008 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.009 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.010 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.011 | CAP-02 | SS-02 (decoder.rs) | P1 | inferred |
| BC-2.02.012 | CAP-02 | SS-02 (decoder.rs) | P1 | unit |
| BC-2.02.013 | CAP-02 | SS-02 (decoder.rs) | P2 | inferred |
| BC-2.02.014 | CAP-02 | SS-02 (decoder.rs) | P1 | unit |
| BC-2.02.015 | CAP-02 | SS-02 (decoder.rs) | P0 | unit |
| BC-2.04.001 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.002 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.003 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.004 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.005 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.006 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.007 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.008 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.009 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.010 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.011 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.012 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.013 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.014 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.015 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.016 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.017 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.018 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.019 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.020 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.021 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.022 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.023 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.024 | CAP-04 | SS-04 (reassembly/) | P0 | inferred |
| BC-2.04.025 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.026 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.027 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.028 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.029 | CAP-04 | SS-04 (reassembly/) | P2 | low |
| BC-2.04.030 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.031 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.032 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.033 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.034 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.035 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.036 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.037 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.038 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.039 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.040 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.041 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.042 | CAP-04 | SS-04 (reassembly/) | P1 | unit |
| BC-2.04.043 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.044 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.045 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.046 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.047 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.048 | CAP-04 | SS-04 (reassembly/) | P2 | low |
| BC-2.04.049 | CAP-04 | SS-04 (reassembly/) | P1 | inferred |
| BC-2.04.050 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.051 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.052 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.053 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.054 | CAP-04 | SS-04 (reassembly/) | P0 | unit |
| BC-2.04.055 | CAP-04 | SS-04 (reassembly/) | P1 | integration |
| BC-2.05.001 | CAP-05 | SS-05 (dispatcher.rs) | P0 | unit |
| BC-2.05.002 | CAP-05 | SS-05 (dispatcher.rs) | P0 | unit |
| BC-2.05.003 | CAP-05 | SS-05 (dispatcher.rs) | P0 | unit |
| BC-2.05.004 | CAP-05 | SS-05 (dispatcher.rs) | P1 | unit |
| BC-2.05.005 | CAP-05 | SS-05 (dispatcher.rs) | P0 | inferred |
| BC-2.05.006 | CAP-05 | SS-05 (dispatcher.rs) | P0 | inferred |
| BC-2.05.007 | CAP-05 | SS-05 (dispatcher.rs) | P1 | unit |
| BC-2.05.008 | CAP-05 | SS-05 (dispatcher.rs) | P1 | unit |
| BC-2.05.009 | CAP-05 | SS-05 (dispatcher.rs) | P0 | inferred |
| BC-2.06.001 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.002 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.003 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.004 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.005 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.006 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.007 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.008 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.009 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.010 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.011 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.012 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.013 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.014 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.015 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.016 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.017 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.018 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.019 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.020 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.021 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.06.022 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.023 | CAP-06 | SS-06 (analyzer/http.rs) | P1 | unit |
| BC-2.06.024 | CAP-06 | SS-06 (analyzer/http.rs) | P2 | inferred |
| BC-2.06.025 | CAP-06 | SS-06 (analyzer/http.rs) | P2 | inferred |
| BC-2.06.026 | CAP-06 | SS-06 (analyzer/http.rs) | P0 | unit |
| BC-2.07.001 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.002 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.003 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.004 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.005 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | inferred |
| BC-2.07.006 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.007 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.008 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.009 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit+integration |
| BC-2.07.010 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.011 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | integration |
| BC-2.07.012 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.013 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.014 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.015 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.016 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.017 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.018 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.019 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.020 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.021 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.022 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.023 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | unit |
| BC-2.07.024 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.025 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | unit |
| BC-2.07.026 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | unit |
| BC-2.07.027 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit |
| BC-2.07.028 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.029 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.030 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.07.031 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | unit+integration |
| BC-2.07.032 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | integration |
| BC-2.07.033 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | inferred |
| BC-2.07.034 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | inferred |
| BC-2.07.035 | CAP-07 | SS-07 (analyzer/tls.rs) | P1 | inferred |
| BC-2.07.036 | CAP-07 | SS-07 (analyzer/tls.rs) | P2 | inferred |
| BC-2.07.037 | CAP-07 | SS-07 (analyzer/tls.rs) | P0 | unit |
| BC-2.08.001 | CAP-08 | SS-08 (analyzer/dns.rs) | P0 | unit |
| BC-2.08.002 | CAP-08 | SS-08 (analyzer/dns.rs) | P0 | unit |
| BC-2.08.003 | CAP-08 | SS-08 (analyzer/dns.rs) | P1 | unit |
| BC-2.08.004 | CAP-08 | SS-08 (analyzer/dns.rs) | P0 | unit |
| BC-2.09.001 | CAP-09 | SS-09 (findings.rs) | P0 | unit |
| BC-2.09.002 | CAP-09 | SS-09 (findings.rs) | P1 | unit |
| BC-2.09.003 | CAP-09 | SS-09 (findings.rs) | P1 | unit |
| BC-2.09.004 | CAP-09 | SS-09 (findings.rs) | P1 | unit |
| BC-2.09.005 | CAP-09 | SS-09 (findings.rs) | P0 | unit+integration |
| BC-2.09.006 | CAP-09 | SS-09 (findings.rs) | P0 | unit |
| BC-2.09.007 | CAP-09 | SS-09 (findings.rs) | P1 | integration |
| BC-2.10.001 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.002 | CAP-10 | SS-10 (mitre.rs) | P1 | unit |
| BC-2.10.003 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.004 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.005 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.006 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.007 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.008 | CAP-10 | SS-10 (mitre.rs) | P0 | unit |
| BC-2.10.009 | CAP-10 | SS-10 (mitre.rs) | P2 | low |
| BC-2.11.001 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.002 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.003 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.004 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.005 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.006 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.007 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.008 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.009 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.010 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.011 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.012 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.013 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.014 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.015 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.016 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.017 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.018 | CAP-11 | SS-11 (reporter/) | P2 | inferred |
| BC-2.11.019 | CAP-11 | SS-11 (reporter/) | P1 | inferred |
| BC-2.11.020 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.021 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.022 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.023 | CAP-11 | SS-11 (reporter/) | P0 | unit |
| BC-2.11.024 | CAP-11 | SS-11 (reporter/) | P1 | unit |
| BC-2.11.035 | CAP-11 | SS-11 (reporter/json.rs, src/mitre.rs) | P1 | unit |
| BC-2.12.001 | CAP-12 | SS-12 (cli.rs) | P0 | unit |
| BC-2.12.002 | CAP-12 | SS-12 (cli.rs) | P1 | unit |
| BC-2.12.003 | CAP-12 | SS-12 (cli.rs) | P1 | unit |
| BC-2.12.004 | CAP-12 | SS-12 (cli.rs) | P0 | unit |
| BC-2.12.005 | CAP-12 | SS-12 (cli.rs) | P0 | unit |
| BC-2.12.006 | CAP-12 | SS-12 (cli.rs) | P1 | unit |
| BC-2.12.007 | CAP-12 | SS-12 (cli.rs) | P0 | inferred |
| BC-2.12.008 | CAP-12 | SS-12 (main.rs) | P1 | inferred |
| BC-2.12.009 | CAP-12 | SS-12 (main.rs) | P0 | inferred |
| BC-2.12.010 | CAP-12 | SS-12 (main.rs) | P2 | inferred |
| BC-2.12.011 | CAP-12 | SS-12 (main.rs) | P1 | inferred (STORY-127) |
| BC-2.12.012 | CAP-12 | SS-12 (main.rs) | P1 | inferred |
| BC-2.12.013 | CAP-12 | SS-12 (main.rs) | P2 | low |
| BC-2.12.014 | CAP-12 | SS-12 (main.rs) | P1 | unit |
| BC-2.12.015 | CAP-12 | SS-12 (main.rs) | P1 | inferred |
| BC-2.12.016 | CAP-12 | SS-12 (main.rs) | P0 | unit |
| BC-2.12.017 | CAP-12 | SS-12 (main.rs) | P0 | unit |
| BC-2.12.018 | CAP-12 | SS-12 (summary.rs) | P0 | unit |
| BC-2.12.019 | CAP-12 | SS-12 (summary.rs) | P1 | unit |
| BC-2.12.020 | CAP-12 | SS-12 (summary.rs) | P1 | unit |
| BC-2.12.021 | CAP-12 | SS-12 (summary.rs) | P1 | unit |
| BC-2.13.001 | CAP-12 | SS-13 (cli.rs) | P0 | unit |
| BC-2.13.002 | CAP-12 | SS-13 (cli.rs) | P0 | unit |
| BC-2.13.003 | CAP-12 | SS-13 (cli.rs) | P0 | unit |
| BC-2.13.004 | CAP-12 | SS-13 (cli.rs) | P2 | unit |
| BC-2.14.001 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit+kani |
| BC-2.14.002 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit+kani |
| BC-2.14.003 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.004 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.005 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit+kani |
| BC-2.14.006 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit+kani |
| BC-2.14.007 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit+kani |
| BC-2.14.008 | CAP-14 | SS-14 (analyzer/modbus.rs) | P1 | unit |
| BC-2.14.009 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.010 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.011 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.012 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.013 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.014 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.015 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.016 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.017 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.018 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.019 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.020 | CAP-14 | SS-14 (analyzer/modbus.rs) | P1 | unit |
| BC-2.14.021 | CAP-14 | SS-14 (analyzer/modbus.rs) | P1 | unit |
| BC-2.14.022 | CAP-14 | SS-14 (analyzer/modbus.rs) | P0 | unit |
| BC-2.14.023 | CAP-14 | SS-12 (cli.rs, main.rs) + SS-14 | P0 | unit+integration |
| BC-2.14.024 | CAP-14 | SS-12 (cli.rs, main.rs) + SS-14 | P0 | unit+integration |
| BC-2.14.025 | CAP-14 | SS-05 (dispatcher.rs) + SS-14 | P0 | unit+kani |
| BC-2.15.001 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.002 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.003 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.004 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.005 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.006 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.007 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit+kani |
| BC-2.15.008 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.009 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.010 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.011 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.012 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.013 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.014 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.015 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.016 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.017 | CAP-15 | SS-12 (cli.rs, main.rs) + SS-15 | P0 | unit+integration |
| BC-2.15.018 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P1 | unit |
| BC-2.15.019 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P1 | unit |
| BC-2.15.020 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P1 | unit |
| BC-2.15.021 | CAP-15 | SS-05 (dispatcher.rs) + SS-15 | P0 | unit+kani |
| BC-2.15.022 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P0 | unit |
| BC-2.15.023 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P1 | unit |
| BC-2.15.024 | CAP-15 | SS-15 (analyzer/dnp3.rs) | P1 | unit |
| BC-2.16.001 | CAP-16 | SS-16 (decoder.rs + analyzer/arp.rs) | P0 | unit+kani |
| BC-2.16.002 | CAP-16 | SS-16 (decoder.rs + analyzer/arp.rs) | P0 | unit+kani |
| BC-2.16.003 | CAP-16 | SS-16 (analyzer/arp.rs) | P0 | unit+kani |
| BC-2.16.004 | CAP-16 | SS-16 (analyzer/arp.rs) | P0 | unit+proptest |
| BC-2.16.005 | CAP-16 | SS-16 (analyzer/arp.rs) | P0 | unit+proptest |
| BC-2.16.006 | CAP-16 | SS-16 (analyzer/arp.rs) | P0 | unit+kani |
| BC-2.16.007 | CAP-16 | SS-16 (analyzer/arp.rs) | P0 | unit |
| BC-2.16.008 | CAP-16 | SS-16 (analyzer/arp.rs) | P1 | unit |
| BC-2.16.009 | CAP-16 | SS-02 (decoder.rs) + SS-16 (analyzer/arp.rs) | P1 | unit |
| BC-2.16.010 | CAP-16 | SS-16 (analyzer/arp.rs) | P1 | unit |
| BC-2.16.011 | CAP-16 | SS-12 (cli.rs, main.rs) + SS-16 | P0 | unit+integration |
| BC-2.16.012 | CAP-16 | SS-12 (cli.rs, main.rs) + SS-16 | P1 | unit+integration |
| BC-2.16.013 | CAP-16 | SS-12 (cli.rs, main.rs) + SS-16 | P1 | unit+integration |
| BC-2.16.014 | CAP-16 | SS-16 (analyzer/arp.rs) | P0 | unit |
| BC-2.16.015 | CAP-16 | SS-02 (decoder.rs) + SS-16 | P0 | unit+integration |
| BC-2.16.016 | CAP-16 | SS-16 (analyzer/arp.rs) | P1 | unit |
| BC-2.17.001 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit+kani |
| BC-2.17.002 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit+kani |
| BC-2.17.003 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit+kani |
| BC-2.17.004 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit+kani |
| BC-2.17.005 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit |
| BC-2.17.006 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit |
| BC-2.17.007 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit+kani |
| BC-2.17.008 | CAP-17 | SS-17 (analyzer/enip.rs) | P1 | unit |
| BC-2.17.009 | CAP-17 | SS-17 (analyzer/enip.rs) | P1 | unit |
| BC-2.17.010 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit |
| BC-2.17.011 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit |
| BC-2.17.012 | CAP-17 | SS-17 (analyzer/enip.rs) | P1 | unit |
| BC-2.17.013 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit |
| BC-2.17.014 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit |
| BC-2.17.015 | CAP-17 | SS-17 (analyzer/enip.rs) | P1 | unit |
| BC-2.17.016 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit |
| BC-2.17.017 | CAP-17 | SS-17 (analyzer/enip.rs) | P1 | unit |
| BC-2.17.018 | CAP-17 | SS-17 (analyzer/enip.rs) | P1 | unit |
| BC-2.17.019 | CAP-17 | SS-05 (dispatcher.rs) + SS-17 | P0 | unit+integration |
| BC-2.17.020 | CAP-17 | SS-12 (cli.rs, main.rs) + SS-17 | P0 | unit+integration |
| BC-2.17.021 | CAP-17 | SS-17 (analyzer/enip.rs) | P1 | unit |
| BC-2.17.022 | CAP-17 | SS-17 (analyzer/enip.rs) | P0 | unit |
| BC-2.17.023 | CAP-17 | SS-12 (cli.rs, main.rs) + SS-17 | P1 | unit+integration |
| BC-2.17.024 | CAP-17 | SS-17 (analyzer/enip.rs) | P1 | unit |
| BC-2.17.025 | CAP-17 | SS-17 (analyzer/enip.rs) | P1 | unit |


## 8. Domain Debt Index

> These open items from domain-debt.md are cross-referenced here for quick lookup.
> They describe CURRENT BEHAVIOR as of develop HEAD, not future requirements.

| Item | Description | Affected BCs |
|------|-------------|--------------|
| ~~O-01~~ | ~~Finding.timestamp always None; RawPacket timestamps never threaded to Finding constructors~~ **[CLOSED — STORY-097/098/099; BC-2.04.054 retains timestamp:None by design]** | ~~BC-2.09.001, BC-2.09.006~~ |
| O-02 | Absent User-Agent (None) intentionally not detected; only Some("") fires | BC-2.06.011 |
| O-03 | Anomaly thresholds not empirically calibrated against labelled traffic | BC-2.04.019, BC-2.04.020, BC-2.04.021 |
| O-04 | MITRE techniques seeded but never emitted: T1040, T1071, T1071.001, T1071.004, T1573, T1692.002, T0885, T1693.001 (staged-not-emitted per ADR-010 Decision 7; GetAndClear firmware detection deferred); T1692.002 replaces revoked T0856 per ATT&CK-ICS v19 remap. T0846 NOW emitted by EtherNet/IP (BC-2.17.010 ListIdentity — removed from not-emitted list). T1692.001/T0836/T0814/T0806/T0835/T0831/T0888 now emitted by Modbus (Feature #7); T1691.001/T0827 now emitted by DNP3 (Feature #8); T0830/T1557.002 now emitted by ARP (Feature #9); T0858/T0816/T0836/T0846/T0888/T0814 now emitted by EtherNet/IP (Feature #316, v0.11.0) — T0858 (IcsExecution TA0104, new catalog entry) and T0816 (IcsInhibitResponseFunction TA0107, new catalog entry) require `technique_info()` arms in src/mitre.rs + `MitreTactic::IcsExecution` new variant. Per ARCH-INDEX v1.7 + F2 EtherNet/IP update: SEEDED=28, EMITTED=20, CATALOGUE-ONLY=8. BC-2.10.005/BC-2.10.008 must be updated in the next BC version-bump pass to reflect T0858+T0816+T0846 emitted entries. | BC-2.10.005, BC-2.10.008 |
| O-05 | reassembly/mod.rs still 691 LOC after partial split (#85) | BC-2.04.* (reassembly module group) |
| O-06 | Weak-cipher Finding evidence Vec has unbounded cardinality (up to ~9216 cipher names) | BC-2.07.009 |
| O-07 | rayon declared in Cargo.toml but never imported; unused transitive dependency | (none -- build/dep debt only) |
| O-08 | dns.rs module doc-comment (lines 1-7) describes DGA/entropy/NXDOMAIN/rare-TLD detection not implemented; DnsAnalyzer is statistics-only (QR-bit counters, always returns empty findings Vec) | BC-2.08.001-004 |
