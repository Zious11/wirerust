# F-F5P1-001 Adjudication: VP-027 Kani harness is tautological (false-green)

- **Finding ID:** F-F5P1-001
- **Severity (reported):** HIGH
- **Phase:** F5 (scoped adversarial), pcapng EPB delta
- **Adjudicator:** formal-verifier
- **Date:** 2026-06-21
- **Status:** UPHELD — finding is valid. Recommendation: **Option A (fix properly now)**.
- **Scope of this document:** analysis + decision + implementation spec ONLY. No edits
  to `src/`, `tests/`, `.factory/specs/`, or the VP-INDEX are made here. No PR opened.

---

## 1. Finding confirmation

The reported defect is **confirmed in full**. `vp027_epb_parse_safety`
(`tests/kani_proofs.rs:192-328`) proves none of the properties VP-027 claims:

| Harness construct | What it actually asserts | Verdict |
|---|---|---|
| Case 1 (`tests/kani_proofs.rs:228-240`) | `if body_len < 20 { kani::assert(body_len < 20, …) }` | Tautology — asserts the `if` guard that gated entry. |
| Case 2 (`:243-255`) | `if … table_size == 0 { kani::assert(table_size == 0, …) }` | Tautology. |
| Case 3 (`:258-270`) | `if … interface_id as usize >= table_size { kani::assert(interface_id as usize >= table_size, …) }` | Tautology. |
| Case 4 (`:274-288`) | `if … captured_len … > body_len-20 { kani::assert(captured_len … > body_len-20, …) }` | Tautology. |
| Case 5 (`:301-314`) | `kani::assert(true, …)` | Vacuous. |
| Discriminant check (`:321-327`) | `kani::assert("E-INP-009" != "E-INP-010", …)` | Compares two distinct string literals — true by construction; tests nothing about the code path. |

The real EPB decode path is never invoked. `grep decode_epb_body src/` returns nothing
(confirmed); the decode is inlined in the `EPB_BLOCK_TYPE` match arm of
`read_pcapng_crate` (`src/reader.rs:930-1087`). The harness's own comments admit it is a
structural stub awaiting a "Phase-6 action item" extraction that never happened
(`tests/kani_proofs.rs:163-178, 222-313`).

**Net effect:** `cargo kani --harness vp027_epb_parse_safety` reports SUCCESSFUL while
proving zero of VP-027's properties. This is a genuine false-green for the gating
parse-safety property of the attacker-controlled EPB path (SEC-004 guard-before-allocate,
SEC-005 no-panic, E-INP-009/E-INP-010 discriminant split per BC-2.01.012 PC5/PC9/AC-001).

### 1a. Severity calibration (one correction to the framing — does not reduce HIGH)

The finding states "VP-027 … passes `cargo kani`." That is accurate for the harness
result. For precision in the audit record, two facts bound the blast radius — **neither
downgrades the finding**:

