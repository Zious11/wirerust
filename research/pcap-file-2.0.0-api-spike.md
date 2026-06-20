# pcap-file 2.0.0 API Spike — Source-Level Findings

**Date:** 2026-06-19
**Purpose:** Resolve API-shape questions gating the F2 pcapng spec remediation. Authoritative
source reading of the vendored crate; no inference from docs prose.

**Source location:**
`/Users/zious/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pcap-file-2.0.0/`

**Crate metadata (Cargo.toml:13-15):** `pcap-file` v2.0.0, `edition = "2021"`.
Deps: `byteorder_slice 3.0.0`, `derive-into-owned 0.2.0`, `thiserror 1.0.35`.

All citations below are `file:line` relative to the crate `src/` root, with quoted source.

---

## KEYSTONE VERDICT (read this first)

**The BC-2.01.014 pure-core timestamp helper is REDUNDANT given pcap-file 2.0.0's own EPB
timestamp handling — BUT only because the crate is itself WRONG. It does NOT apply
`if_tsresol`; it hard-codes nanosecond resolution.**

Concretely, for `EnhancedPacketBlock`:

- The struct field is `pub timestamp: Duration` — a *normalized* type, NOT raw split ticks.
  (`pcapng/blocks/enhanced_packet.rs:27`)
- On parse, the crate combines the two 32-bit halves into a raw u64 tick count and then calls
  `Duration::from_nanos(timestamp)` — i.e. it **assumes the ticks are nanoseconds**
  unconditionally. (`enhanced_packet.rs:46-48`, `:65`)
- The `IfTsResol(u8)` option *is* parsed off the IDB (`interface_description.rs:179-184`,
  `:110`) but is **never read back** to scale the EPB timestamp anywhere in the crate.
  (Confirmed: no consumer of `IfTsResol` outside the parse/write option round-trip.)

**Consequence for wirerust:**

1. You cannot call the BC-2.01.014 helper on `EnhancedPacketBlock::timestamp` — by the time
   you receive the block, the raw ticks are gone (collapsed into a `Duration` under a false
   ns assumption). The helper as specified (raw `timestamp_high`/`timestamp_low` + `if_tsresol`
   → normalized) has **no raw input to operate on** if you consume the parsed `Block` API.
2. Therefore the H-1 / SEC-001 overflow cluster (`timestamp_high << 32` overflow, tsresol
   exponent overflow in the pure-core helper) is **a delete, not a fix**, *if and only if*
   wirerust consumes `Block`/`EnhancedPacketBlock` directly. The overflow the helper guards
   against has already happened (or been mooted) inside the crate at `enhanced_packet.rs:48`
   (`(timestamp_high << 32) + timestamp_low`, done in u64 with no overflow guard — but u64
   cannot overflow from two u32 halves, so it is actually safe there; see Q1 detail).
3. **However** — if wirerust needs *correct* tsresol-aware timestamps (any IDB with
   `if_tsresol != 9`, i.e. not nanoseconds — the default per the pcapng spec is **microseconds**,
   tsresol=6), it must bypass the parsed `Block` API and read raw via `next_raw_block` /
   `RawBlock`, then do the split-tick + tsresol math itself. In that path the BC-2.01.014
   helper is **NEEDED and correct**, and the H-1/SEC-001 overflow guards are real.

**Net recommendation:** The helper is REDUNDANT *only* on the high-level `Block` path, and on
that path the crate is silently wrong for any non-nanosecond capture (the common case is
microsecond, tsresol=6). The right call is almost certainly to **keep the helper and feed it
from `RawBlock`** (or from the obsolete `PacketBlock`, which DOES expose raw `timestamp: u64`
— see Q1). Treat the overflow cluster as a real fix, not a delete, because the correct
implementation path is the raw path, which is exactly where the helper lives.

---

## Q1 — EnhancedPacketBlock timestamp shape (KEYSTONE)

**Verdict: NORMALIZED (`Duration`), with `if_tsresol` NOT applied — the crate hard-codes
nanoseconds. Raw split ticks are NOT exposed on the EPB type.**

Struct field (`pcapng/blocks/enhanced_packet.rs:26-27`):
```rust
/// Number of units of time that have elapsed since 1970-01-01 00:00:00 UTC.
pub timestamp: Duration,
```

