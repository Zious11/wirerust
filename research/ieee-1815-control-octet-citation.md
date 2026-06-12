# IEEE 1815-2012 (DNP3) Data Link Control Octet — Authoritative Citation

**Research type:** general (protocol-standard citation cross-check)
**Date:** 2026-06-12
**Purpose:** Provide an independently-sourced, authoritative citation for policy
`DF-CANONICAL-FRAME-HOLDOUT-001`. The canonical control-octet byte value (`0xC4`)
must be traceable to the IEEE 1815-2012 protocol standard, NOT derived from this
project's own behavioral contracts. **Project BC files were deliberately NOT read.**

---

## Citation Block (transcribe verbatim)

> Per IEEE 1815-2012 §9.2.4.1 (data-link fixed-frame header; CONTROL field
> validity in §9.2.4.1.3 with Annex B "Valid Data Link Layer Control Codes"),
> the data-link CONTROL octet — the fourth octet of the frame — places DIR at
> bit 7 (0x80) and PRM at bit 6 (0x40), with the function code in the low nibble.
> DIR = 1 denotes the master-to-outstation direction (frame sourced from the
> designated master). A canonical master-to-outstation UNCONFIRMED_USER_DATA
> primary frame therefore has CONTROL octet = DIR(0x80) | PRM(0x40) | FC(0x04) = 0xC4.

**Short form (for a test docstring):**

> Per IEEE 1815-2012 §9.2.4.1, the data-link CONTROL octet places DIR at bit 7.
> A canonical master-to-outstation UNCONFIRMED_USER_DATA frame has
> control octet = DIR(0x80) | PRM(0x40) | FC(0x04) = 0xC4.

---

## Point-by-point findings

### 1. Section number defining the CONTROL octet

**Clause 9 = "Data Link Layer." The fixed-frame header / CONTROL field family is
§9.2.4.1, with the CONTROL field's validity rules at §9.2.4.1.3.**

This is supported by the **DNP Users Group** Technical Bulletin
*AN2013-004 "Validation of Incoming DNP3 Data"* (the DNP Users Group is the body
that maintains DNP3 and co-authored IEEE 1815 — quasi-primary). Its
validation-rules table maps each acceptance rule to an explicit
"IEEE 1815-2012 Reference" clause:

| Validation rule | IEEE 1815-2012 Reference (verbatim from bulletin) |
|---|---|
| First octet not `0x05`; second octet not `0x64` | **9.2.4.1.1** |
| LENGTH field (third octet) rules | **9.2.4.1.2** (and 9.2.4) |
| CRC field(s) incorrect | 9.2.4, **9.2.4.3**, Annex E |
| **PRM bit + FCV bit + function code in the CONTROL field (fourth octet) invalid / unsupported** | **9.2.4.1.3; Annex B "Valid Data Link Layer Control Codes"** |
| PRM bit clear but device not expecting a Secondary message | **9.2.4.1.3.2** |
| Secondary function code not a permitted response | **Table 9-1** |

This establishes the clause family **9.2.4.1.x** as the data-link fixed-header
clauses, with **§9.2.4.1.3 + Annex B** governing the CONTROL octet's valid
bit/function-code combinations. **Confidence: HIGH** (quasi-primary, dnp.org-authored).

> Note on precision: I confirmed the **clause numbers** from a quasi-primary
> source. I did **not** independently confirm the clause *titles* (e.g., a title
> like "Control Field Format") or any specific Figure/Table identifiers beyond
> **Table 9-1** and **Annex B** as named in the bulletin. See "Do NOT assert" below.

### 2. Bit-field layout — DIR = bit 7 (0x80), PRM = bit 6 (0x40)

**Confirmed.** The DNP Users Group bulletin reproduces the CONTROL-octet bit
diagram (bit 7 = MSB on the left):

```
 bit:   7    6    5    4    3 2 1 0
       DIR  PRM  FCB  FCV  [ Func Code ]   <- Primary to secondary (PRM=1)
       DIR  PRM  ...  DFC  [ Func Code ]   <- Secondary to primary (PRM=0)
```

- **DIR = bit 7 → mask 0x80** (MSB). Confirmed.
- **PRM = bit 6 → mask 0x40.** Confirmed.
- Function code occupies the low bits (bits 3..0 of the octet, low nibble).

