# E17 — MACsec (802.1AE) ARP Offset Deep-Dive

**Date:** 2026-06-16
**Type:** general (technology / library-behavior verification)
**Subject:** Is wirerust's lax ARP-offset computation (`14 + Σ link_exts.header_len()` at `src/decoder.rs` ~315-325) correct for MACsec-tagged frames carrying ARP, or is there a latent off-by-N bug?
**Method:** Direct source inspection of the **vendored** etherparse 0.20.2 crate in the local cargo registry (the authoritative artifact actually linked into this build), cross-checked against etherparse's own conformance proptests, plus a web sweep for public MACsec-over-ARP PCAPs.

---

## VERDICT: (A) — OFFSET IS PROVABLY CORRECT FOR ALL REACHABLE MACsec VARIANTS

The "documented-limitation" position is **evidence-backed, not hand-waving.** There is **no off-by-N bug.**

Two independent facts settle it:

1. **header_len() includes the 8 SCI bytes.** etherparse's `MacsecHeaderSlice::header_len()` — the *exact* function our decoder calls via `LaxLinkExtSlice::header_len()` — returns `6 + (sci ? 8 : 0) + (unmodified ? 2 : 0)`. The "if the SCI case doesn't include the 8 SCI bytes, that's the bug" hypothesis is **refuted** by source line `macsec_header_slice.rs:246-248` and the matching proptest that asserts `16` for SCI+Unmodified.

2. **The only variant that can ever reach `Layer::Arp` is Unmodified MACsec.** Modified/Encrypted/EncryptedUnmodified payloads are opaque: etherparse's lax driver hits `LaxMacsecPayloadSlice::Modified` and `return result` **before** the inner ARP-parse block. So `stop_err == Layer::Arp` is unreachable for encrypted/modified MACsec — our offset code never even runs for them.

3. **Same-function identity.** etherparse advances its *own* internal `offset` accumulator using the identical `macsec.header.header_len()` (`lax_packet_headers.rs:352`) immediately before it parses the inner ARP. Our decoder sums the same per-extension `header_len()`. By construction the two offsets are equal — there is no second, independent arithmetic that could drift.

---

## Pinned-Fact Confirmations (all cited to vendored etherparse 0.20.2 source)

Vendored crate root: `/Users/zious/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/etherparse-0.20.2/`
Version confirmed in `Cargo.lock` lines 469-471: `etherparse v0.20.2`.

### Fact 1 — SecTag wire layout & next-ethertype position — CONFIRMED

`MacsecHeader::MIN_LEN = 6`, `MAX_LEN = 16` (`macsec_header.rs:33,36`).

Wire layout proven by the byte offsets etherparse reads:
- TCI/AN = byte 0, SL = byte 1, PN = bytes 2-5 → **6-byte base SecTag** (`macsec_header_slice.rs:174-186`, `sci()` reads PN-then-SCI).
- Optional SCI = bytes 6-13 (8 bytes), read only when TCI bit `0b10_0000` is set (`macsec_header_slice.rs:196-216`, `sci_present()` at :190-192).
- Next ethertype (2 bytes): read at bytes **[6..8] when SCI absent**, at bytes **[14..16] when SCI present** (`macsec_header_slice.rs:222-241` and `ptype()` :135-150).

So: SecTag is **6 bytes without SCI, 14 bytes with SCI**; the 2-byte next-ethertype sits *after* the SCI in the Unmodified case. Exactly as pinned in the question.

### Fact 2 — header_len() for each {SCI} × {Modified/Unmodified} combination — CONFIRMED, QUOTED

`MacsecHeaderSlice::header_len()` (`macsec_header_slice.rs:246-248`):
```rust
pub fn header_len(&self) -> usize {
    6 + if self.sci_present() { 8 } else { 0 } + if self.is_unmodified() { 2 } else { 0 }
}
```
Identical owned-header version `MacsecHeader::header_len()` (`macsec_header.rs:204-211`):
```rust
6 + if self.sci.is_some() { 8 } else { 0 }
  + if matches!(self.ptype, MacsecPType::Unmodified(_)) { 2 } else { 0 }
```

etherparse's **own proptest asserts the exact table** (`macsec_header.rs:309-350`):