Parse path (`enhanced_packet.rs:45-48`, `:62-69`):
```rust
let interface_id = slice.read_u32::<B>().unwrap();
let timestamp_high = slice.read_u32::<B>().unwrap() as u64;
let timestamp_low  = slice.read_u32::<B>().unwrap() as u64;
let timestamp = (timestamp_high << 32) + timestamp_low;   // raw u64 ticks
...
let block = EnhancedPacketBlock {
    interface_id,
    timestamp: Duration::from_nanos(timestamp),           // <-- ticks ASSUMED to be ns
    ...
};
```

Round-trip back to wire (`enhanced_packet.rs:79-83`) confirms the same ns assumption:
```rust
let timestamp = self.timestamp.as_nanos();
let timestamp_high = (timestamp >> 32) as u32;
...
let timestamp_low  = (timestamp & 0xFFFFFFFF) as u32;
```

**Overflow notes for H-1 / SEC-001:**
- `(timestamp_high << 32) + timestamp_low` at line 48: both operands are `u64` (each is a
  `u32 as u64`). A u32 shifted left 32 in u64 occupies the high half exactly; adding the low
  u32 cannot overflow u64. So *this specific* shift is safe inside the crate.
- The danger the BC-2.01.014 helper guards against is the **tsresol scaling** step
  (`ticks * 10^-resol` or `ticks << resol` for binary tsresol), which the crate **never
  performs** — so the crate has no overflow there because it has no scaling there. The
  helper's overflow risk reappears the moment wirerust does the scaling the crate omitted.

**Contrast — obsolete `PacketBlock` DOES expose a raw u64 timestamp**
(`pcapng/blocks/packet.rs:28-30`, `:53`):
```rust
/// ... a single 64-bit unsigned integer ... units of time ...
pub timestamp: u64,
...
let timestamp = slice.read_u64::<B>().unwrap();   // raw, NOT converted
```
So if wirerust ever needs raw ticks from the high-level API, the *obsolete* PacketBlock is the
only variant that hands them over unconverted. EPB does not.

---

## Q2 — Does pcap-file apply if_tsresol?

**Verdict: NO. The crate parses `IfTsResol` into the IDB but never applies it to any packet
timestamp. EPB ticks are hard-coded as nanoseconds; the IDB tsresol is dead data w.r.t.
timestamp conversion.**

- `IfTsResol(u8)` variant declared: `interface_description.rs:109-110`.
- Parsed (option code 9, length must be 1): `interface_description.rs:179-184`.
- Written back: `interface_description.rs:231`.
- **No application site.** The EPB parser (`enhanced_packet.rs:40-72`) takes only the
  block body slice; it has no access to the IDB and makes no tsresol query. The parser's
  `packet_interface()` helper (`pcapng/parser.rs:124-126`) lets the *caller* look up the IDB
  for a given EPB, but the crate itself never uses it to scale the timestamp.

So tsresol conversion is entirely the caller's responsibility — and the caller can only do it
correctly by reading raw ticks (raw-block path or PacketBlock), because EPB has already
thrown the raw ticks away behind a ns-assuming `Duration`.

---

## Q3 — SimplePacketBlock data accessor & length math

**Verdict: SPB exposes `data: Cow<'a, [u8]>` and `original_len: u32`. The crate does NOT
compute captured length and does NOT clamp to snaplen — `data` is the entire remaining block
body, raw. Caller must derive captured_len from `min(original_len, snaplen)`.**

Struct (`pcapng/blocks/simple_packet.rs:19-25`):
```rust
pub struct SimplePacketBlock<'a> {
    pub original_len: u32,
    pub data: Cow<'a, [u8]>,
}
```

Parse (`simple_packet.rs:28-37`):
```rust
if slice.len() < 4 {
    return Err(PcapError::InvalidField("SimplePacketBlock: block length < 4"));
}
let original_len = slice.read_u32::<B>().unwrap();
let packet = SimplePacketBlock { original_len, data: Cow::Borrowed(slice) };
Ok((&[], packet))
```

Key points relevant to adversary **H-2 (hand-computed SPB length math)**:
- `data` is assigned the *entire* remaining slice after the 4-byte original_len field — the
  crate performs **no** captured-length computation and **no** snaplen clamp.
- There is no `captured_len` field on SPB at all (the pcapng spec says SPB has no captured
  length; it is implicitly `min(original_len, snaplen)` and the on-disk payload is padded).
  The crate hands you the padded body verbatim as `data` — note the padding bytes are
  **included** in `data` here (parse returns `Cow::Borrowed(slice)` with no de-padding),
  unlike EPB which slices to `captured_len` before padding (see Q3b).
