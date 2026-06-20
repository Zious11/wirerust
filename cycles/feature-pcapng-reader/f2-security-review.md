---
document_type: security-review
review_id: SR-PCAPNG-F2
phase: F2
feature: FE-001-pcapng-reader-support
reviewer: security-reviewer
date: 2026-06-19
scope: specification-only (no implementation exists; F4 is the implementation phase)
total_findings: 9
critical: 0
high: 2
medium: 4
low: 3
files_reviewed: 13
artifacts_reviewed:
  - .factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.009.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.010.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.011.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.012.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.013.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.014.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.015.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.016.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.017.md
  - .factory/specs/behavioral-contracts/ss-01/BC-2.01.018.md
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/specs/prd-supplements/nfr-catalog.md
  - src/reader.rs (classic-pcap reference path for parity analysis)
  - .factory/research/pcapng-parser-dependency-eval.md
---

# Security Review: F2 pcapng Reader Specification

## Scope and Context

This is a **specification-only** security review. No implementation exists yet (F4 is the
implementation phase). The goal is to determine whether the F2 specification — ADR-009,
BC-2.01.009 through BC-2.01.018, error taxonomy entries E-INP-008 through E-INP-012, and
relevant NFRs — **adequately requires** defensive behavior against the pcapng malformed-input
threat surface before the implementer writes a single line of code.

pcapng is exclusively attacker-controlled binary input. The classic-pcap path in
`src/reader.rs` is the security baseline: it provides a worked example of the discipline the
pcapng path must match or exceed. Any spec requirement that is weaker than the classic path
posture is a pre-implementation gap.

**Threat model:** An adversary delivers a malformed or adversarially-crafted `.pcapng` file
(or a file with a `.cap` / `.pcap` extension containing pcapng bytes) to wirerust's CLI. The
adversary controls every byte of the block stream. The adversary's goals are: (1) cause a
panic/abort that discredits the tool; (2) cause an unbounded allocation that exhausts memory;
(3) cause an infinite loop that hangs the process; (4) cause an out-of-bounds read producing
garbage results or information disclosure.

---

## Executive Summary

The F2 specification is substantially well-designed for the malformed-input threat surface.
The spec team correctly identified the key defensive requirements: fail-to-Err rather than
panic, explicit error taxonomy entries for each pcapng error class, anyhow context chains
(BC-2.01.017), no unwrap on the pcapng path, and a pure-core timestamp function designated
for Kani proof (BC-2.01.014). These are genuine strengths.

However, the review identifies **2 HIGH findings and 4 MEDIUM findings** — all spec gaps
where a required defense is either absent, partially specified, or not testable as written.
None are CRITICAL, but the two HIGH findings (SEC-002 and SEC-004) must be resolved before
F3 story decomposition to avoid implementing with under-specified safety contracts. The most
significant gap is the timestamp arithmetic in BC-2.01.014, which specifies an intermediate
expression that overflows u64 for certain legal `if_tsresol` values — a CWE-190 that Kani
is being asked to prove but cannot, because the spec itself is the source of the defect.

A cargo-fuzz harness on the pcapng reader path should be mandated as an F6 hardening
deliverable (not F3), but the spec should already require the property it would exercise:
no panic for any byte sequence.

---

## Findings

### SEC-001: CWE-190 Integer Overflow in Intermediate ts_usecs Computation (BC-2.01.014)

- **Severity:** HIGH
- **CWE:** CWE-190 (Integer Overflow or Wraparound)
- **OWASP:** A04:2021 – Insecure Design
- **Spec Coverage:** Partial — the spec acknowledges overflow risk for `ts_sec` and mandates
  saturating behavior, but does NOT address overflow in the `ts_usecs` intermediate expression.
- **Attack Vector:** An adversary crafts an EPB with `ts_high` and `ts_low` set to near-maximum
  values, combined with a legal `if_tsresol` exponent that produces a large `ticks_per_sec`
  (e.g., `if_tsresol = 6`, so `ticks_per_sec = 1_000_000`). The spec requires (Postcondition 2):
  `ts_usecs = ((ticks % ticks_per_sec) * 1_000_000 / ticks_per_sec) as u32`.
  The intermediate expression `(ticks % ticks_per_sec) * 1_000_000` is a u64 multiplication.
  `ticks % ticks_per_sec` is at most `ticks_per_sec - 1`. For `if_tsresol = 6`:
  `max_intermediate = (1_000_000 - 1) * 1_000_000 = 999_999_000_000`, which fits in u64
  (max u64 ≈ 1.8 × 10^19). For `if_tsresol = 9` (nanoseconds): `max_intermediate =
  (1_000_000_000 - 1) * 1_000_000 ≈ 10^15`, still fits. However, for base-2 exponents with
  large e (e.g., `if_tsresol = 0x80 | 0x3E` → 2^62 ticks/sec): `ticks_per_sec = 2^62 ≈
  4.6 × 10^18`, and `ticks % ticks_per_sec` may be up to `2^62 - 1`. The intermediate
  expression `(2^62 - 1) * 1_000_000` overflows u64 (max is ~1.8 × 10^19; `2^62 × 10^6
  ≈ 4.6 × 10^24`). In Rust's default checked arithmetic at debug builds this panics; in
  release builds (even with `overflow-checks = true` in Cargo.toml) this panics in a
  structured way — but the spec claims no panic for any u8 input (BC-2.01.014 Invariant 1).