1. **VP-027 is not locked.** There is no `verification_lock: true` for VP-027 anywhere
   (`grep -rln verification_lock .factory/specs/verification-properties/` does not list a
   VP-027 file; there is no standalone `vp-027-*.md` — VP-027 exists only as the catalog
   row in `VP-INDEX.md:80` and BC-2.01.012's Verification Properties section). VP-027
   status is `draft` (`VP-INDEX.md:80`), with the lifecycle note (`VP-INDEX.md:175-177`)
   stating VP-025/026/027 "transition to verified at F6 hardening." So no immutable
   `verification_lock` document is being violated, and no withdrawal process is owed.
2. **Kani is not in CI.** `grep -rln kani .github/workflows/` returns nothing; the proofs
   run only under a manual `cargo kani` invocation during formal hardening. So the
   false-green is not currently gating a merge.

Why it remains HIGH regardless: VP-027 is the **sole formal proof obligation** for the
SEC-004/SEC-005 attacker-controlled EPB parse path. The Kani summary count in the index
already lists VP-027 among the 14 Kani properties (`VP-INDEX.md:38`), and the F6 lifecycle
explicitly intends to lock it. If VP-027 is carried into F6 in its current form it will be
*locked as verified while proving nothing* — at which point it becomes an immutable
false-green requiring the full withdrawal process to unwind. Catching it now, pre-lock, is
exactly the right time. The integrity defect (a "proof" that asserts its own guard
conditions) is the canonical formal-verification anti-pattern and must not be allowed to
reach `verification_lock`.

---

## 2. Decision: Option A (extract a pure `decode_epb_body` and write a real harness)

**Recommended: Option A.** Reasoning:

### 2a. Feasibility — the EPB body decode is cleanly pure-extractable

The EPB arm (`src/reader.rs:930-1087`) is already structured as a self-contained decode
over three inputs that are all in-scope at the arm:

- `blk_body: &[u8]` — the raw EPB body slice (`raw_block.body.as_ref()`).
- `interfaces: &[InterfaceInfo]` — the interface table (only `.is_empty()`, `.len()`, and
  `interfaces[id].if_tsresol` are read; the table is **not mutated** in this arm).
- `section_endianness: SectionEndianness` — a `Copy` scalar enum.

Its only outward effects are (a) `packets.push(RawPacket { … })` and (b)
`packets_emitted = packets_emitted.saturating_add(1)`. Both are trivially hoisted out of
the decode: a pure `decode_epb_body(...) -> Result<RawPacket>` returns the `RawPacket`, and
the caller performs the `push` + counter increment. Everything between
`src/reader.rs:937` (body-length gate) and `:1084` (the `push`) is pure arithmetic, slice
bounds, byte-order reads, and a call to the already-pure `pcapng_timestamp_to_secs_usecs`
(`src/reader.rs:344`, already a Kani target for VP-025). No I/O, no global state, no
mutation of shared structures. **It is pure by construction once the `push`/counter are
lifted to the caller.**

This is not a speculative refactor: the architecture **already mandates exactly this
target**. Footnote `[^vp025-027-module-anchor]` (`VP-INDEX.md:191-204`) specifies the
VP-027 Kani anchor as "**pure EPB fixed-field-decode function (takes `&[u8]`, interface
table size; returns parsed fields or Err)**" that "live[s] in the `src/reader.rs`
compilation unit but [is a] pure-core sub-function." Extraction is the intended design; the
implementer simply skipped it and left a stub. Option A discharges a pre-existing,
documented obligation rather than inventing new architecture.

### 2b. Risk

- **Behavioral risk: very low.** The extraction is a mechanical move of an existing
  contiguous block into a function with the same logic and the same error strings. The full
  STORY-125 regression suite (`bc_2_01_story125_epb_tests.rs` and the broader pcapng
  suite) plus the holdout fixtures (`arp-baseline-16pkt.cap`, HS-104, HS-108) pin the
  observable behavior. Any divergence is caught by `cargo test --all-targets`.
- **Kani tractability risk: low, but bounded.** The harness must symbolically construct a
  body buffer. Kani BMC requires a finite buffer bound; we bound `body.len()` to a small
  representative range (`<= 28` bytes — enough to cover the 20-byte fixed-field minimum
  plus a few captured/padding bytes and the EC-009/EC-010 boundary) and use a fixed-size
  interface table of length 0 or 1 (the only two discriminant-relevant table states per
  BC-2.01.012 PC5a/PC5b). This keeps the proof in the same tractability class as the
  existing VP-025 harness, which already runs.

### 2c. Why not Option B (justified deferral)

Option B (demote VP-027 to a documented deferred-to-F6 stub, decrement the Kani count, and
record an F6 obligation) is **viable and honest** — it would remove the false-green by no
longer counting VP-027 as a satisfied Kani property. But it is the weaker choice here:

1. The work to make the proof real is small and the target is already specified (2a). There
   is no genuine blocker that justifies deferral — the stub exists because of a skipped
   action item, not because the property is hard to prove.
