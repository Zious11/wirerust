---
document_type: holdout-scenario
level: ops
version: "1.2"  # H-4 / Pass-4 R4 / ADR-009 rev 7: initial authoring. Three cases: (a) valid pcapng SHB+IDB, no EPB/SPB → stdout empty, exactly one stderr notice (no skip count), exit 0; (b) valid pcapng with 2 unknown-type skipped blocks, no packets → notice includes skipped-block count, exit 0; (c) malformed pcapng (EPB before any IDB) → E-INP-009, exit 1, NO zero-packet notice. Maps to BC-2.01.009 PC6 / BC-2.01.015 PC9. Pass-5 S4 (ADR-009 rev 8): added cases (d) OPB-only fixture → notice with OPB count + mergecap hint, exit 0; (e) 2 NRBs + 1 OPB fixture → notice shows OPB count distinctly from NRB skips, exit 0. Canonical notice format updated to "notice: <filename>: 0 packets read from pcapng file (...)". Pass-5 re-audit: (P5-001) removed stale VP-025 from verification_properties (VP-025 is the SHB Kani timestamp proof; no relationship to zero-packet NOTICE); (P5-002) standardized Cases D/E mergecap hint to BC-2.01.009 PC6 canonical form "re-save with mergecap".
status: draft
producer: product-owner
timestamp: 2026-06-20T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.009.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.015.md
input-hash: "tbd"
traces_to: .factory/specs/prd.md
id: "HS-108"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-1"
behavioral_contracts:
  - BC-2.01.009
  - BC-2.01.015
verification_properties: []
lifecycle_status: active
introduced: v0.9.x-pcapng-reader
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: pcapng Zero-Packet Notice — End-to-End Stderr Notice, Skip-Count Inclusion, and Error vs. Notice Disambiguation

> **WARNING:** This file must NEVER be shown to the implementer or test-writer agents.

## Scenario

When a structurally valid pcapng file produces zero packets, wirerust MUST emit exactly
one stderr notice (BC-2.01.009 PC6 / BC-2.01.015 PC9). This scenario tests three boundary
conditions that together define the complete public-observable contract for the notice:

1. A valid pcapng with SHB + IDB but no packet-bearing blocks (EPB/SPB) → notice without
   skip count, exit 0, stdout empty.
2. A valid pcapng with 2 unknown-type skipped blocks and no packets → notice WITH skip
   count ("2 block(s) skipped"), exit 0.
3. A malformed pcapng where an EPB appears before any IDB → E-INP-009 error on stderr,
   exit 1. This is NOT a zero-packet success; the zero-packet notice is NOT emitted.

The third case is the critical disambiguation: a zero-packet condition caused by a
structural error (E-INP-009) must produce an error + non-zero exit, not the notice + exit 0.
BC-2.01.009 v1.4 (ADR-009 rev 7 H-4) explicitly states: "a file is 'structurally-valid
zero-packet' (notice, exit 0) IFF it parses to EOF with no error AND packets.len()==0; an
EPB/SPB before any IDB is an ERROR (E-INP-009, exit 1), NOT a zero-packet success."

### SHB and IDB Wire Layouts (shared across all fixtures)

**SHB (28 bytes, LE):**
```
Hex: 0A 0D 0D 0A  1C 00 00 00  4D 3C 2B 1A  01 00  00 00
     FF FF FF FF FF FF FF FF  1C 00 00 00
```
Breakdown: block_type=`0A0D0D0A`, btl=28 (`1C 00 00 00`), BOM=`4D 3C 2B 1A` (LE sentinel),
major=1 (`01 00`), minor=0 (`00 00`), section_length=-1 (`FF FF FF FF FF FF FF FF`),
trailing_btl=28 (`1C 00 00 00`).

**IDB (20 bytes, LE, linktype=Ethernet, snaplen=65535):**
```
Hex: 01 00 00 00  14 00 00 00  01 00  00 00  FF FF 00 00  14 00 00 00
```
Breakdown: block_type=`00000001` (`01 00 00 00`), btl=20 (`14 00 00 00`),
linktype=1 (`01 00`), reserved=0 (`00 00`), snaplen=65535 (`FF FF 00 00`),
trailing_btl=20 (`14 00 00 00`).

