# Research: Public Real-World EtherNet/IP + CIP PCAP Sources for SS-17 Holdouts (HS-110..122)

- **Type:** general (technology / corpus location)
- **Date:** 2026-06-27
- **Researcher:** vsdd-factory:research-agent
- **Scope:** LOCATE publicly-available, REAL-WORLD ENIP/CIP captures (TCP/44818, UDP/2222) and map
  them to the 13 EtherNet/IP holdout scenarios. This is a research/citation pass — no files were
  downloaded, hashed, or executed. A follow-on step performs acquisition + verification.

> **Provenance discipline.** Every candidate below is cited to a live URL. Where a claim about
> ENIP/CIP *content* could not be verified against the source (only inferred from filename), it is
> marked **[content-inferred]**. File sizes are from the GitHub Contents API at research time.

---

## TL;DR — Top recommendation

The single best found source is **`scy-phy/bro-cip-enip`** (`testing/btest/Traces/enip/`), an
**MIT-licensed** corpus from SUTD's SCy-Phy lab built around the **SWaT testbed** (Allen-Bradley
ControlLogix / 1756-ENBT). It contains a full spread of normal ENIP/CIP traffic **and** a
`enip_metasploit.pcapng` capture of the Metasploit `auxiliary/admin/scada/multi_cip_command`
module issuing **STOPCPU / CRASHCPU / CRASHETHER / RESETETHER** — i.e. real CIP **Stop** and
**Reset** command traffic. This one repo plausibly satisfies HS-110, HS-111, HS-112, HS-114,
HS-115, HS-116, HS-120, and both halves of HS-122, under a clean redistributable license.

The **canonical-frame holdouts that pin exact byte layouts (HS-110 partial, HS-118, HS-119) and
the synthetic DoS-bound holdouts (HS-117, HS-121, HS-113) are NOT well served by found
captures** — they need crafted fixtures (the holdout specs already declare F4 fixture obligations
for these). See the satisfaction matrix at the end.

---

## Ranked candidate captures

