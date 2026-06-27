# Holdout Evaluation — SS-17 EtherNet/IP (HS-110..122)

- **Evaluated:** 2026-06-27
- **Binary:** target/release/wirerust (develop f17d270)
- **Corpus:** 22 real-world ENIP/CIP pcaps (scy-phy/bro-cip-enip MIT + ITI/ICS-Security-Tools CC-BY-4.0)
- **Method:** Black-box. Analyzer JSON outputs cross-checked against raw pcap bytes
  (Python pcapng/pcap dissector); detection logic probed with byte-exact synthetic replays.
  No analyzer source read.

## Summary

- Scenarios evaluated: 13 (HS-110..122)
- Satisfied by real captures: 5  (HS-110, HS-114, HS-115, HS-116, HS-120)
- Validated as correct negative / deferral by real captures: 2 (HS-119, and HS-111/112 non-detection)
- Synthetic-by-design (no real capture can exercise the boundary): 4 (HS-113, HS-117, HS-118, HS-121)
- Real-world corpus mandate: HS-122 SATISFIED (clean parse, no panic, plausible TP/FP profile)
- **REAL DETECTION BUG found: NONE.** The zero T0858/T0816 on enip_metasploit.pcapng is
  CORRECT behavior — determination (c)/(b), not a bug.
- **Gate: PASS** (no detection defect; all observable detections behave to spec; zero false
  positives for attack techniques on known-good captures; zero parse errors / panics across 22 pcaps).

---

## Task 1 — enip_metasploit.pcapng zero-Stop/Reset: DETERMINATION = (c), with (b) for the Stop frame. NOT A BUG.

### Capture facts
- Transport: TCP/44818, loopback (127.0.0.1) — Metasploit `multi_cip_command` against a local target.
- 13 ENIP PDUs total: 8x RegisterSession (0x0065) + 5x SendRRData (0x006F) = exactly matches analyzer.
- 4 SendRRData *requests* (src ports 51310/42942/36768/53860), each on its own RegisterSession'd flow,
  + 1 zero-length SendRRData reply.

### Per-request raw-byte dissection (ENIP encapsulation `length` field vs CPF framing)

| Flow  | ENIP length | Encap data (hex)                              | Valid CPF 0x00B2 item? | CIP service reachable by spec-compliant CPF walk |
|-------|-------------|-----------------------------------------------|------------------------|--------------------------------------------------|
| 53860 | **8**       | `0503200124013003`                            | **NO** (len 8 < 10 min CPF) | none — encap is raw CIP with no item framing |
| 42942 | 12          | `0e0320f52401104324011043`                    | NO (item_count=17168 absurd) | none — malformed CPF |
| 36768 | 26          | `52022006240103f00c000a...`                   | NO (item_count=61443 absurd) | none — malformed CPF |
| 51310 | 42          | `00000000 0200 0200 00000000 b2001a00 52...`  | YES (item_count=2, 0x00B2 len=26) | offset-0 service = **0x52 Unconnected_Send**; Stop (0x07) is EMBEDDED inside the Unconnected_Send wrapper, NOT at item offset 0 |

**Key bytes — frame 53860 (RESETETHER):** encapsulation `length`=8, so the only encap data is
the 8 bytes `05 03 20 01 24 01 30 03` — that is **raw CIP** (service 0x05 Reset, path size 3,
Identity class 0x01 inst 1 attr 3). A spec-compliant SendRRData requires
interface_handle(4)+timeout(2)+item_count(2)+item_header(4) = 10 bytes minimum before any CIP
payload. With length=8 there is **no CPF wrapper and no 0x00B2 item at all**; the `b2 00 08 00`
bytes that appear in the wire frame fall inside the ENIP header `options` region, not as a CPF
item type. The analyzer's CPF item-walk correctly finds no 0x00B2 item and emits nothing.

**Key bytes — frame 51310 (STOPCPU):** this IS a well-formed CPF with a 0x00B2 item, but the
item's offset-0 CIP service is **0x52 (Unconnected_Send)**. The Stop service `0x07` is the
*embedded* message inside the Unconnected_Send (`...f0 0c 00 | 07 0220642401 ...`). v0.11.0
classifies the CIP service at offset 0 of the 0x00B2 item (0x52, which is not Stop/Reset/Write),
so it does not fire — and does not unwrap Unconnected_Send embedded messages (a v0.12.0-class
capability). This is consistent with the ADR-010 / HS-119 0x00B2-offset-0 detection model.