**Unknown-type block (20 bytes, LE, type=0x00000099):**
```
Hex: 99 00 00 00  14 00 00 00  [8 bytes of dummy body]  14 00 00 00
```
Breakdown: block_type=`0x00000099` (unknown; not EPB/SPB/IDB/SHB), btl=20 (`14 00 00 00`),
8 dummy body bytes (e.g. `AA BB CC DD EE FF 00 11`), trailing_btl=20 (`14 00 00 00`).
The pcap-file crate delivers this as a `RawBlock` with body = 8 dummy bytes; wirerust
silently skips it and increments `skipped_blocks`.

---

### Case A — Valid pcapng (SHB + IDB, zero EPB/SPB, zero skipped blocks) → notice without skip count, exit 0

1. A crafted pcapng file is presented containing ONLY a SHB (28 bytes) + IDB (20 bytes).
   No EPB, SPB, OPB, or unknown blocks follow. Total file = 48 bytes.
   ```
   File layout (48 bytes total):
     [SHB: 28 bytes — see above]
     [IDB: 20 bytes — see above]
     [EOF]
   ```
   The file is structurally valid. The block walk reaches EOF with `packets.len() == 0`
   and `skipped_blocks == 0`.
2. The user runs `wirerust analyze idb_only_no_packets.pcapng --json 2>&1`.
3. Expected public-observable outcomes:
   - **Exit code: 0** (not an error — the file is valid).
   - **Stdout:** empty (no JSON output — `--json` produces no output when no packets).
   - **Stderr:** contains exactly ONE notice. The notice MUST match the canonical format:
     `"notice: idb_only_no_packets.pcapng: 0 packets read from pcapng file"` (no
     parenthetical segment when no blocks were skipped and no OPBs present).
   - The notice MUST NOT include a parenthetical segment (e.g., "(0 block(s) skipped)"
     or "(2 block(s) skipped)") — the parenthetical is omitted when `skipped_blocks == 0`
     and no OPBs were encountered.
   - The notice MUST NOT appear MORE THAN ONCE on stderr (one-shot guard).
   - No JSON, no CSV, no terminal report is emitted on stdout.

**Byte-exact assertion:** `stderr` contains the substring
`"0 packets read from pcapng file"` AND does NOT contain `"skipped"` AND does NOT
contain `"obsolete"`. `stdout` is empty. Exit code 0.

---

### Case B — Valid pcapng (SHB + IDB + 2 unknown blocks, zero packets) → notice WITH skip count, exit 0

1. A crafted pcapng file is presented containing SHB + IDB + 2 unknown-type blocks.
   No EPB or SPB blocks are present. Total file = 28 + 20 + 20 + 20 = 88 bytes.
   ```
   File layout (88 bytes total):
     [SHB: 28 bytes]
     [IDB: 20 bytes]
     [Unknown block 1: 20 bytes, type=0x00000099]
     [Unknown block 2: 20 bytes, type=0x00000099]
     [EOF]
   ```
   The file is structurally valid. The block walk reaches EOF with `packets.len() == 0`
   and `skipped_blocks == 2` (each unknown block increments the counter once per
   BC-2.01.015 PC9).
2. The user runs `wirerust analyze unknown_blocks_no_packets.pcapng --json 2>&1`.
3. Expected public-observable outcomes:
   - **Exit code: 0** (not an error — the file is structurally valid; skipping unknown
     block types is normal pcapng behavior).
   - **Stdout:** empty.
   - **Stderr:** contains exactly ONE notice. The notice MUST match the canonical format
     with both the zero-packet phrase and a parenthetical skip count:
     `"notice: unknown_blocks_no_packets.pcapng: 0 packets read from pcapng file (2 block(s) skipped as unsupported)"`.
     The notice MUST contain both:
     - The zero-packet substring: `"0 packets read from pcapng file"`.
     - The skip-count segment: `"2 block(s) skipped"` (or equivalent phrasing confirming
       the count is 2 and the blocks were skipped as unsupported).
   - The notice MUST appear exactly ONCE on stderr (one-shot guard).
   - No other output on stderr beyond the single notice line.

