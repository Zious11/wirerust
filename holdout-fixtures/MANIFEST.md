# Holdout Fixtures Manifest

PCAP fixtures for the information-asymmetric holdout evaluation of the wirerust CLI.
Built **independently** from the authoritative protocol specs (ASHRAE 135-2016 Annex J,
RFC 1006, RFC 1035) — no implementation or implementer fixtures were consulted.

Generator: `make_holdout_fixtures.py` (Python 3, stdlib only).
All fixtures: classic libpcap, global-header magic `0xa1b2c3d4` (little-endian, microsecond
timestamps), version 2.4, snaplen 65535, `LINKTYPE_ETHERNET` (1). Ethernet II + IPv4 +
TCP/UDP. IPv4 and TCP/UDP checksums are computed and valid; base timestamp `t0 = 1717000000`
(2024-05-29 UTC), one second per packet.

## Common addressing

| Symbol | Value |
|--------|-------|
| MAC_A | `00:11:22:33:44:55` |
| MAC_B | `00:66:77:88:99:aa` |
| MAC_BCAST | `ff:ff:ff:ff:ff:ff` |
| IP_A | `192.168.1.10` |
| IP_B | `192.168.1.100` |
| IP_C | `192.168.1.20` |
| IP_BCAST | `192.168.1.255` |

**TCP open+close flow** (used by fixtures 2–6) is a clean 6-packet handshake+teardown that
guarantees a flow-open and flow-close event:
1. `A→B` SYN (seq 1000)
2. `B→A` SYN-ACK (seq 5000, ack 1001)
3. `A→B` ACK (handshake complete)
4. `A→B` FIN-ACK (initiate close)
5. `B→A` FIN-ACK
6. `A→B` ACK (flow fully closed)

All TCP headers: data-offset 5 (no options), window 65535. IPv4: IHL 5, TTL 64, DF set.

---

## Crafted fixtures

### 1. `hs-bacnet-udp47808.pcap` — 90 bytes, 1 packet
- **Serves:** HS-129 Case A/B, HS-128 (non-empty).
- **Spec:** BACnet/IP over UDP port 47808 (0xBAC0), ASHRAE 135-2016 Annex J §J.2.1. BVLC
  header `0x81` (BACnet/IP type) + function + 2-byte length.
- **Packet:** Eth `MAC_A→MAC_B`, IPv4 `192.168.1.10→192.168.1.100` proto 17, UDP
  src 54321 → dst **47808**. Payload (8 bytes, BVLC-shaped):
  `81 0a 00 08 01 00 10 08` = BVLC type 0x81, function 0x0a (Original-Unicast-NPDU),
  length 0x0008; NPDU version 0x01, control 0x00; APDU 0x10 0x08 (Unconfirmed-Request, Who-Is).