- Therefore any wirerust SPB length logic (the H-2 hand-rolled math) is genuinely the
  caller's burden — the crate gives no help and no validation. The correct captured length
  must be computed by the caller as `min(original_len, snaplen)` per spec, and the caller
  must also strip the SPB padding that the crate left in `data`.

---

## Q3b — EnhancedPacketBlock data accessor & length fields

**Verdict: EPB exposes `data: Cow<'a, [u8]>` (already sliced to captured_len, padding
stripped) and `original_len: u32`. There is NO `captured_len` field retained — captured_len
is consumed at parse time and recoverable only as `data.len()`.**

Struct (`enhanced_packet.rs:29-33`):
```rust
/// Actual length of the packet when it was transmitted on the network.
pub original_len: u32,
/// The data coming from the network, including link-layer headers.
pub data: Cow<'a, [u8]>,
```
Note: **no `captured_len` field** on EPB (contrast `PacketBlock` which keeps
`captured_len: u32`, `packet.rs:32-33`).

Parse — captured_len read, validated, used to slice `data`, then discarded
(`enhanced_packet.rs:49-60`):
```rust
let captured_len = slice.read_u32::<B>().unwrap();
let original_len  = slice.read_u32::<B>().unwrap();

let pad_len = (4 - (captured_len as usize % 4)) % 4;
let tot_len = captured_len as usize + pad_len;

if slice.len() < tot_len {
    return Err(PcapError::InvalidField("EnhancedPacketBlock: captured_len + padding > block length"));
}
let data = &slice[..captured_len as usize];   // data == captured bytes, no padding
slice = &slice[tot_len..];                      // padding skipped
```

For wirerust: `captured_len == block.data.len()`. The crate validates
`captured_len + pad <= remaining body` and errors cleanly if violated (line 55-57) — no panic
on a too-large captured_len (the bounds check precedes the slice). It does NOT validate
`captured_len <= snaplen` (see Q7) and does NOT validate `captured_len <= original_len`.

---

## Q4 — Block enum variants & exhaustiveness

**Verdict: `pcap_file::pcapng::Block<'a>` has 9 variants. It is NOT `#[non_exhaustive]`
(confirmed zero occurrences of `non_exhaustive` in the entire crate source). Per-variant skip
arms ARE possible; no `_ => skip` catch-all is *required* by the type system — but `Unknown`
is itself the catch-all for unrecognised block types on the wire.**

Definition (`pcapng/blocks/block_common.rs:146-166`):
```rust
#[derive(Clone, Debug, IntoOwned, Eq, PartialEq)]
pub enum Block<'a> {
    SectionHeader(SectionHeaderBlock<'a>),
    InterfaceDescription(InterfaceDescriptionBlock<'a>),
    Packet(PacketBlock<'a>),                 // obsolete Packet Block (type 0x02)
    SimplePacket(SimplePacketBlock<'a>),
    NameResolution(NameResolutionBlock<'a>),
    InterfaceStatistics(InterfaceStatisticsBlock<'a>),
    EnhancedPacket(EnhancedPacketBlock<'a>),
    SystemdJournalExport(SystemdJournalExportBlock<'a>),
    Unknown(UnknownBlock<'a>),
}
```

Exact variant inventory vs the question's checklist:
- SectionHeader — yes
- InterfaceDescription — yes
- **Packet (obsolete) — yes** (`Packet`, block type `0x02`, `block_common.rs:153`, `:29`)
- SimplePacket — yes
- **NameResolution — yes** (`block_common.rs:157`)
- **InterfaceStatistics — yes** (`block_common.rs:159`)
- EnhancedPacket — yes
- **SystemdJournalExport — yes** (`block_common.rs:163`)
- Unknown — yes (catch-all for unrecognised block types, `block_common.rs:250`)
- **DecryptionSecrets — NO.** There is no DecryptionSecrets variant. DSB (type 0x0A) falls
  through `try_from_raw_block` to `Block::Unknown` (`block_common.rs:217-251`; only the 8
  known block-type constants at `:25-39` are matched, DSB is not among them).

Exhaustiveness for adversary **M-2:** Because the enum is *not* `#[non_exhaustive]`, a match
in wirerust can list all 9 arms and the compiler will enforce completeness (a future crate
upgrade that adds a variant would be a breaking change caught at compile time). So per-variant
skip arms are fully supported; a `_ => skip` is optional, not mandated. Practically, unknown
on-the-wire block types never reach a new variant — they arrive as `Block::Unknown`
(`block_common.rs:250`), so the "skip unknown" policy maps to matching `Block::Unknown`.