2. SEC-004/SEC-005 over attacker-controlled `captured_len` + `interface_id` is precisely
   the class of property formal verification exists to discharge. Deferring it leaves the
   single highest-value EPB safety obligation unproven while the surrounding lower-value
   properties (VP-025 timestamp totality) are proven — an inverted risk priority.
3. Deferral still leaves a live F6 obligation that someone must later execute under the
   same constraints; it moves the cost without reducing it.

Option B remains the **fallback** if, during implementation, the harness proves
Kani-intractable at the chosen bounds (e.g., unwind blowup). In that case: implement the
extraction anyway (it has standalone value and unblocks future proofs), reduce the harness
to the no-panic + discriminant core, and if even that is intractable, demote per Option B
with the extraction retained. This is noted in §4.3.

---

## 3. Implementation spec (Option A)

Target files: `src/reader.rs`, `tests/kani_proofs.rs`. (Spec only — not applied here.)

### 3.1 New pure function `decode_epb_body` in `src/reader.rs`

Add a `pub`, `#[doc(hidden)]` pure-core function. Place it near the other pure pcapng
helpers (e.g. just after `pcapng_timestamp_to_secs_usecs`, around `src/reader.rs:430`, or
immediately before the `impl PcapSource` block — colocation is cosmetic). Signature:

```rust
/// Pure-core EPB body decoder (BC-2.01.012; VP-027 Kani target).
///
/// Decodes one Enhanced Packet Block body into a `RawPacket`, applying the
/// 5-step evaluation order of BC-2.01.012 PC9 in the mandated sequence:
///   (i)   body.len() >= EPB_FIXED_OVERHEAD_BYTES else E-INP-008
///   (ii)  read interface_id
///   (iii) empty interface table -> E-INP-009
///   (iv)  interface_id OOB on non-empty table -> E-INP-010
///   (v)   PC6a bound-by-body and PC6b padding-overrun -> E-INP-008
///
/// Pure: no I/O, no global state, no mutation of `interfaces`. The caller owns
/// `packets.push(...)` and the `packets_emitted` increment. This is the VP-027
/// Kani anchor per VP-INDEX footnote [^vp025-027-module-anchor].
///
/// `#[doc(hidden)]`: exported solely so the `#[cfg(kani)]` harness can call it
/// without an I/O source; not part of the supported public API surface.
#[doc(hidden)]
pub fn decode_epb_body(
    body: &[u8],
    interfaces: &[InterfaceInfo],
    endianness: SectionEndianness,
) -> anyhow::Result<RawPacket> {
    // ... body below ...
}
```

Function body — a **verbatim lift** of `src/reader.rs:937-1084`, with the two exact
substitutions noted, preserving every error string and check order unchanged:

```rust
    use anyhow::anyhow;

    // (i) Minimum body length gate — E-INP-008 (BC-2.01.012 PC9 step i / AC-003).
    if body.len() < EPB_FIXED_OVERHEAD_BYTES {
        return Err(anyhow!(
            "pcapng EPB body too short: expected at least {} bytes, got {} \
             (E-INP-008: body-too-short)",
            EPB_FIXED_OVERHEAD_BYTES,
            body.len()
        ));
    }

    // (ii) Read interface_id (bytes 0-3) in section endianness.
    let interface_id = match endianness {
        SectionEndianness::BigEndian => {
            u32::from_be_bytes([body[0], body[1], body[2], body[3]])
        }
        SectionEndianness::LittleEndian => {
            u32::from_le_bytes([body[0], body[1], body[2], body[3]])
        }
    };

    // (iii) Empty-table check — E-INP-009 (PC5a / PC9 step iii).
    if interfaces.is_empty() {
        return Err(anyhow!(
            "pcapng Enhanced Packet Block encountered before any Interface \
             Description Block: EPB references interface_id={interface_id} but \
             interface table is empty — no IDB has been parsed (E-INP-009)"
        ));
    }

    // (iv) OOB-on-non-empty check — E-INP-010 (PC5b / PC9 step iv).
    if interface_id as usize >= interfaces.len() {
        let table_size = interfaces.len();
        return Err(anyhow!(
            "EPB interface_id={interface_id} out of range (table size={table_size}) \
             (E-INP-010)"
        ));
    }

    let iface = &interfaces[interface_id as usize];

    // (v) Read remaining fixed fields (ts_high@4-7, ts_low@8-11, captured_len@12-15,
    //     original_len@16-19) in section endianness.
    let (ts_high, ts_low, captured_len, _original_len) = match endianness {
        SectionEndianness::BigEndian => (
            u32::from_be_bytes([body[4], body[5], body[6], body[7]]),
            u32::from_be_bytes([body[8], body[9], body[10], body[11]]),
            u32::from_be_bytes([body[12], body[13], body[14], body[15]]),
            u32::from_be_bytes([body[16], body[17], body[18], body[19]]),
        ),
        SectionEndianness::LittleEndian => (
            u32::from_le_bytes([body[4], body[5], body[6], body[7]]),
            u32::from_le_bytes([body[8], body[9], body[10], body[11]]),
            u32::from_le_bytes([body[12], body[13], body[14], body[15]]),
            u32::from_le_bytes([body[16], body[17], body[18], body[19]]),
        ),
    };

    // PC6a — bound-by-body (live reachable guard) -> E-INP-008.
    let available = body.len().saturating_sub(EPB_FIXED_OVERHEAD_BYTES);
    if captured_len as usize > available {
        return Err(anyhow!(
            "pcapng EPB captured_len {captured_len} exceeds available body \
             bytes {available} (E-INP-008: captured_len > body extent)"
        ));
    }

    // PC6b — padding-aware overrun (defense-in-depth) -> E-INP-008.
    let pad_len = (4usize.wrapping_sub(captured_len as usize % 4)) % 4;
    if EPB_FIXED_OVERHEAD_BYTES
        .saturating_add(captured_len as usize)
        .saturating_add(pad_len)
        > body.len()
    {
        return Err(anyhow!(
            "pcapng EPB padding-overrun: 20 + {captured_len} + {pad_len} > {} \
             (E-INP-008: wirerust body-decode padding overrun; defense-in-depth)",
            body.len()
        ));
    }

    // Slice packet data bounded by captured_len (PC3 / Invariant 2).
    let packet_data =
        &body[EPB_FIXED_OVERHEAD_BYTES..EPB_FIXED_OVERHEAD_BYTES + captured_len as usize];

    // Timestamp routing via the pure-core helper (PC2 / BC-2.01.014).
    let (ts_sec, ts_usecs) =
        pcapng_timestamp_to_secs_usecs(ts_high, ts_low, iface.if_tsresol);

    Ok(RawPacket {
        timestamp_secs: ts_sec,
        timestamp_usecs: ts_usecs,
        data: packet_data.to_vec(),
    })
```

The two substitutions vs. the inlined original:
- the four `blk_body` reads become `body`, and `section_endianness` becomes `endianness`
  (parameter renames only);
- the terminal `packets.push(RawPacket { … })` / `packets_emitted += 1` are **removed**
  from the function and replaced by `Ok(RawPacket { … })`.

### 3.2 Rewire the call site `src/reader.rs:930-1087`

Replace the entire inlined EPB body (lines 935-1086 inclusive, i.e. everything from the
`(i) Minimum body length gate` comment through the `packets_emitted = …` line) with a call
plus the lifted side effects:

```rust
                EPB_BLOCK_TYPE => {
                    // BC-2.01.012 / ADR-009 Decision 2: EPB carries packet data.
                    // Decode is delegated to the pure-core `decode_epb_body` (VP-027
                    // Kani target); the caller owns the push + emitted-counter side effects.
                    let blk_body = raw_block.body.as_ref();
                    let packet = decode_epb_body(blk_body, &interfaces, section_endianness)?;
                    packets.push(packet);
                    packets_emitted = packets_emitted.saturating_add(1);
                }
