---
title: "F6 Targeted Security Scan — pcapng Reader Delta"
date: "2026-06-21"
scope: "src/reader.rs + src/main.rs delta (b73b242..662bd85)"
reviewer: "security-reviewer (claude-sonnet-4-6)"
total_findings: 5
critical: 0
high: 0
medium: 2
low: 2
info: 1
files_reviewed: 2
verdict: "PASS — no CRITICAL or HIGH findings; two MEDIUM require tracking"
---

# F6 Targeted Security Scan — pcapng Reader Delta

**Scope:** `src/reader.rs` and `src/main.rs`, commits `b73b242..662bd85`
**Date:** 2026-06-21
**Reviewer:** security-reviewer (claude-sonnet-4-6)
**Threat model:** Forensic CLI parsing untrusted `.pcap`/`.pcapng` files supplied by an attacker
**Overall Verdict:** PASS — no CRITICAL or HIGH findings; two MEDIUM findings require tracking; two LOW findings noted

---

## 1. Scope and Methodology

The delta introduces the pcapng capture-format reader: `BufReader`-based magic-byte probe,
`PcapNgParser`-driven block walk (SHB/IDB/EPB/SPB/skip arms), pure-core helpers
(`decode_epb_body`, `decode_epb_body_discriminant`, `pcapng_timestamp_to_secs_usecs`,
`spb_captured_len`, `parse_idb_options`), per-file isolation loop, and
`format_zero_packet_notice`.

The review covers:
- All attacker-reachable integer arithmetic (length fields, captured_len, original_len,
  option lengths, timestamp ticks, pad computations)
- Slice bounds safety on every indexed path
- Allocation without size limit (DoS)
- Forward-progress in the block-walk loop
- TOCTOU in the directory scan
- Sensitive data exposure (DSB body, error messages)
- Supply-chain: `cargo audit` and `cargo deny check`

---

## 2. Security Findings

### SEC-001: Unbounded Memory Allocation from Attacker-Controlled pcapng File
- **Severity:** MEDIUM
- **CWE:** CWE-400 (Uncontrolled Resource Consumption)
- **OWASP:** A05:2021 - Security Misconfiguration (resource limit absent)
- **Attack Vector:** Attacker supplies a valid pcapng file (correct SHB magic `0x0A0D0D0A`) of
  arbitrarily large size. `from_pcap_reader` calls `read_to_end` into a `Vec<u8>` with no byte
  limit, followed by block-walk that pushes one `RawPacket` per EPB/SPB into a second `Vec`.
  A crafted 4 GB pcapng file with 134 M minimal-overhead EPBs (each 32 bytes) would cause
  ~4 GB raw allocation plus ~4 GB for the packet vector, totalling ~8 GB RSS before any analysis.
- **Impact:** Out-of-memory termination (SIGKILL on Linux). Denial of service against the
  analyst's workstation. Not exploitable for code execution or data exfiltration.
- **Evidence:**
  - `src/reader.rs:826-829`: `let mut raw = Vec::new(); buf_reader.read_to_end(&mut raw)`
  - `src/reader.rs:1153-1154`: `packets.push(packet)` in EPB arm, unbounded loop
  - Module doc comment at lines 23-24 explicitly acknowledges this as a known limitation.
- **Exploitability under threat model:** REALISTIC — an adversary with the ability to supply
  a large capture file (via email attachment, shared drive, network share) can trigger OOM.
  Requires no special privileges; only that the analyst opens the file with wirerust.
- **Proposed Mitigation:** Add a configurable or hardcoded `MAX_PCAP_BYTES` guard before
  `read_to_end` (e.g., `fs::metadata(path)?.len() > MAX_PCAP_BYTES → bail`). Also consider
  `Vec::with_capacity` with a cap. The module notes "streaming-reader follow-up" in the
  technical-debt register; a file-size guard is a lower-effort interim mitigation.
  A guard of 2 GB or 4 GB would match typical analyst workstation constraints.

---

### SEC-002: Interface Table Amplification (Many-IDB DoS)
- **Severity:** MEDIUM
- **CWE:** CWE-770 (Allocation of Resources Without Limits or Throttling)
- **OWASP:** A05:2021 - Security Misconfiguration
- **Attack Vector:** Attacker crafts a pcapng with no packet blocks but N IDB blocks (each 32
  bytes minimum, all using the same linktype to pass E-INP-011 conflict check). Each IDB
  pushes one `InterfaceInfo` (8 bytes) to `interfaces: Vec<InterfaceInfo>`. Before any EPB/SPB
  (so `packets_emitted == 0`), the E-INP-013 guard does not fire. A 4 GB file filled with
  minimal IDB blocks yields ~128 M table entries (~1 GB) plus the 4 GB raw buffer.