Corroborated independently by: DNP3 cheat-sheets, the original DNP3 "Basic 4"
data-link documentation (circa 2002), ABB (REF615/REC523) and Emerson DNP3
protocol manuals, an Automatak/opendnp3 mailing-list thread, and DNP3 training
material — all consistent. **Confidence: HIGH.**

### 3. DIR semantics — DIR = 1 means master-to-outstation

**Confirmed.** The DNP Users Group bulletin states directly:

> "DIR: 1 = From Master, 0 = From Outstation"

i.e., **DIR = 1 ⇒ frame sourced from the designated master station
(master → outstation)**. ABB's REC523 manual corroborates: "The direction bit
indicates the physical direction of the frame with relation to the designated
master station. Value 1 indicates a frame from the designated master station;
value 0 indicates a frame from a non-master station." **Confidence: HIGH.**

The bulletin separately defines **PRM**: "1 = Primary to Secondary,
0 = Secondary to Primary" (PRM is the initiating/primary bit, distinct from DIR).

### 4. Function code 0x4 = UNCONFIRMED_USER_DATA; PRM = 1 for primary

**Confirmed.** In the primary-to-secondary (PRM = 1) function-code table,
function code **4 = UNCONFIRMED_USER_DATA**. This appears in the DNP3 data-link
function-code table reproduced across the DNP3 "Basic 4" docs, cheat-sheets, and
vendor manuals:

```
Primary (PRM = 1) function codes:
  0  RESET_LINK_STATES
  2  TEST_LINK_STATES
  3  CONFIRMED_USER_DATA
  4  UNCONFIRMED_USER_DATA   <-- FC = 0x4
  9  REQUEST_LINK_STATUS
```

**PRM = 1** for a primary (initiating) message → bit 6 → **0x40**. Confirmed.
**Confidence: HIGH** (multi-source).

### 5. Canonical octet derivation = 0xC4

**Confirmed by derivation AND by observed implementation practice.**

```
  DIR = 1  (master -> outstation)  -> bit 7 -> 0x80
  PRM = 1  (primary message)       -> bit 6 -> 0x40
  FC  = 4  (UNCONFIRMED_USER_DATA) -> low nibble -> 0x04
  ------------------------------------------------------
  0x80 | 0x40 | 0x04 = 0xC4   (binary 1100 0100)
```

The DNP "Valid Data Link Layer Control Codes" table (reproduced in the DNP3
"Basic 4" docs and cheat-sheets) lists, for Master-to-Outstation,
**`C4  UNCONFIRMED_USER_DATA`** (and `44` for the Outstation-to-Master direction,
where DIR = 0). DNP3 training material states verbatim: "the data link control
octet in almost all messages from the master is a hex C4 which is function code
four with the direction and primary bits set ... from the outstation is a hex 44."
An opendnp3 mailing-list capture shows a real master using control byte `C4`
(DIR = 1, PRM = 1) and the slave using `44` (DIR = 0, PRM = 1). **Confidence: HIGH.**

> Framing caveat: The standard defines the **bit meanings** and the **valid
> code-point set** (Annex B). `0xC4` is the **derived and universally-implemented**
> CONTROL byte for a master→outstation UNCONFIRMED_USER_DATA primary frame. It is
> safe to present `0xC4` as "derived per §9.2.4.1 bit layout and listed among the
> valid control codes (Annex B)", rather than as a single verbatim "shall be 0xC4"
> sentence lifted from the standard text.

---

## Explicit verification / limitation flags

**VERIFIED against a quasi-primary DNP Users Group source (dnp.org-authored
bulletin):**
- Clause family **9.2.4.1.x** is the data-link fixed-header; **§9.2.4.1.3 + Annex B**
  govern the CONTROL octet's valid PRM/FCV/function-code combinations.
- Bit layout: DIR = bit 7 (0x80), PRM = bit 6 (0x40), function code in low bits.
- DIR = 1 ⇒ From Master; PRM = 1 ⇒ Primary/initiating.
- CONTROL is the fourth octet of the frame.
- **Table 9-1** governs permitted Secondary response function codes.

**VERIFIED against multiple independent secondary sources** (vendor manuals,
cheat-sheets, training material, opendnp3 capture):
- FC 0x4 = UNCONFIRMED_USER_DATA (primary).
- Master→outstation UNCONFIRMED_USER_DATA control byte = `0xC4`; outstation→master = `0x44`.