### Proof the detector itself is correct (byte-exact synthetic replay)
- Synthetic pcap with a **well-formed** CPF 0x00B2 item, item_data = `0503200124013003`
  (the exact Reset CIP bytes from 53860): **fires exactly one T0816.**
- Same with item_data `070220642401` (Stop 0x07 at offset 0): **fires exactly one T0858.**
- Replaying frame 53860's literal encapsulation bytes verbatim (length=8, no CPF):
  **0 findings** — reproduces the metasploit result in isolation, confirming the cause is the
  Metasploit module's non-standard / CPF-less framing, not a wirerust defect.

### Determination
**(c)** for frames 53860/42942/36768: the Stop/Reset/probe service bytes are present but NOT
carried in a valid 0x00B2 CPF Unconnected Data Item — the Metasploit module emits raw-CIP /
malformed-CPF encapsulation, so there is no spec-compliant item-walk path to the service byte.
**(b)** for frame 51310: Stop rides offset-0 service 0x52 (Unconnected_Send) with Stop embedded;
v0.11.0 correctly does not unwrap embedded messages.
**Neither is determination (a). There is NO real detection bug. Non-detection is CORRECT.**
Consequence: enip_metasploit.pcapng does NOT satisfy HS-111/HS-112 (it is not the crafted
0x00B2-offset-0 fixture those scenarios require); it instead corroborates the v0.11.0
0x00B2-offset-0 scope model behind HS-119.

---

## Per-Scenario Results

| Scenario | Status | Score | Evidence |
|----------|--------|-------|----------|
| HS-110 canonical LE-header decode | SATISFIED | 1.0 | EthernetIP-CIP.pcap 8799 PDUs / 0 parse_errors; cmd dist {0x006F:438, 0x0070:8361} — LE header decoded across the largest clean run. All 22 captures decode 0x006F/0x0065/0x0070 correctly (no 0x6F00 big-endian regression). Confirmed via synthetic canonical SendRRData (cmd 0x006F parsed). |
| HS-111 CIP Stop / T0858 | NO-REAL-CAPTURE (detector verified by synthetic) | n/a / detector=1.0 | No found capture carries Stop 0x07 at offset 0 of a valid 0x00B2 item. enip_metasploit STOP rides 0x52 Unconnected_Send (embedded). Synthetic 0x00B2/offset-0 0x07 -> exactly one T0858. Detector behaves to spec; real corpus cannot exercise it. |
| HS-112 CIP Reset / T0816 | NO-REAL-CAPTURE (detector verified by synthetic) | n/a / detector=1.0 | enip_metasploit RESET frame has ENIP length=8 -> no CPF / no 0x00B2 item (determination c). enip_mitm_..reboot.. has no Reset 0x05 on 0x00B2 either. Synthetic 0x00B2/offset-0 0x05 -> exactly one T0816. Detector correct; real corpus cannot exercise it. |
| HS-113 write-burst / T0836 | NOT-APPLICABLE (synthetic) | n/a | Strict >50-in-1s boundary. write_count=0 across all 22 real captures; no real capture issues CIP writes on 0x00B2, let alone 51 in 1s. Boundary is synthetic-by-design. |
| HS-114 ListIdentity / T0846 | SATISFIED | 1.0 | enip_test.pcap: cmd dist {0x0063:2}; exactly ONE T0846 finding ("ListIdentity broadcast... network-wide device enumeration (T0846)") — confirms detection + one-shot guard (2 frames, 1 finding). |
| HS-115 error-burst / T0888 | SATISFIED (Pattern A) / NO-REAL-CAPTURE (Pattern B) | 0.9 | enip_enum_attr_PLC.pcapng: 202x T0888 (GetAttributesAll/GetAttributeSingle on Identity class 0x01) — strong real TP for Identity-read recon (Pattern A). The *error-burst Pattern B* (>5 error responses/10s) boundary is synthetic; enum_attr's 190 error_count did not (and need not) drive Pattern B since the 202 findings are the request-side Pattern A. |
| HS-116 ForwardOpen/Close empty-MITRE | SATISFIED | 1.0 | enip_connect_to_plc1 (17), enip_rw_attr_priv (1), enip_write_read_tag_bad (4), enip_mitm_reboot (4): all CIP service=0x54 ForwardOpen findings, category=Anomaly, verdict=Possible/Low, and the finding JSON carries NO mitre_techniques key (empty MITRE) — exactly the v0.11.0 ADR-010 Decision 7 behavior. |
| HS-117 malformed / T0814 | NOT-APPLICABLE (synthetic) | n/a | Requires >=3 ENIP frames with unknown command codes in a window. All real captures use valid commands (0x0063/65/66/6F/70); 0 parse_errors corpus-wide; even metasploit's malformed-CPF frames carry valid command 0x006F so they do not trip the encapsulation-command validity gate. No real capture exercises T0814. (Robustness corroborated: malformed-CPF frames handled with 0 panics / 0 parse errors.) |
| HS-118 oversize carry/skip | NOT-APPLICABLE (synthetic) | n/a | Requires a frame with declared length forcing 24+len>600 followed by a valid frame. No real capture contains such a crafted oversize-declared frame. Synthetic-by-design. |
| HS-119 0x00B1 deferral negative | SATISFIED (corroborated) | 1.0 | enip_metasploit corroborates the offset-0/0x00B2-only scope model: services NOT presented at offset 0 of a valid 0x00B2 item produce zero detections (determination b/c above). No real 0x00B1 connected-data CIP request in corpus to exercise directly, but the negative behavior (no spurious Stop/Reset on non-0x00B2-offset-0 carriers) is demonstrated. |
| HS-120 dispatch port 44818 | SATISFIED | 1.0 | All 22 captures (all on 44818) dispatched to the ENIP analyzer (enip_summary present in every JSON; total PDUs 1..8799). enip_test on 44818 + --enip -> T0846 fired, confirming port-7-rule routing reaches the analyzer. |
| HS-121 max-findings DoS bound | NOT-APPLICABLE (synthetic) | n/a | Requires 10,001 Stop frames. Largest real capture = 8799 PDUs (EthernetIP-CIP), 0 findings; no real flood of detecting frames. Cap boundary is synthetic-by-design. |
| HS-122 real-world corpus | SATISFIED | 1.0 | Known-good arm: SWaT/ControlLogix normal captures (enip_read_*, EthernetIP-CIP, enip_rw_dummy_tag) -> ZERO T0858/T0816/T0836 (zero attack-technique false positives). Known-problematic arm: enip_test -> T0846; enip_enum_attr -> 202x T0888 (non-zero recon TP). All 22 exit 0, 0 panics, 0 parse_errors. Mandate met. |