- **Impact:** Process panic on adversary-crafted pcapng with a high-resolution base-2
  `if_tsresol` combined with large timestamp values. This also means the Kani proof target
  (BC-2.01.014 Verification Properties VP-NNN "no panic for any (u32, u32, u8) input") will
  FAIL if the spec formula is implemented literally — exposing the gap at F6, not F3.
- **Evidence:** BC-2.01.014 Postcondition 2:
  `ts_usecs = ((ticks % ticks_per_sec) * 1_000_000 / ticks_per_sec) as u32`
  The multiplication is on the u64 type. For `if_tsresol & 0x7F >= 43` (base-2 exponent ≥ 43,
  giving `ticks_per_sec ≥ 2^43 ≈ 8.8 × 10^12`), the product can approach u64::MAX before
  the division. For `e = 62` the product overflows definitively.
  BC-2.01.014 Postcondition 2 also specifies `ticks_per_sec: u64 = 10u64.pow(e as u32)`.
  For base-10, `e = 20` yields `10^20`, which already overflows u64::MAX (≈ 1.8 × 10^19).
  `if_tsresol & 0x7F = 20` is a valid byte value an adversary can supply. This is an
  additional overflow path: `10u64.pow(20)` panics in both debug and release.
- **Proposed Mitigation:** The spec must explicitly require saturating or checked arithmetic
  for both the `ticks_per_sec` computation and the intermediate `ts_usecs` product:
  1. For base-10: `ticks_per_sec = 10u64.checked_pow(e as u32).unwrap_or(u64::MAX)` — if
     the exponent overflows the denominator, the result is zero ticks per second of remainder,
     so `ts_usecs = 0`.
  2. For base-2: the shift `1u64 << e` is already specified; for `e >= 64` (impossible since
     e is `u8 & 0x7F`, so max 127, but `1u64 << 63` is valid and `1u64 << 64` is UB in
     Rust) — the spec must clamp e to 63 before the shift or use `u64::MAX` for overflow.
  3. For the intermediate multiplication: use `u128` for the intermediate computation or
     apply a saturating multiply: if the product overflows u64, `ts_usecs = 0` (or clamp).
  **Recommended BC addition:** Add an AC or Invariant to BC-2.01.014 explicitly requiring
  saturating behavior for all intermediate u64 arithmetic. The Kani proof objective already
  asserts no-panic; make the spec's arithmetic safe so the proof can succeed.

---

### SEC-002: CWE-835 Infinite Loop Risk — Forward Progress Not Mandated in Unknown-Block Skip (BC-2.01.015)

- **Severity:** HIGH
- **CWE:** CWE-835 (Loop with Unreachable Exit Condition / Infinite Loop)
- **OWASP:** A04:2021 – Insecure Design
- **Spec Coverage:** N — the spec does not require forward-progress guarantees on the
  block-walk loop, and the skip logic it mandates has a structural hole for adversary-crafted
  `block_total_length` values.