### 2. `hs-bacnet-tcp47808.pcap` — 444 bytes, 6 packets
- **Serves:** HS-129 Case C (TCP on the BACnet port; flow-close occurs).
- **Spec:** port 47808 observed over TCP (transport mismatch vs. BACnet/IP's UDP norm).
- **Packets:** TCP open+close flow, Eth `MAC_A↔MAC_B`, IPv4 `192.168.1.10↔192.168.1.100`,
  src port 54321 ↔ dst port **47808**. No application payload.

### 3. `hs-bacnet-combined.pcap` — 510 bytes, 7 packets
- **Serves:** HS-129 Case D (both UDP and TCP on 47808 in one capture).
- **Packets:** packet 1 = the UDP/47808 BVLC datagram from fixture 1 (t0); packets 2–7 =
  a TCP open+close flow to dst **47808** (src port 54322, starting at t0+10).

### 4. `hs-tcp102.pcap` — 444 bytes, 6 packets
- **Serves:** HS-130 present-case (flow-close on port 102).
- **Spec:** ISO-on-TCP / TPKT, TCP port 102 (S7comm / S7comm-plus / IEC 61850 MMS /
  ICCP-TASE.2), RFC 1006.
- **Packets:** TCP open+close flow, `192.168.1.10↔192.168.1.100`, src 49200 ↔ dst **102**.
  No application payload.

### 5. `hs-tcp53.pcap` — 444 bytes, 6 packets
- **Serves:** HS-131 Case C (TCP/53 → transport-mismatch vs. DNS's UDP default).
- **Spec:** DNS, port 53, RFC 1035 §4.2.1 (DNS-over-TCP variant).
- **Packets:** TCP open+close flow, `192.168.1.10↔192.168.1.100`, src 49300 ↔ dst **53**.
  No application payload.

### 6. `hs-unclassified-tcp9600.pcap` — 444 bytes, 6 packets
- **Serves:** HS-127, HS-128 (non-empty), HS-130 absent-case (a gap entry exists but NOT on 102).
- **Spec:** arbitrary unclassified TCP port 9600 (no canonical protocol assignment used here).
- **Packets:** TCP open+close flow, `192.168.1.10↔192.168.1.100`, src 49600 ↔ dst **9600**.
  No application payload.

### 7. `hs-empty.pcap` — 24 bytes, 0 packets
- **Serves:** HS-127 (empty), HS-128 (empty-entries).
- **Contents:** valid 24-byte libpcap global header only; zero packet records.
- Note: the CLI emits `notice: ... 0 packets read` and still exits 0 (readable, empty).

### 8. `hs-bacnet-corpus.pcap` — 620 bytes, 8 packets
- **Serves:** HS-132 Case C (known-problematic ICS corpus stand-in).
- **Spec:** BACnet/IP UDP 47808, ASHRAE 135-2016 Annex J, with authentic BVLC + NPDU + APDU
  framing (Who-Is / I-Am discovery exchange).
- **Packets** (all UDP src 47808 → dst 47808):
  1. `IP_A→IP_BCAST` (bcast MAC) — Who-Is broadcast:
     `81 0b 00 0c | 01 20 ff ff 00 ff | 10 08`
     (BVLC 0x0b Original-Broadcast-NPDU len 0x0c; NPDU v1 control 0x20 dest-present,
     DNET 0xFFFF global, DLEN 0, hop 0xFF; APDU Unconfirmed Who-Is).
  2. `IP_B→IP_BCAST` — I-Am (device 260, vendor 15).
  3. `IP_C→IP_BCAST` — I-Am (device 1001, vendor 42).
  4. `IP_A→IP_BCAST` — I-Am (device 7, vendor 8).
  5. `IP_A→IP_BCAST` — Who-Is broadcast (second round).
  6. `IP_B→IP_BCAST` — I-Am (device 260, vendor 15).
  7. `IP_A→IP_B` — unicast Who-Is (`81 0a 00 08 01 00 10 08`).
  8. `IP_B→IP_A` — unicast I-Am (device 1001, vendor 42).
- **I-Am APDU shape:** `10 00 c4 <objid:4> 22 01 e0 91 00 21 <vendor>` — Unconfirmed I-Am,
  BACnetObjectIdentifier (app tag 0xC4, device object-type 8 + instance), Max-APDU 480
  (tag 0x22), Segmentation both (tag 0x91 val 0x00), Vendor Id (tag 0x21). BVLC function
  0x0b, type 0x81. Between hosts 192.168.1.10 / .100 / .20.

---

## Reusable existing fixtures (do NOT recreate)

Confirmed readable via black-box `wirerust analyze` (exit 0):

| Fixture | Size | Packets | Serves |
|---------|------|---------|--------|
| `tests/fixtures/dns.cap` | 4338 B | 38 | HS-131 Case A (valid UDP/53 DNS) |
| `tests/fixtures/dns-remoteshell.pcap` | 25005 B | 58 | HS-131 Case A alt (UDP/53 DNS; also has non-IP L2 frames — CLI emits a benign "No IP layer found" notice, still reads) |
| `tests/fixtures/http-full.cap` | 25803 B | 43 | HS-132 Case B (known-good IT baseline) |
| `tests/fixtures/tls.pcap` | 25057 B | 58 | HS-132 Case B (known-good IT baseline) |

---

## Verification summary

| Fixture | Bytes | Packets | Parses (exit 0) |
|---------|-------|---------|-----------------|
| hs-bacnet-udp47808.pcap | 90 | 1 | yes |
| hs-bacnet-tcp47808.pcap | 444 | 6 | yes |
| hs-bacnet-combined.pcap | 510 | 7 | yes |
| hs-tcp102.pcap | 444 | 6 | yes |
| hs-tcp53.pcap | 444 | 6 | yes |
| hs-unclassified-tcp9600.pcap | 444 | 6 | yes |
| hs-empty.pcap | 24 | 0 | yes (0 packets, notice) |
| hs-bacnet-corpus.pcap | 620 | 8 | yes |

Readability check only — analysis output was **not** interpreted for correctness (that is the
holdout evaluator's job).
