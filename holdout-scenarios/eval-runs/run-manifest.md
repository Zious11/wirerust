# ENIP Holdout Evaluation — Run Manifest

- **Generated:** 2026-06-27
- **Wirerust HEAD:** f17d2704c0bf153bb01038a45344faaec00c2f06 (develop)
- **Binary:** `target/release/wirerust` (cargo build --release, edition 2024)
- **Canonical command template:**
  ```
  wirerust --json --reassemble analyze --enip <pcap>
  ```
  Flags: `--reassemble` forces TCP stream reassembly on (required for ENIP per BC-2.17.020);
  `--enip` enables the EtherNet/IP analyzer; `--json` emits JSON to stdout.
- **Eval dir (pcaps + raw outputs):** `/tmp/enip-holdout-pcaps/` (local, not committed)
- **Outputs dir (JSON + manifest):** `.factory/holdout-scenarios/eval-runs/`

## Acquisition Summary

### Acquired (22 files total)

| Source Repo | License | Files Acquired |
|-------------|---------|---------------|
| `scy-phy/bro-cip-enip` (MIT) | MIT (redistributable with notice) | 14 `.pcapng` files from `testing/btest/Traces/enip/` |
| `ITI/ICS-Security-Tools` (CC-BY-4.0) | CC-BY-4.0 (redistributable with attribution) | 8 `.pcap` files from `pcaps/EthernetIP/` |

### Skipped (per task instructions)

| Source | Reason Skipped |
|--------|---------------|
| `EmreEkin/ICS-Pcaps` | No license stated — redistribution rights unclear |
| `automayt/ICS-pcap` CL5000EIP-* | Every .pcap is 131 bytes (global pcap header + stub only); no usable ENIP payload. Also no license. |
| Any registration-gated sources | Not attempted |

---

## Per-Pcap Results

### scy-phy/bro-cip-enip (MIT)

---

#### enip_metasploit.pcapng
- **SHA256:** `da409883fbbf9c32a316071029c9984f73dece19f02cf0b6c792771fae8ef5ba`
- **Size:** 374 KB
- **Expected content:** Metasploit `multi_cip_command` — STOPCPU/RESETETHER/CRASHETHER/CRASHCPU vs Allen-Bradley PLC
- **Command:** `wirerust --json --reassemble analyze --enip enip_metasploit.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 0
- **ENIP summary:**
  - total_pdu_count: 13
  - flows_analyzed: 4
  - command_distribution: `{"0x0065": 8, "0x006F": 5}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** none
- **Anomaly:** The Metasploit capture contains only 13 ENIP PDUs — 8 RegisterSession (0x0065) + 5 SendRRData (0x006F). The analyzer processed it without error but detected **no Stop/Reset CIP service bytes**. The TCP reassembly ran 10 flows total, 4 classified as ENIP. CIP Stop (service 0x07) and Reset (service 0x05) should have been in the SendRRData payloads; the absence of findings means either (a) those service bytes were not observed in the 5 SendRRData frames, or (b) the Stop/Reset detection path was not triggered by the PDU content seen. **This is a notable observation for the holdout evaluator — zero T0858/T0816 findings from the primary attack capture.**

---

#### enip_enum_attr_PLC.pcapng
- **SHA256:** `26b282d14a6399b752bccde3201a84c16df9c86792d55be600675d368dbca047`
- **Size:** 73 KB
- **Expected content:** CIP GetAttribute reads against Identity/tag objects (enumeration)
- **Command:** `wirerust --json --reassemble analyze --enip enip_enum_attr_PLC.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 202
- **ENIP summary:**
  - total_pdu_count: 406
  - flows_analyzed: 1
  - command_distribution: `{"0x0065": 2, "0x006F": 404}`
  - write_count: 0 / error_count: 190 / parse_errors: 0
- **Techniques fired:** T0888 (x202, verdict=Likely, confidence=High)
  - category: Reconnaissance
  - summary: "CIP Identity Object attribute read: single-device reconnaissance (T0888)"
  - evidence sample: "CIP service=0x01 (GetAttributesAll) path targets Identity Object (class 0x01) src=192.168.1.250"
  - also service=0x0E (GetAttributeSingle) → Identity Object reads

---

#### enip_enumarate_plc1_tags.pcapng
- **SHA256:** `6ddd42666f459c9659d7ed5241d25e9e8b7b6f975b34e0cdaa80646926ec7aaf`
- **Size:** 833 KB
- **Expected content:** CIP enumeration / tag reads
- **Command:** `wirerust --json --reassemble analyze --enip enip_enumarate_plc1_tags.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 0
- **ENIP summary:**
  - total_pdu_count: 4254
  - flows_analyzed: 17
  - command_distribution: `{"0x0065": 34, "0x006F": 4220}`
  - write_count: 0 / error_count: 17 / parse_errors: 0
