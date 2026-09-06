# F2/F3 — S7comm PCAP Fixture Sourcing & Licensing Analysis

**Feature:** S7comm (Siemens S7 protocol over ISO-on-TCP, TCP/102)
**Phase:** F2/F3 groundwork — test-fixture acquisition for TDD acceptance criteria + E2E fixtures
**Date:** 2026-09-06
**Agent:** vsdd-factory:research-agent
**Precedent:** `tests/fixtures/iec104-iti-diverse.pcap` (committed, CC-BY-4.0) and the
`tests/fixtures/README.md` / `tests/fixtures/E2E-PCAPS.md` commit policy.

> **Research only.** No files were downloaded, modified, or committed. All sizes/paths
> below are read from source-repository metadata (GitHub API / directory listings), not
> from local copies.

---

## TL;DR / Recommendation

**No public S7comm PCAP was found that is simultaneously (a) real, (b) small, (c) clean
classic-vs-plus content, AND (d) covered by a clearly permissive/first-party
redistribution license with no positive evidence of third-party origin.** This is the
same conclusion the deep-research sweep reached independently.

The single most important finding for this repo:

- The **ITI/ICS-Security-Tools** `pcaps/s7/` directory — which is CC-BY-4.0 at the repo
  level and is *already the exact source wirerust uses for its committed IEC-104 and ENIP
  fixtures* — turns out to be, for S7, **substantially a re-host of the GPLv2 S7comm
  Wireshark-dissector plugin's own `doc/test-traces` directory** (moki-ics/s7commwireshark,
  author Thomas Wiens). Verified: `s7-1200-hmi.pcap`, `S7-1511_db2_var1_HMI.pcap`,
  `S7-1200-Uploading-OB1-TIAV12.pcap`, `V13_1200_*.pcapng` are byte-for-byte the same
  filenames present in the GPLv2 plugin repo.
- Under wirerust's **own F-009 / D-524 precedent** — where `iec104-iti-dissect.pcap` was
  kept **fetch-only (NOT committed)** precisely because its upstream filename
  (`TestDissectIec104.pcap`) indicated Wireshark-dissector-test-suite origin — these S7
  captures fall into the **same DO-NOT-COMMIT class**. The CC-BY wrapper does not cleanly
  launder GPLv2 upstream test assets.

### Recommended fixture strategy (matches existing repo pattern)

1. **Committed AC / unit + small E2E fixtures → SYNTHESIZE.** Hand-craft minimal
   TPKT (RFC 1006) + COTP (ISO 8073 / ITU-T X.224) + S7comm-PDU byte fixtures with a
   first-party generator (`tests/fixtures/mk_s7comm_pcap.py`), dedicated under CC0/MIT.
   This is exactly what wirerust already does for `modbus-large.pcap`
   (`mk_modbus_large_pcap.py`) and the synthetic SPB pcapng tests. It is the **only clean
   MIT/Apache commit posture** and gives byte-level determinism for TDD ground-truth.
2. **Real-world E2E validation → fetch-only, gitignored** under
   `tests/fixtures/local-samples/`, wired into `bin/fetch-e2e-pcaps` with SHA-256 pins —
   the same treatment as `iec104-iti-dissect.pcap`, `dnp3dataset_capture.pcap`, and the
   4SICS captures. Best candidates: the ITI `pcaps/s7/` set and cisagov
   `testing/traces/*.pcap`. **Never committed.**
3. **Do NOT commit** any Wireshark-wiki S7comm capture, any SourceForge s7commwireshark
   sample, or any ITI `s7/` file — all carry GPL/GPLv2 or third-party-origin evidence.