**Byte-exact assertion:** `stderr` contains the substring
`"0 packets read from pcapng file"` AND contains `"2 block(s) skipped"` (or
a phrase including the literal `2` and "skipped"). `stdout` is empty. Exit code 0.

---

### Case C — Malformed pcapng (EPB before any IDB) → E-INP-009, exit 1, NO zero-packet notice

1. A crafted pcapng file is presented containing SHB + an EPB (with interface_id=0)
   but NO IDB before the EPB. The interface table is empty when the EPB arrives.
   Total file = 28 (SHB) + 48 (EPB) = 76 bytes.

   **Minimal EPB (48 bytes, LE):**
   ```
   block_type:         06 00 00 00   # EPB type
   block_total_length: 30 00 00 00   # 48 decimal
   interface_id:       00 00 00 00   # references interface 0 — but table is EMPTY
   ts_high:            00 00 00 00
   ts_low:             00 00 00 00
   captured_len:       10 00 00 00   # 16 decimal
   original_len:       10 00 00 00   # 16 decimal
   packet_data:        [16 bytes: AA BB CC DD EE FF 00 11 22 33 44 55 66 77 88 99]
   trailing_total_len: 30 00 00 00
   ```
   Full EPB hex (48 bytes):
   ```
   06 00 00 00  30 00 00 00  00 00 00 00  00 00 00 00
   00 00 00 00  10 00 00 00  10 00 00 00
   AA BB CC DD EE FF 00 11 22 33 44 55 66 77 88 99
   30 00 00 00
   ```

   ```
   File layout (76 bytes total):
     [SHB: 28 bytes]
     [EPB: 48 bytes — interface_id=0, NO prior IDB]
     [EOF]
   ```
   This file is structurally MALFORMED: an EPB references interface 0, but no IDB has
   been seen. This is an error condition (E-INP-009 — empty interface table), not a
   zero-packet success.
2. The user runs `wirerust analyze epb_before_idb.pcapng --json 2>&1`.
3. Expected public-observable outcomes:
   - **Exit code: 1** (non-zero — a structural error was encountered).
   - **Stdout:** empty (no JSON output on stdout; the file failed to parse).
   - **Stderr:** contains an error message consistent with **E-INP-009** (no interface
     defined / EPB before any IDB / empty interface table). Message must not be a raw
     Rust panic backtrace (`thread 'main' panicked`). Must not be empty.
   - **The zero-packet notice MUST NOT appear on stderr.** The notice (`"0 packets read
     from pcapng file"` in canonical format) is for structurally valid zero-packet files
     only. A file that produces an error (E-INP-009) before reaching EOF with zero packets
     is NOT a "structurally valid zero-packet" file — it is an error. Emitting the notice
     alongside the error would be incorrect.
   - No JSON, no CSV, no terminal report is emitted on stdout.

**Byte-exact assertion:** `stderr` contains a non-empty error string consistent with
E-INP-009. `stderr` does NOT contain `"0 packets read from pcapng file"`.
`stdout` is empty. Exit code non-zero (1 or 2).

---

### Case D — Valid pcapng (SHB + IDB + 1 OPB, zero EPB/SPB) → notice WITH OPB count + mergecap hint, exit 0

**Added: Pass-5 S4 (ADR-009 rev 8) — OPB-distinction scenarios.**

An Obsolete Packet Block (OPB, block_type=0x00000002) is a legacy pcapng block type from
pre-1.0 spec drafts. It carries packet data but is treated as a skipped/unsupported block
by modern readers that use EPB as the canonical packet block. wirerust MUST track OPBs
distinctly in the notice because OPBs contain packet data that was NOT analyzed — this
requires a mergecap remediation hint so users understand that captured packets may have
been silently omitted from analysis.