**NOT independently verified — DO NOT assert as verbatim-from-standard:**
The IEEE 1815-2012 full text is paywalled; it was not read directly. A separate
AI deep-research narrative produced several specifics that could NOT be
corroborated by any primary or quasi-primary source and show hallucination
patterns. Avoid citing these as verbatim standard text:
- A clause **title** such as "Control Field Format" for §9.2.4.1 (clause *numbers*
  confirmed; *titles* not confirmed).
- A specific **"Figure 9-1"** bit diagram identifier.
- A **"Table 9-2 Primary Function Codes"** identifier (only **Table 9-1** is
  corroborated, and that is for permitted *response* codes).
- Any long verbatim **"shall ..."** sentences attributed to the standard.
- Any **page/line numbers** inside IEEE 1815-2012 or vendor manuals.
- A literal standard sentence of the form "the CONTROL byte shall be 0xC4 for ..."
  (the standard defines bit meanings + a valid-code set; `0xC4` is derived).

**Recommendation for the holdout/policy:** Cite **§9.2.4.1 (CONTROL field;
validity per §9.2.4.1.3 and Annex B)** as the section anchor, and present `0xC4`
as the derived `DIR(0x80) | PRM(0x40) | FC(0x04)` composition. This is fully
defensible from independent sources. Do not attach a fabricated figure/table
number or a verbatim "shall" quote.

---

## Primary / corroborating sources

1. **DNP Users Group** — Technical Bulletin AN2013-004(b) "Validation of Incoming
   DNP3 Data" (maps acceptance rules to IEEE 1815-2012 clause numbers incl.
   9.2.4.1.1/.2/.3, 9.2.4.1.3.2, 9.2.4.3, Table 9-1, Annex B, Annex E).
   dnp.org (download via LinkClick fileticket=bTubmc6O7kg) and Chipkin mirror:
   `https://cdn.chipkin.com/assets/uploads/imports/resources/AN2013-004b Validation of Incoming DNP3 Data.pdf`
   — **quasi-primary (standards-maintaining body).**
2. DNP3 "Basic 4" Data Link Layer documentation (circa 2002) & DNP3 Protocol
   Stack cheat-sheets — control-octet bit diagram + function-code table +
   "Valid Data Link Layer Control Codes" (`C4`/`44`). — secondary, multi-source.
3. ABB REC523 / REF615 DNP3 protocol manuals; Emerson FB1000 DNP3 spec —
   CONTROL field DIR/PRM semantics. — secondary (vendor).
4. opendnp3 / Automatak mailing-list thread — observed master `C4` / slave `44`.
   `https://groups.google.com/g/automatak-dnp3/c/v2V4Q-8iRhg` — secondary (field capture).
5. DNP3 training video (data-link lesson) — "almost all messages from the master
   ... hex C4 ... function code four with the direction and primary bits set."
   — secondary (training).
6. IEEE 1815-2012 standard record (paywalled, not read directly):
   `https://standards.ieee.org/ieee/1815/5414/` — primary (referenced, not accessed).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source sweep of IEEE 1815-2012 §9 control-octet definition, DIR/PRM bit layout, FC table, 0xC4 derivation (reasoning_effort=high). Surfaced narrative; treated specific figure/table/quote claims as suspect. |
| Perplexity perplexity_reason | 1 | Synthesis over gathered evidence: separate quasi-primary/secondary-confirmed facts from likely-hallucinated specifics; identify most defensible section citation. |
| Perplexity perplexity_search | 1 | Raw ranked URLs; surfaced the DNP Users Group AN2013-004 bulletin (the load-bearing quasi-primary source mapping rules to IEEE clause numbers) and the C4/44 control-code tables. |
| WebFetch | 1 | Attempted direct extraction of the dnp.org AN2013-004 PDF (binary/compressed; snippet already captured the IEEE-reference table via perplexity_search). |
| Training data | 1 area | General DNP3/IEC 60870-5 background framing only; all load-bearing facts sourced above. |

**Total MCP tool calls:** 4 (3 Perplexity + 1 WebFetch).
**Training data reliance:** low — every load-bearing claim (clause numbers, bit
masks, FC value, 0xC4 derivation) is sourced to the DNP Users Group bulletin
and/or multiple corroborating secondary sources; hallucination-risk specifics
from the AI narrative were explicitly excluded.