A good committed synthetic set to target (mirrors `iec104-iti-diverse.pcap`'s role):
Setup Communication (0xF0), Read Var (0x04), Write Var (0x05), PLC STOP, PLC START/Warm
restart, Request Download / Download Block / Download Ended, Upload, and a Userdata/SZL
read (0x00/0x04 subfunction) — classic S7comm (0x32). Optionally one S7comm-plus (0x72)
S7-1200 skeleton for dispatch coverage.

---

## Ranked candidate table

| # | Source | Contents (S7 function codes / classic 0x32 vs plus 0x72) | Format / size | Redistribution license | Commit-safe? | Notes |
|---|--------|----------------------------------------------------------|---------------|------------------------|--------------|-------|
| 1 | **Synthetic / first-party** — new `tests/fixtures/mk_s7comm_pcap.py` (to be written) | Whatever we encode: Setup Comm 0xF0, Read Var 0x04, Write Var 0x05, PLC STOP/START, Download/Upload, SZL/Userdata. Classic S7comm 0x32 (and optionally a 0x72 skeleton). | pcap, hand-sized (bytes–KB) | **CC0 / MIT / Apache — first-party, we own it** | ✅ **YES — recommended primary** | Byte-level deterministic; no third-party provenance risk. Precedent: `modbus-large.pcap` (`mk_modbus_large_pcap.py`), synthetic SPB tests. Build over TPKT/RFC1006 + COTP/ISO8073 + S7comm PDU. |
| 2 | **ITI/ICS-Security-Tools** `pcaps/s7/` — <https://github.com/ITI/ICS-Security-Tools/tree/master/pcaps/s7> | 34 captures. Classic S7comm 0x32: `snap7_s300_*`, `step7_s300_*`, `tia_s300_*`, `wincc_s300/s400_*` (S7-300/400 targets). S7comm-plus 0x72: `s7-1200-hmi.pcap`, `S7-1200-Uploading-OB1-TIAV12.pcap`, `S7-1511_*`, `V13_1200_*` (S7-1200/1500). Covers Read/Write Var, Setup Comm, STOP, copy-RAM-to-ROM, download OB1/HwConfig, upload OB1, auth password, flash LED, read diag/SZL, firmware update, WinCC alarms. | pcap + pcapng; 216 B – 14.3 MB (most 216 B – 74 KB; two firmware-update files 14.27 MB each) | Repo LICENSE.md = **CC-BY-4.0** … **BUT** the S7 files are a re-host of the **GPLv2** s7comm-Wireshark-plugin `doc/test-traces` (moki-ics/s7commwireshark, Thomas Wiens) — *verified filename match*. | ⚠️ **DO-NOT-COMMIT** (fetch-only) | Positive evidence of third-party GPLv2 origin → identical situation to `iec104-iti-dissect.pcap` (F-009 D-524 fetch-only ruling). Use gitignored under `local-samples/` + `bin/fetch-e2e-pcaps`. Excellent E2E coverage; **best fetch-only real source.** |
| 3 | **cisagov/icsnpp-s7comm** `testing/traces/` — <https://github.com/cisagov/icsnpp-s7comm/tree/main/testing/traces> | `s7comm_plus.pcap` (S7comm-plus 0x72), `s7ident.pcap` (identification/SZL), `snap7.pcap` (classic S7comm 0x32 via snap7 — likely Setup Comm + Read/Write Var). Per-file function inventory not published; confirm on download. | pcap; sizes not exposed by API tree (small — Zeek unit-test traces) | Repo `LICENSE.txt` = **BSD-3-Clause**, © 2023 Battelle Energy Alliance. **No trace-file-specific license or provenance statement.** | ⚠️ **DO-NOT-COMMIT** without written INL/CISA confirmation | BSD-3-Clause is permissive *for the analyzer code*; the traces have **no separate grant**. Same gap the deep-research flagged. Safe as fetch-only E2E. If INL confirms the traces are Battelle-generated & BSD-covered, they become the cleanest small real fixtures. |
| 4 | **Netresec 4SICS Geek Lounge** — <https://www.netresec.com/?page=PCAP4SICS> | Mixed ICS lab; contains SIMATIC S7-1200 @ 192.168.88.30:102 + another S7 @ 10.10.10.10:102. Function-code inventory not published (needs dissection). Classic vs plus unconfirmed. | pcap; 25 MB / 134 MB / 200 MB | **No named license.** Netresec requests credit to CS3Sthlm/4SICS on redistribution but grants no MIT/BSD/CC/public-domain license. | ❌ **DO-NOT-COMMIT** | Already present in wirerust as **fetch-only** (`E2E-PCAPS.md`, credit CS3Sthlm/4SICS). Too large to commit regardless. Keep fetch-only; S7 content is incidental/sparse. |
| 5 | **Wireshark SampleCaptures wiki** — <https://wiki.wireshark.org/S7comm> (`s7comm_downloading_block_db1.pcap`, `s7comm_program_blocklist_onlineview.pcap`, `s7comm_reading_plc_status.pcap`, `s7comm_reading_setting_plc_time.pcap`, `s7comm_varservice_libnodavedemo.pcap`, `_bench`) | Classic S7comm 0x32: Setup Comm 0xF0; download program block DB1 (Request Download/Download Block/Download Ended); program-block list online view; read PLC status (Userdata/SZL); read+set PLC time; variable-service Read Var 0x04 across memory areas. Good, clearly-labeled function coverage. | classic pcap; small (2.8 KB – 1.57 MB — `_bench` is the large one) | Wiki site-wide content = **GNU GPL**; **no per-file permissive license**. | ❌ **DO-NOT-COMMIT** | Same "de-facto Wireshark class" the repo already documents (`README.md` licensing notice). Usable **fetch-only** only. Note: ITI `pcaps/s7/` also mirrors five of these under the same names — mirroring does not improve the license. |
| 6 | **SourceForge s7commwireshark Sample-captures** — <https://sourceforge.net/projects/s7commwireshark/files/Sample-captures/> | 4 classic S7comm 0x32 captures: (1) Read Var DB1.DBD0; (2) cyclic vars, reads+writes + SZL reads; (3) VAT reads MB100/MW200/MD300/M400.0; (4) password login + download DB1 to S7-414. Cleanest function labeling of any single source. | classic pcap; 3.5 KB / 72.5 KB / 4.4 KB / 7.6 KB | Plugin code = **GPLv2**; no capture-specific permissive license. | ❌ **DO-NOT-COMMIT** | GPLv2 provenance. Fetch-only at most. This is the upstream of the ITI mirror (row 2). |
| 7 | **QUT 2017 S7comm** — <https://github.com/qut-infosec/2017QUT_S7comm> | Control set + manipulated/attack set from an S7comm testbed (Rodofile thesis). Classic S7comm implied; plus not established. No function-code breakdown published. | pcap + process logs; sizes not exposed | **No LICENSE / CC notice in repo.** Thesis making data "publicly accessible" ≠ redistribution grant. | ❌ **DO-NOT-COMMIT** | License unclear. Fetch-only for research validation only, not redistributed (same class as `dnp3dataset_capture.pcap`). |
| 8 | **KIT — S7 Data Modification Attacks** — <https://publikationen.bibliothek.kit.edu/1000181992> | S7-1512/1516 BSEND/BRECV data-block traffic + data-modification attacks. Parsed S7 data-block fields. | **Gzip'd Apache Parquet — NOT pcap/pcapng** | **CC BY 4.0** (explicit — commit-safe *license*) | ❌ Not a PCAP fixture | License is clean, but format is not a capture. Converting Parquet→pcap creates an attributed CC-BY derivative requiring validation — not worth it vs. synthesizing. Noted for completeness. |
| 9 | **Univ. Murcia "Electra" S7Comm dataset** — <http://perception.inf.um.es/ICS-datasets/> | Electric-traction substation S7comm: read/write, command manipulation, replay, response manipulation, MITM. ~2.81M S7comm records. | **CSV/derived records — NOT pcap** | No explicit reuse license exposed. | ❌ Not a PCAP fixture / license unclear | Records, not captures. Excluded. |
| 10 | **Univ. Coimbra `ICS_PCAPS`** — <https://github.com/tjcruz-dei/ICS_PCAPS> | Modbus/TCP SCADA + generic attacks. **No S7 capture present.** | pcap | Code MIT / content CC-BY-3.0 | ❌ No S7 content | Clean license but supplies no S7 fixture. |
| 11 | **scy-phy/bro-cip-enip** — <https://github.com/scy-phy/bro-cip-enip> | ENIP/CIP only. **No S7comm.** | pcap/pcapng | MIT | ❌ No S7 content | This is wirerust's MIT gold source for ENIP fixtures; it has nothing for S7. Listed to close the loop. |
| 12 | **malware-traffic-analysis.net** — <https://www.malware-traffic-analysis.net/> | **No S7comm / Siemens PLC / TCP-102 capture found.** Site is Windows-malware infection traffic. | password-protected zips | No general redistribution license. | ❌ Not a candidate | Also blocked by the repo's "no live malware C2/exploit traffic" rule (`README.md`). Excluded. |

---

## Why synthesize (the safer path here)

Redistribution licensing for real S7comm captures is uniformly murky:

- Every "real" small S7comm capture with clean, labeled function coverage traces back to
  **one GPLv2 lineage** (Thomas Wiens' s7comm Wireshark dissector) — via the Wireshark
  wiki, SourceForge, moki-ics, OR the ITI CC-BY re-host. None is permissively licensed at
  the file level.
- The only *explicitly* permissive/first-party S7 dataset found with a clean license
  (KIT, CC-BY-4.0) is **Parquet, not pcap**.
- wirerust's committed-fixture bar is deliberately strict: **redistributable license
  (CC-BY-4.0/MIT), ≤100 KB, and *no positive evidence of non-redistributable third-party
  origin*** (`E2E-PCAPS.md` "Adding a capture"). The S7 real-capture pool fails the third
  clause almost everywhere.

Hand-crafted S7comm fixtures avoid all of this and are well-precedented in this repo
(`modbus-large.pcap` via `mk_modbus_large_pcap.py`; synthetic SPB pcapng tests;
`bin/fetch-e2e-pcaps` "regenerates the synthetic one"). S7comm-over-TCP/102 is a
well-documented stack — TPKT (RFC 1006) 4-byte header + COTP (ISO 8073 / ITU-T X.224)
DT-data TPDU + S7comm PDU (protocol id 0x32, ROSCTR job/ack-data/userdata, parameter +
data blocks) — entirely feasible to emit deterministically for the AC ground-truth set.

### Suggested synthetic fixture matrix (for the committed set)

| Fixture (proposed) | S7comm content | Purpose |
|--------------------|----------------|---------|
| `s7comm-setup-read-write.pcap` | Setup Comm 0xF0 → Read Var 0x04 → Write Var 0x05 (classic 0x32) | core happy-path AC ground-truth |
| `s7comm-plc-control.pcap` | PLC STOP + PLC START/warm-restart (0x32 job, control functions) | control-command detection AC |
| `s7comm-download-upload.pcap` | Request Download / Download Block / Download Ended; Upload sequence | program transfer detection AC |
| `s7comm-userdata-szl.pcap` | Userdata (0x07) SZL/diagnostic read | userdata/SZL parse path |
| `s7commplus-s71200-skel.pcap` (optional) | S7comm-plus 0x72 header skeleton | plus-vs-classic dispatch coverage |

Keep each ≤ a few KB; document the generator + exact byte layout in the fixture header
and `tests/fixtures/README.md`, dedicate under CC0/MIT.

---

## Fetch-only real captures (recommended for `local-samples/` + `bin/fetch-e2e-pcaps`)

Use these for *manual/CI E2E robustness validation only* — gitignored, SHA-pinned, never
committed — exactly like `iec104-iti-dissect.pcap`:

| Candidate | Direct URL (raw) | Why fetch-only |
|-----------|------------------|----------------|
| ITI `snap7_s300_everything.pcapng` (8.3 KB) | `https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/s7/snap7_s300_everything.pcapng` | GPLv2 plugin-trace origin (D-524 class) |
| ITI `tia_s300_downloadOb1.pcapng` (14.3 KB) | `https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/s7/tia_s300_downloadOb1.pcapng` | program download; GPLv2-origin |
| ITI `S7-1200-Uploading-OB1-TIAV12.pcap` (19.9 KB) | `https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/s7/S7-1200-Uploading-OB1-TIAV12.pcap` | S7comm-plus upload; GPLv2-origin |
| ITI `wincc_s400_production.pcapng` (294 KB) | `https://raw.githubusercontent.com/ITI/ICS-Security-Tools/master/pcaps/s7/wincc_s400_production.pcapng` | larger real WinCC/S7-400 baseline |
| cisagov `s7comm_plus.pcap` | `https://raw.githubusercontent.com/cisagov/icsnpp-s7comm/main/testing/traces/s7comm_plus.pcap` | S7comm-plus; BSD repo but no trace-file license |
| cisagov `snap7.pcap` | `https://raw.githubusercontent.com/cisagov/icsnpp-s7comm/main/testing/traces/snap7.pcap` | classic S7comm; same license gap |
| cisagov `s7ident.pcap` | `https://raw.githubusercontent.com/cisagov/icsnpp-s7comm/main/testing/traces/s7ident.pcap` | S7 identification/SZL |

> **DO-NOT-COMMIT flag applies to every row above.** They are validation inputs, not
> redistributed artifacts. Record provenance + SHA-256 in `E2E-PCAPS.md` when added, and
> credit the upstream (ITI/ICS-Security-Tools CC-BY-4.0 wrapper **and** Thomas Wiens /
> s7comm-wireshark GPLv2 for the S7 traces; Battelle Energy Alliance / INL for cisagov).

---

## Full ITI `pcaps/s7/` inventory (reference — all DO-NOT-COMMIT, fetch-only)

Source: `https://github.com/ITI/ICS-Security-Tools/tree/master/pcaps/s7` (CC-BY-4.0 repo
wrapper; S7 files = GPLv2 s7comm-wireshark test-traces re-host).

| File | Size (bytes) | Likely dialect | Likely content |
|------|-------------|----------------|----------------|
| snap7_s300_setupCommunication.pcapng | 216 | classic 0x32 | Setup Communication |
| snap7_s300_stop.pcapng | 217 | classic 0x32 | PLC STOP |
| snap7_s300_readVar.pcapng | 220 | classic 0x32 | Read Var |
| step7_s300_copyRamToRom.pcapng | 219 | classic 0x32 | copy RAM→ROM |
| step7_s300_stop.pcapng | 431 | classic 0x32 | PLC STOP |
| step7_s300_AuthPassword.pcapng | 722 | classic 0x32 | password auth |
| step7_s300_rwVarTab.pcapng | 2,638 | classic 0x32 | read/write VarTab |
| step7_s300_readVarTab.pcapng | 56,508 | classic 0x32 | read VarTab |
| step7_s300_readDiagData.pcapng | 73,759 | classic 0x32 | read diag/SZL |
| step7_s300_download.pcapng | 13,005 | classic 0x32 | block download |
| snap7_s300_everything.pcapng | 8,260 | classic 0x32 | mixed ops |
| tia_s300_flashLed.pcapng | 484 | classic 0x32 | flash LED |
| tia_s300_downloadOb1.pcapng | 14,297 | classic 0x32 | download OB1 |
| tia_s300_downloadHwConfig.pcapng | 18,792 | classic 0x32 | download HW config |
| tia_s300_goOnline.pcapng | 62,240 | classic 0x32 | go-online session |
| tia_s300_updateFirmware.pcapng | 14,266,140 | classic 0x32 | firmware update (large) |
| tia_s300_updateFirmware_2.pcapng | 14,266,140 | classic 0x32 | firmware update (large) |
| wincc_s300_setup-alarm-read.pcapng | 24,442 | classic 0x32 | WinCC alarm read |
| wincc_s300_setup-alarm-read_2.pcapng | 63,830 | classic 0x32 | WinCC alarm read |
| wincc_s300_setup-alarm-read-write.pcapng | 21,368 | classic 0x32 | WinCC alarm read/write |
| wincc_s400_production.pcapng | 294,072 | classic 0x32 | WinCC/S7-400 production baseline |
| s7-1200-hmi.pcap | 11,559 | plus 0x72 | S7-1200 HMI |
| S7-1200-Uploading-OB1-TIAV12.pcap | 19,895 | plus 0x72 | upload OB1 (TIA V12) |
| S7-1511_db2_var1_HMI.pcap | 6,024 | plus 0x72 | S7-1500 DB var HMI |
| S7-1511_db3_var1_HMI.pcap | 7,334 | plus 0x72 | S7-1500 DB var HMI |
| S7-1511_db6w0_HMI.pcap | 3,396 | plus 0x72 | S7-1500 DB var HMI |
| S7-1511-opc-request-all-types.pcap | 10,270 | plus 0x72 | OPC request, all types |
| V13_1200_..._Timer_sync.pcapng | 19,948 | plus 0x72 | S7-1200 cyclic/timer sim |
| V13_1200_..._FehlerbeiMW100.pcapng | 18,488 | plus 0x72 | S7-1200 cyclic + fault |
| s7comm_downloading_block_db1.pcap | 9,523 | classic 0x32 | download DB1 (Wireshark-wiki mirror) |
| s7comm_program_blocklist_onlineview.pcap | 13,981 | classic 0x32 | block-list online view (wiki mirror) |
| s7comm_reading_plc_status.pcap | 25,112 | classic 0x32 | read PLC status/SZL (wiki mirror) |
| s7comm_reading_setting_plc_time.pcap | 5,355 | classic 0x32 | read/set PLC time (wiki mirror) |
| s7comm_varservice_libnodavedemo.pcap | 2,846 | classic 0x32 | Read Var / libnodave (wiki mirror) |
| s7comm_varservice_libnodavedemo_bench.pcap | 1,566,710 | classic 0x32 | var-service benchmark (wiki mirror) |

*Dialect column is inferred from target model (S7-300/400 → classic 0x32; S7-1200/1500 →
S7comm-plus 0x72) and MUST be confirmed by byte-level dissection on download — do not
treat as authoritative.*

---

## Open items / to confirm on download (F3)

1. **Confirm classic-vs-plus per file** by dissecting the protocol-id byte (0x32 vs 0x72)
   — the model-based inference above is a heuristic, not a guarantee (TIA→S7-300 can be
   classic even from a V1x engineering station).
2. **cisagov trace license**: optionally request written INL/CISA confirmation that
   `testing/traces/*.pcap` are Battelle-generated and covered by the repo BSD-3-Clause. If
   granted, `s7comm_plus.pcap` / `snap7.pcap` / `s7ident.pcap` (small, BSD) would become
   the *only* commit-safe real captures and could replace some synthetic fixtures.
3. **Build `mk_s7comm_pcap.py`** (F3 implementation task) covering the synthetic matrix
   above; wire the regenerated fixtures into `bin/fetch-e2e-pcaps` per repo convention.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source sweep of all S7comm PCAP source families (4SICS/Netresec, S4/ICS-village, MTA, cisagov icsnpp-s7comm, Wireshark SampleCaptures, academic SCADA datasets) with per-source license verification (`reasoning_effort` implicit high via `search_context_size: high`). |
| WebFetch | 8 | Verified: ITI/ICS-Security-Tools `pcaps/` + `pcaps/s7/` directory contents + exact file sizes (GitHub API); ITI repo LICENSE.md (= CC-BY-4.0); cisagov `testing/traces/` paths + LICENSE.txt (= BSD-3-Clause, Battelle); moki-ics/s7commwireshark `doc/test-traces` filename cross-match (GPLv2 origin proof). |
| Read | 3 | Existing fixture precedent: `tests/fixtures/README.md`, `tests/fixtures/E2E-PCAPS.md` (commit policy, iec104/ENIP provenance), CLAUDE.md. |
| Glob | 3 | Located existing fixtures + IEC-104 precedent artifacts. |
| Training data | 2 areas | S7comm/TPKT/COTP protocol-stack structure (RFC 1006 / ISO 8073) and classic-0x32-vs-plus-0x72 dialect heuristics — flagged explicitly; the dialect column requires byte-level confirmation. |

**Total MCP tool calls:** 1 (`perplexity_research`, high context) + 8 WebFetch verifications.
**Training data reliance:** low — every license/provenance claim is web-verified against the
source repository; only protocol-structure background and the model-based dialect heuristic
draw on training data, and both are flagged as to-confirm.