**OPB wire layout (32 bytes, LE, 0 captured bytes):**
```
block_type:         02 00 00 00   # OPB type (0x00000002)
block_total_length: 20 00 00 00   # 32 decimal
interface_id:       00 00          # references interface 0 (IDB[0])
drops_count:        00 00          # no drops
ts_high:            00 00 00 00
ts_low:             00 00 00 00
captured_len:       00 00 00 00   # 0 bytes of captured data
original_len:       00 00 00 00   # 0 original bytes
trailing_total_len: 20 00 00 00
```
Full OPB hex (32 bytes):
```
02 00 00 00  20 00 00 00  00 00  00 00  00 00 00 00
00 00 00 00  00 00 00 00  00 00 00 00  20 00 00 00
```

1. A crafted pcapng file is presented containing SHB + IDB + 1 OPB (no EPB or SPB).
   Total file = 28 + 20 + 32 = 80 bytes.
   ```
   File layout (80 bytes total):
     [SHB: 28 bytes — see above]
     [IDB: 20 bytes — see above]
     [OPB: 32 bytes, type=0x00000002, captured_len=0]
     [EOF]
   ```
   The file is structurally valid. The block walk reaches EOF with `packets.len() == 0`,
   `skipped_blocks == 0` (OPBs are tracked separately from generic unknown-type skips),
   and `obsolete_packet_blocks == 1`.
2. The user runs `wirerust analyze opb_only_no_epb.pcapng --json 2>&1`.
3. Expected public-observable outcomes:
   - **Exit code: 0** (not an error — the file is structurally valid).
   - **Stdout:** empty (no JSON output — no packets were analyzed).
   - **Stderr:** contains exactly ONE notice in the canonical format. The notice MUST:
     - Contain the zero-packet substring: `"0 packets read from pcapng file"`.
     - Include the OPB count with mergecap remediation hint, for example:
       `"notice: opb_only_no_epb.pcapng: 0 packets read from pcapng file (includes 1 obsolete Packet Block whose data was not analyzed; re-save with mergecap)"`.
     - The literal `1` (OPB count) and the word `"obsolete"` MUST both appear in the notice.
     - The mergecap hint MUST be present (guideline: `"convert with mergecap"` or equivalent
       explicit remediation directive).
   - The notice MUST appear exactly ONCE on stderr (one-shot guard).
   - No OPB packet body bytes appear in the notice message (SEC-007 applies).

**Byte-exact assertion:** `stderr` contains the substring `"0 packets read from pcapng file"`
AND contains `"1"` AND contains `"obsolete"` AND contains `"mergecap"`. `stdout` is empty.
Exit code 0. `stderr` does NOT contain `"skipped as unsupported"` (OPB is reported via the
distinct OPB-count path, not the generic skip-count path).

---

### Case E — Valid pcapng (SHB + IDB + 2 NRBs + 1 OPB, zero packets) → OPB count distinct from NRB skips, exit 0

**Added: Pass-5 S4 (ADR-009 rev 8) — OPB-distinction scenarios.**

This case verifies that OPBs are counted and reported DISTINCTLY from non-packet skipped
blocks (NRBs, unknown-type blocks, etc.). The notice must show the OPB count separately
from the general skip count so users can tell whether their capture contained actual (but
unanalyzed) packet data.

**NRB wire layout (16 bytes, LE, empty record list):**
```
block_type:         04 00 00 00   # NRB type (0x00000004)
block_total_length: 10 00 00 00   # 16 decimal
nrb_record_type:    00 00          # record type 0 = end-of-records sentinel
nrb_record_length:  00 00          # 0 bytes
trailing_total_len: 10 00 00 00
```
Full NRB hex (16 bytes):
```
04 00 00 00  10 00 00 00  00 00  00 00  10 00 00 00
```

1. A crafted pcapng file is presented containing SHB + IDB + NRB + NRB + OPB (no EPB or SPB).
   Total file = 28 + 20 + 16 + 16 + 32 = 112 bytes.
   ```
   File layout (112 bytes total):
     [SHB: 28 bytes — see above]
     [IDB: 20 bytes — see above]
     [NRB: 16 bytes, type=0x00000004, empty record list]
     [NRB: 16 bytes, type=0x00000004, empty record list]
     [OPB: 32 bytes, type=0x00000002, captured_len=0]
     [EOF]
   ```
   The file is structurally valid. The block walk reaches EOF with `packets.len() == 0`,
   `skipped_blocks == 2` (the two NRBs are non-packet, non-IDB, non-EPB/SPB blocks),
   and `obsolete_packet_blocks == 1`.