---

## Q5 — Block-walk / forward progress (SEC-002, CWE-835)

**Verdict: The CRATE owns forward progress. `next_block` / `next_raw_block` return the
remainder slice `rem` that the crate computed by consuming the full block (header + body +
trailer), and the buffered reader advances by exactly that consumed amount. A caller using
the documented loop cannot spin on `block_total_length = 8` — that input is rejected as an
error before any advance, and on the parser (slice) API the caller simply stops because no
progress slice is returned on error.**

Mechanics:

1. `RawBlock::from_slice` computes the block extent from `initial_len` and returns
   `Ok((rem, block))` where `rem` is positioned *after* the trailer
   (`block_common.rs:110-123`):
   ```rust
   let body_len = initial_len - 12;
   let body = &slice[..body_len as usize];
   let mut rem = &slice[body_len as usize..];
   let trailer_len = rem.read_u32::<B>().unwrap();
   if initial_len != trailer_len { return Err(...); }
   let block = RawBlock { ... };
   Ok((rem, block))
   ```
   The advance is driven by `initial_len`, not by the caller.

2. A malicious `block_total_length = 8` is rejected before any block is produced
   (`block_common.rs:101-103`): `if initial_len < 12 { return Err(InvalidField("Block: initial_len < 12")) }`.
   Also `initial_len % 4 != 0` is rejected (`block_common.rs:97-99`). So the degenerate
   short/misaligned length cannot create a zero/under-advance block.

3. On the buffered `PcapNgReader`/`ReadBuffer` path, advancement is computed from the returned
   `rem` pointer, not from caller arithmetic (`read_buffer.rs:48-50`, `:96-103`):
   ```rust
   Ok((rem, value)) => { self.advance_with_slice(rem); return Ok(value); }
   ...
   fn advance_with_slice(&mut self, rem: &[u8]) {
       let diff_len = (rem.as_ptr() as usize)
           .checked_sub(self.buffer().as_ptr() as usize)
           .expect("Rem is not a sub slice of self.buffer");
       self.advance(diff_len)   // assert!(self.pos + nb_bytes <= self.len)
   }
   ```
   So the consumed length equals the real block extent the parser walked.

4. On an error (e.g. the rejected `initial_len < 12`), `parse_with` returns `Err(...)` and
   does **not** advance (`read_buffer.rs:65`: `Err(e) => return Err(e)`). The documented reader
   loop terminates on the `Err(_)` arm; the parser-API loop in the rustdoc example
   (`parser.rs:29-44`) likewise should terminate on `Err(_)`. Forward progress is the crate's
   responsibility; the caller's only obligation is to stop on error rather than retry the same
   `src`.

**SEC-002 conclusion:** CWE-835 infinite-loop risk on `block_total_length = 8` is mitigated by
the crate (rejected at `block_common.rs:101`). wirerust does not need to hand-roll a min-block
length guard for forward progress — but should ensure its own loop breaks on `Err(_)` and does
not feed the same slice back in (the rustdoc example's `Err(_)` arm is empty and, taken
literally, would spin — wirerust must `break`/return there).

---

## Q6 — Malformed-input behavior (SEC-008 / no-panic)

**Verdict: MIXED. The block framing layer returns clean `Result` errors on malformed/truncated
input. BUT there are `unwrap()` calls in the field-decode hot path and `unimplemented!()` /
`panic!()` reachable under specific conditions. None of the hot-path `unwrap()`s are on
attacker-controlled lengths *after* the framing bounds checks — but the pattern is fragile and
wirerust should treat a fuzz-hardening wrapper as warranted.**

Clean-error evidence:
- `RawBlock::from_slice` length/alignment/trailer checks all return `Err`
  (`block_common.rs:69-71`, `:97-119`).
- EPB / SPB / Packet / IDB bounds checks return `Err` before slicing
  (`enhanced_packet.rs:41-43`, `:55-57`; `simple_packet.rs:29-31`; `packet.rs:47-49`, `:60-62`;
  `interface_description.rs:41-43`).
- Option parsing bounds-checks length+pad before slicing (`opt_common.rs:31-46`).
- `next_block` returns `Result` (`parser.rs:69-83`); `PcapNgReader::next_block` wraps it in
  `Option<Result<...>>` and converts IO errors cleanly (`reader.rs:47-59`).