- **Impact:** Memory pressure; combined with SEC-001 can amplify OOM risk. Denial of service.
- **Evidence:**
  - `src/reader.rs:1141-1145`: `interfaces.push(InterfaceInfo { ... })` in IDB arm, no limit
  - `src/reader.rs:1070-1075`: E-INP-013 guard fires only when `packets_emitted > 0`
- **Exploitability under threat model:** REALISTIC with same preconditions as SEC-001.
  Lower standalone risk than SEC-001 since the IDB table is smaller than packet data.
- **Proposed Mitigation:** Cap `interfaces.len()` at a hard limit (e.g., 65535, matching
  the maximum EPB `interface_id` u16 range in practice) and return E-INP-008 on overflow.
  This also tightens the security invariant for the EPB `interface_id` OOB check.

---

### SEC-003: TOCTOU Window in Directory Scan (`resolve_targets`)
- **Severity:** LOW
- **CWE:** CWE-367 (Time-of-Check Time-of-Use (TOCTOU) Race Condition)
- **OWASP:** A01:2021 - Broken Access Control
- **Attack Vector:** In `resolve_targets`, `path.is_file()` is called, then `read_magic(&path)`
  opens the same path. Between these two calls, a local attacker who controls the filesystem
  could replace the file (e.g., via symlink swap) with a different file — a device file,
  a named pipe, or a different content — that passes the magic check but causes unexpected
  behavior downstream (e.g., blocking on a named pipe read in `read_to_end`).
- **Impact:** If the attacker can write to the scanned directory, they could cause wirerust
  to block indefinitely (blocking pipe read via `read_to_end`) or process an unintended file.
  Not exploitable for memory corruption.
- **Evidence:**
  - `src/main.rs:654-656`: `if path.is_file() && let Some(magic) = read_magic(&path) && ...`
  - `src/main.rs:626-631`: `read_magic` opens the path independently; no `O_NOFOLLOW` or
    file descriptor re-use between `is_file()` and the subsequent `read_magic()` + `from_file()`
- **Exploitability under threat model:** LOW — requires local filesystem write access to the
  scanned directory. In the primary threat model (attacker supplies crafted capture files),
  the attacker does not have local filesystem access. Relevant only if wirerust is run on a
  shared filesystem or in a CI/scanning pipeline where directory contents are attacker-influenced.
- **Proposed Mitigation:** Use an `openat`-based approach or open the file once and pass the
  file descriptor to both magic detection and parsing. In Rust, this means:
  `File::open(path)` once, read first 4 bytes with `read_exact`, then `seek(SeekFrom::Start(0))`
  and pass the open `File` to `from_pcap_reader`. This eliminates the TOCTOU window entirely
  and reduces the number of `open()` syscalls.

---

### SEC-004: Crate Error Messages Surfaced with Internal Detail
- **Severity:** LOW
- **CWE:** CWE-209 (Generation of Error Message Containing Sensitive Information)
- **OWASP:** A09:2021 - Security Logging and Monitoring Failures
- **Attack Vector:** When pcap-file crate parsing fails, the raw crate error string is
  included verbatim in the wirerust error message (e.g., `"pcapng SHB parse failed: {e} (E-INP-010)"`
  at `src/reader.rs:943`). Crate error messages may expose internal parser state, version strings,
  or implementation details. For a CLI forensic tool emitting to stderr, this is standard practice,
  but the messages propagate through `anyhow` context chains and could reach log aggregators.
- **Impact:** Minimal for a forensic CLI. Crate error messages do not contain cryptographic
  material, credentials, or file content. Risk is limited to exposing crate version fingerprints.
- **Evidence:**
  - `src/reader.rs:935,939,943,1027`: crate `PcapError` surfaced in anyhow context strings
  - `src/reader.rs:878`: unrecognized magic bytes echoed in error (4 bytes from file, acceptable)
- **Exploitability under threat model:** NOT EXPLOITABLE in the attacker-supplies-file model.
  The attacker already knows what bytes they wrote into the file.
- **Proposed Mitigation:** Document that error messages are intended for operator consumption
  only and should not be forwarded to untrusted parties. No code change required in the
  current threat model. If wirerust is ever exposed as a service (API), this rating should
  be elevated.

---