| ptype | SCI | header_len() asserted by upstream test | line |
|-------|-----|----------------------------------------|------|
| Modified / Encrypted / EncryptedUnmodified | absent | **6** | :323 |
| Modified / Encrypted / EncryptedUnmodified | present | **14** (6 + 8 SCI) | :330 |
| Unmodified (inner ethertype) | absent | **8** (6 + 2 etype) | :341 |
| Unmodified (inner ethertype) | present | **16** (6 + 8 SCI + 2 etype) | :347 |

This matches the question's hypothesized values exactly **and the SCI bytes ARE included.**

### Fact 3 — REACHABILITY (the crux) — CONFIRMED from the lax driver source

`LaxPacketHeaders` lax driver, MACsec arm (`lax_packet_headers.rs:349-374`):

- **(a) Unmodified MACsec → CAN reach `Layer::Arp`.** The `LaxMacsecPayloadSlice::Unmodified(l)` arm (:350-362) sets `rest = l.payload`, `offset += macsec.header.header_len()` (:352), `ether_type = l.ether_type` (the inner ethertype), then **falls through** to the `match ether_type` block. If the inner ethertype is `ARP` (:397), it calls `ArpPacket::from_slice(rest)`; on a length failure it sets `result.stop_err = Some((Len(err), Layer::Arp))` (:402). **This is the one and only path to `Layer::Arp`.**

- **(b) Modified / Encrypted MACsec → NEVER reaches `Layer::Arp`.** The `LaxMacsecPayloadSlice::Modified { incomplete, payload }` arm (:364-373) sets `result.payload = LaxPayloadSlice::MacsecModified { .. }` and **`return result;`** immediately (:372). It never reaches the `match ether_type` ARP block. The payload is opaque by design (`LaxMacsecPayloadSlice` has only `Unmodified` and `Modified` variants — `lax_macsec_payload_slice.rs:4-10`; `next_ether_type()` returns `None` for all non-Unmodified ptypes — `macsec_header.rs:53-59`).

Conclusion: **our `stop_err == Layer::Arp` None-arm offset code is only ever entered for Unmodified MACsec carrying an inner ARP that was truncated.** For every encrypted/modified variant the code is dead — so there is nothing for an offset bug to mis-handle there.

### Fact 4 — Arithmetic lands exactly on the ARP fixed header — CONFIRMED

Our decoder (`src/decoder.rs:315-321`): for `LinkSlice::Ethernet2`, `arp_offset = 14 + Σ ext.header_len()`.

Worked through for the two **reachable** Unmodified cases (full Ethernet2 frame, single MACsec tag):

| Case | Ethernet2 base | MACsec header_len() | computed arp_offset | wire position of ARP byte 0 |
|------|----------------|---------------------|---------------------|-----------------------------|
| no-SCI, Unmodified, inner=ARP | 14 | 8 | **22** | 14 (eth) + 6 (SecTag) + 2 (etype) = **22** ✓ |
| SCI, Unmodified, inner=ARP | 14 | 16 | **30** | 14 (eth) + 6 (SecTag) + 8 (SCI) + 2 (etype) = **30** ✓ |

Both land exactly on ARP byte 0. The SCI-present case yields **30, not 22** — i.e. the 8 SCI bytes are correctly accounted for. (If `header_len()` had *omitted* the SCI it would have returned 8 and produced 22 — the bug the question worried about. The source shows it returns 16 → 30. No bug.)

**Stacked-extension proof (upstream conformance test):** `lax_packet_headers.rs:1371-1419` (`from_x_slice_net_variants`) builds frames with **up to three stacked link extensions** (any mix of MACsec/VLAN, with the final ext flipped to `MacsecPType::Unmodified(ARP)` at :1361), sets ether_type=ARP, then truncates the ARP region byte-by-byte (:1396) and asserts:
```rust
layer_start_offset: base_len,   // = test.len(&[]) - arp.packet_len()
... Layer::Arp
```
i.e. etherparse itself asserts the ARP start offset equals **the summed length of all preceding headers** — exactly `14 + Σ header_len()`. wirerust's formula is the same quantity, derived the same way, from the same `header_len()` function.

### Fact 5 — Public MACsec-over-ARP PCAPs — RE-CONFIRMED: none known

