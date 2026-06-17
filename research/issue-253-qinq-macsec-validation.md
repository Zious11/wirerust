# DF-VALIDATION-001 — Issue #253 Validation

**Issue:** #253 — "Add QinQ / MACsec decoder test fixtures for ARP VLAN-offset path"
**Validation date:** 2026-06-16
**HEAD validated against:** `480f8ae` (develop)
**Validator:** vsdd-factory:research-agent
**Policy:** DF-VALIDATION-001 (`.factory/policies.yaml`) — a deferred finding MUST be
research-validated before a GitHub issue drives implementation.

---

## VERDICT (summary)

| Field | Result |
|-------|--------|
| **Genuine?** | **GENUINE** — confirmed coverage gap, not a false report. |
| **Still OPEN on `480f8ae`?** | **YES** — no QinQ (0x88a8) or MACsec (0x88e5) test exists anywhere in the tree. |
| **Bug vs. coverage?** | Primarily a **coverage gap** for QinQ (offset code is provably correct for double-VLAN). For **MACsec it is potentially a latent bug** — see the warning in Part C. |
| **Severity / user impact** | LOW–MEDIUM. The shipped offset code is structurally sum-over-`link_exts`, so QinQ is almost certainly already handled correctly; the risk is an *unverified* MACsec edge that could mis-offset. No correctness regression is currently demonstrated. |
| **Effort** | SMALL for QinQ fixtures (~2–4 code-built `#[test]` cases, ~1–2 h). MEDIUM if MACsec is included (must first resolve the header_len/encryption nuance, ~3–5 h). |

---

## PART A — Codebase validation (read against `480f8ae`)

### A.1 — The ARP VLAN-offset logic (`src/decoder.rs`)

The research note's citation is accurate. The current code (decoder.rs lines 315–325, with the
peek at 337–348) reads **verbatim**:

```rust
let arp_offset: Option<usize> =
    lax.link.as_ref().and_then(|link| match link {
        etherparse::LinkSlice::Ethernet2(_) => {
            let link_exts_len: usize =
                lax.link_exts.iter().map(|ext| ext.header_len()).sum();
            Some(14 + link_exts_len)
        }
        // `_` covers any future LinkSlice variants added by etherparse;
        // conservatively treat as unreadable (case c).
        _ => None,
    });
```

The malformed classification peek (lines 337–348):

```rust
let malformed = arp_offset.is_some_and(|offset| {
    data.get(offset..offset + 8).is_some_and(|arp_hdr| {
        let htype = u16::from_be_bytes([arp_hdr[0], arp_hdr[1]]);
        let ptype = u16::from_be_bytes([arp_hdr[2], arp_hdr[3]]);
        let hlen = arp_hdr[4];
        let plen = arp_hdr[5];
        htype != 0x0001 || ptype != 0x0800 || hlen != 6 || plen != 4
    })
});
```

**Confirmed:** the offset computation genuinely sums `header_len()` over **all** entries in
`lax.link_exts` (`14 + Σ ext.header_len()`). It is **NOT** hard-coded to a single VLAN. The
formula in the research note (`arp_offset = 14 + lax.link_exts.iter().map(|ext| ext.header_len()).sum()`)
matches the shipped code exactly. The classification then distinguishes malformed
(`htype/ptype/hlen/plen` non-standard → D11) from truncated (valid fixed header → generic
decode-error). All accesses are bounds-checked (`.get()`); no panic path.

The comments at lines 290–294 and 311–313 already assert QinQ "+8" and MACsec "variable" are
"handled via `LaxLinkExtSlice::header_len()` without hardcoding" — but **no test exercises
either claim**. That unverified assertion is exactly what issue #253 targets.

### A.2 — Existing VLAN unit tests (`tests/bc_2_16_d078_vlan_offset_tests.rs`)

**Confirmed exists.** Exactly **4 tests**, all inside `mod d078_vlan_offset`:

| # | Test fn | Frame | hlen | Expected |
|---|---------|-------|------|----------|
| 1 | `test_F1_vlan_tagged_truncated_benign_arp_no_false_positive_d11` | single 802.1Q (0x8100) | 6 | no D11 (truncated) |
| 2 | `test_F1_vlan_tagged_truncated_malformed_arp_routes_to_d11` | single 802.1Q (0x8100) | 8 | D11 |
| 3 | `test_F1_nonvlan_truncated_benign_unchanged` | plain Ethernet | 6 | no D11 (regression guard) |
| 4 | `test_F1_nonvlan_truncated_malformed_unchanged` | plain Ethernet | 8 | D11 (regression guard) |