- **Attack Vector:** An adversary crafts a block with `block_total_length = 8` (the minimum
  the spec permits as non-erroring per BC-2.01.015 EC-004: "block_total_length = 8 (block
  with empty body) → 0 bytes skipped; no error"). The spec says `block_total_length - 8 = 0`
  bytes are consumed and parsing continues. If the underlying block-walk loop (driven by
  `pcap-file 2.0.0`'s iterator) does NOT itself advance past the 8-byte header for this
  case, the position in the stream is unchanged and the next iteration reads the same block
  header again — producing an infinite loop. The adversary simply repeats this pattern in
  any crafted unknown block.
  Additionally: if the upstream library's block walker does not itself ensure the cursor
  advances at least 8 bytes on any returned `Block`, wirerust's loop over `PcapNgParser`
  results cannot guarantee forward progress purely from the spec as written.
- **Impact:** Infinite loop (or very long loop) consuming 100% CPU, effectively a
  denial-of-service against the wirerust process. A single crafted pcapng file hangs the
  CLI indefinitely.
- **Evidence:** BC-2.01.015 EC-004: `block_total_length = 8 (block with empty body) → 0
  bytes consumed; no error`. The spec mandates consuming `block_total_length - 8` bytes,
  which is 0. No invariant anywhere requires that the block-walk loop advances the stream
  by at least 8 bytes total (header + body) per iteration. BC-2.01.015 Postcondition 7
  guards `block_total_length < 8` with an error, but `= 8` is a valid no-consume case
  that creates the zero-advance condition.
  Separately: the classic-pcap path in `src/reader.rs` avoids this entirely because
  `next_raw_packet()` on the `PcapReader` iterator always advances by at least a full packet
  record. The pcapng block-walk equivalent relies on `pcap-file 2.0.0`'s `PcapNgParser`
  to provide this guarantee, but the spec does not document that trust or verify it.
- **Proposed Mitigation:** Add an explicit invariant to BC-2.01.015 (or to BC-2.01.017 as a
  cross-cutting loop invariant):
  "The block-walk loop MUST make forward progress of at least 8 bytes on every iteration
  (the 4-byte block-type field plus the 4-byte block-total-length field are always consumed
  before any dispatching). An implementation that relies on `pcap-file 2.0.0`'s block walker
  MUST verify that the library never re-returns the same block position."
  Additionally add a testable AC: "A pcapng file containing a block with `block_total_length =
  8` MUST be processed in finite time and return within one iteration of the block loop."
  If the implementation hand-rolls the loop rather than delegating to the library iterator,
  the loop must track the stream position before and after each iteration and return
  `Err(E-INP-010)` if no forward progress was made.

---

### SEC-003: CWE-125 Out-of-Bounds Read via EPB interface_id Not Bounded by Table Size — Mapping to Error is Spec-Specified but Error Code is Wrong (BC-2.01.012)

- **Severity:** MEDIUM
- **CWE:** CWE-125 (Out-of-Bounds Read)
- **OWASP:** A04:2021 – Insecure Design
- **Spec Coverage:** Partial — the spec identifies the threat and mandates an error, but
  maps it to the wrong error code.
- **Attack Vector:** An adversary crafts an EPB where `interface_id` is an arbitrary 32-bit
  unsigned integer (the pcapng spec defines it as u32). If the implementation indexes into
  the interface table (a Vec or similar) using this value without bounds checking, and the
  value exceeds the Vec length, a panic or OOB read results.
- **Evidence:** BC-2.01.012 Postcondition 5: "An EPB with `interface_id` referencing an
  interface index not in the table returns `Err` mapping to **E-INP-008**." The error
  taxonomy entry E-INP-008 covers "structural parse failures at the SHB or IDB level:
  truncated file, missing BOM, malformed block-total-length, unsupported major version."
  An out-of-range `interface_id` on an EPB is semantically distinct from a structural SHB/IDB
  failure. The dedicated error entry for this condition is **E-INP-009**, whose description
  covers "EPB encountered before any IDB" (a related but different condition). Neither entry
  exactly describes "EPB interface_id out of range." This taxonomy mismatch risks the
  implementer producing an unhelpful error message or the test-writer testing the wrong error
  code, creating a latent defect.
- **Impact:** If the error mapping is wrong, the implementer may not add a bounds-check guard
  and may rely on the library to handle it — which may or may not produce a clean error vs.
  a panic. The correct error message ("interface_id=N references interface not in table (max
  index M)") is not specified anywhere in the taxonomy.
- **Proposed Mitigation:**
  1. Correct BC-2.01.012 Postcondition 5 to map to E-INP-010 (block-level data inconsistency)
     with a context string naming the specific violation: `"EPB interface_id={id} out of range
     (interface table size={n})"`.
  2. OR: add a new E-INP-013 entry specifically for "EPB interface_id out of table range" —
     this provides the clearest implementer signal.
  3. Add an explicit AC to BC-2.01.012: "The interface_id field from EPB MUST be checked
     against the current interface table size before any indexing operation. An unchecked
     array index on interface_id is not permitted."
  This makes the guard a testable acceptance criterion rather than an implied implementation
  detail.

---

### SEC-004: CWE-789 / CWE-400 Memory Exhaustion — No Allocation Bound Specified for the Packet Vector (All BCs)

- **Severity:** MEDIUM
- **CWE:** CWE-789 (Memory Allocation with Excessive Size Value), CWE-400 (Uncontrolled Resource Consumption)
- **OWASP:** A05:2021 – Security Misconfiguration / A04:2021 – Insecure Design
- **Spec Coverage:** N — neither the BCs nor the NFR catalog impose any upper bound on the
  total number of packets collected from a pcapng file, or on the total memory allocated by
  `PcapSource.packets`.
- **Attack Vector:** An adversary crafts a pcapng file containing an extremely large number
  of EPBs, each with a small (e.g., zero-byte) payload. Each EPB produces one `RawPacket`
  (a struct with two u32 fields and a `Vec<u8>`). With no limit, the parser iterates over
  millions or billions of blocks, allocating a `RawPacket` for each, until the host OOM-kills
  the process. On a 64-bit system with address space but not physical RAM, this causes an
  extended period of swap thrash before termination.
  Alternatively: an adversary crafts an EPB with a maliciously large `captured_length` value
  that is consistent with `block_total_length`. The spec says (BC-2.01.012 Postcondition 3)
  the data is "copied from the EPB body bounded by `captured_length`." If `captured_length`
  is, e.g., 2 GB and `block_total_length` also encodes that size, the implementation
  allocates a 2 GB Vec for a single packet.
- **Impact:** Out-of-memory crash (denial of service) when processing a malicious pcapng file.
  This is not a crash in the security sense but defeats the tool's availability guarantee.
- **Parity Context:** The classic-pcap path in `src/reader.rs` has no explicit packet count
  bound either (it is a known limitation: NFR-PERF-002, NFR-VIO-001, "eager Vec<RawPacket>
  load, RAM = pcap_file_size × 1.5"). However, the classic-pcap path is bounded implicitly
  by the file size of a well-formed `.pcap` file — each packet record has overhead, and files
  are real. pcapng block sizes are also real-file-bounded for benign inputs. The threat
  surface widens for pcapng because the format allows block-total-length to be adversarially
  large in a compact on-disk representation (a single 8-byte header can claim 2 GB of
  payload, which a real classic pcap file cannot do because the data bytes would need to be
  present in the file).
- **Proposed Mitigation:** The spec should add at minimum a note acknowledging this limitation
  and, for pcapng specifically, should require bounds on per-packet captured_length. Two
  recommended additions:
  1. Add an AC to BC-2.01.012: "captured_length MUST be validated against the physical
     block_total_length MINUS the EPB fixed-field overhead (20 bytes) before any allocation.
     If captured_length exceeds `block_total_length - 20`, the reader MUST return
     `Err(E-INP-010)` rather than attempting the allocation."
  2. Add an NFR or note to NFR-PERF-002 / NFR-RES that pcapng files with extreme packet
     counts are subject to the same eager-load limitation as classic pcap, and that the
     pcapng reader inherits this known limitation — no new per-packet count cap is mandated
     for this cycle, but the captured_length vs. block_total_length consistency check IS
     required because it prevents single-block multi-GB allocations.
  Note: item 1 is partly implied by BC-2.01.012 Postcondition 6 ("captured_length exceeding
  block_total_length minus EPB fixed-field overhead → Err E-INP-010"), but that postcondition
  does not explicitly tie the data allocation to that check. The AC should make it explicit
  that the guard precedes the allocation.

---

### SEC-005: CWE-248 / CWE-617 Panic Discipline — No-Panic Contract Is Asserted but Not Testable as a Standalone AC (BC-2.01.017)

- **Severity:** MEDIUM
- **CWE:** CWE-248 (Uncaught Exception), CWE-617 (Reachable Assertion)
- **OWASP:** A04:2021 – Insecure Design
- **Spec Coverage:** Partial — BC-2.01.017 states "No pcapng parse error produces a `panic!`
  or an `unwrap` in production code" and the Verification Properties section lists "No panic
  on malformed pcapng (any truncation point) | fuzz: truncate well-formed pcapng at every
  offset; assert no panic." However, this is listed as a VP property backed by fuzzing that
  does not yet exist, not as an AC in any individual parsing BC.
- **Attack Vector:** An adversary supplies a crafted pcapng where specific byte offsets within
  a block header or body trigger an unwrap or expect call in the implementation. The spec
  requires Err return, but without a testable AC at the BC level, an implementer might add an
  unwrap on what they believe is an "impossible" parse failure and ship it.
- **Evidence:** BC-2.01.017 Postcondition 3 says "No panic, no `unwrap`, no `expect` in the
  pcapng code path." This is an implementation constraint in BC-2.01.017, a cross-cutting BC.
  However: none of BC-2.01.010 through BC-2.01.015 have a standalone AC that reads "this
  block parser MUST return Err (not panic) for any malformed input." The constraint lives only
  in BC-2.01.017, which an implementer working on STORY-124 (IDB parsing) may not read in
  full.
  In the classic-pcap path (`src/reader.rs`), NFR-REL-008 ("Errors propagate via
  `anyhow::Result` from all file/reader/decoder paths; no `panic!` on bad input") is an
  explicitly tested NFR. The pcapng spec should similarly have this as a per-BC AC.
- **Proposed Mitigation:** Add a standard AC to each of BC-2.01.010, BC-2.01.011,
  BC-2.01.012, BC-2.01.013, BC-2.01.015 (and any additional block-parsing BCs added in F3):
  "**No-panic AC:** This block parser MUST return `Err(anyhow::Error)` for any malformed or
  truncated input. `unwrap()`, `expect()`, and `panic!()` are prohibited in the implementation
  of this BC. This requirement is testable by the fuzzing harness in F6 and by truncation
  unit tests at every fixed-field boundary."
  This elevates the no-panic requirement from a read-once cross-cutting note (BC-2.01.017)
  to a per-story, per-block testable acceptance criterion.

---

### SEC-006: CWE-20 Insufficient Input Validation — if_tsresol base-2 Shift Exponent Not Clamped (BC-2.01.014)

- **Severity:** MEDIUM
- **CWE:** CWE-20 (Improper Input Validation)
- **OWASP:** A03:2021 – Injection (numeric input)
- **Spec Coverage:** Partial — the spec defines the formula for base-2 exponents but does not
  address the case where `e = if_tsresol & 0x7F` is 63 or higher.
- **Attack Vector:** An adversary supplies `if_tsresol = 0xFF` (bit 7 set = base-2; lower
  7 bits = 0x7F = 127). The spec requires `ticks_per_sec = 1u64 << 127`. In Rust, a shift
  amount >= 64 on a u64 value is a panic (debug) or undefined behavior (in unchecked mode).
  With `overflow-checks = true` in `[profile.release]` (which wirerust does set — NFR-REL-001),
  this panics at runtime in both debug and release. The adversary thus achieves a panic by
  supplying a single byte value in the IDB's `if_tsresol` option.
- **Impact:** Process panic triggered by a single crafted IDB option byte.
- **Evidence:** BC-2.01.014 Postcondition 3: `ticks_per_sec: u64 = 1u64 << e` where `e =
  if_tsresol & 0x7F`. The maximum value of `e` is 127 (0x7F). Rust's `u64 << 127` panics
  with overflow-checks = true. EC-006 in BC-2.01.014 acknowledges `if_tsresol = 0x3F`
  (base-2, e=63 → `2^63`) but only comments "ticks likely < ticks_per_sec; ts_sec=0,
  ts_usecs=0" — it does not note the shift itself (`1u64 << 63`) is valid (the maximum
  non-panicking shift), while `1u64 << 64` and above would panic. EC-006 uses e=63 which is
  the safe boundary, but the spec does not mandate clamping e to 63.
  An adversary controlling `if_tsresol` can set `e = 64` or higher (e.g., `if_tsresol =
  0x80 | 0x40 = 0xC0`, giving e=64), triggering the panic.
- **Proposed Mitigation:** Add to BC-2.01.014 (Invariants or Postcondition 3):
  "The base-2 shift exponent `e` MUST be clamped to the range [0, 63] before computing
  `ticks_per_sec`. If `e >= 64`, the exponent is treated as saturating: `ticks_per_sec =
  u64::MAX` (effectively 1-tick-per-u64MAX-seconds; `ts_sec = 0` for any plausible
  timestamp). No panic is permitted for any value of `e` in [0, 127]. The `1u64 << e`
  expression MUST be wrapped with a bounds check or replaced with `1u64.checked_shl(e as
  u32).unwrap_or(u64::MAX)`."
  This is closely related to SEC-001 but is a distinct attack vector (shift vs. multiply).

---

### SEC-007: CWE-693 / DSB Block — Secret Material Passed to Skip Path Not Explicitly Guarded Against Logging (BC-2.01.015)

- **Severity:** LOW
- **CWE:** CWE-693 (Protection Mechanism Failure), CWE-532 (Insertion of Sensitive Information into Log File)
- **OWASP:** A09:2021 – Security Logging and Monitoring Failures
- **Spec Coverage:** Y — DSB is in the skip-AC list in BC-2.01.015 AC-001.
- **Analysis:** BC-2.01.015 AC-001 explicitly names `DecryptionSecretsBlock (DSB, type
  0x0000000A) — TLS key log material; no packet data; silently skipped`. EC-009 confirms
  "silently skipped; TLS key material NOT used; no warning." The spec is correct: DSB bytes
  are consumed via the `block_total_length` skip and discarded. They are never parsed into
  a structured type, never logged to stderr, and never included in any Finding.
  This is correctly specified. The risk is LOW rather than non-existent because:
  1. If an implementer adds debug logging (e.g., `log::debug!("{:?}", block_bytes)`) to the
     unknown-block skip path, TLS key material would appear in debug logs.
  2. The spec does not prohibit debug log emission from the skip path; it only prohibits
     stderr warnings.
- **Proposed Mitigation:** Add a note to BC-2.01.015 AC-002 explicitly prohibiting logging
  of block body bytes in the skip path: "Block body bytes MUST NOT be logged, printed, or
  included in any diagnostic output, regardless of log level. This applies especially to
  DSB blocks which may carry TLS session keys." This is a low-cost addition that makes the
  constraint explicit for the implementer.

---

### SEC-008: CWE-345 / Supply Chain — pcap-file 2.0.0 pcapng Parser Bounds Behavior Unverified at Runtime (ADR-009, Research Eval)

- **Severity:** LOW
- **CWE:** CWE-345 (Insufficient Verification of Data Authenticity), from a supply-chain
  trust perspective; CWE-20 (Improper Input Validation) from the delegated-validation angle.
- **OWASP:** A06:2021 – Vulnerable and Outdated Components
- **Spec Coverage:** Partial — the research evaluation explicitly flags this as "partially
  inconclusive": "I did not exercise a runtime test proving the reader correctly applies
  [if_tsresol] to EPB 64-bit timestamps."
- **Analysis:** The ADR selects `pcap-file 2.0.0`'s `PcapNgReader` as the parser (Option A).
  The research evaluation notes the crate has no known RUSTSEC advisories (as of 2026-06-19).
  The dependency has 10M+ downloads and is well-exercised in the field. However:
  1. The crate's pcapng parsing behavior under malformed inputs (specifically: what does
     `PcapNgParser::next_block()` return if `block_total_length` is internally inconsistent?)
     is not verified at the spec level. The spec assumes the library surfaces errors as
     `PcapError` values that can be mapped to anyhow, but does not require a test to confirm
     this assumption.
  2. The `pcap-file` 2.0.0 crate's last stable release was 2023-02-01 (over 3 years ago).
     While there are no known advisories, the long interval since last release means any
     issues discovered since then would be in the RC-only `3.0.0-rc.2` track, which wirerust
     cannot adopt.
- **Impact:** If `pcap-file 2.0.0` panics internally on certain malformed inputs rather than
  returning a clean error, the spec's no-panic guarantee cannot be satisfied regardless of
  how well the wrapper code is written.
- **Proposed Mitigation:** The spec (via ADR-009 Consequences or a new BC-2.01.017 AC) should
  require a validation test confirming that `PcapNgParser::next_block()` returns
  `Err(PcapError)` (not panic) for the following crafted inputs: (a) block with
  `block_total_length = 0`, (b) truncated SHB with only 8 bytes, (c) EPB with
  `captured_length > block_total_length`. This is a one-time confirmation that the library
  behaves as expected under adversarial conditions, complementing the fuzzing requirement.
  This check can be a unit test in STORY-126 or -127.

---

### SEC-009: CWE-190 / Parity Gap — Classic-pcap Path Has Explicit ts_frac Division-by-1000 Guard; pcapng Path Has No Equivalent for Nanosecond Conversion (BC-2.01.014 vs. src/reader.rs)

- **Severity:** LOW
- **CWE:** CWE-190 (Integer Overflow or Wraparound)
- **OWASP:** A04:2021 – Insecure Design
- **Spec Coverage:** Partial — BC-2.01.014 Postcondition 5 correctly specifies the nanosecond
  case as `ts_usecs = ((ticks % 1_000_000_000) / 1_000) as u32`, which is safe (no overflow).
  The parity concern is about which cases are explicitly safe vs. which rely on the general
  formula.
- **Analysis:** The classic-pcap path in `src/reader.rs` (lines 71-74) handles nanosecond
  timestamps with explicit integer division: `raw_packet.ts_frac / 1_000`. This is safe
  because `ts_frac` is a u32 and `u32 / 1000` never overflows. The pcapng timestamp
  conversion covers the `if_tsresol = 9` case explicitly in Postcondition 5 (the nanosecond
  fast path), which is also safe. The general formula in Postcondition 2
  (`((ticks % ticks_per_sec) * 1_000_000 / ticks_per_sec)`) is used for all other
  resolutions and is where SEC-001 and SEC-006 live. The nanosecond path is correctly
  specified separately and is the safer path. This finding notes the parity relationship and
  confirms no regression on the nanosecond case itself, but recommends the general formula be
  split into explicit fast paths (µs, ns) and a guarded general case, to bound the overflow
  risk to the rare exotic-resolution path.
- **Proposed Mitigation:** Amend BC-2.01.014 to enumerate three distinct, explicitly safe
  formulas — one for the µs default (if_tsresol=6), one for the ns common case
  (if_tsresol=9), and a guarded general formula for all other values — rather than one
  general formula with the µs and ns cases as special cases. This reduces the attack surface
  of the general formula to exotic resolutions only, and makes each case independently
  reviewable.

---

## Threat Surface Assessment Matrix

| # | Threat | CWE | Severity | Spec Coverage | Must Fix Before F3? |
|---|--------|-----|----------|---------------|---------------------|
| T1 | Integer overflow in ts_usecs intermediate (e.g. base-10 e=20 overflows u64, base-2 e>=43 overflows intermediate multiply) | CWE-190 | HIGH | Partial | YES |
| T2 | Infinite loop on zero-advance unknown block (block_total_length=8 consumes 0 bytes) | CWE-835 | HIGH | N | YES |
| T3 | OOB read via EPB interface_id > table size (wrong error code in spec) | CWE-125 | MEDIUM | Partial | YES |
| T4 | Memory exhaustion — no per-packet allocation size bound vs. block_total_length | CWE-789/400 | MEDIUM | N (implied but not AC) | YES (clarify the implied check) |
| T5 | No-panic requirement not testable as per-BC AC | CWE-248/617 | MEDIUM | Partial | YES |
| T6 | base-2 shift exponent not clamped (e >= 64 panics with overflow-checks=true) | CWE-20 | MEDIUM | N | YES |
| T7 | DSB secret material could appear in debug logs (skip path) | CWE-693/532 | LOW | Y | OPTIONAL |
| T8 | pcap-file 2.0.0 internal error behavior unverified at runtime | CWE-345/20 | LOW | Partial | NO (validate in STORY-126) |
| T9 | Parity: nanosecond conversion safe; general formula unsafe paths not isolated | CWE-190 | LOW | Partial | Combined with T1 |

---

## Classic-pcap Parity Assessment

The classic-pcap path in `src/reader.rs` sets the security baseline. Assessment of parity:

| Classic-pcap Discipline | pcapng Spec Parity |
|------------------------|-------------------|
| `next_raw_packet()` returns `Result`; no unwrap | BC-2.01.017 requires same — PARITY OK (but not per-BC testable: see SEC-005) |
| Link-type whitelist check on all accepted files | BC-2.01.016 mirrors exactly — PARITY OK |
| Nanosecond ts_frac / 1000 explicit division | BC-2.01.014 Postcondition 5 explicit — PARITY OK |
| `BufReader` wrapping for non-seekable streams | BC-2.01.009 Invariant 2 mandates BufReader — PARITY OK |
| No per-packet count bound (NFR-PERF-002, NFR-VIO-001) | pcapng inherits same limitation — PARITY OK (documented gap, not new) |
| Snaplen-truncated packets: `incl_len` used, not `orig_len` | BC-2.01.012 Postcondition 3 mandates `captured_length` — PARITY OK |
| No overflow guards on ts_frac (u32 arithmetic, safe by type) | BC-2.01.014 uses u64; intermediate overflow NOT guarded — PARITY GAP (SEC-001, SEC-006) |
| No block-walk loop (single stream of records) | pcapng block-walk: forward-progress not mandated — PARITY GAP (SEC-002) |
| No interface_id concept (global datalink type) | EPB interface_id bounds check implied but not AC'd — PARITY GAP (SEC-003) |

---

## Risk Register Dispositions

The pcapng feature BCs (BC-2.01.009 through BC-2.01.018) were introduced in F2 and do not
appear in an L2 Domain Spec Risk Register with explicit R-NNN entries for security (no
`.factory/specs/domain/risk-register.md` was loaded as part of this review scope — the
artifacts provided do not include a Risk Register). The NFR catalog does not contain any
OPEN or OPEN-DEBT security NFRs that are specifically scoped to the pcapng reader. The
relevant security NFRs (NFR-SEC-006 "no shell-out", NFR-SEC-007 "no network I/O") are
N/A for the pcapng reader feature as the reader has no shell-out or network egress.

The security-relevant NFR gaps identified in this review are:

- NFR-REL-008 ("no panic on bad input") applies to the pcapng path and is correctly inherited.
  However, its testability for pcapng is not yet wired (SEC-005).
- NFR-RES series: no equivalent of NFR-RES-011/016 (buffer caps) exists for the pcapng
  reader's per-packet allocation. This is an intentional design choice (same as classic pcap)
  but should be documented (SEC-004).

---

## Must-Fix Before F3 Story Decomposition

The following findings MUST be resolved (via BC amendment or NFR addition) before F3 stories
are written, because the story-level ACs and test vectors will be derived from the BCs:

1. **SEC-001** (HIGH): Fix BC-2.01.014 timestamp formula to specify saturating/checked
   arithmetic for base-10 pow (clamped to prevent u64 overflow at e >= 20) and for the
   intermediate usecs multiplication. The Kani proof target fails if this is not fixed.

2. **SEC-002** (HIGH): Add a forward-progress invariant to BC-2.01.015 requiring the
   block-walk loop to advance by at least 8 bytes per iteration. Add a testable AC for
   the `block_total_length = 8` edge case.

3. **SEC-003** (MEDIUM): Correct BC-2.01.012 Postcondition 5 to reference the correct
   error code (E-INP-010, not E-INP-008) and add an explicit AC requiring a bounds check
   before any interface_id-based indexing.

4. **SEC-004** (MEDIUM): Add an explicit AC to BC-2.01.012 requiring that the
   captured_length vs. block_total_length check precedes any data allocation. This is implied
   by BC-2.01.012 Postcondition 6 but must be stated as a guard-before-allocate requirement.

5. **SEC-005** (MEDIUM): Add a "no-panic AC" to each of BC-2.01.010, BC-2.01.011,
   BC-2.01.012, BC-2.01.013, BC-2.01.015 so the per-story test-writer has a concrete
   AC to test at the story level.

6. **SEC-006** (MEDIUM): Add shift-exponent clamping to BC-2.01.014 (e must be clamped to
   [0, 63] before `1u64 << e`). This is prerequisite for the Kani proof to succeed.

---

## Fuzzing (cargo-fuzz) Recommendation

**Recommendation: mandate a cargo-fuzz harness on the pcapng reader as an F6 hardening
deliverable.**

Rationale:
- BC-2.01.017's Verification Properties already list "fuzz: truncate well-formed pcapng at
  every offset; assert no panic" as a VP. This is the right instinct but needs to be elevated
  to a formal F6 deliverable with a specific harness requirement.
- The timestamp conversion function (BC-2.01.014) is already designated as a Kani target.
  A fuzz harness on the full pcapng reader path (feeding arbitrary bytes to
  `PcapSource::from_pcap_reader`) would complement the Kani proof by exercising the
  integration boundary between the timestamp function and the block-parsing loop.
- The pcapng format's variable-length block structure (block-total-length framing, TLV
  options, multi-IDB interleaving) creates a large irregular input space that unit tests
  alone cannot cover.

**Suggested F6 harness target:** A `cargo-fuzz` target `fuzz_pcapng_reader` that feeds
arbitrary bytes to `PcapSource::from_pcap_reader(&mut Cursor::new(data))` and asserts
(a) no panic, and (b) either `Ok(_)` or `Err(_)` with a concrete error message (not an
empty error chain). This harness would catch all of SEC-001, SEC-002, SEC-005, and SEC-006
if any remain unaddressed after the spec fixes above.

Note: the fuzzing harness itself is NOT mandated in F3 (the TDD implementation phase), as
the F3 implementer's scope is making the BCs pass, not writing security tooling. F6 is the
correct home per wirerust's pipeline.

---

## Error Taxonomy Coverage

Review of E-INP-008 through E-INP-012 against the threat surface:

| Error Code | Threat Covered | Assessment |
|-----------|----------------|------------|
| E-INP-008 | SHB/IDB structural truncation; missing BOM; unsupported major version | Adequate |
| E-INP-009 | EPB before any IDB (empty interface table) | Adequate |
| E-INP-010 | EPB/SPB captured_length inconsistency; unknown block with block_total_length < 8 | Adequate for the cases named, but EPB interface_id OOB is currently misassigned to E-INP-008 (see SEC-003) |
| E-INP-011 | Multi-IDB linktype conflict | Adequate; directory-mode isolation documented in BC-2.01.018 AC-002 |
| E-INP-012 | Multi-section pcapng (second SHB) | Adequate; remediation hint present |

Gap: There is no error code for "EPB interface_id references an interface not in the table."
This is distinct from E-INP-009 (pre-IDB EPB) and distinct from E-INP-010 (data truncation).
Either E-INP-013 should be added or E-INP-010's Notes should be amended to explicitly include
this case.

---

## Summary Statistics

| Severity | Count | Findings |
|----------|-------|---------|
| CRITICAL | 0 | — |
| HIGH | 2 | SEC-001 (CWE-190 timestamp arithmetic overflow), SEC-002 (CWE-835 infinite loop) |
| MEDIUM | 4 | SEC-003 (CWE-125 wrong error code), SEC-004 (CWE-789 allocation bound), SEC-005 (CWE-248 no-panic not AC'd), SEC-006 (CWE-20 shift clamp) |
| LOW | 3 | SEC-007 (CWE-693 DSB log guard), SEC-008 (CWE-345 library behavior), SEC-009 (CWE-190 parity note) |
| **Total** | **9** | |

**Verdict:** REQUEST CHANGES on F3 story decomposition until SEC-001, SEC-002, SEC-003,
SEC-004, SEC-005, and SEC-006 are resolved via BC amendments. No CRITICAL findings. The
two HIGH findings are spec defects (not implementation defects) and are fixable with targeted
BC edits before F3 begins.