2. The user runs `wirerust analyze nrb_plus_opb_no_packets.pcapng --json 2>&1`.
3. Expected public-observable outcomes:
   - **Exit code: 0** (not an error).
   - **Stdout:** empty.
   - **Stderr:** contains exactly ONE notice. The notice MUST:
     - Contain the zero-packet substring: `"0 packets read from pcapng file"`.
     - Report the OPB count DISTINCTLY from the NRB skip count. Both counts MUST appear
       separately in the notice. A compliant example:
       `"notice: nrb_plus_opb_no_packets.pcapng: 0 packets read from pcapng file (2 block(s) skipped as unsupported; includes 1 obsolete Packet Block whose data was not analyzed; re-save with mergecap)"`.
     - The literal `2` (NRB skip count) AND the literal `1` (OPB count) AND the word
       `"obsolete"` AND `"mergecap"` MUST ALL appear in the notice.
     - The OPB count (`1`) and the generic skip count (`2`) MUST be presented as distinct
       values — the notice MUST NOT collapse them into a single count of `3`.
   - The notice MUST appear exactly ONCE on stderr.

**Byte-exact assertion:** `stderr` contains the substring `"0 packets read from pcapng file"`
AND contains `"2"` (skip count) AND contains `"1"` (OPB count) AND contains `"obsolete"`
AND contains `"mergecap"`. `stderr` does NOT collapse OPB and non-packet skips into a single
aggregate count. `stdout` is empty. Exit code 0.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.01.009 | PC6 — valid file + zero packets → one-shot stderr notice (no parenthetical when skipped_blocks==0 and no OPBs) | Case A: SHB+IDB only; canonical notice "notice: <file>: 0 packets read from pcapng file" without parenthetical |
| BC-2.01.009 | PC6 — valid file + zero packets → one-shot stderr notice WITH skip count when skipped_blocks>0 | Case B: 2 unknown blocks skipped; notice includes "(2 block(s) skipped)" in canonical format |
| BC-2.01.009 | PC6 / H-4 disambiguation — EPB before IDB is E-INP-009 error, NOT zero-packet success; notice MUST NOT appear | Case C: malformed file (EPB before IDB) produces E-INP-009 exit 1 with no notice |
| BC-2.01.015 | PC9 — skipped_blocks counter incremented once per skipped unknown block; count passed to BC-2.01.009 notice | Case B: two unknown blocks → skipped_blocks=2 → count appears in notice |
| BC-2.01.015 | PC9 — skipped_blocks=0 when no blocks were skipped; BC-2.01.009 omits skip-count from notice | Case A: no skipped blocks → notice has no skip-count segment |
| BC-2.01.009 | One-shot guard — notice emitted exactly once per file, not once per block | Cases A, B, D, E: notice appears exactly once on stderr |
| BC-2.01.009 | SEC-007 — block body content NOT included in the notice message | Cases A, B, D, E: notice is a one-line human-readable string with no raw body bytes |
| BC-2.01.009 | OPB-distinction — obsolete_packet_blocks count appears in notice DISTINCTLY from skipped_blocks; mergecap hint included | Case D: 1 OPB → notice includes OPB count (1) and mergecap hint; no generic skip-count |
| BC-2.01.009 | OPB+NRB co-occurrence — both skip count (NRBs) and OPB count appear in notice as separate values; not collapsed | Case E: 2 NRBs + 1 OPB → notice shows "2 block(s) skipped" AND "1 obsolete Packet Block" as distinct entries |

## Verification Approach

