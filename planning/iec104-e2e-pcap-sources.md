---
document_type: research
producer: research-agent
date: 2026-07-17
title: IEC 60870-5-104 (IEC-104) E2E PCAP Source Candidates
---

# IEC 60870-5-104 (IEC-104) E2E PCAP Source Candidates

## Task

Find authoritative, publicly-downloadable IEC-104 (TCP port 2404) sample captures
to add to the wirerust E2E test corpus (`bin/fetch-e2e-pcaps` +
`tests/fixtures/E2E-PCAPS.md`). Need at least one `.pcap` and at least one native
`.pcapng`. Prefer small, redistributable captures with rich IEC-104 content
(U-frames, I-frame ASDUs, control commands, interrogation).

All URLs below were verified reachable via HTTP HEAD/GET on 2026-07-17 (status +
content-length shown). No files were downloaded.

---

## Recommended shortlist (add these)

Two of these give the required pcap + native pcapng pair. I recommend adding all
four: the two ITI captures are CC-BY-4.0 (cleanly redistributable) and the two
Wireshark wiki captures are the canonical reference samples (and the pcapng is the
only native IEC-104 pcapng I could find).

| Rank | Filename to use | Format | Size | License / redistribution | Why |
|------|-----------------|--------|------|--------------------------|-----|
| 1 | `iec104.pcap` | classic pcap | 10,135 B | Wireshark public sample, no per-file license — credit Wireshark Foundation, treat local-use-only (same handling as `rsasnakeoil2.pcap`, `teardrop.cap`) | Canonical IEC-104 reference sample; U-frames + I-frame ASDUs + general interrogation lifecycle |
| 2 | `iec104-sq.pcapng` | **native pcapng** | 584 B | Wireshark public sample, no per-file license — credit Wireshark Foundation, local-use-only | Only native IEC-104 pcapng found; exercises SQ-bit (sequence-of-IOs) ASDU encoding through the pcapng reader |
| 3 | `iec104-iti-diverse.pcap` | classic pcap | 13,952 B | **CC-BY-4.0** (redistributable with attribution to ITI / Illinois Institute of Technology) | Diverse ASDU type mix; redistributable — safest license in the set |
| 4 | `iec104-iti-dissect.pcap` | classic pcap | 11,409 B | **CC-BY-4.0** (attribution to ITI) | Wireshark-dissector test capture — deliberately broad Type ID / COT coverage incl. control commands |

### Candidate details

#### 1. Wireshark `iec104.pcap` (RECOMMENDED — primary classic-pcap fixture)

- **Direct raw URL (canonical, matches existing corpus GitLab pattern):**
  `https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/iec104.pcap`
- Mirror (302-redirects to the GitLab URL above):
  `https://wiki.wireshark.org/uploads/__moin_import__/attachments/SampleCaptures/iec104.pcap`
- **Verified:** `HTTP/2 200`, `content-length: 10135`, `application/octet-stream`.
- **Size:** ~10 KB (small).
- **Content:** Real IEC 60870-5-104 SCADA session on TCP 2404. Contains U-format
  frames (STARTDT / STOPDT / TESTFR link management), I-format frames carrying
  ASDUs, and the general-interrogation (C_IC, TypeID 100) request/response
  lifecycle (COT 6 act → 7 con → 20 inrogen → 10 actterm). Good coverage of the
  APCI frame-type dispatch and ASDU header extraction paths.
- **License note:** Wireshark SampleCaptures have no per-file license; public
  sample. Credit "Wireshark Foundation", mark local-use-only / not redistributed,
  exactly as `rsasnakeoil2.pcap`, `arp-storm.pcap`, `teardrop.cap`, and the pcapng
  test captures already in `bin/fetch-e2e-pcaps` are handled.

#### 2. Wireshark `IEC104_SQ.pcapng` (RECOMMENDED — native pcapng fixture)

- **Direct raw URL (canonical GitLab pattern):**
  `https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/IEC104_SQ.pcapng`
- Mirror (302-redirects to the GitLab URL above):
  `https://wiki.wireshark.org/uploads/__moin_import__/attachments/SampleCaptures/IEC104_SQ.pcapng`