Panic / unwrap surface (relevant to SEC-008):
- **`unwrap()` after a manual length check** — e.g. `enhanced_packet.rs:45-50`,
  `simple_packet.rs:32`, `packet.rs:51-55`, `interface_description.rs:45-52`,
  `block_common.rs:73,77,81,115`. These read fixed-width fields *after* an explicit
  `slice.len() < N` guard, so under correct guards they cannot panic. Risk is brittle: the
  guard and the reads must stay in sync (e.g. EPB checks `< 20` then reads exactly 20 bytes —
  OK).
- **`opt_common.rs:36-37`** reads option code/length with `unwrap()` *after* `slice.len() < 4`
  check (`:32`) — OK.
- **`block_common.rs:213-215` `panic!("The raw block is not borrowed")`** — reachable only if
  a caller constructs an owned `RawBlock` and calls `try_from_raw_block`; not reachable from
  the normal borrowed parse path. Low risk for read pipeline.
- **`unknown.rs:36` `unimplemented!(...)`** in `UnknownBlock::from_slice` — never called by the
  crate's own dispatch (UnknownBlock is built via `UnknownBlock::new`, `block_common.rs:250`),
  but a panic landmine if wirerust ever calls `UnknownBlock::from_slice` directly. Do not.
- **`read_buffer.rs:91 assert!`** and **`:100 .expect(...)`** are internal invariants on the
  buffered path; not attacker-controlled (they assert the parser returned a sub-slice).
- **`block_common.rs:193` `.unwrap()`** is on a write path (`std::io::sink()` fake-write to
  compute length) — not on the read hot path.

**SEC-008 conclusion:** No panic is reachable from well-formed-framing-then-malformed-fields on
the documented borrowed read path *given the crate's own guards hold* — and they appear to hold
(every fixed-width `unwrap` is preceded by a matching length check). The residual risks are
(a) the `unimplemented!` in `UnknownBlock::from_slice` (avoid calling it), and (b) the
`panic!` in `try_from_raw_block` for non-borrowed input (only use borrowed raw blocks). wirerust
should still run `catch_unwind`-free but treat the crate as "errors-clean on truncation,
panic-on-misuse"; a fuzz target over `PcapNgReader::next_block` is the right verification.

---

## Q7 — snaplen validation (adversary O-4 parity)

**Verdict: NO. `PcapNgReader` / the EPB/SPB parsers do NOT validate `captured_len <= snaplen`.
The crate does not even hold snaplen in scope at packet-parse time (EPB parser only sees the
block body slice, not the IDB). snaplen is parsed and stored on the IDB
(`interface_description.rs:33`, `:52`) and is otherwise unused for validation.**

Evidence:
- IDB stores snaplen (`interface_description.rs:29-33`, parsed at `:52`):
  ```rust
  pub snaplen: u32,
  ...
  let snaplen = slice.read_u32::<B>().unwrap();
  ```
- EPB parse validates `captured_len + pad <= remaining body`
  (`enhanced_packet.rs:55-57`) but performs **no** snaplen comparison — it has no access to the
  IDB. Same for SPB (`simple_packet.rs:28-37`, no snaplen reference at all).
- There is no code path anywhere that compares a packet's captured length against
  `InterfaceDescriptionBlock::snaplen`.

**O-4 conclusion:** The classic-pcap workaround (using `next_raw_packet` to dodge a
snaplen-rejection bug) has **no analog here** — pcapng never rejects on snaplen in the first
place, so there is nothing to work around, and there is also no built-in snaplen enforcement to
rely on. If wirerust wants snaplen parity / enforcement, it must implement the
`captured_len <= snaplen` check itself, looking up the IDB via
`parser.packet_interface(&epb)` (`parser.rs:124-126`) or `reader.packet_interface(&epb)`
(`reader.rs:87-89`). The crate gives the lookup hook but not the check.

---

## Cross-cutting summary table