**Confirmed gap:** every VLAN fixture uses a **single** 802.1Q tag — outer EtherType `0x81 0x00`
(decoder.rs test lines 155, 197). There is **NO** fixture with:
- outer EtherType `0x88a8` (802.1ad / provider bridging / QinQ),
- a second (inner) VLAN tag producing two `link_exts` entries (offset +8),
- MACsec EtherType `0x88e5` / 802.1AE.

A tree-wide grep for `88a8|88e5|macsec|MacSec|Macsec|QinQ|double.?tag` in `src/` returns **only
comment mentions in `decoder.rs`** (lines 293–294, 312) — no code, no test. Confirmed via
`Grep` over `src/`. The single-VLAN-only coverage is precisely the gap the issue claims.

### A.3 — BC references and detection codes

Both cited BCs reference this exact path and were already updated for VLAN/link-extension
offsets:

- **BC-2.16.009 v1.7** (D11 malformed ARP). PC-2 lax path and **EC-008** explicitly state the
  offset is "Ethernet2 base header length ... PLUS the summed byte-lengths of all
  link-extension headers in `lax.link_exts` (VLAN 802.1Q/802.1ad, MACsec, etc.)". The v1.7
  changelog entry is the "F-1 fix — VLAN/link-extension offset correction".
- **BC-2.16.015 v1.6** (decode-vs-analysis separation). PC-7a/7b and **EC-008** carry the same
  "+ summed `lax.link_exts` lengths" language and explicitly name "VLAN-tagged and
  MACsec-wrapped frames".

**Detection-code semantics (confirmed from BC text):**
- **D11 = malformed.** Triggered when the peeked fixed header has `htype != 0x0001 OR
  ptype != 0x0800 OR hlen != 6 OR plen != 4` → `Err("Non-Ethernet/IPv4 ARP frame")` →
  `record_malformed` → LOW/Anomaly finding, `mitre_techniques: []` (T0814 withheld per
  DF-VALIDATION-001).
- **Truncated (NOT D11).** Valid Ethernet/IPv4 fixed header but short variable section, OR
  frame too short for the 8-byte peek, OR non-Ethernet link → `Err("truncated ARP frame")` →
  generic decode-error, no finding.
- **D-078 / F-1** is the drift/finding identifier for the lax-None-arm offset; the VLAN
  extension is the v1.6/v1.7 "F-1 fix".

Both BCs are **active** and their text already anticipates QinQ/MACsec. So the fixtures the
issue proposes are squarely in-contract — they verify an existing, specified requirement that
currently has no test.

---

## PART B — External validation (etherparse registry, version-pinned)

### B.1 — Dependency version (verified, not from memory)

- `Cargo.toml` line 28: `etherparse = "0.20"`.
- `Cargo.lock` lines 469–472: **`etherparse 0.20.2`** (checksum
  `e8b5355e41024c070dd6c1a9b3340e5026a71a4222fb6c64606f21c9f6b502c1`). This is the exact
  resolved version the tests compile against.

### B.2 — `LaxLinkExtSlice` exact variant names and inner types (PINNED — no longer inferred)

The prior research note flagged these names as INFERRED. They are now **documented fact**,
verified against docs.rs/0.20.2 and the etherparse release notes (verbatim source):

```rust
pub enum LinkExtSlice<'a> {
    /// Slice containing a VLAN header & payload.
    Vlan(SingleVlanSlice<'a>),
    /// Slice containing MACsec header & payload.
    Macsec(MacsecSlice<'a>),
}
```

For the **lax** parser the variants are identically named, with lax inner types
(docs.rs/0.20.2 `enum.LaxLinkExtSlice.html`):

| Enum | Variant | Inner type (0.20.2) |
|------|---------|---------------------|
| `LinkExtSlice<'a>` | `Vlan(_)` | `SingleVlanSlice<'a>` |
| `LinkExtSlice<'a>` | `Macsec(_)` | `MacsecSlice<'a>` |
| **`LaxLinkExtSlice<'a>`** | **`Vlan(_)`** | **`SingleVlanSlice<'a>`** |
| **`LaxLinkExtSlice<'a>`** | **`Macsec(_)`** | **`LaxMacsecSlice<'a>`** |