- **Techniques fired:** none
- **Note:** 17 error responses observed but below T0888 Pattern B burst threshold (default >5 in detection window).

---

#### enip_connect_to_plc1_and_upload.pcapng
- **SHA256:** `4d05d9433a477d95c4eb8a4f9ce6aed13bed84e9c429400ef87c2919f5b2d7fc`
- **Size:** 3.2 MB
- **Expected content:** CIP ForwardOpen/ForwardClose + program upload
- **Command:** `wirerust --json --reassemble analyze --enip enip_connect_to_plc1_and_upload.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 17 (all verdict=Possible, confidence=Low, category=Anomaly)
- **ENIP summary:**
  - total_pdu_count: 4094
  - flows_analyzed: 1
  - command_distribution: `{"0x006F": 82, "0x0070": 4012}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** none (MITRE unmapped)
- **Findings detail:** ForwardOpen connection lifecycle anomalies (service=0x54) from src=192.168.1.201. No dedicated MITRE ICS technique — logged as low-confidence anomaly.

---

#### enip_upload_plc1.pcapng
- **SHA256:** `4ae857b455109416a459b15fd5a2300231859a214b4cf7df14adc4043f5d6cd3`
- **Size:** 3.8 MB
- **Expected content:** CIP program upload session
- **Command:** `wirerust --json --reassemble analyze --enip enip_upload_plc1.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 13
- **ENIP summary:**
  - total_pdu_count: 3604
  - flows_analyzed: 1
  - command_distribution: `{"0x006F": 66, "0x0070": 3538}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** T1036 (x2, verdict=Likely, confidence=High) — conflicting TCP segment overlap; (none) x11 excessive out-of-window segments
- **Note:** T1036 finding is TCP-layer (segment overlap), not ENIP-layer. ENIP itself processed cleanly.

---

#### enip_rw_dummy_tag.pcapng
- **SHA256:** `c87e997aa031ef11a7b4573196edf35cca75e0122120987a8df9d39b25b23ef9`
- **Size:** 4.5 KB
- **Expected content:** CIP read/write tag operations
- **Command:** `wirerust --json --reassemble analyze --enip enip_rw_dummy_tag.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 0
- **ENIP summary:**
  - total_pdu_count: 16
  - flows_analyzed: 2
  - command_distribution: `{"0x0065": 4, "0x006F": 12}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** none

---