| Q | Topic | Verdict | Key cite |
|---|-------|---------|----------|
| 1 | EPB timestamp shape | `Duration`, ns hard-coded; raw ticks NOT exposed on EPB | `enhanced_packet.rs:27,46-48,65` |
| 2 | if_tsresol applied? | NO — parsed to IDB, never applied | `interface_description.rs:110,179-184` |
| 3 | SPB data/len | `data: Cow<[u8]>` (incl. padding), `original_len`; no captured_len, no snaplen clamp | `simple_packet.rs:19-37` |
| 3b| EPB data/len | `data: Cow<[u8]>` (captured bytes, padding stripped), `original_len`; no captured_len field | `enhanced_packet.rs:29-33,49-60` |
| 4 | Block enum | 9 variants, NOT `#[non_exhaustive]`; no DecryptionSecrets (→Unknown); Packet/NRB/ISB/SJE present | `block_common.rs:146-166`; no `non_exhaustive` in crate |
| 5 | Forward progress | CRATE owns it; `block_total_length=8` rejected (`<12`); reader advances by parser-returned rem | `block_common.rs:101-123`, `read_buffer.rs:48-103` |
| 6 | Malformed input | Result-clean on truncation; `unwrap` guarded; `unimplemented!`/`panic!` only on misuse | `enhanced_packet.rs:41-57`, `unknown.rs:36`, `block_common.rs:213` |
| 7 | snaplen validation | NO check; snaplen stored on IDB only; lookup hook exists, enforcement does not | `interface_description.rs:33,52`, `parser.rs:124-126` |

---

## Decisive answer to the gating question

**Is the BC-2.01.014 pure-core timestamp helper NEEDED or REDUNDANT?**

- On the high-level `Block` / `EnhancedPacketBlock` API path: the helper is **REDUNDANT** in
  the narrow sense that you can no longer call it (no raw ticks survive), **but the crate's
  substitute is incorrect** for any capture whose `if_tsresol != 9` (notably the spec-default
  microsecond resolution, tsresol=6). Consuming `EnhancedPacketBlock::timestamp` directly
  yields wrong wall-clock times for the common case.
- The only correct path is the **raw path** (`next_raw_block` / `RawBlock`, or the obsolete
  `PacketBlock::timestamp: u64`), and on that path the helper is **NEEDED and correct**, and
  the **H-1 / SEC-001 overflow guards are a real fix, not a delete** — because that is exactly
  where wirerust performs the split-tick combine and the `10^-tsresol` / binary-tsresol scaling
  that overflow.

**Therefore: keep the helper, drive it from the raw-block path, and keep the H-1/SEC-001
overflow cluster as a live fix.** Do not wire the helper onto `EnhancedPacketBlock::timestamp`
(double-application / lost-resolution hazard). This conclusion should be reflected in the F2
pcapng spec remediation. (No spec was modified by this spike.)

---

# ADDENDUM — F2 pcapng raw-block path bootstrapping (HS-103 / BC-2.01.010 / IDB offsets)

**Date:** 2026-06-19
**Scope:** Three questions gating the F2 pcapng raw-block path. Authoritative source reading of
the vendored crate (same source tree as above). All citations `file:line` relative to crate
`src/` root, quoted verbatim.

## VERDICTS (read first)

1. **Does `RawBlock` expose the SHB body VERBATIM for self-detection?**
   **YES — but with one critical nuance the F2 design MUST account for.** The `RawBlock.body`
   field is the raw, uninterpreted block body, framed as `Cow::Borrowed(&slice[..body_len])`
   (`block_common.rs:111,121`). For the SHB, the body begins at the **Byte-Order Magic (BOM)**:
   the crate does NOT consume the BOM into the body. `body` starts at body offset 0 =
   BOM(4) | major(2) | minor(2) | section_length(8) | options... — exactly the verbatim SHB
   body layout. wirerust CAN read `RawBlock.body[0..4]` and self-detect section endianness from
   the BOM. **Nuance:** `next_raw_block` / `RawBlock::from_slice` PEEKS the BOM internally
   (`block_common.rs:80-86`) only to choose how to read the SHB's `initial_len` and to validate
   the trailer — it does NOT strip or rewrite the body. So self-detection is both *possible* and
   *redundant-but-harmless*: the crate already knows the endianness, but it leaves the BOM in the
   body for you to re-derive. BC-2.01.010 raw BOM-detection is fully implementable on
   `RawBlock.body`.

2. **Is endianness bootstrapping handled such that BC-2.01.010 raw BOM-detection is
   implementable?**
   **YES.** The crate solves the chicken-and-egg (need endianness to read `block_total_length`,
   but endianness lives in the BOM inside the body) by treating the SHB as a special case: it
   reads the SHB `initial_len` as BigEndian, peeks the BOM, then byte-swaps `initial_len` if the
   BOM says little-endian (`block_common.rs:76-89`). The `RawBlock` it returns has
   `initial_len`/`trailer_len` **already resolved to native u32 values** AND `body` **left raw**
   (BOM intact at body offset 0). So wirerust gets both: a correctly-framed block extent for
   forward progress, AND the raw BOM to re-derive endianness for major/minor/section_length.
   BC-2.01.010 is implementable: read `body[0..4]` as u32-BE, compare against `0x1A2B3C4D`
   (big) / `0x4D3C2B1A` (little), then read major @4, minor @6, section_length @8 in the
   detected endianness.