| # | Capture (file) | Source URL | ENIP/CIP content | Size | License | Satisfies HS |
|---|----------------|-----------|------------------|------|---------|--------------|
| 1 | `enip_metasploit.pcapng` | github.com/scy-phy/bro-cip-enip `testing/btest/Traces/enip/` | Metasploit `multi_cip_command`: **STOPCPU (CIP Stop 0x07), RESETETHER/CRASHETHER (CIP Reset/crash), CRASHCPU** vs Allen-Bradley PLC. SendRRData + 0x00B2 CIP service requests. **Verified** via repo's `detect-metasploit.bro` signatures. | 383 KB | **MIT** (redistributable w/ notice) | **HS-111 (T0858)**, **HS-112 (T0816)**, HS-110, HS-120, **HS-122 Case B** |
| 2 | `enip_enum_attr_PLC.pcapng` / `enip_enumarate_plc1_tags.pcapng` | same repo, same folder | CIP enumeration: GetAttribute reads against Identity/tag objects. **[content-inferred + repo-context]** Likely T0888 (Identity reads) and read-class traffic. | 75 KB / 853 KB | MIT | HS-115 (T0888 candidate), HS-122 Case B (T0888), HS-120 |
| 3 | `enip_rw_dummy_tag.pcapng`, `enip_rw_dummy_tag_priv_violation.pcapng`, `enip_rw_attr_plc1_priv_violation.pcapng`, `enip_write_read_tag_bad.pcapng` | same repo, same folder | CIP read/write tag ops incl. **privilege-violation / bad** responses → CIP **error responses** (general_status != 0). **[content-inferred]** Candidate T0888 error-burst + T0836 write source. | 2.8–13 KB | MIT | HS-113 (write src), **HS-115 (T0888 error burst)**, HS-122 |
| 4 | `enip_connect_to_plc1_and_upload.pcapng`, `enip_upload_plc1.pcapng` | same repo, same folder | CIP connection setup + program upload → **ForwardOpen (0x54)/ForwardClose (0x4E)** Connection Manager on 0x00B2. **[content-inferred]** | 3.4 MB / 4.0 MB | MIT | **HS-116**, HS-110, HS-120, HS-122 |
| 5 | `enip_readDI_WIFI_PLC_1.pcapng`, `enip_read_tags.pcapng`, `enip_read_P201AUTO.pcapng`, `enip_mitmcapturePLC1.pcapng` | same repo, same folder | **Normal** read/poll traffic from SWaT operation (no Stop/Reset). Known-good candidate. **[content-inferred]** | 1.7 KB–20.8 MB | MIT | **HS-122 Case A (known-good)**, HS-110, HS-120 |
| 6 | `enip_mitm_hmi-plc1_reboot-hmi-vnc.pcapng` | same repo, same folder | MITM scenario incl. **reboot** of PLC → candidate CIP Reset (T0816). **[content-inferred from filename]** | 2.9 MB | MIT | HS-112 (secondary), HS-122 Case B |
| 7 | `ControlLogix_Logix5000_download_upload_run.pcap` | github.com/EmreEkin/ICS-Pcaps `Ethernet_IP/` | **Real** Allen-Bradley ControlLogix engineering session: program download, upload, **mode change to RUN** (CIP services on 0x00B2). **[content-inferred from filename + Rockwell convention]** | 79 KB | **No license stated** (ICS Defense/Savunma) — redistribution unclear | HS-110, HS-120, HS-122 (mode-change/write src) |
| 8 | `ControlLogix_FactoryTalk_HMI.pcap` | same repo | Real ControlLogix↔FactoryTalk HMI polling = normal reads. Known-good candidate. **[content-inferred]** | 98 KB | No license stated | **HS-122 Case A (known-good)**, HS-120 |
| 9 | `ENIP_CIP-CM.pcap` | same repo | EtherNet/IP **CIP Connection Manager** = ForwardOpen/ForwardClose exchange. **[content-inferred from "CM"]** | 2.8 KB | No license stated | **HS-116**, HS-110 |
| 10 | `cip_stop_plc.pcap` | github.com/ITI/ICS-Security-Tools `pcaps/EthernetIP/` | Filename indicates **CIP Stop PLC** command. **[content-inferred — README only documents `EthernetIP-CIP.pcap` provenance]** Needs verification. | unknown | **CC-BY-4.0** (redistributable w/ attribution) | HS-111 (T0858) candidate, HS-122 Case B |
| 11 | `cip_start_plc.pcap` | same repo | CIP Start/Run command. **[content-inferred]** Known-good-ish (start, not stop). | unknown | CC-BY-4.0 | HS-122, HS-120 |
| 12 | `cip_unlock_cpu.pcap` | same repo | CIP CPU unlock sequence. **[content-inferred]** | unknown | CC-BY-4.0 | HS-122, HS-120 |
| 13 | `EthernetIP-CIP.pcap` | same repo / mirrored in EmreEkin | General ENIP+CIP sample (sourced from CloudShark `76038eaa4a3b` per repo README). RegisterSession + SendRRData + CIP services. **Verified** provenance link in README. | ~2.0 MB | CC-BY-4.0 (ITI copy) | HS-110, HS-120, HS-122 |
| 14 | `cip-multiple-1.pcap`, `cip-multiple-2.pcap`, `cip-eth-set-2.pcap`, `enip_test.pcap` | same repo | Multiple-service CIP requests / **SetAttribute** ("set") traffic — candidate write source. **[content-inferred from filenames]** | unknown | CC-BY-4.0 | HS-113 (write src) candidate, HS-110, HS-120 |

### Sources verified to be NOT usable / not what was hoped