`LaxLinkExtSlice` has **exactly two** variants — `Vlan` and `Macsec`. There is **no** `VlanDouble`
variant (confirmed). The existing test already uses `matches!(ext, LaxLinkExtSlice::Vlan(_))`
(test line 299), which is correct.

**`header_len()` confirmed present** on both `LinkExtSlice` and `LaxLinkExtSlice`:
`pub fn header_len(&self) -> usize` — "Returns the header length of the link extension"
(docs.rs/0.20.2). So the shipped `ext.header_len()` call is a real, documented API.

`LaxSlicedPacket.link_exts` is `ArrayVec<LaxLinkExtSlice<'a>, { LaxSlicedPacket::LINK_EXTS_CAP }>`,
and `LINK_EXTS_CAP = 3` (release notes). It is iterable; the shipped `.iter()...sum()` is valid.

### B.3 — QinQ representation: TWO Vlan entries, NOT one VlanDouble

**Confirmed (documented).** etherparse 0.20.x parses each VLAN tag as a separate
`SingleVlanSlice` entry. For a QinQ frame `[Eth2 EtherType=0x88a8][outer tag][inner
EtherType=0x8100][inner tag][... 0x0806 ARP ...]`:
- outer `0x88a8` (PROVIDER_BRIDGING) → first `LaxLinkExtSlice::Vlan(SingleVlanSlice)`,
  `header_len() == 4`;
- inner `0x8100` (VLAN_TAGGED_FRAME) → second `LaxLinkExtSlice::Vlan(SingleVlanSlice)`,
  `header_len() == 4`;
- total `link_exts` byte length = **8** → `arp_offset = 14 + 8 = 22`.

So for QinQ the shipped `Σ header_len()` formula yields **+8** correctly. **No bug for QinQ**;
the issue is pure coverage.

### B.4 — On-wire offsets a fixture must produce

| Framing | EtherType chain | `link_exts` | Σ header_len | ARP starts at |
|---------|-----------------|-------------|--------------|---------------|
| Plain Ethernet | `0x0806` | (empty) | 0 | **14** |
| Single 802.1Q | `0x8100` → `0x0806` | 1 × Vlan | 4 | **18** |
| QinQ (802.1ad) | `0x88a8` → `0x8100` → `0x0806` | 2 × Vlan | 8 | **22** |
| MACsec (no SCI) | `0x88e5` → (SecTAG) → `0x0806` | 1 × Macsec | **8 + 2 = 10** (see B.5) | **24** (see warning) |
| MACsec + SCI | `0x88e5` → (SecTAG+SCI) → `0x0806` | 1 × Macsec | **16 + 2 = 18** (see B.5) | **32** (see warning) |

Classification per BC text: at the **correct** offset, `hlen != 6` (or any other non-standard
fixed-header field) → **D11 malformed**; a valid 8-byte fixed header with a short variable
section → **truncated** (not D11).

### B.5 — MACsec header length — the critical nuance

`MacsecHeaderSlice::header_len()` is documented (docs.rs/0.19.0, unchanged in 0.20.2) as:
> "Length of the MACsec header (**SecTag + next ether type if available**)."

The SecTAG base is **8 bytes without SCI** and **16 bytes with SCI** (TCI/AN + SL + PN, plus
optional 8-byte SCI). The "next ether type" (2 bytes) is included in `header_len()` **only when
the payload is unencrypted & unmodified** — i.e. when `next_ether_type()` is `Some`. This is
the part the prior note could not pin and the part that creates real risk (see Part C).

`MacsecSlice` / `LaxMacsecSlice` are structs with `header: MacsecHeaderSlice` and
`payload: {Lax}MacsecPayloadSlice`. The payload enum:

```rust
pub enum LaxMacsecPayloadSlice<'a> {
    Unmodified(LaxEtherPayloadSlice<'a>),   // can be parsed onward (ARP visible)
    Modified { incomplete: bool, payload: &'a [u8] },  // encrypted/modified — opaque
}
```