- **Verified:** `HTTP/2 200`, `content-length: 584`, `application/octet-stream`.
- **Size:** 584 B (tiny — only a few packets, but a genuine native pcapng: SHB/IDB/EPB blocks).
- **Content:** IEC-104 communication log with the **SQ bit set** in the ASDU
  variable structure qualifier (sequence-of-information-objects encoding). This is
  a distinct ASDU-parsing path from the individually-addressed IO case, so it is a
  useful complement to `iec104.pcap`. Small packet count means it is a
  sanity/format fixture rather than a rich behavioral sample.
- **License note:** same as #1 — Wireshark public sample, no per-file license,
  credit Wireshark Foundation, local-use-only.
- **Why this one:** native IEC-104 pcapng captures are genuinely scarce (see
  "pcapng scarcity" below). This is the only native IEC-104 pcapng I located on an
  authoritative, directly-downloadable source.

#### 3. ITI `090813_diverse.pcap` (RECOMMENDED — redistributable pcap)

- **Direct raw URL:**
  `https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/090813_diverse.pcap`
- **Verified:** `HTTP/2 200`, `content-length: 13952`.
- **Size:** ~14 KB (small).
- **Content:** IEC-104 traffic with a diverse mix of ASDU Type IDs (filename +
  directory context). Real capture in the ITI/ICS-Security-Tools ICS pcap corpus
  (the same repo wirerust already uses for the ENIP `enip_test.pcap` and
  `EthernetIP-CIP.pcap` fixtures).
- **License note:** repo license is **CC-BY-4.0** (verified via GitHub license
  API). Redistributable with attribution — "ICS Security Tools, Illinois Institute
  of Technology (ITI)", the same attribution string already used for the ITI ENIP
  captures in `bin/fetch-e2e-pcaps`.

#### 4. ITI `TestDissectIec104.pcap` (RECOMMENDED — redistributable, broad Type ID coverage)

- **Direct raw URL:**
  `https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/TestDissectIec104.pcap`
- **Verified:** `HTTP/2 200`, `content-length: 11409`.
- **Size:** ~11 KB (small).
- **Content:** A Wireshark-dissector *test* capture ("TestDissectIec104") —
  constructed to exercise the IEC-104 dissector across many ASDU Type IDs and
  Causes of Transmission, so it is dense in monitor/control ASDU variety (a good
  stressor for TypeID coverage incl. control commands C_SC/C_DC/C_SE and
  interrogation). Because it is synthetic-for-testing rather than an organic
  session, U-frame session setup may be minimal — pair it with #1 for realistic
  session framing.
- **License note:** CC-BY-4.0 (ITI repo), attribution as in #3.

---

## Secondary / not recommended for now

### ITI `JavaRMI_and_IEC_Misc.pcap`
- URL: `https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/JavaRMI_and_IEC_Misc.pcap`
- Verified `HTTP/2 200`, `content-length: 28889` (~28 KB). CC-BY-4.0 (ITI).
- Mixed Java-RMI + miscellaneous IEC-104 traffic. Noisier / less focused than the
  other two ITI files; only add if you want a mixed-protocol dispatch stressor.