```

Notes:
- `?` preserves the existing immediate-propagation semantics (PC7 — no silent drop).
- `interfaces` is borrowed immutably and not mutated by the decode; the borrow ends before
  the next loop iteration, so the existing `interfaces.push(...)` in the IDB arm is
  unaffected.
- The error strings are byte-identical to the originals, so all error-message assertions in
  the STORY-125 suite continue to pass unchanged.

### 3.3 Rewrite the VP-027 harness `tests/kani_proofs.rs:192-328`

Replace the tautological body with one that constructs a symbolic body buffer and a
symbolic interface table, calls the **real** `decode_epb_body`, and asserts the
discriminant + safety properties. Suggested harness (single proof; bounds chosen for
tractability):

```rust
    use wirerust::reader::{
        decode_epb_body, InterfaceInfo, SectionEndianness,
    };
    use wirerust::DataLink; // adjust path to the actual DataLink export

    /// VP-027: EPB parse safety — real-call proof.
    ///
    /// Proves over symbolic EPB bodies + interface tables that `decode_epb_body`:
    ///   1. Never panics (totality / SEC-005 / AC-003).
    ///   2. Empty table  -> Err containing "E-INP-009" (and NOT "E-INP-010")  [PC5a].
    ///   3. OOB on non-empty table -> Err containing "E-INP-010" (NOT "E-INP-009") [PC5b].
    ///   4. body.len() < 20 -> Err containing "E-INP-008" (EC-011).
    ///   5. PC6a: captured_len > available -> Err "E-INP-008".
    ///   6. The two interface discriminants are distinct on the same fixed body.
    #[kani::proof]
    #[kani::unwind(32)]
    fn vp027_epb_parse_safety() {
        // Symbolic body length bounded for BMC tractability.
        // 28 covers: <20 (EC-011), exactly 20 (zero captured), and a small data+pad zone
        // spanning the EC-009/EC-010 boundary.
        const MAX_BODY: usize = 28;
        let body_len: usize = kani::any_where(|n: &usize| *n <= MAX_BODY);

        // Symbolic body bytes. A fixed-capacity array sliced to body_len keeps the
        // allocation static for Kani.
        let mut buf = [0u8; MAX_BODY];
        for b in buf.iter_mut() {
            *b = kani::any();
        }
        let body: &[u8] = &buf[..body_len];

        let endianness = if kani::any() {
            SectionEndianness::LittleEndian
        } else {
            SectionEndianness::BigEndian
        };

        // ---- Case A: EMPTY table -> E-INP-009 (PC5a) ----
        {
            let empty: [InterfaceInfo; 0] = [];
            let r = decode_epb_body(body, &empty, endianness);
            // Totality: the call returns (Ok or Err); it never panics. If body_len >= 20,
            // the empty-table branch fires before any captured_len arithmetic (PC9 step iii).
            if body_len >= EPB_FIXED_OVERHEAD_BYTES_TEST {
                let e = r.expect_err("empty table with valid-length body must Err");
                let s = format!("{e:#}");
                kani::assert(s.contains("E-INP-009"), "empty table -> E-INP-009 (PC5a)");
                kani::assert(!s.contains("E-INP-010"), "empty table must NOT be E-INP-010");
            } else {
                // body too short -> E-INP-008 (PC9 step i precedes empty-table check).
                let e = r.expect_err("short body must Err");
                let s = format!("{e:#}");
                kani::assert(s.contains("E-INP-008"), "body < 20 -> E-INP-008 (EC-011)");
            }
        }

        // ---- Case B: NON-EMPTY table (len 1), symbolic interface_id ----
        {
            let table = [InterfaceInfo { linktype: DataLink::Ethernet, if_tsresol: 6 }];
            let r = decode_epb_body(body, &table, endianness);

            if body_len < EPB_FIXED_OVERHEAD_BYTES_TEST {
                let s = format!("{:#}", r.expect_err("short body must Err"));
                kani::assert(s.contains("E-INP-008"), "body < 20 -> E-INP-008 (EC-011)");
            } else {
                // interface_id is read from body[0..4]; with table.len()==1 it is OOB
                // iff interface_id != 0.
                let id = match endianness {
                    SectionEndianness::LittleEndian =>
                        u32::from_le_bytes([body[0], body[1], body[2], body[3]]),
                    SectionEndianness::BigEndian =>
                        u32::from_be_bytes([body[0], body[1], body[2], body[3]]),
                };
                if id as usize >= 1 {
                    let s = format!("{:#}", r.expect_err("OOB id must Err"));
                    kani::assert(s.contains("E-INP-010"), "OOB non-empty -> E-INP-010 (PC5b)");
                    kani::assert(!s.contains("E-INP-009"), "OOB must NOT be E-INP-009");
                } else {
                    // id == 0: in-bounds; result is Ok unless PC6a/PC6b reject captured_len.
                    // Either way the call must not panic, and any Err here is E-INP-008
                    // (PC6a/PC6b are the only remaining failure modes once id is valid).
                    match r {
                        Ok(_) => {}
                        Err(e) => {
                            let s = format!("{e:#}");
                            kani::assert(
                                s.contains("E-INP-008"),
                                "valid id, body-decode reject -> E-INP-008 (PC6a/PC6b)",
                            );
                        }
                    }
                }
            }
        }
    }