```
wirerust analyze idb_only_no_packets.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: exit 0, stderr contains `"0 packets read from pcapng file"` (no "skipped" or
"obsolete" substring), stdout empty.

Verify notice appears exactly once:
```
wirerust analyze idb_only_no_packets.pcapng --json 2>&1 | grep -c "0 packets read"
```
Expect: output `1`.

```
wirerust analyze unknown_blocks_no_packets.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: exit 0, stderr contains `"0 packets read from pcapng file"` AND contains
`"2 block(s) skipped"` (or equivalent with literal `2` and "skipped"), stdout empty.

```
wirerust analyze epb_before_idb.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: non-zero exit, stderr contains E-INP-009 error message, stderr does NOT contain
`"0 packets read from pcapng file"`, stdout empty.

```
wirerust analyze opb_only_no_epb.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: exit 0, stderr contains `"0 packets read from pcapng file"` AND contains `"1"`
AND contains `"obsolete"` AND contains `"mergecap"`. stderr does NOT contain
`"skipped as unsupported"`. stdout empty.

```
wirerust analyze nrb_plus_opb_no_packets.pcapng --json 2>&1
echo "Exit: $?"
```
Expect: exit 0, stderr contains `"0 packets read from pcapng file"` AND contains `"2"`
(skip count) AND contains `"1"` (OPB count, presented distinctly) AND contains `"obsolete"`
AND contains `"mergecap"`. stdout empty. The counts `2` and `1` MUST NOT be merged into `3`.

## Evaluation Rubric

- **Case A correctness** (weight: 0.20): SHB+IDB file exits 0 with exactly one notice
  on stderr. Notice matches canonical format `"notice: <file>: 0 packets read from pcapng file"`
  without parenthetical segment. Stdout empty. Confirms PC6 broadened condition (zero-packet
  even without skipped blocks).
- **Case B skip-count inclusion** (weight: 0.20): Two-skipped-block file exits 0 with
  notice containing both the zero-packet phrase and the literal skip count (2). The
  skip-count segment is present IFF `skipped_blocks > 0`. Confirms BC-2.01.015 PC9
  counter handoff to BC-2.01.009 notice logic.
- **Case C error vs. notice disambiguation** (weight: 0.25): EPB-before-IDB file exits
  non-zero with E-INP-009 error on stderr. The zero-packet notice MUST NOT appear.
  Conflating a structural error with a zero-packet success is the primary H-4 defect
  class this case guards against.
- **Case D OPB-count with mergecap hint** (weight: 0.20): SHB+IDB+1 OPB file exits 0
  with notice containing the zero-packet phrase, the OPB count (1), the word "obsolete",
  and a mergecap remediation hint. The OPB is NOT reported via the generic skip-count path.
  Confirms OPB-specific notice path is implemented and distinct from unknown-block skipping.
- **Case E OPB count distinct from NRB skips** (weight: 0.10): SHB+IDB+2 NRBs+1 OPB
  file exits 0 with notice showing both the skip count (2 for NRBs) and the OPB count (1)
  as separate values. The notice MUST NOT collapse both into a single count of 3.
  Confirms that wirerust distinguishes "blocks with packet data that were skipped" (OPBs)
  from "blocks with no packet data that were skipped" (unknown/NRB).
- **One-shot guard** (weight: 0.05): In Cases A, B, D, and E, the notice appears exactly
  once on stderr regardless of how many blocks were walked before EOF. The notice is not
  emitted per-block.

## Edge Conditions

- **Case A (SHB+IDB, no EPB/SPB, skipped_blocks=0):** The IDB itself is not a packet-
  bearing block. A file with only SHB+IDB is valid and complete per the pcapng spec
  (it represents a capture session that was opened but produced no packets). The notice
  fires because `valid file + zero packets` — not because blocks were skipped.
- **Case B (unknown block type):** Block type `0x00000099` is not defined in the pcapng
  spec. The pcap-file crate delivers it as a `RawBlock` (with body = 8 dummy bytes).
  wirerust silently discards the body and increments `skipped_blocks`. SEC-007 applies:
  the body bytes MUST NOT appear in the notice message.