- Wireshark `SampleCaptures` wiki lists exactly one MACsec capture: **`macsec_cisco_trunk.pcap`** (3750X switch-to-switch TrustSec). Its description enumerates VTP, RSTP/RPVST+, CDP, EIGRP — **ARP is not listed**, and as Cisco TrustSec switch-to-switch traffic its MACsec payloads are encrypted/modified (the opaque, non-`Layer::Arp` path). (WebFetch of wiki.wireshark.org/SampleCaptures.)
- Deep web sweep (Perplexity `sonar-deep-research`) across Wireshark sample/test captures, packetlife, cloudshark, and GitHub macsec fixtures returned **"No Evidence of MACsec-over-ARP"** / "no curated MACsec-over-ARP" capture — consistent with prior research. No publicly known capture carries *unmodified* MACsec with an inner ARP payload.

**Implication:** the reachable code path (Unmodified MACsec + inner truncated ARP) has **no public on-wire fixture** to test against. This is a real, accurate documented limitation — but the *logic* is proven correct by (i) the etherparse source, (ii) etherparse's own stacked-extension conformance proptest covering exactly this case, and (iii) the same-function identity argument. A synthetic fixture (hand-built Unmodified-MACsec-over-ARP frame, both no-SCI and SCI) is the only way to add a wire-level regression test, and would be sound to add.

---

## Per-Variant Summary Table

| MACsec variant | SCI | `header_len()` | Reaches `Layer::Arp`? | arp_offset (single tag, Eth2) | Correct? |
|----------------|-----|----------------|------------------------|-------------------------------|----------|
| Unmodified (inner=ARP) | no  | 8  | **YES** (only path) | 14+8 = **22** | ✓ exact |
| Unmodified (inner=ARP) | yes | 16 | **YES** (only path) | 14+16 = **30** | ✓ exact |
| Modified | no  | 6  | NO (opaque, early return) | n/a — code unreachable | ✓ vacuous |
| Modified | yes | 14 | NO (opaque, early return) | n/a — code unreachable | ✓ vacuous |
| Encrypted | no  | 6  | NO | n/a | ✓ vacuous |
| Encrypted | yes | 14 | NO | n/a | ✓ vacuous |
| EncryptedUnmodified | no  | 6  | NO | n/a | ✓ vacuous |
| EncryptedUnmodified | yes | 14 | NO | n/a | ✓ vacuous |

No reachable variant computes a wrong offset. No `UNCERTAIN` facts remain on the correctness question.

---

## Recommendation

- **Keep the implementation as-is.** `14 + Σ ext.header_len()` is provably correct for every MACsec variant the code can reach.
- The "no public MACsec-over-ARP PCAP exists" statement is the *honest, evidence-backed* limitation — it concerns **test coverage**, not **correctness**. Do NOT downgrade it to imply the code is unverified: the code is verified against etherparse source + upstream proptest.
- **Optional hardening (not a bug fix):** add a synthetic unit fixture for Unmodified-MACsec-over-truncated-ARP in both no-SCI (expect offset 22) and SCI (expect offset 30) forms, to lock the 8-SCI-byte accounting against future etherparse changes. This closes the only real gap (absence of a wire-level regression test), without any code change to the decoder.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep web sweep for public MACsec-over-ARP PCAP fixtures (Wireshark, packetlife, cloudshark, GitHub) — confirmed none known |
| WebFetch | 1 | Wireshark SampleCaptures wiki — enumerate MACsec captures and whether any carry ARP |
| Read (vendored etherparse 0.20.2 source) | 6 files | `macsec_header.rs`, `macsec_header_slice.rs`, `lax_macsec_slice.rs`, `lax_macsec_payload_slice.rs`, `lax_link_ext_slice.rs`, `lax_packet_headers.rs` — authoritative behavior of `header_len()`, payload dispatch, and `Layer::Arp` reachability |
| Read/Grep (local) | 4 | `Cargo.lock` version pin; `src/decoder.rs` offset code + lax ARP arm |
| Training data | 0 areas | None relied upon for any load-bearing claim — every fact cited to vendored source line numbers or live web |

**Total MCP tool calls:** 1 (perplexity_research) + 1 WebFetch
**Training data reliance:** low — all correctness claims are grounded in the exact etherparse 0.20.2 source linked into this build (vendored cargo registry copy), cross-checked against etherparse's own conformance proptests. The PCAP-availability finding is web-grounded. No version number or API behavior taken from model memory.