```

Implementation notes for the harness author (resolve at apply time, not in this spec):
- `EPB_FIXED_OVERHEAD_BYTES_TEST` is a local `const … = 20;` in the harness (the
  crate-private `EPB_FIXED_OVERHEAD_BYTES` is not re-exported; do not export it solely for
  the test — duplicate the literal `20` with a comment citing BC-2.01.012 Invariant 5).
- Confirm the `DataLink` import path (`wirerust::DataLink` vs `wirerust::reader::DataLink`)
  and that `InterfaceInfo` / `SectionEndianness` are already `pub` (they are —
  `src/reader.rs:155,175`). `DataLink::Ethernet` may need to be any valid whitelisted
  variant; pick whichever variant exists.
- If `#[kani::unwind(32)]` is insufficient or excessive, tune per the VP-025 precedent
  (footnote `[^vp025-027-module-anchor]` notes the unwind-bound consideration). Start at
  the smallest bound that covers `MAX_BODY` loop iterations.
- The PC6b synthetic non-4-aligned case (original Case 5) can be added as a concrete
  assertion (`body_len = 23`, `captured_len = 1`) once the symbolic core verifies, mirroring
  the VP-025 concrete saturation vector. It is optional for closing the false-green; the
  symbolic body already admits non-4-aligned lengths, so PC6b is exercised within the
  bounded space.
- Update the module doc-comment block at `tests/kani_proofs.rs:1-50` so the "VP-027" header
  no longer describes a modeled stub.

### 3.4 VP-INDEX / BC consequence (record only — applied by spec-steward in the fix burst)

No VP **count** change and no VP **addition/retirement** — VP-027 already exists and is
already counted as Kani (`VP-INDEX.md:38,80`). The only catalog action at lock time (F6) is
the normal `draft -> verified` + `verification_lock: true` transition, which now becomes
*legitimate* because a real proof backs it. The fix burst should add a `modified:` note to
`VP-INDEX.md` recording that the VP-027 harness was converted from a tautological stub to a
real `decode_epb_body` call (cite F-F5P1-001), with no count change. BC-2.01.012 is
`verification_lock`-free for VP-027 (no standalone doc) and needs no content edit — its
PC9/AC-001 text already matches the extracted function exactly.

> Per the global rule, **do not** make the VP-INDEX/lock edits in this adjudication. They
> belong to the fix PR / F6 lock burst.

