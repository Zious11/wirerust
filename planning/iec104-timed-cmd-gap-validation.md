---
document_type: research-validation
producer: research-agent
status: complete
finding_id: IEC104-TIMED-CMD-GAP-001
policy: DF-VALIDATION-001
date: 2026-07-23
verdict: CONFIRMED
confidence: HIGH
inputs:
  - src/analyzer/iec104.rs
  - src/mitre.rs
  - src/reporter/json.rs
  - .factory/STATE.md
input-hash: "afccdd7"
---

# Validation Report — IEC104-TIMED-CMD-GAP-001

## Finding under validation

> TypeIDs 58–64 (timed control variants) fall into `detect_iec104_threats` `_` silent arm;
> no T1692.001/T0836 findings emitted. Evasion gap.
> (Active Carry-Forwards, `.factory/STATE.md:225`)

## Verdict

**CONFIRMED — HIGH confidence.**

The detection gap is real on all three axes: the code silently drops TypeIDs 58–64, those
TypeIDs are the CP56Time2a-tagged and functionally-equivalent twins of the plain control
commands 45–51 that *are* detected, and the technique IDs the finding cites (T1692.001, T0836)
are correct and current in the version of MITRE ATT&CK for ICS this repo pins to. One wording
correction to the finding: T1692.001 is **not** a mis-citation — see Axis 3.

---

## Axis 1 — Ground-truth code check (CONFIRMED)

File: `src/analyzer/iec104.rs`, function `detect_iec104_threats` (lines 730–929).

### (a) Which TypeIDs currently trigger threat findings

| Match arm | TypeIDs | Technique(s) emitted | Verdict | Lines |
|-----------|---------|----------------------|---------|-------|
| `45..=47` | C_SC_NA_1, C_DC_NA_1, C_RC_NA_1 | `T1692.001` | Possible | 748–774 |
| `48..=51` | C_SE_NA_1, C_SE_NB_1, C_SE_NC_1, C_BO_NA_1 | `T1692.001` + `T0836` | Possible | 781–830 |
| `105`     | C_RP_NA_1 (Reset Process) | `T0827` | Likely | 836–860 |
| `100 \| 101 \| 103` | Interrogation / Counter-Interr. / Clock-Sync | none (benign) | — | 866–870 |
| `0 \| 128..=255` | undefined / private-reserved | `T0814` | Possible | 878–910 |
| `_` (catch-all) | all remaining defined TypeIDs in [1,127] | **none — silent** | — | 915–918 |

### (b) TypeIDs 58–64 fall through to the silent `_` arm — CONFIRMED

The match has no arm covering 52–99. The code's own comment at lines 912–914 names the
fall-through set explicitly:

> "Defined-but-unhandled TypeIDs in [1, 127] not covered by the arms above:
>  TypeIDs 1–44 (monitoring direction), **52–99**, 102 (C_RD_NA_1), 104, 106–127.
>  No finding emitted — silently logged (BC-2.19.022 invariant 1; AC-170-005)."

58–64 sit inside the 52–99 span, so an I-format frame carrying any of these TypeIDs reaches
`detect_iec104_threats` via the live `on_data` dispatch path
(`src/analyzer/iec104.rs:1331–1339`) and produces **zero findings**. The dispatch is real and
reachable — I-format frames are parsed (`parse_asdu`, line 1331) and passed straight into
`detect_iec104_threats`; there is no upstream TypeID filter that would stop 58–64 from arriving.

### (c) What the untimed equivalents emit

- TypeIDs 45–47 → one `T1692.001` "Unauthorized Message: Command Message" finding, Verdict
  `Possible`, with CASDU/first_ioa target-address evidence (lines 748–774).
- TypeIDs 48–51 → **two** findings: `T1692.001` (command-message indicator) **and** `T0836`
  "Modify Parameter" (parameter/value write), both Verdict `Possible` (lines 781–830).

So the detection asymmetry is exact: the plain commands are attributed, their timed twins are
not.

---

## Axis 2 — Protocol semantics (CONFIRMED)

IEC 60870-5-101 (the companion standard whose ASDU type catalog IEC 60870-5-104 inherits)
defines TypeIDs 58–64 as the CP56Time2a time-tagged control commands, in exact 1:1
correspondence with the untagged base commands 45–51:

| Timed TypeID | Mnemonic | Untimed twin | Mnemonic | Operation |
|--------------|----------|--------------|----------|-----------|
| 58 | C_SC_TA_1 | 45 | C_SC_NA_1 | Single command |
| 59 | C_DC_TA_1 | 46 | C_DC_NA_1 | Double command |
| 60 | C_RC_TA_1 | 47 | C_RC_NA_1 | Regulating step command |
| 61 | C_SE_TA_1 | 48 | C_SE_NA_1 | Set-point, normalized value |
| 62 | C_SE_TB_1 | 49 | C_SE_NB_1 | Set-point, scaled value |
| 63 | C_SE_TC_1 | 50 | C_SE_NC_1 | Set-point, short float |
| 64 | C_BO_TA_1 | 51 | C_BO_NA_1 | Bitstring of 32 bits |

The `TA` suffix denotes the CP56Time2a-tagged variant of the corresponding `NA` base command;
the only wire-format difference is a trailing 7-byte CP56Time2a timestamp appended to the
information object. The control semantics — actuating a switch, driving a set-point, writing a
32-bit output — are identical. These are control-direction messages (master → RTU/IED).