## Findings (behavioral gaps / observations)

1. **No detection defect.** The headline anomaly (zero T0858/T0816 from the Metasploit
   capture) is explained entirely by Metasploit's non-standard ENIP framing (raw-CIP /
   malformed-CPF encapsulation and Unconnected_Send-embedded Stop), not by a wirerust bug.
   Byte-exact synthetic replays prove the T0858/T0816 detectors fire correctly on
   spec-compliant 0x00B2 offset-0 frames. **No HIGH item to route to the orchestrator.**

2. **Coverage gap (capability, not defect):** v0.11.0 detects CIP services only at offset 0
   of a valid 0x00B2 Unconnected Data Item. Real attacker traffic (Metasploit multi_cip_command)
   wraps the dangerous command inside an Unconnected_Send (0x52) and/or uses non-standard
   encapsulation. Detecting these requires Unconnected_Send unwrapping — a known v0.12.0-class
   item, consistent with the documented 0x00B1/embedded deferral. Recommend a backlog note
   (LOW/informational) that the primary public Stop/Reset attack tool is not detected at the
   offset-0/0x00B2 layer; this is an expected scope limit for v0.11.0, not a release blocker.

3. **Real-capture satisfiability of HS-111/HS-112:** the found corpus contains NO capture with
   a spec-compliant offset-0 0x00B2 Stop/Reset. These two scenarios remain satisfiable only by
   the F4 crafted fixtures (as their specs already declare fixture_needed: true). The real
   corpus neither satisfies nor refutes them; the underlying detectors are verified correct.

4. **Robustness:** 22/22 captures exit 0 with zero parse_errors and no panic, including frames
   with absurd CPF item counts (17168, 61443) and CPF-less encapsulation — strong evidence the
   item-walk and header validation are bounds-safe.

## Release-readiness implication

ENIP SS-17 is release-ready with respect to this holdout pass. All detections observable in the
real corpus (T0846, T0888, ForwardOpen empty-MITRE anomalies) behave to spec; zero attack-technique
false positives on known-good captures; the synthetic-boundary scenarios (HS-113/117/118/121) and
the crafted-fixture detections (HS-111/112) are out of reach of the found corpus by design and are
independently verified correct via byte-exact replay. No HIGH/blocking defect. The only follow-up
is an informational backlog note on Unconnected_Send / non-standard-framing coverage for v0.12.0.