#### enip_rw_dummy_tag_priv_violation.pcapng
- **SHA256:** `2843ea3df894d5c33c7061a8e151fafaad289f472343f9261aa83699d8e8e3b9`
- **Size:** 13 KB
- **Expected content:** CIP privilege-violation responses (general_status != 0)
- **Command:** `wirerust --json --reassemble analyze --enip enip_rw_dummy_tag_priv_violation.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 0
- **ENIP summary:**
  - total_pdu_count: 48
  - flows_analyzed: 6
  - command_distribution: `{"0x0065": 12, "0x006F": 36}`
  - write_count: 0 / error_count: 1 / parse_errors: 0
- **Techniques fired:** none
- **Note:** 1 error response counted in enip_summary. Below burst threshold.

---

#### enip_rw_attr_plc1_priv_violation.pcapng
- **SHA256:** `528fc6a1abaedff680792fb67120b0d1495dfc605fa971300571a234d3e5ad25`
- **Size:** 2.8 KB
- **Expected content:** CIP attribute privilege-violation error responses
- **Command:** `wirerust --json --reassemble analyze --enip enip_rw_attr_plc1_priv_violation.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 1 (verdict=Possible, confidence=Low, category=Anomaly)
- **ENIP summary:**
  - total_pdu_count: 10
  - flows_analyzed: 1
  - command_distribution: `{"0x0065": 2, "0x006F": 2, "0x0070": 6}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** none (MITRE unmapped)
- **Findings detail:** ForwardOpen lifecycle anomaly (service=0x54) from src=192.168.1.250.

---

#### enip_write_read_tag_bad.pcapng
- **SHA256:** `a657b9df4859c3b5e92782e55e212b61942a8451563c909f5feabcd10e02435d`
- **Size:** 4.9 KB
- **Expected content:** CIP write/read operations with bad/error responses
- **Command:** `wirerust --json --reassemble analyze --enip enip_write_read_tag_bad.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 4 (all verdict=Possible, confidence=Low, category=Anomaly)
- **ENIP summary:**
  - total_pdu_count: 18
  - flows_analyzed: 2
  - command_distribution: `{"0x0065": 4, "0x0066": 2, "0x006F": 8, "0x0070": 4}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** none (MITRE unmapped)
- **Findings detail:** ForwardOpen lifecycle anomalies (service=0x54) from src=192.168.1.233. Command 0x0066 = ListServices.

---

#### enip_readDI_WIFI_PLC_1.pcapng
- **SHA256:** `5d985e12c07c33033acfd6ac02db18c27046e3d28f72026cdedc988488a629b9`
- **Size:** 1.5 MB
- **Expected content:** Normal read/poll traffic from SWaT operation (known-good)
- **Command:** `wirerust --json --reassemble analyze --enip enip_readDI_WIFI_PLC_1.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** Warning: failed to decode packet (No IP layer found). Further errors counted silently.
- **Findings:** 0
- **ENIP summary:**
  - total_pdu_count: 4
  - flows_analyzed: 1
  - command_distribution: `{"0x0065": 2, "0x006F": 2}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** none

---

#### enip_read_tags.pcapng
- **SHA256:** `154870500484c72f64dddf16b94810ddb218d721bf049ca4ba314a9377c09f9d`
- **Size:** 3.0 KB
- **Expected content:** Normal CIP read tags (known-good)
- **Command:** `wirerust --json --reassemble analyze --enip enip_read_tags.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 0
- **ENIP summary:**
  - total_pdu_count: 8
  - flows_analyzed: 2
  - command_distribution: `{"0x0065": 4, "0x006F": 4}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** none

---

#### enip_read_P201AUTO.pcapng
- **SHA256:** `7a7c0da2d8f39e7eece3d4f8c4af8ad295541aef848f7bc73bf0dffe9c203d59`
- **Size:** 1.7 KB
- **Expected content:** Normal CIP reads from SWaT (known-good)
- **Command:** `wirerust --json --reassemble analyze --enip enip_read_P201AUTO.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 0
- **ENIP summary:**
  - total_pdu_count: 4
  - flows_analyzed: 1
  - command_distribution: `{"0x0065": 2, "0x006F": 2}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** none

---

#### enip_mitmcapturePLC1.pcapng
- **SHA256:** `71fece33b6103732df58f0c41b9f587fd325b674ac18da56cbd87f3a91f17897`
- **Size:** 20 MB
- **Expected content:** MITM capture of PLC1 traffic
- **Command:** `wirerust --json --reassemble analyze --enip enip_mitmcapturePLC1.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** Warning: failed to decode packet (No IP layer found). Further errors counted silently.
- **Findings:** 2 (verdict=Likely, confidence=High, category=Anomaly)
- **ENIP summary:**
  - total_pdu_count: 1390
  - flows_analyzed: 1
  - command_distribution: `{"0x0070": 1390}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** T1036 (x2, verdict=Likely, confidence=High)
  - summary: "Conflicting TCP segment overlap on flow 192.168.1.10:44818 → 192.168.1.200:60976"
  - evidence: "Retransmitted segment contains different data"
  - Note: T1036 = Masquerading (TCP-layer evasion indicator). Consistent with MITM scenario.

---

#### enip_mitm_hmi-plc1_reboot-hmi-vnc.pcapng
- **SHA256:** `633d585e59501809d862b96d51dba8b03ce871e1395d8536ada5a90ef5edcdd8`
- **Size:** 2.7 MB
- **Expected content:** MITM scenario including reboot of PLC (candidate CIP Reset/T0816)
- **Command:** `wirerust --json --reassemble analyze --enip enip_mitm_hmi-plc1_reboot-hmi-vnc.pcapng`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 5
  - 1x verdict=Inconclusive, confidence=Medium, category=Anomaly (TCP small-segment run)
  - 4x verdict=Possible, confidence=Low, category=Anomaly (ForwardOpen lifecycle)
- **ENIP summary:**
  - total_pdu_count: 902
  - flows_analyzed: 2
  - command_distribution: `{"0x0065": 2, "0x006F": 8, "0x0070": 892}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** none (MITRE unmapped)