### SEC-005: `wrapping_sub` in Padding Computation (Defense-in-Depth Note)
- **Severity:** INFO
- **CWE:** CWE-191 (Integer Underflow) — theoretical, not exploitable as written
- **Attack Vector:** In `decode_epb_body` and `decode_epb_body_discriminant`, padding is
  computed as `(4usize.wrapping_sub(captured_len as usize % 4)) % 4`. The `wrapping_sub` call
  is intentional: `4.wrapping_sub(0 % 4) = 4.wrapping_sub(0) = 4`, then `% 4 = 0`. This is
  the correct idiom for 4-byte alignment padding, and the outer `% 4` clamps the result to
  `[0, 3]` regardless of the intermediate wrapping value.
- **Impact:** None — the arithmetic is correct and the final PC6b overrun check validates
  the total before any slice access. The `wrapping_sub` does not produce a security-relevant
  value that escapes bounds checking.
- **Evidence:** `src/reader.rs:500,585`: `let pad_len = (4usize.wrapping_sub(captured_len as usize % 4)) % 4;`
- **Exploitability under threat model:** NOT EXPLOITABLE. The PC6b guard at lines 501-511 and
  586-592 validates `EPB_FIXED_OVERHEAD_BYTES + captured_len + pad_len <= body.len()` before
  any slice indexing. Verified manually for all boundary values of `captured_len % 4`.
- **Proposed Mitigation:** None required. The idiom is widely-used and correct. Optionally
  replace with the clearer `(4 - (captured_len as usize % 4)) % 4` using debug-mode overflow
  checks that catch mistakes in testing (the `% 4` before subtraction ensures the subrahend
  is always `<= 4`, making wrapping impossible). This is a cosmetic improvement only.

---

## 3. Specific Guard Verification (What Was Checked and Found Clean)

### 3.1 Slice Bounds Safety

All slice indexing on attacker-controlled length fields was verified:

| Path | Field | Guard | Result |
|------|-------|-------|--------|
| `decode_epb_body` | `captured_len` (u32) | PC6a: `captured_len as usize > body.len() - 20` → E-INP-008 | SAFE |
| `decode_epb_body` | `pad_len` | PC6b: `20 + captured_len + pad_len > body.len()` → E-INP-008 | SAFE |
| `decode_epb_body` | `interface_id` (u32) | `interface_id as usize >= interfaces.len()` → E-INP-010 | SAFE |
| `parse_idb_options` | `opt_len` (u16→usize) | `cursor + opt_len > remaining.len()` before read | SAFE |
| `parse_idb_options` | `cursor += padded` | Only increments cursor; next-iteration `cursor+4` check prevents dereference past end | SAFE |
| `spb_captured_len` | `original_len` (u32) | `min(original_len, (body.len()-4) as u32)`; btl is u32 so body.len()-4 ≤ u32::MAX | SAFE |
| `SPB arm` | `captured_len` | `body.len() >= SPB_FIXED_OVERHEAD_BYTES (4)` checked before `spb_captured_len` call | SAFE |
| `parse_shb_body` | body access | `body.len() >= SHB_BODY_FIXED_BYTES (16)` checked first | SAFE |
| `read_magic` | short file | `read_exact` returns `None` on short read | SAFE |

### 3.2 Integer Overflow / Arithmetic

| Computation | Overflow Guard | Result |
|-------------|---------------|--------|
| `(ts_high as u64) << 32` | Both operands are u64; shift is exactly 32 | SAFE |
| `ticks / ticks_per_sec` | `ticks_per_sec >= 1` always (base-10: `BASE10_POWERS[0]=1`; base-2: `1u64.checked_shl(e)` with e clamped to 63) | SAFE |
| `(ticks % ticks_per_sec) as u128 * 1_000_000u128` | u128 max = 3.4e38; max product = (u64::MAX-1) * 1e6 ≈ 1.84e25; no overflow | SAFE |
| `(ticks / ticks_per_sec).min(u32::MAX as u64) as u32` | `.min()` saturates; `as u32` cannot wrap | SAFE |
| `1u64.checked_shl(e)` | e clamped to 63 before call; `checked_shl` returns `None` for e>=64, handled with `unwrap_or(u64::MAX)` | SAFE |
| `block_seq.saturating_add(1)` | saturating at u32::MAX | SAFE |
| `packets_emitted.saturating_add(1)` | saturating at u32::MAX | SAFE |
| `skipped_blocks.saturating_add(1)` | saturating | SAFE |
| `opb_skipped.saturating_add(1)` | saturating | SAFE |
| `skipped_blocks.saturating_sub(opb_skipped)` | saturating; invariant opb_skipped <= skipped_blocks | SAFE |
| `EPB_FIXED_OVERHEAD_BYTES.saturating_add(captured_len as usize).saturating_add(pad_len)` | saturating chain | SAFE |

### 3.3 Panic / Unwrap Analysis (Production Paths Only)