Only `Unmodified` exposes the inner ether payload (and hence an inner ARP). `is_unmodified()` /
`encrypted()` / `next_ether_type()` on the header gate this.

---

## PART C — Verdict, fixture plan, and bug warning

### C.1 — Verdict

- **GENUINE, OPEN on `480f8ae`.** The offset code is sum-over-`link_exts`, the two referenced
  BCs (2.16.009 v1.7, 2.16.015 v1.6) explicitly require correct handling of QinQ/802.1ad and
  MACsec offsets, and yet the only existing fixtures use a single 0x8100 tag. The gap is real.
- **Severity LOW–MEDIUM.** No correctness regression is currently *demonstrated*; the formula
  is provably correct for QinQ. The value of the issue is converting an unverified comment
  ("QinQ adds 8, MACsec variable") into a regression-guarded test, and surfacing the MACsec
  edge below before it bites a real capture.
- **Effort:** QinQ fixtures SMALL; MACsec MEDIUM (blocked on the C.3 decision).

### C.2 — Concrete fixture plan (code-built `#[test]`, mirroring the existing style)

Add to a new `tests/bc_2_16_qinq_macsec_offset_tests.rs` (or extend the existing file with a new
`mod qinq_macsec_offset`), using the same `probe_lax_arm` pattern, `decode_packet` +
`ArpAnalyzer::record_malformed` simulation, and byte-built `Vec<u8>` builders. **Do not download
pcaps.** Recommended cases:

1. **QinQ benign-truncated → NO false D11** (PRIMARY). Frame:
   `dst(6) src(6) | 0x88a8 | outer TCI 0x0064 | 0x8100 | inner TCI 0x0064 | 0x0806 |
   ARP fixed-header htype=0x0001 ptype=0x0800 hlen=6 plen=4 oper=0x0001` (no variable section).
   Total = `6+6+2 + 4 + 4 + 8 = 30` bytes. ARP starts at offset **22**. Assert:
   `probe_lax_arm` shows `link_exts` has **two** Vlan entries (`.filter(Vlan).count() == 2`),
   `net == None`, `stop_err == Layer::Arp`; `decode_packet` → `Err("truncated ARP frame")`;
   `malformed_findings == 0`. (Guards the offset = +8 path.)

2. **QinQ malformed `hlen=8` → D11.** Same framing, inner ARP `hlen=8`. Total 30 bytes, ARP at
   offset 22 → `hlen != 6` → `Err("Non-Ethernet/IPv4 ARP frame")` → `malformed_findings >= 1`.
   Add the finding-quality asserts (category `Anomaly`, empty `mitre_techniques`) as Test 2 in
   the existing file does.

3. **(Regression sanity) assert `Σ header_len() == 8` for the QinQ frame** directly via a probe
   (sum `ext.header_len()` over `link_exts`) so the test fails loudly if etherparse ever changes
   QinQ representation to a single double-VLAN entry.

4. **(OPTIONAL, gated on C.3) MACsec + ARP, unencrypted/unmodified.** Build a SecTAG with
   `encrypted=false`, `userdata_changed=false`, no SCI, `next_ether_type = 0x0806`. **First write
   a probe-only assertion** that `link_exts` contains exactly one `Macsec` entry and record the
   actual `ext.header_len()` value etherparse returns; only then assert the resulting ARP offset.
   If `Modified`/encrypted, the inner ARP is not parseable and the frame will NOT reach
   `stop_err == Layer::Arp` via the ARP path — so an encrypted MACsec fixture is **not** a valid
   ARP-offset test (it never hits the None arm with `Layer::Arp`).

Each builder ends with a `frame.len()` assertion (matching existing convention), and each test
calls `probe_lax_arm` before the D11 assertions to confirm it reaches the lax None arm.

### C.3 — WARNING: MACsec may be a real bug, not just missing coverage

The QinQ path is provably correct, but **MACsec is genuinely uncertain** for two independent
reasons. Flag both to the implementer:

1. **`header_len()` includes the trailing "next ether type" (2 bytes) only when
   unencrypted/unmodified.** The decoder assumes `14 + Σ header_len()` lands exactly on the ARP
   fixed header. For MACsec, `MacsecHeaderSlice::header_len()` = "SecTag + next ether type if
   available". If etherparse counts the 2-byte inner EtherType (0x0806) as part of the link-ext
   header (because it sits inside the SecTAG framing), then `14 + header_len()` already skips the
   EtherType and points at the ARP fixed header — **good**. But if the inner EtherType is *also*
   present as separate on-wire bytes that `header_len()` double-counts (or omits), the offset
   could be off by 2. **This must be confirmed empirically** by a probe test that prints the real
   `header_len()` for a hand-built SecTAG, before trusting the offset. The decoder's "MACsec adds
   its variable header length" comment is an **unverified assumption**.

2. **Encrypted / modified MACsec frames never expose an inner ARP at all.** Only
   `MacsecPayloadSlice::Unmodified` carries a parseable ether payload. An encrypted ARP-in-MACsec
   frame is opaque to etherparse — it will not produce `stop_err == Layer::Arp`, so the lax
   None-arm ARP peek is never reached. Reading raw bytes at `14 + macsec_header_len` for an
   encrypted frame and interpreting them as an ARP fixed header would be **semantically wrong**
   (those bytes are ciphertext). The current code only peeks when `stop_err == Layer::Arp`, so it
   is *probably* safe (encrypted frames won't enter the arm) — but this has **never been tested**,
   and it is the most likely place a latent defect hides.

**Recommendation:** implement QinQ fixtures now (clear win, no risk). For MACsec, first land a
**probe-only** test that records etherparse's actual `header_len()` and `link_exts` shape for a
hand-built unencrypted SecTAG+ARP frame; only after that empirical result is known should an
offset assertion (and any decoder change) be made. Treat the MACsec offset as **unverified** —
do **not** assume `+8`/`+16` from the spec alone.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source synthesis of etherparse 0.20.2 link-ext API (variants, header_len, QinQ/MACsec representation, version history). reasoning_effort=high. |
| Perplexity perplexity_search | 1 | Raw URLs + verbatim release-note enum definition and `MacsecHeaderSlice` method list (header_len = "SecTag + next ether type"). |
| WebFetch | 4 | docs.rs/0.20.2 verbatim: `LaxLinkExtSlice` (2 variants Vlan/Macsec, header_len sig), `LinkExtSlice`, `LaxMacsecSlice` (struct fields, encrypted/modified gating). One 404 (wrong-cased URL) re-resolved via search. |
| Read | 6 | decoder.rs (offset code), Cargo.toml, Cargo.lock (pinned 0.20.2), the 4-test VLAN file, BC-2.16.009, BC-2.16.015. |
| Grep | 2 | Confirmed etherparse 0.20.2 in lock; confirmed no QinQ/MACsec/0x88a8/0x88e5 in `src/` (comments only). |
| Glob | 2 | Located VLAN test file and BC files. |
| Training data | 1 area | MACsec 802.1AE SecTAG general structure (8/16-byte, SCI) — cross-checked against etherparse `sci_present()`/`sci()` docs, not relied on alone. |

**Total MCP tool calls:** 6 (1 perplexity_research + 1 perplexity_search + 4 WebFetch).
**Training data reliance:** low — every load-bearing claim (variant names, header_len semantics,
QinQ two-entry representation, version 0.20.2, BC text) is sourced from docs.rs/0.20.2, the
etherparse release notes, or the local tree; the only training-data input (MACsec wire format) is
corroborated by the documented `MacsecHeaderSlice` accessors.

### Sources
- docs.rs/etherparse/0.20.2 — `enum.LaxLinkExtSlice.html`, `enum.LinkExtSlice.html`,
  `struct.LaxMacsecSlice.html`, `struct.MacsecHeaderSlice.html`, `enum.LaxMacsecPayloadSlice.html`
- github.com/JulianSchmid/etherparse — Releases (verbatim `link_exts` migration + enum defs,
  `LINK_EXTS_CAP = 3`, `MacsecPayloadSlice` Unmodified/Modified), README.md (supported protocols)
- Local tree @ `480f8ae`: `src/decoder.rs`, `Cargo.toml`, `Cargo.lock`,
  `tests/bc_2_16_d078_vlan_offset_tests.rs`,
  `.factory/specs/behavioral-contracts/ss-16/BC-2.16.009.md` (v1.7),
  `.factory/specs/behavioral-contracts/ss-16/BC-2.16.015.md` (v1.6)