| Candidate | Why excluded |
|-----------|--------------|
| `automayt/ICS-pcap` `ETHERNET_IP/digitalbond pcaps/CL5000EIP-*` (Remote-Mode-Change, Reboot-or-Restart, Software-Download/Upload, Lock/Unlock-PLC, View-Device-Status, etc.) | **Every `.pcap` is 131 bytes** — a global pcap header + at most a stub packet. The actual captured behavior lives in the sibling Bro `conn.log`/`weird.log`, NOT in the pcap. Despite ideal filenames (these are the DigitalBond/Quickdraw ControlLogix attack scenarios), the pcaps carry no usable ENIP payload. **Also: repo has NO LICENSE file** (only `README.md` + `AdditionalNotes.txt`) → redistribution/use rights unclear. Verified via GitHub Contents API. |
| `cpppo` (github.com/pjkundert/cpppo) | Ships **no committed `.pcap` files**. It is a Python ENIP/CIP **simulator + client**; you would *run* it to *generate* a capture (GetAttributeSingle `@1/1/7` against the Identity object). This is a **fixture-generation tool**, not a found real-world capture. Useful for HS-115/HS-122 Case B fixture creation if needed (GPLv3). |
| Electra dataset (perception.inf.um.es/ICS-datasets) | HTTP-only host (HTTPS refused at research time). Prior literature indicates it is predominantly **Modbus/S7** and largely distributed as **CSV flow features**, not raw ENIP pcap. **Not confirmed to contain ENIP pcap** — flag inconclusive. |
| iTrust SWaT/WADI raw datasets (itrust.sutd.edu.sg) | **Registration-gated** (request form, signed agreement). The *public, MIT-licensed* slice of SWaT ENIP traffic we actually need is already mirrored in candidate #1–#6 (`scy-phy/bro-cip-enip`), so the gated dataset is not required. |
| Lemay & Fernandez SCADA dataset (USENIX CSET'16) | **Modbus-focused**, not ENIP/CIP. Not applicable. |
| WUSTL-IIoT, MSU/ORNL (Morris) datasets | Primarily Modbus/RTU + CSV feature sets; no confirmed ENIP pcap. Not applicable. |
| Wireshark SampleCaptures wiki (wiki.wireshark.org / gitlab mirror) | An ENIP/CIP attachment is referenced anecdotally but **could not be confirmed/located** on the current GitLab-hosted wiki via automated fetch (page is very long; attachment not surfaced). **Inconclusive** — a manual browse + Ctrl-F "EtherNet/IP" is recommended before relying on it. The ITI repo (#10–#14) is the reliable substitute. |

---

## Holdout-by-holdout satisfaction assessment

| HS | Need | Best found real capture | Verdict |
|----|------|--------------------------|---------|
| **HS-110** canonical LE header decode | any valid ENIP frame (SendRRData/RegisterSession) on 44818 | #1, #5, #7, #13 (any real ENIP frame) | **CAN** satisfy *Case A* with a real capture. **Cases B/C (exact 28-byte vector, 23-byte truncation) need the crafted fixture** the spec already mandates. |
| **HS-111** CIP Stop (0x07) → T0858 | CIP Stop service request | **#1 `enip_metasploit.pcapng` (STOPCPU — verified)**; #10 `cip_stop_plc.pcap` (inferred) | **CAN** — strong, verified candidate (#1). |
| **HS-112** CIP Reset (0x05) → T0816 | CIP Reset service request | **#1 `enip_metasploit.pcapng` (RESETETHER/CRASHETHER — verified module)**; #6 reboot (inferred) | **CAN** — #1 carries Reset-class commands. Verify the exact 0x05 service byte on extraction. |
| **HS-113** write burst >50/1s → T0836 | 51 CIP SetAttribute writes within 1s | #3/#14 (write traffic) — but **none guaranteed to hit >50 writes in a 1s window** | **PARTIAL / NO.** Found captures have writes but the **rate/timing boundary (51 in 1s) is synthetic**. The spec already declares a crafted-fixture obligation. Use found captures only for the write-detection sanity, not the threshold. |
| **HS-114** ListIdentity (0x0063) → T0846 | ENIP ListIdentity request | #2 (enumeration) **[content-inferred]**; NMAP `enip-enumerate.nse` capture (generatable) | **LIKELY** — enumeration captures usually include ListIdentity; **verify command=0x0063 on extraction**. If absent, generate via NMAP/cpppo. |
| **HS-115** error burst >5/10s → T0888 | >5 CIP error responses (general_status != 0) | #3 priv-violation / `*_bad` / `*priv_violation` captures **[content-inferred]** | **LIKELY** for *presence* of error responses; the **>5-in-10s burst boundary is synthetic**. Found captures support the detection path; the threshold case needs a fixture. |
| **HS-116** ForwardOpen/Close on 0x00B2 | CIP 0x54 / 0x4E | **#9 `ENIP_CIP-CM.pcap`**, #4 connect/upload captures | **CAN** — Connection Manager captures contain ForwardOpen/Close. Verify service bytes 0x54/0x4E. |
| **HS-117** malformed/garbage → T0814 | ≥3 malformed frames on 44818 | none (real captures are well-formed by design) | **NO found source.** Crafted fixture (spec already mandates) — deliberate garbage frames. |
| **HS-118** oversize declared frame | oversized ENIP length field | none | **NO found source.** Crafted fixture required (oversized `length` field, carry/skip path). |
| **HS-119** CIP req on 0x00B1 (negative) | CIP request misplaced in 0x00B1 connected item | none (protocol-illegal in real traffic — spec notes a ForwardOpen in 0x00B1 is a protocol violation) | **NO found source.** Crafted fixture required (negative control). |
| **HS-120** dispatch on 44818 | any 44818 traffic | **any of #1–#14** | **CAN** — trivially satisfied by every candidate. |
| **HS-121** max-findings DoS bound | high-volume synthetic | none (synthetic by design) | **NO / N/A.** Spec marks this synthetic; no pcap needed. |
| **HS-122** real-world corpus (Case A good + Case B problematic) | known-good + known-problematic real captures | **Case A:** #5/#8 (normal reads/HMI poll). **Case B:** **#1 metasploit (verified Stop/Reset)**, #2 enumeration (T0846/T0888) | **CAN** — fully satisfiable. Pick #5 or #8 for Case A (verify zero Stop/Reset), #1 for Case B (verified attack commands). |

### Summary verdict

- **Satisfiable by FOUND real captures (8/13):** HS-110 (Case A), HS-111, HS-112, HS-114*, HS-116, HS-120, HS-122 (both cases), and HS-115* (*presence verification pending extraction).
- **Synthetic / fixture-only (5/13):** HS-113 (rate boundary), HS-117 (malformed), HS-118 (oversize), HS-119 (0x00B1 negative), HS-121 (DoS bound). These already carry F4 fixture-creation obligations in their specs — no real-pcap source is expected or appropriate.

The two genuinely *threshold-boundary* holdouts where a real capture is unlikely to land exactly on
51-writes-in-1s (HS-113) or 6-errors-in-10s (HS-115) should keep their crafted fixtures for the
**boundary** case; found captures (#3) can additionally serve as a real-world *sanity* corpus for
the underlying write/error detection paths.

---

## Acquisition guidance for the follow-on step (not performed here)

1. **Primary pull (clean license):** clone `scy-phy/bro-cip-enip` (MIT) and use
   `testing/btest/Traces/enip/*.pcapng`. Record per-file SHA256. The repo's
   `scripts/policy/protocols/enip/detect-metasploit.bro` documents the exact CIP STOPCPU byte
   signature (`0x52,0x02,0x20,0x06,0x24,0x01,...`) — use it to confirm the Stop/Reset bytes for
   HS-111/HS-112 before declaring satisfaction.
2. **Secondary (CC-BY-4.0):** ITI `cip_stop_plc.pcap` / `ENIP_CIP-CM.pcap` / `EthernetIP-CIP.pcap`
   — attribute ITI/ICS-Security-Tools per CC-BY.
3. **Tertiary (license unclear — use for local eval only, do NOT redistribute):** EmreEkin
   ControlLogix captures (#7–#9). Good real-Rockwell content but no license; treat as local
   evaluation input only, not as a checked-in fixture, until licensing is clarified.
4. **Verify before trusting filenames:** open each candidate, confirm port 44818, ENIP command
   codes, CPF item type (0x00B2 vs 0x00B1), and CIP service bytes. Several `[content-inferred]`
   rows above are named optimistically; confirm the actual service byte (0x07 Stop, 0x05 Reset,
   0x54 ForwardOpen, 0x4E ForwardClose, 0x10 SetAttributeSingle, 0x0063 ListIdentity).
5. **Avoid** the automayt 131-byte `CL5000EIP-*.pcap` stubs entirely — they are not real captures.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | Deep multi-source sweep of public ICS/OT ENIP pcap repositories and of academic ICS datasets (which ship pcap vs CSV; licensing/registration gates). Both returned oversized payloads saved to disk; URL lists extracted via Grep. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — |
| WebFetch | 14 | GitHub Contents API directory listings (automayt ETHERNET_IP + digitalbond subfolders, scy-phy Traces/enip, EmreEkin Ethernet_IP, ITI EthernetIP dir + README + license), Wireshark wiki/GitLab, Electra (refused), repo READMEs/licenses. |
| WebSearch | 6 | Wireshark sample-capture location, automayt repo listing, EmreEkin/scy-phy/cpppo provenance + content verification. |
| Training data | 1 area | General ENIP/CIP protocol structure (command codes, CPF 0x00B2/0x00B1, CIP service bytes) used to interpret findings — cross-checked against ODVA/Wireshark dissector references surfaced in search. |

**Total MCP tool calls:** 2 (both `perplexity_research`, the mandated primary tool) + 14 WebFetch + 6 WebSearch = 22 external lookups.
**Training data reliance:** low — all capture candidates, sizes, licenses, and the Metasploit STOPCPU/RESETETHER content claim are sourced to live URLs (GitHub Contents API, repo READMEs/LICENSE, repo detection scripts). Protocol-structure knowledge from training data was used only to map verified content to holdout requirements.

### Confidence & limitations

- **HIGH confidence:** scy-phy repo file inventory + sizes + MIT license + `enip_metasploit.pcapng` containing STOPCPU/CRASHCPU/CRASHETHER/RESETETHER (verified via the repo's own detection script and README); ITI repo CC-BY-4.0 + file list; EmreEkin file list/sizes + no-license status; automayt 131-byte stub finding.
- **MEDIUM/INFERRED:** exact CIP service bytes inside individual scy-phy/ITI/EmreEkin captures whose names imply content (rows marked `[content-inferred]`). These MUST be confirmed by opening the pcap in the follow-on step.
- **INCONCLUSIVE:** Wireshark SampleCaptures ENIP attachment (not located via automated fetch); Electra dataset ENIP-pcap presence (HTTPS refused). Neither is required given the confirmed sources above.