- **Note:** No T0816 fired. ENIP processed 902 PDUs (892 SendUnitData = 0x0070), no Reset CIP service detected.

---

### ITI/ICS-Security-Tools (CC-BY-4.0)

Attribution: ICS Security Tools, Illinois Institute of Technology (ITI). License: CC-BY-4.0.

---

#### enip_test.pcap
- **SHA256:** `0ba6c01fde28912e9f890d839b991cff71cca8e8259e1d93e9c3a312c43bc255`
- **Size:** 925 B
- **Expected content:** Basic ENIP test capture
- **Command:** `wirerust --json --reassemble analyze --enip enip_test.pcap`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 1 (verdict=Likely, confidence=High, category=Reconnaissance)
- **ENIP summary:**
  - total_pdu_count: 2
  - flows_analyzed: 1
  - command_distribution: `{"0x0063": 2}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** T0846 (x1, verdict=Likely, confidence=High)
  - summary: "EtherNet/IP ListIdentity broadcast observed: network-wide device enumeration (T0846)"
  - evidence: "ENIP command=0x0063 (ListIdentity) src=10.1.1.167 session=0"

---

#### EthernetIP-CIP.pcap
- **SHA256:** `c50b510b3242f94c8aed9a4b6723962f182d04feca8a8dac09a96a135649461d`
- **Size:** 2.0 MB
- **Expected content:** General ENIP + CIP sample (CloudShark provenance, RegisterSession + SendRRData)
- **Command:** `wirerust --json --reassemble analyze --enip EthernetIP-CIP.pcap`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 0
- **ENIP summary:**
  - total_pdu_count: 8799
  - flows_analyzed: 4
  - command_distribution: `{"0x006F": 438, "0x0070": 8361}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** none
- **Note:** 8799 PDUs processed, 0 parse errors. Largest clean ENIP dispatch run in this corpus.

---

#### cip_stop_plc.pcap
- **SHA256:** `93e5d388193fcaff3f3f38878de34269afcd01318897a2d36647c773d1e5ed3d`
- **Size:** 146 B (truncated — single frame only, likely incomplete)
- **Command:** `wirerust --json --reassemble analyze --enip cip_stop_plc.pcap`
- **Exit code:** 0 (no panic)
- **Stderr:** none
- **Findings:** 0
- **ENIP summary:**
  - total_pdu_count: 1
  - flows_analyzed: 1
  - command_distribution: `{"0x0070": 1}`
  - write_count: 0 / error_count: 0 / parse_errors: 0
- **Techniques fired:** none
- **Note:** 146-byte file — effectively a pcap header + 1 SendUnitData frame (0x0070). Insufficient to reconstruct CIP Stop service byte. No T0858 fired.

---

#### cip_start_plc.pcap
- **SHA256:** `3b341bee9e7790bf0fae8468f55c9b90ac49e014cfb1dc3cc97437f811d0933f`
- **Size:** 146 B
- **Command:** `wirerust --json --reassemble analyze --enip cip_start_plc.pcap`
- **Exit code:** 0 / Findings: 0
- **ENIP summary:** total_pdu_count: 1, command: 0x0070
- **Techniques fired:** none
- **Note:** Same truncation issue as cip_stop_plc.pcap.

---

#### cip_unlock_cpu.pcap
- **SHA256:** `26d072e506ffd939b82532b597bdb27684ca79e2abac8ab62b8d65e9ad87313c`
- **Size:** 150 B
- **Command:** `wirerust --json --reassemble analyze --enip cip_unlock_cpu.pcap`
- **Exit code:** 0 / Findings: 0
- **ENIP summary:** total_pdu_count: 1, command: 0x0070
- **Techniques fired:** none

---

#### cip-eth-set-2.pcap
- **SHA256:** `fe674ddc3e4964c10321dd76f649097c87db6d48091327c6977656a5a3c04b26`
- **Size:** 138 B
- **Command:** `wirerust --json --reassemble analyze --enip cip-eth-set-2.pcap`
- **Exit code:** 0 / Findings: 0
- **ENIP summary:** total_pdu_count: 1, command: 0x006F
- **Techniques fired:** none

---