3. **IDB snaplen byte offset — CONFIRM adversary C-1.**
   **CONFIRMED: snaplen is at IDB body offset 4-7, with a 2-byte reserved field @2-3.** The
   wire body field order is exactly: `linktype: u16 @0-1`, `reserved: u16 @2-3`,
   `snaplen: u32 @4-7`. Source proves snaplen is NOT at offset 2. The crate additionally
   *validates* `reserved == 0` and errors otherwise — a detail F2 should mirror or tolerate.

---

## Q-A1 — SHB raw-body exposure (KEYSTONE for HS-103 / BC-2.01.010)

**Verdict: `RawBlock` exposes the SHB body VERBATIM starting at the BOM. The crate peeks the
BOM to resolve framing but does NOT consume/rewrite it; `RawBlock.body[0]` = BOM byte 0.**

`RawBlock` struct (`block_common.rs:54-64`):
```rust
pub struct RawBlock<'a> {
    pub type_: u32,         // block type (resolved to native u32)
    pub initial_len: u32,   // block_total_length (resolved to native u32)
    pub body: Cow<'a, [u8]>,// raw block body, VERBATIM — BOM intact for SHB
    pub trailer_len: u32,
}
```

How `from_slice` frames the SHB (`block_common.rs:73-93`):
```rust
let type_ = slice.read_u32::<B>().unwrap();
// Special case for the section header because we don't know the endianness yet
if type_ == SECTION_HEADER_BLOCK {
    let initial_len = slice.read_u32::<BigEndian>().unwrap();
    // Check the first field of the Section header to find the endianness
    let mut tmp_slice = slice;
    let magic = tmp_slice.read_u32::<BigEndian>().unwrap();   // PEEK only — tmp_slice, not slice
    let res = match magic {
        0x1A2B3C4D => inner_parse::<BigEndian>(slice, type_, initial_len),
        0x4D3C2B1A => inner_parse::<LittleEndian>(slice, type_, initial_len.swap_bytes()),
        _ => Err(PcapError::InvalidField("SectionHeaderBlock: invalid magic number")),
    };
    return res;
}
```
Key proof that the BOM survives into `body`: the magic peek is done on `tmp_slice` (a copy of
the cursor, `:80-81`), so the real `slice` cursor is still positioned at the BOM when handed to
`inner_parse`. Then `inner_parse` carves the body from offset 0 of that slice
(`block_common.rs:110-111`):
```rust
let body_len = initial_len - 12;
let body = &slice[..body_len as usize];   // body[0] == BOM byte 0
```
The 12 subtracted is type(4) + initial_len(4) + trailer_len(4) — i.e. the block header/trailer
framing, NOT the BOM. The BOM is inside the body.

**Contrast with the parsed path:** if wirerust instead used `Block`/`SectionHeaderBlock`, the
crate consumes the BOM and hands back a resolved `endianness: Endianness` field with the BOM
gone (`section_header.rs:19-21,47-52`). So self-detection is ONLY possible on the **raw** path —
which is exactly the F2 raw-block path. On the raw path it is fully possible.

**BC-2.01.010 implementability:** CONFIRMED. wirerust reads `raw_block.body[0..4]` as a u32 in
big-endian, matches `0x1A2B3C4D` (section is big-endian) or `0x4D3C2B1A` (little-endian), and
proceeds to read major/minor/section_length from the body at the offsets below in the detected
endianness. This is the same logic the crate uses at `section_header.rs:47-52`.

## Q-A2 — Endianness bootstrapping on the raw path

**Verdict: The crate resolves SHB framing fields (`initial_len`, `trailer_len`) to native u32
by peeking the BOM, while leaving the body raw. wirerust does NOT need to solve framing
endianness itself, AND retains the raw BOM to detect body-field endianness. Both halves of the
bootstrap are satisfied.**

Mechanics, step by step (`block_common.rs:76-119`):
1. SHB `type_` (`0x0A0D0D0A`) is endian-agnostic (palindromic magic), so it reads identically
   either way (`:73`, const `:25`).
2. `initial_len` is read as BigEndian first (`:77`), then the BOM is peeked (`:81`); if the BOM
   is little-endian, `initial_len` is byte-swapped before use
   (`:84` `initial_len.swap_bytes()`). So the returned `RawBlock.initial_len` is a correct
   native value regardless of section endianness.