- **Case C (EPB before IDB):** This is the H-4 disambiguation case. Before BC-2.01.009
  v1.4, it was ambiguous whether a zero-packet condition caused by a structural error
  should produce the notice or the error. The v1.4 rule is unambiguous: the notice fires
  only when the file parses to EOF with no error. An E-INP-009 error terminates the parse
  before EOF; the notice must not fire.
- **Case D (SHB+IDB+OPB, no EPB/SPB, obsolete_packet_blocks=1):** The OPB contains
  packet data bytes (captured_len=0 in the fixture, but non-zero in real captures). wirerust
  tracks OPBs separately from unknown-type skipped blocks because OPBs carry actual packet
  content that was NOT decoded or included in the analysis output. The notice must reflect
  this with a mergecap hint so users understand data may be missing.
- **Case E (2 NRBs + 1 OPB):** NRBs (Name Resolution Blocks) carry IP-to-name mappings,
  not packet data. They are non-packet blocks (like unknown-type blocks) and contribute to
  `skipped_blocks`. OPBs carry packet data and contribute to `obsolete_packet_blocks`. These
  two counters serve different user-facing purposes and MUST remain distinct in the notice.
  A count of `3` in the notice would be incorrect; the correct representation has `2` (NRBs)
  and `1` (OPB) as separate values with separate labels.
- **Ordering of notice vs. JSON output:** In Cases A, B, D, and E (exit 0, --json flag),
  the notice appears on stderr and the JSON output (if any) appears on stdout. When
  `packets.len() == 0`, the JSON output may be empty or contain a zero-packet summary
  object — either is acceptable. The key observable is that the notice is on stderr, not
  embedded in stdout.
- **Notice message format:** The canonical format is
  `"notice: <filename>: 0 packets read from pcapng file"` with an optional parenthetical
  for skip count and/or OPB count. The evaluator checks for the substring
  `"0 packets read from pcapng file"` — it does not require the `"notice: "` prefix to
  be an exact prefix, but the zero-packet substring must be present. The skip-count segment
  (when present) must include the literal integer `2` and the word "skipped". The OPB
  segment (when present) must include the literal integer `1` and the word "obsolete" and
  a reference to "mergecap".

## Failure Guidance

"HOLDOUT LOW: HS-108 (satisfaction: 0.XX) — zero-packet notice contract has defects.
Case A failure (exit non-zero): valid SHB+IDB file is being rejected as an error.
Case A failure (no notice on stderr): PC6 zero-packet notice is absent or gated on skipped_blocks>0 instead of 'valid file + zero packets'.
Case A failure (notice contains skip count or obsolete): parenthetical segment included when skipped_blocks==0 and no OPBs; must be omitted.
Case A failure (notice appears >1 times): one-shot guard is absent; notice must fire exactly once.
Case B failure (skip count not in notice): skipped_blocks counter is not passed to the notice emitter, or the count is wrong (expected 2, got something else).
Case B failure (no notice at all): same as Case A PC6 absence; notice must fire for skipped-blocks files too.
Case C failure (exit 0): EPB-before-IDB is being treated as zero-packet success instead of E-INP-009 error.
Case C failure (notice on stderr): zero-packet notice must NOT appear when the file produced a parse error; notice is for valid-file-zero-packet only (H-4 disambiguation rule, BC-2.01.009 v1.4).
Case D failure (no OPB count in notice): obsolete_packet_blocks counter not tracked or not passed to notice emitter; notice must include OPB count (1) and word 'obsolete'.
Case D failure (no mergecap hint): OPB notice must include a mergecap remediation hint; missing hint means users cannot discover how to recover the packet data.
Case D failure (OPB counted as generic skip): OPB appears in '2 block(s) skipped' path instead of the distinct OPB path; must be separate.
Case E failure (counts collapsed): NRB skip count (2) and OPB count (1) merged into a single count of 3; they must appear as distinct values with distinct labels.
Case E failure (OPB count absent): same as Case D failure; OPB must be counted and reported even when generic skip count is also present.
See BC-2.01.009 PC6, BC-2.01.015 PC9, ADR-009 rev 7 H-4 disambiguation rule, ADR-009 rev 8 OPB-distinction scenarios."