### automayt/ICS-pcap `IEC 60870/iec104/` and `IEC 60870/IEC104_SQ/`
- `iec104.pcap`: `https://raw.githubusercontent.com/automayt/ICS-pcap/master/IEC%2060870/iec104/iec104.pcap` — only **130 bytes** (truncated/trivial; not useful).
- `IEC104_SQ.pcapng`: `https://raw.githubusercontent.com/automayt/ICS-pcap/master/IEC%2060870/IEC104_SQ/IEC104_SQ.pcapng` — **584 bytes**, byte-identical in size to the Wireshark wiki `IEC104_SQ.pcapng` (this repo appears to have copied it from the Wireshark wiki).
- **License:** automayt/ICS-pcap has **NO license file** (verified) → local-use-only, not redistributable. Prefer the authoritative Wireshark-wiki source (#2) over this mirror.

### 4SICS-GeekLounge captures (already in corpus)
- The three `4SICS-GeekLounge-1510{20,21,22}.pcap` files are already fetched by
  `bin/fetch-e2e-pcaps`. They are broad ICS-lab mixed-protocol captures and *may*
  contain some IEC-104 frames, but there is no small dedicated IEC-104-only 4SICS
  slice. No action needed; do not rely on them for targeted IEC-104 coverage.

### Attack / malformed IEC-104 datasets (flagged — large, not directly fetchable)
For U-frame/interrogation coverage the shortlist above is sufficient. If a
dedicated **attack / malformed** IEC-104 fixture is wanted later:
- **IEC 60870-5-104 Intrusion Detection Dataset** — Zenodo record 7108614
  (`https://zenodo.org/records/7108614`). Twelve labeled IEC-104 cyberattacks
  (unauthorized-command injection + DoS) with PCAP files. **Flagged:** PCAPs are
  packaged inside per-entity `.7z` archives (not a direct raw single-file URL) and
  are larger than the "few MB" preference; extraction + slicing required. License:
  check the Zenodo record terms before any redistribution. Not verified as a
  direct raw pcap download.
- Could not locate a single small GitHub-raw labeled IEC-104 *attack* pcap meeting
  the direct-URL + small-size criteria. Marking that gap explicitly.

---

## pcapng scarcity note (explicit)

Native IEC-104 `.pcapng` captures are **scarce**. The only authoritative, directly
downloadable native IEC-104 pcapng found is Wireshark's `IEC104_SQ.pcapng` (584 B,
#2 above) — and it is tiny. If a larger native-pcapng IEC-104 fixture is needed,
the pragmatic path (already used in this corpus for `220703_arp-storm-nrb.pcapng`)
is to re-export one of the ITI classic pcaps to pcapng locally via Wireshark/`editcap`
and add it as a synthetic-derived fixture with documented provenance. That is a
local generation step, not an external source.

---

## Suggested `bin/fetch-e2e-pcaps` entries (for the later wiring step)

Filenames chosen to be self-describing and collision-free within the corpus.
SHA256 values must be computed at fetch time (files not downloaded here).

```
# --- IEC 60870-5-104 (IEC-104) captures (STORY-17x / feature-iec104 e2e) ---
# Wireshark canonical IEC-104 sample: U-frames (STARTDT/STOPDT/TESTFR) + I-frame
# ASDUs + general interrogation (C_IC TypeID 100) lifecycle. TCP 2404.
# Credit: Wireshark Foundation; no per-file license; public sample; not redistributed.
"iec104.pcap|<sha256>|https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/iec104.pcap"
# Native pcapng: IEC-104 with SQ-bit (sequence-of-IOs) ASDU. Small; exercises
# pcapng reader + SQ ASDU path. Credit: Wireshark Foundation; public sample; not redistributed.
"iec104-sq.pcapng|<sha256>|https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/IEC104_SQ.pcapng"
# ITI diverse-ASDU IEC-104 capture (CC-BY-4.0). Credit: ICS Security Tools, Illinois
# Institute of Technology (ITI).
"iec104-iti-diverse.pcap|<sha256>|https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/090813_diverse.pcap"
# ITI dissector-test IEC-104 capture: broad Type ID / COT coverage incl. control
# commands. CC-BY-4.0. Credit: ICS Security Tools, Illinois Institute of Technology (ITI).
"iec104-iti-dissect.pcap|<sha256>|https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/IEC60870-5-104/TestDissectIec104.pcap"
```

---

## Sources

- Wireshark SampleCaptures wiki: <https://wiki.wireshark.org/SampleCaptures> and GitLab mirror <https://gitlab.com/wireshark/wireshark/-/wikis/SampleCaptures>
- Wireshark `iec104.pcap` (verified 200, 10135 B): <https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/iec104.pcap>
- Wireshark `IEC104_SQ.pcapng` (verified 200, 584 B): <https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/IEC104_SQ.pcapng>
- ITI/ICS-Security-Tools IEC60870-5-104 directory (CC-BY-4.0): <https://github.com/ITI/ICS-Security-Tools/tree/master/pcaps/IEC60870-5-104>
- automayt/ICS-pcap `IEC 60870` directory (no license): <https://github.com/automayt/ICS-pcap/tree/master/IEC%2060870>
- Wireshark IEC 60870-5-104 display-filter reference: <https://www.wireshark.org/docs/dfref/i/iec60870_104.html>
- IEC 60870-5-104 Intrusion Detection Dataset (Zenodo, attack pcaps): <https://zenodo.org/records/7108614>