This makes the gap a **real evasion vector, not a theoretical one**: an attacker who wants to
issue an unauthorized single/double/regulating command or a set-point write while staying
invisible to this analyzer need only select the timestamped TypeID (58–64) instead of the plain
one (45–51). Same physical effect on the controlled process; zero findings emitted. Timed
control commands are ordinary, standards-conformant traffic that real masters send, so the
evasion carries no protocol-anomaly side effect that another arm might catch.

Sources:
- [FreyrSCADA IEC-60870-5-101 README (TypeID catalog)](https://github.com/FreyrSCADA/IEC-60870-5-101/blob/master/README.md)
- [knaj/IEC-60870 TypeId.cs enum](https://github.com/knaj/IEC-60870/blob/master/IEC60870/Enum/TypeId.cs)
- [Wikipedia — IEC 60870-5 (ASDU type overview)](https://en.wikipedia.org/wiki/IEC_60870-5)
- [scadaprotocols.com — IEC-104 CP56Time2a technical guide](https://scadaprotocols.com/iec-104-time-synchronization-cp56time2a/)

---

## Axis 3 — MITRE mapping validation (CONFIRMED — finding IDs are correct, NOT mis-cited)

The task flagged T1692.001 as a possible mis-citation. It is **not**. Both cited IDs are valid
and current:

- **T1692.001 — "Unauthorized Message: Command Message"** is a live MITRE ATT&CK for ICS
  sub-technique under parent **T1692 "Unauthorized Message"** (tactic: Impair Process Control).
  It is the restructured home of the older **T0855 "Unauthorized Command Message"**: MITRE moved
  T0855's content under the T1692 parent as sub-technique `.001` (the T1692.001 description is
  verbatim the old T0855 text). The repo already tracks this remap deliberately —
  `src/mitre.rs:347–348, 437` map `"T1692.001" => ("Unauthorized Message: Command Message", …)`
  and annotate it "remapped from T0855, v19 remap issue #222". Using T1692.001 for control
  commands is therefore the *correct current* attribution; citing the retired T0855 would be the
  error.
- **T0836 — "Modify Parameter"** (tactic: Impair Process Control) is valid and current;
  `src/mitre.rs:215` maps it to that name. It is the appropriate second attribution for
  set-point / bitstring writes (parameter modification), which is exactly why the untimed
  48–51 arm co-emits it.

Currency is anchored in-repo: `src/reporter/json.rs:25–29` pins the catalog to
**ATT&CK for ICS v19.1 (released 2026-04-28)** and states all emitted IDs — explicitly listing
`T1692.001`, `T0836`, `T0814`, and others — were "confirmed valid and active in v19.1" against
the `ics-attack-19.1.json` STIX bundle (validation record:
`.factory/research/attack-ics-version-pin.md`).

**Correction to the finding text:** the finding writes "T1692.001/T0836" as the missing
attributions — that pairing is correct. No mis-citation exists. (The one detail I could not
independently pin to a public release note is the specific label "v19"; external sources
confirm the T0855→T1692.001 restructuring is real but did not name the version. The repo's own
v19.1 pin resolves this internally, so it does not affect the verdict.)

Sources:
- [MITRE ATT&CK — T1692 Unauthorized Message (ICS)](https://attack.mitre.org/techniques/T1692/)
- [MITRE ATT&CK — T1692.001 Unauthorized Message: Command Message (ICS)](https://attack.mitre.org/techniques/T1692/001/)
- [MITRE ATT&CK — T0855 Unauthorized Command Message (ICS, superseded)](https://attack.mitre.org/techniques/T0855/)

---

## Recommended detection scope (input to a new behavioral contract)

Mirror the existing untimed arms exactly, preserving the C_SE/C_BO parameter-write split so the
timed twins carry identical attribution to their plain counterparts:

| New arm | TypeIDs | Technique(s) | Verdict | Parity with |
|---------|---------|--------------|---------|-------------|
| `58..=60` | C_SC_TA_1, C_DC_TA_1, C_RC_TA_1 | `T1692.001` | Possible | `45..=47` arm |
| `61..=64` | C_SE_TA_1, C_SE_TB_1, C_SE_TC_1, C_BO_TA_1 | `T1692.001` + `T0836` | Possible | `48..=51` arm |

Parity requirements:
1. **Evidence parity** — include the same CASDU and (when `count > 0` and present) `first_ioa`
   target-address context the untimed arms already build (lines 752–758, 784–799).
2. **[TEST] tagging** — the `cot_test` → ` [TEST]` suffix loop (lines 924–928) already runs over
   all newly-pushed findings, so timed-command findings inherit it automatically once the arms
   are added; no extra wiring needed.
3. **Verdict/confidence parity** — `Verdict::Possible`, `Confidence::Medium`, `ThreatCategory::Impact`,
   matching the untimed arms.
4. **Summary wording** — distinguish the timed variant in the human-readable summary (e.g.,
   "C_SC_TA/C_DC_TA/C_RC_TA — time-tagged switching control command") so analysts can tell timed
   from untimed, while keeping the same technique IDs.
5. **No new MITRE catalog entries required** — T1692.001 and T0836 are already registered in
   `src/mitre.rs`; the VP-007 atomic-catalog obligation does not trigger for this change.

Implementation note: these two arms slot cleanly ahead of the existing `_` catch-all in
`detect_iec104_threats`. AC-170-005 / BC-2.19.022's "silently logged" set would need its listed
range narrowed (the 52–99 comment at lines 912–914) to remove 58–64. Regression check: the
existing "defined-but-unhandled" tests for neighboring TypeIDs (52–57, 65–99) must still assert
zero findings.