| Location | Usage | Safety |
|----------|-------|--------|
| `reader.rs:379` | `1u64.checked_shl(e).unwrap_or(u64::MAX)` | Safe: `unwrap_or` never panics |
| `reader.rs:1273` | `.unwrap_or(DataLink::from(0))` | Safe: `unwrap_or` never panics |
| Test code (`expect_err`, `expect`) | All in `#[cfg(kani)]` or `#[cfg(test)]` blocks | Out of production scope |

No bare `unwrap()`, `expect()`, `panic!()`, or `unreachable!()` on attacker-reachable production paths.

### 3.4 Forward-Progress Guard (CWE-835)

The block-walk loop at `reader.rs:978-1265` guards against infinite loops:
- `prev_len = src.len()` recorded before `next_raw_block`
- After `Ok` return: `if src.len() >= prev_len { return Err(...) }`
- Since `next_raw_block` returns a sub-slice, `src.len()` can only decrease or stay equal;
  `>= prev_len` means "zero or negative progress" → immediate error
- Correctly handles the edge case where `src.len() == prev_len` (zero advance)

**Verified:** This guard was introduced specifically for CWE-835 per commit `ebbc961` and
is correctly positioned after the `?`-propagation so it fires on successful (non-error) returns only.

### 3.5 DSB (Decryption Secrets Block) Handling — SEC-007

The DSB arm (`reader.rs:1247-1254`) correctly:
- Matches `DSB_BLOCK_TYPE = 0x0000_000A` directly (no crate `Block` enum used)
- Only increments `skipped_blocks`; `raw_block.body` is never accessed, logged, or assigned
- `pcap-file` crate's `next_raw_block_inner` falls through the wildcard match for DSB (type
  `0x000A` is not `SECTION_HEADER_BLOCK` or `INTERFACE_DESCRIPTION_BLOCK`), so the crate
  also does not process the body; body is `Cow::Borrowed` slice from the pre-loaded `raw` Vec

**Verified:** TLS key material is not surfaced at any severity level. SEC-007 correctly implemented.

### 3.6 Endianness Handling

Section endianness is established once from `parser.section().endianness` after SHB parsing and
propagated as `SectionEndianness` to all downstream decoders (IDB, EPB, SPB body fields,
IDB options TLV). No block re-detects endianness per-block. Correct for both BE and LE sections.

### 3.7 `spb_captured_len` Truncation Analysis

Theoretical concern: `(body.len().saturating_sub(SPB_FIXED_OVERHEAD_BYTES)) as u32` could
truncate if `body.len() > u32::MAX + 4`. Verified safe: pcapng block-total-length (`btl`) is
a `u32` field, so `btl_max = u32::MAX = 4_294_967_295`. The crate validates `btl ≥ 12` and
`body = btl - 12` bytes. Maximum `body.len() = 4_294_967_283 < u32::MAX`. The `as u32` cast
cannot truncate. No overflow possible.

---

## 4. Supply-Chain / Dependency Audit

### 4.1 cargo audit

**Result:** 1 warning (allowed), 0 vulnerabilities

```
RUSTSEC-2026-0097  rand 0.8.5  "unsound with custom logger using rand::rng()"
  Dependency chain: wirerust → tls-parser → phf_codegen [BUILD-DEP] → phf_generator → rand 0.8.5
  Status: SUPPRESSED in CI with --ignore RUSTSEC-2026-0097 (ci.yml:152)
  Analysis: phf_codegen is a BUILD dependency of tls-parser (listed under
    [build-dependencies] in tls-parser-0.12.2/Cargo.toml). rand 0.8.5 is
    invoked only during `cargo build` (build.rs generates PHF maps at compile time).
    The unsound path (custom logger + rand::rng() race) cannot be triggered by
    attacker-supplied pcapng files; no runtime exploit vector exists.
    Suppression rationale: upgrade is tls-parser upstream's responsibility.
    Verdict: FALSE POSITIVE for the runtime threat model.
```

### 4.2 cargo deny check

**Result:** CLEAN — `advisories ok, bans ok, licenses ok, sources ok`

Warnings were license-not-encountered (allowlist entries for licenses not present in current
deps — cosmetic, not a security issue) and a duplicate `syn` 1.x/2.x entry (build-time
proc-macro dep only, not a security issue).

### 4.3 pcap-file 2.0.0 Supply-Chain

- Cargo.lock checksum: `1fc1f139757b058f9f37b76c48501799d12c9aa0aa4c0d4c980b062ee925d1b2` (verified)
- No RUSTSEC advisories for pcap-file 2.0.0 in the current advisory database
- `RawBlock::from_slice` returns `Cow::Borrowed` — zero-copy block body slices into the
  pre-loaded `raw` Vec. No per-block heap allocation by the crate.
