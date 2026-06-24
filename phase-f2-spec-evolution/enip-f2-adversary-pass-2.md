# Adversarial Spec Review — feature-enip-v0.11.0 (SS-17), Pass 2
VERDICT: FAIL — 4 CRITICAL, 3 HIGH, 3 MEDIUM, 2 LOW. Novelty: HIGH (dominant pattern: half-propagated Pass-1 fixes left anchor docs stale).

| ID | Sev | Axis | File | Description | Fix |
|----|-----|------|------|-------------|-----|
| F-P2-001 | CRITICAL | consistency | ADR-010 ~415/502/638 | ADR says EMITTED 17→19 + "T0846 already emitted"; PRD/research/BC-INDEX say 17→20 (ENIP emits T0846 first time). VP-007 obligation must add T0858+T0816+T0846 (3 ids). | ADR EMITTED→20; add 3 ids to EMITTED_IDS; remove "already emitted". |
| F-P2-002 | CRITICAL | protocol/Kani | BC-2.17.007 vs VP-032 Sub-D | BC maps 0x0A→ApplyAttributes / 0x09→MSP; ODVA + VP-032 say 0x0A→MultipleServicePacket. BC fails its own Kani partition. | Align BC-2.17.007 to VP-032 Sub-D NAMED_SERVICES (13 named, 0x0A=MSP); remove 0x09/ApplyAttributes. |
| F-P2-003 | CRITICAL | consistency | BC-2.17.007 H1/Inv-2 vs enum | "13 named" vs 14 enum variants. Resolves once bogus ApplyAttributes removed. | After F-P2-002, confirm exactly 13 named + Response + Unknown = 16. |
| F-P2-004 | CRITICAL | protocol | VP-032 line ~64 | Residual "big-endian offsets" for BC-2.17.002 (body is LE). | VP-032: big-endian→little-endian; grep whole file for BE residue. |
| F-P2-005 | HIGH | partial-fix | PRD §2.17 ~1342/1349/1362-1364 | Write-burst default 20→50 not propagated to PRD §2.17 body (3 sites). | PRD §2.17: 20→50; align OA-001 note. |
| F-P2-006 | HIGH | counter parity | BC-2.17.018 vs BC-2.17.016 | BC-016 increments malformed_in_window at 3 sites incl. oversized-frame-skip; BC-018 (owns T0814 threshold) lists only 2. | Add oversized-frame-skip path to BC-2.17.018 Desc + PC-1. |
| F-P2-007 | HIGH | canonical-frame holdout | BC-2.17.015 ~42/67-70 | Hard-coded connection-serial offset 14-15, no holdout; ODVA offset is variable → likely wrong. | Drop literal offset; "serial extraction deferred to v0.12.0; record 0 in v0.11.0" (matches best-effort framing). |
| F-P2-008 | MEDIUM | consistency | BC-2.17.008 PC vs EC-007 | Unconnected-only (0x00B2) scope in EC-007 not enforced in postconditions; literal PC reads 0x00B1 status at wrong offset. | Add PC gate: extract only when CPF item type_id==0x00B2; skip 0x00B1 in v0.11.0. |
| F-P2-009 | MEDIUM | clarity | BC-2.17.009 PC-2 | PC-2 says "apply mask &0xFC ... compare ==0x20 exact" — incoherent; implementer could code (t&0xFC)==0x20 (wrongly matches 0x21-0x23). | Remove &0xFC from v0.11.0 comparison; exact-match only; keep &0xFC note in Rationale as future. |
| F-P2-010 | MEDIUM [process-gap] | consistency | BC-2.10.005 PRD row ~858 | "25 Total" stale vs O-04 SEEDED=28; BC-2.10.005/008 version-bump pending (BC-INDEX line 875). | Update 25→28; land BC-2.10.005/008 version bump before F3. |
| F-P2-011 | LOW | realism | BC-2.17.014/ADR Dec 4 | ENIP_ERROR_BURST_THRESHOLD=5/10s plausibly low for noisy SCADA commissioning. | No F2 change; carry OA-005 to F6 recalibration. |
| F-P2-012 | LOW | stale label | PRD §2.17 ~1325/1339, §7 RTM | "24 BCs" + RTM stops at BC-2.17.024; BC-2.17.025 missing from PRD §2.17 table + §7 RTM. | Add BC-2.17.025 to §2.17 table + §7 RTM; "24 BCs"→"25 BCs". |

Pass-1 fixes verification: endianness propagated to BCs/ADR but MISSED VP-032 line 64 (F-P2-004); T0846 fixed in PRD/BC-INDEX but ADR stale (F-P2-001); frame-skip sound in BC-016 but BC-018 not reconciled (F-P2-006); BC-2.17.025 created OK but PRD/§7 not updated (F-P2-012); write-burst 50 in BCs/ADR but PRD body 20 (F-P2-005).