#### cip-multiple-1.pcap
- **SHA256:** `ecf46cb2bfb1e7cda70d9b966431fe2e1f1056d2a158cf54ddce6f03fc82474e`
- **Size:** 304 B
- **Command:** `wirerust --json --reassemble analyze --enip cip-multiple-1.pcap`
- **Exit code:** 0 / Findings: 0
- **ENIP summary:** total_pdu_count: 1, command: 0x0070
- **Techniques fired:** none

---

#### cip-multiple-2.pcap
- **SHA256:** `4a6350da7f5d5b2ee383e9e92e6a5a846931785f849798799a323d0b8a8ba3d4`
- **Size:** 387 B
- **Command:** `wirerust --json --reassemble analyze --enip cip-multiple-2.pcap`
- **Exit code:** 0 / Findings: 0
- **ENIP summary:** total_pdu_count: 1, command: 0x0070
- **Techniques fired:** none

---

## Technique / Holdout Satisfaction Summary

| Technique | Fired | File(s) | Count | Notes |
|-----------|-------|---------|-------|-------|
| T0846 (ListIdentity) | YES | enip_test.pcap | 1 | Likely/High |
| T0888 (Identity reads) | YES | enip_enum_attr_PLC.pcapng | 202 | Likely/High — GetAttributesAll + GetAttributeSingle on class 0x01 |
| T0858 (CIP Stop) | NO | enip_metasploit.pcapng (expected), cip_stop_plc.pcap (truncated) | 0 | Zero findings despite Metasploit STOPCPU presence — see anomaly note |
| T0816 (CIP Reset) | NO | enip_metasploit.pcapng (expected), enip_mitm_hmi-plc1_reboot-hmi-vnc.pcapng (expected) | 0 | Zero findings |
| T0836 (Write burst) | NO | all | 0 | No capture reached write-burst threshold |
| T0814 (Malformed) | NO | all | 0 | All real captures are well-formed ENIP |
| T1036 (TCP overlap) | YES | enip_mitmcapturePLC1.pcapng, enip_upload_plc1.pcapng | 4 | TCP-layer evasion finding (not ENIP-layer technique) |
| ForwardOpen lifecycle | YES | enip_connect_to_plc1_and_upload, enip_rw_attr_plc1_priv_violation, enip_write_read_tag_bad, enip_mitm_hmi-plc1_reboot-hmi-vnc | 22 | Low-confidence anomaly, no MITRE mapping |
| Zero panic / crash | YES | all 22 pcaps | — | exit=0, no panic output on any capture |

## Key Anomalies for Holdout Evaluator Attention

1. **enip_metasploit.pcapng — T0858/T0816 not fired:** This is the primary attack capture
   (Metasploit multi_cip_command STOPCPU/RESETETHER). The ENIP summary shows only 13 PDUs
   (8 RegisterSession + 5 SendRRData). The analyzer observed command_distribution 0x0065 and
   0x006F but zero Stop (CIP service 0x07) or Reset (service 0x05) detections. Possible
   explanations: (a) the CIP Stop/Reset commands are buried inside Connected data (0x0070
   ConnectedData items) rather than Unconnected (0x00B2) — but no 0x0070 appears here; or
   (b) the CIP service bytes in the SendRRData payloads don't match the stop/reset detection
   pattern for some reassembly reason; or (c) the capture actually contains the ENIP session
   setup phase and the command execution happened on a different port/session not captured here.
   The holdout evaluator should inspect the raw pcap bytes (service offset in the CIP header)
   to determine ground truth.

2. **ITI cip_stop_plc.pcap — truncated at 146 bytes:** A pcap with 1 SendUnitData frame is
   insufficient for TCP reassembly to reconstruct a CIP payload. The ENIP summary correctly
   shows 1 PDU dispatched but zero service detection. These ITI single-packet captures are
   essentially frame stubs for Wireshark display, not full session captures.

3. **enip_mitmcapturePLC1.pcapng — no ENIP findings, only T1036:** 1390 SendUnitData (0x0070)
   frames processed without a single ENIP-layer finding. T1036 (conflicting TCP overlap) is
   noteworthy — consistent with a MITM injection scenario, but fires from TCP reassembly,
   not the ENIP analyzer proper.

4. **enip_enum_attr_PLC.pcapng — 190 error_count, 202 T0888 findings:** The error_count in
   the ENIP summary (190) represents CIP error responses — the server returned error replies
   to many attribute reads. The 202 T0888 findings are the Identity Object read requests
   themselves (not the error responses). This is the strongest real-world positive in the corpus.