- Inner parse validates `btl % 4 == 0`, `btl >= 12`, `slice.len() >= btl - 8`, and
  `initial_len == trailer_len`. These guards prevent btl-based OOB reads in the crate.

### 4.4 nom 7.1.3 and byteorder_slice 3.0.0

No known advisories. nom is used by pcap-file for internal parsing; byteorder_slice for
endian-aware field reads. Both are transitive-only.

---

## 5. Threat Model Coverage

| Attack Vector | Covered By | Result |
|--------------|-----------|--------|
| Crafted captured_len > body size | PC6a guard (EPB) | BLOCKED → E-INP-008 |
| Crafted captured_len + padding overrun | PC6b guard (EPB) | BLOCKED → E-INP-008 |
| EPB before any IDB | Empty-table guard (step iii) | BLOCKED → E-INP-009 |
| EPB with interface_id beyond table | OOB guard (step iv) | BLOCKED → E-INP-010 |
| Invalid SHB BOM | `parse_shb_body` BOM table check | BLOCKED → E-INP-008 |
| Unsupported major version | `major_version != 1` check | BLOCKED → E-INP-008 |
| Second SHB (multi-section) | `SHB_BLOCK_TYPE` arm → immediate Err | BLOCKED → E-INP-012 |
| IDB after packets emitted | `packets_emitted > 0` guard | BLOCKED → E-INP-013 |
| Mixed linktypes across IDBs | E-INP-011 conflict check | BLOCKED |
| Block-walk infinite loop | `src.len() >= prev_len` zero-advance guard | BLOCKED → E-INP-010 |
| SPB body too short | `body.len() < SPB_FIXED_OVERHEAD_BYTES` | BLOCKED → E-INP-008 |
| SPB before any IDB | Empty-table guard | BLOCKED → E-INP-009 |
| Malformed IDB options TLV overrun | `cursor + opt_len > remaining.len()` | BLOCKED → E-INP-008 |
| Malformed if_tsresol TLV length | `opt_len != 1` for code 9 | BLOCKED → E-INP-008 |
| DSB TLS key material disclosure | DSB arm skips without reading body | BLOCKED (SEC-007) |
| Timestamp overflow (base-10/base-2) | Saturating arithmetic + u128 intermediate | BLOCKED |
| Large file / OOM | No size limit (SEC-001) | MEDIUM RISK — known limitation |
| Many-IDB amplification | No table size limit (SEC-002) | MEDIUM RISK |
| Directory scan TOCTOU | Consecutive syscalls (SEC-003) | LOW RISK — requires local access |

---

## 6. Risk Register Disposition

No L2 Domain Spec Risk Register was provided for this F6 scan. Security-category risks
identified in the delta are captured directly as SEC-NNN findings above.

Previously-addressed security findings from earlier adversarial passes visible in commit history:
- `be667da`: F-8 saturating_add for block_seq counter (SEC-005, now resolved)
- `ebbc961`: F-6 forward-progress guard (CWE-835, resolved)
- `489f3ae`: H-1/H-2/H-3 adversarial remediation (resolved)
- `6e54ed8`: F-F5P5-002 read_exact in read_magic (resolved)

---

## 7. Overall F6 Security Verdict

**VERDICT: PASS**

The pcapng reader delta does not introduce any CRITICAL or HIGH security vulnerabilities.
All attacker-controlled length fields (captured_len, original_len, option lengths, interface_id)
are bounds-checked before use. Arithmetic uses exclusively saturating, checked, or provably
safe operations. The forward-progress guard prevents CWE-835. DSB body bytes are never
accessed. No panics exist on attacker-reachable production paths.

**Two MEDIUM findings require tracking:**
- SEC-001: Unbounded `read_to_end` allocation (CWE-400) — recommend file-size guard as interim
  mitigation before the streaming-reader follow-up in the technical-debt register
- SEC-002: Unbounded interface table growth (CWE-770) — recommend hard cap of 65535 IDB entries

**Two LOW findings noted but do not block progression:**
- SEC-003: TOCTOU in directory scan (CWE-367) — low risk for stated threat model; eliminate
  by opening the file once and reusing the file descriptor
- SEC-004: Crate error message disclosure (CWE-209) — acceptable for forensic CLI tool

**Supply chain:** `cargo audit` shows one suppressed warning (RUSTSEC-2026-0097, build-time
only, no runtime exploit vector). `cargo deny` is clean. `pcap-file 2.0.0` checksum verified.
