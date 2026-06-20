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