---

## 4. Verification plan for the fix PR

### 4.1 Behavior preservation (must pass before the proof matters)

```bash
cargo build
cargo clippy --all-targets -- -D warnings
cargo test --all-targets          # full STORY-125 EPB + pcapng regression suite
```

Expected: all green, unchanged. The extraction is behavior-preserving; identical error
strings keep every message assertion passing. Confirm `arp-baseline-16pkt.cap` (16 packets,
byte-fidelity) and the HS-104 / HS-108 empty-table + OOB cases still pass.

### 4.2 The real proof

```bash
cargo kani --harness vp027_epb_parse_safety
```

Expected result: `VERIFICATION:- SUCCESSFUL` with **non-trivial coverage** — i.e. the
report shows the `decode_epb_body` call reached and the E-INP-009 / E-INP-010 / E-INP-008
assertions checked over the symbolic space (contrast with the current run, where the proof
is vacuous). Sanity check the harness is non-tautological by temporarily flipping one
expected code (e.g. assert empty-table -> "E-INP-010") and confirming Kani then reports
`FAILED`; revert after. (Kani environment confirmed available: `cargo-kani 0.67.0` at
`~/.cargo/bin/cargo-kani`.)

Optionally run the sibling proof to confirm no regression in the shared harness file:

```bash
cargo kani --harness vp025_pcapng_timestamp_totality
```

### 4.3 Fallback to Option B (only if 4.2 is intractable)

If `vp027_epb_parse_safety` cannot be made to converge within a reasonable unwind/time
budget at `MAX_BODY = 28`:

1. Keep the `decode_epb_body` extraction (§3.1/§3.2) — it has independent value and
   converts the stub into a real-but-narrower proof target.
2. Reduce the harness to the totality (no-panic) + discriminant assertions over a smaller
   `MAX_BODY` (e.g. 24) and drop the data-zone bytes from symbolic exploration.
3. If even the reduced harness is intractable, demote VP-027 per **Option B**: in the fix
   burst, spec-steward marks VP-027 status as an explicit **deferred-to-F6 documented
   stub** in `VP-INDEX.md` (remove it from the satisfied-Kani enumeration so it is no longer
   a false-green), decrement `kani_count` accordingly, and record a tracked F6 obligation
   citing F-F5P1-001. The extracted `decode_epb_body` remains in place as the future proof
   target. This fallback is documented here so the implementer is not blocked if Kani
   resists.

---

## 5. Summary

| Item | Outcome |
|---|---|
| Finding valid? | **Yes** — VP-027 harness is tautological; proves zero VP-027 properties. |
| Currently locked / in CI? | No (`status: draft`, no `verification_lock`, Kani not in CI) — but on the F6 lock path. |
| Severity | **HIGH** upheld — sole formal obligation for the SEC-004/SEC-005 attacker-controlled EPB path; must not reach `verification_lock` as a false-green. |
| Decision | **Option A** — extract pure `decode_epb_body`, rewrite harness to call it. |
| Why | Pure extraction is already mandated by the VP-027 module-anchor footnote; low behavioral + tractability risk; deferral would leave the highest-value EPB safety property unproven. |
| Deliverables for the fix PR | (1) `pub #[doc(hidden)] fn decode_epb_body` in `src/reader.rs`; (2) rewire EPB arm to call it; (3) rewrite `vp027_epb_parse_safety` to call the real function and assert E-INP-008/009/010 over symbolic input; (4) spec-steward VP-INDEX `modified:` note (no count change). |
| Verify | `cargo test --all-targets` green; `cargo kani --harness vp027_epb_parse_safety` SUCCESSFUL with non-vacuous coverage (confirm via deliberate-flip negative test). |
| Fallback | Option B (documented deferral + kani_count decrement) only if the real proof is Kani-intractable; keep the extraction regardless. |

**No code, test, spec, or VP-INDEX edits were made by this adjudication. No PR opened.**
The single artifact produced is this document.