3. The trailer length is read in the resolved endianness `B` and checked against `initial_len`
   (`:115-119`): `if initial_len != trailer_len { return Err(...) }`. This is the forward-progress
   / integrity guard; it is done FOR wirerust.
4. The body (BOM-first) is returned raw (`:111,121`).

**SHB body field offsets (for BC-2.01.010), derived from `section_header.rs:47-67`:**

| Field | Body offset | Width | How crate reads it |
|-------|-------------|-------|--------------------|
| Byte-Order Magic (BOM) | 0 | 4 | `read_u32::<BigEndian>` then match (`section_header.rs:47`) |
| major_version | 4 | 2 | `read_u16::<B>` (`:65`) |
| minor_version | 6 | 2 | `read_u16::<B>` (`:66`) |
| section_length | 8 | 8 (i64) | `read_i64::<B>` (`:67`) |
| options | 16 | var | `opts_from_slice::<B>` (`:68`) |

Note the crate reads the BOM itself in BigEndian and *matches the raw 32-bit value* against the
two magic constants (`section_header.rs:47-52`) — it does NOT need to know endianness in advance
because both magic constants are distinct 32-bit patterns. wirerust must do the same: read BOM
as a fixed-endian u32 and compare, not "read then interpret."

**Conclusion:** BC-2.01.010 raw BOM-detection is implementable. The crate already proves the
exact algorithm at `section_header.rs:47-52` and proves the framing-bootstrap at
`block_common.rs:76-89`. wirerust on the raw path gets resolved framing (free) + raw body
(BOM intact) — everything needed for self-detection.

## Q-A3 — IDB body field layout (confirms adversary C-1)

**Verdict: CONFIRMED. IDB wire body is `linktype: u16 @0-1`, `reserved: u16 @2-3`,
`snaplen: u32 @4-7`. snaplen is at body offset 4, NOT 2. The crate also validates
`reserved == 0`.**

Parse source (`interface_description.rs:40-57`), in read order = wire order:
```rust
if slice.len() < 8 {
    return Err(PcapError::InvalidField("InterfaceDescriptionBlock: block length < 8"));
}
let linktype = (slice.read_u16::<B>().unwrap() as u32).into();  // @0-1  (u16)
let reserved = slice.read_u16::<B>().unwrap();                  // @2-3  (u16)
if reserved != 0 {
    return Err(PcapError::InvalidField("InterfaceDescriptionBlock: reserved != 0"));
}
let snaplen = slice.read_u32::<B>().unwrap();                   // @4-7  (u32)
let (slice, options) = InterfaceDescriptionOption::opts_from_slice::<B>(slice)?;
```
The reads are sequential off the body slice with no seeking, so offsets are cumulative:

| Field | Body offset | Width | Cite |
|-------|-------------|-------|------|
| linktype | 0 | 2 (u16, widened to u32) | `interface_description.rs:45` |
| reserved | 2 | 2 (u16, must be 0) | `interface_description.rs:47-50` |
| snaplen | 4 | 4 (u32) | `interface_description.rs:52` |
| options | 8 | var | `interface_description.rs:53` |

**Adversary C-1 disposition:** the claim that snaplen sits at body offset 2 is WRONG. snaplen is
at offset 4-7; a 2-byte reserved field occupies @2-3. Any F2 logic computing snaplen position
must use offset 4. Additional crate behavior F2 should note: the crate REJECTS a non-zero
reserved field (`:48-49`), and requires body length >= 8 before reading (`:41-42`). If F2's raw
path hand-parses the IDB body, mirroring the `reserved == 0` check is optional (the spec leaves
reserved "should be zero"), but the offsets are non-negotiable: linktype@0, reserved@2,
snaplen@4.

## Addendum summary table

| Q | Topic | Verdict | Key cite |
|---|-------|---------|----------|
| A1 | SHB raw-body verbatim? | YES — `RawBlock.body[0]` = BOM byte 0; crate peeks BOM on `tmp_slice`, leaves real body raw | `block_common.rs:54-64,80-93,110-111,121` |
| A2 | Endianness bootstrap implementable? | YES — framing (`initial_len`/trailer) resolved by crate; BOM left in body for self-detect; algorithm proven | `block_common.rs:76-119`, `section_header.rs:47-67` |
| A3 | IDB snaplen offset | CONFIRMED @4-7; reserved u16 @2-3; linktype u16 @0-1; crate enforces reserved==0 | `interface_description.rs:45-53` |

(No spec was modified by this addendum.)
