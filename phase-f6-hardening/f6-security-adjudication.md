---
title: "F6 Security Adjudication — pcapng Reader MEDIUM/LOW/INFO Findings"
date: "2026-06-21"
author: "architect (claude-sonnet-4-6)"
source_scan: "pcapng-f6-security-scan.md"
develop_head: "662bd85"
status: "FINAL"
---

# F6 Security Adjudication — pcapng Reader

Source scan: `.factory/phase-f6-hardening/pcapng-f6-security-scan.md`
develop HEAD at adjudication: `662bd85`

---

## F6-SEC-A (SEC-001) — Unbounded `read_to_end` → OOM DoS (CWE-400, MEDIUM)

### Decision: DEFER — Accept-and-Document with Tracked Hardening Item

**Rationale (why not fix-in-F6):**

The all-in-memory model is a pre-existing ADR-009 Decision 13 architectural
commitment. Adding a `fs::metadata` size-gate before `read_to_end` is a
product-level policy choice with a non-trivial tradeoff: the legitimate E2E
corpus already contains a 200 MB file (`4SICS-GeekLounge-151022.pcap`) and a
tracked-but-optional 1 GB file (`maccdc2012_00000.pcap`, recorded in
`tests/fixtures/E2E-PCAPS.md` as a scale stressor). Any ceiling that does not
break legitimate forensic workflows must be placed above these floor values.

The scan report itself proposes 2 GB or 4 GB as "typical analyst workstation"
bounds. That ceiling value is a product-level policy decision — specifically,
it answers "what file size is wirerust willing to refuse?" — and the human
must make that call. The decision cannot be delegated to a constant chosen by
the factory because:

- A 2 GB ceiling would pass the 200 MB corpus entry but silently refuse the
  1 GB MACCDC entry (which is in the E2E index, even if optional). A forensic
  analyst running wirerust against that file would receive an E-INP-NNN abort
  with no other recovery path (no streaming mode exists yet).
- A 4 GB ceiling would pass all current corpus entries but is only 1x the
  attack size (the scan posits a crafted 4 GB file → ~8 GB RSS). A 4 GB
  ceiling does not meaningfully bound memory on a 16 GB workstation; it only
  prevents pathological supersized files.
- The streaming-reader follow-up already in the technical-debt register is the
  correct architectural fix. A ceiling now is interim mitigation; the question
  is whether the product wants to constrain users during the interim period.

**This item must be raised to the human at the F6 gate. See FLAG below.**

**Accepted limitation documentation:** The existing module doc comment at
`src/reader.rs:23-24` already acknowledges "for very large captures the
all-in-memory model is a known limitation; see the technical-debt register for
a streaming-reader follow-up." No spec text needs to be added; the ADR-009
Decision 13 annotation and module doc are sufficient.

**Tracked hardening item to record in STATE.md drift items:**

```
F6-SEC-A [MEDIUM, deferred] — CWE-400 unbounded read_to_end OOM DoS.
ADR-009 Decision 13 all-in-memory model has no file-size ceiling. A crafted
~4 GB pcapng file → ~8 GB RSS → OOM SIGKILL. Deferred pending human policy
decision on ceiling value and streaming-reader follow-up. Pre-condition for
resolving: (1) human chooses ceiling (product-level policy); (2) implement
fs::metadata guard before read_to_end and assign E-INP-014 for the rejection;
(3) wire BC-2.01.009 AC-NNN for the size-gate. The streaming-reader rework is
the permanent architectural fix (future cycle).
```

**F6 gate impact:** Does NOT block F6→F7 transition. The product currently
only parses files through deliberate CLI invocation by an analyst who has
chosen to load the file; there is no network-facing or automated-ingestion
path. The exploitability is real but bounded by the operational context.

---

### FLAG FOR HUMAN (F6 gate)

**F6-SEC-A requires a human policy decision before any code can land.**

Question: "What is the maximum pcapng file size wirerust should load into
memory before refusing with a clear error?"

Context for the decision:
- The E2E corpus contains a 200 MB file (4SICS-151022) and tracks an optional
  1 GB file (MACCDC 2012) as a scale stressor.
- Any ceiling below 1 GB would refuse a tracked corpus entry.
- A ceiling at 2 GB or 4 GB reduces (but does not eliminate) the DoS surface;
  it does not address the 2x or 3x memory amplification from the packet vector.
- The streaming-reader follow-up (already in the technical-debt register)
  eliminates the vulnerability entirely; the question is whether to add an
  interim ceiling now or wait for the streaming reader.

If a ceiling is chosen, the remediation spec is as follows:

- **Constant location:** `src/reader.rs`, new top-level const adjacent to the
  pcapng block-type constants block (around line 50-100):
  `const MAX_PCAPNG_FILE_BYTES: u64 = <human_decides_value>;`
- **Gate location:** `src/reader.rs` within `from_pcap_reader`, at line 826,
  immediately before `let mut raw = Vec::new();`, after the `PCAPNG_MAGIC`
  branch is entered. Because the caller passes a generic `R: Read`, a file size
  check requires the file path, not the reader. The gate therefore must move to
  `from_file` (`src/reader.rs:1284-1288`) using `fs::metadata(path)?.len()`
  before constructing the BufReader. Classic-pcap files share `from_file`, so
  the gate should be applied inside the pcapng arm only (after the magic peek,
  not before format detection). Alternatively, thread the file size through a
  new overload; the simplest correct approach is a separate
  `from_pcapng_file_with_size_check` internal method called from `from_file`
  when the magic is `PCAPNG_MAGIC`.
- **Error code:** E-INP-014 (next free per error-taxonomy v3.7). New entry in
  `specs/prd-supplements/error-taxonomy.md`: category INP, severity broken,
  exit 1, message:
  `"pcapng file too large: {size} bytes exceeds limit of {MAX_PCAPNG_FILE_BYTES} bytes (E-INP-014); use a streaming tool or split the capture"`.
- **BC impact:** BC-2.01.009 requires a new Precondition (PC-NNN) and an Error
  Condition cross-referencing E-INP-014. BC-2.01.017 error-code table requires
  E-INP-014 addition.
- **Directory scan interaction (`resolve_targets`):** `resolve_targets` in
  `src/main.rs:645-667` calls `read_magic` then later calls `PcapSource::from_file`.
  The size-gate fires inside `from_file` and returns `Err(E-INP-014)`. Because
  directory mode already isolates per-file errors (BC-2.01.018 AC-002), this
  error propagates as a per-file broken result without aborting the scan.
  No changes are needed to `resolve_targets` or `main.rs`; the isolation
  already handles it correctly.
- **ADR change:** ADR-009 new Decision N: "Interim file-size guard (E-INP-014)
  is applied at `from_file` entry on the pcapng path via `fs::metadata`. The
  ceiling `MAX_PCAPNG_FILE_BYTES` is a product-level constant; see human gate
  decision at F6. This guard is superseded when the streaming-reader follow-up
  lands."

---

## F6-SEC-B (SEC-002) — Uncapped Interface Table (CWE-770, MEDIUM)

### Decision: DEFER — Accept-and-Document, contingent on F6-SEC-A outcome

**Rationale:**

The IDB-amplification vector is real but its standalone exploitability is
substantially lower than SEC-001 for the following reasons:

1. **Per-IDB allocation is bounded by block overhead.** Each IDB block is a
   minimum of 12 bytes (block header/trailer) on disk. `InterfaceInfo` is 2
   fields: `linktype: DataLink` (4 bytes on most targets) and `if_tsresol: u8`
   (1 byte), for ~8 bytes per entry (with alignment). A 4 GB file filled
   entirely with minimum-size IDBs yields ~4 GB / 12 bytes = ~333 M entries,
   not 128 M as the scan calculated (the scan used 32-byte IDB minimum, which
   is the standard minimum including if_name option; the absolute minimum btl
   is 20 per the pcapng spec). Either way the interface table amplification
   factor is 8/20 = 0.4x the file size, while the raw buffer alone is 1x the
   file size. The dominant term is always the raw Vec from `read_to_end` (SEC-001).

2. **F6-SEC-A already bounds both vectors.** If the human chooses to add a
   file-size ceiling (SEC-001 fix path), the `read_to_end` raw buffer is
   capped at `MAX_PCAPNG_FILE_BYTES`, which also transitively caps the number
   of IDB blocks that can physically exist in the raw buffer. Under a 2 GB
   ceiling, maximum IDB entries is bounded at 2 GB / 20 bytes = ~100 M; at
   the `InterfaceInfo` size of ~8 bytes that is ~800 MB of interface table.
   This is still non-trivial but paired with the 2 GB raw buffer is
   well under 4 GB total and bounded. Under a 4 GB ceiling: ~200 M entries
   / ~1.6 GB interface table alongside the 4 GB raw buffer.
   An explicit IDB cap adds a secondary, defense-in-depth bound.

3. **E-INP-008 already covers the post-table-overflow EPB OOB path.** The
   existing EPB and SPB `interface_id >= interfaces.len()` guard (E-INP-010)
   prevents any OOB memory access even with an inflated table. The security
   risk is DoS by allocation, not by memory corruption.

4. **The 65535 cap requires a BC decision.** Capping the interface table at
   65535 entries changes the error-surface semantics: a legitimate (if exotic)
   pcapng file with more than 65535 interfaces would be rejected. The pcapng
   spec does not mandate a maximum interface count; EPB `interface_id` is a
   u32 field, not u16. The scan's 65535 suggestion is based on u16 maximum (the
   maximum EPB `interface_id` expressible before the field overflows, but the
   field is u32 in the spec). No real-world file is expected to have >65535
   interfaces; but this is a new behavioral rejection the product must own.

**If F6-SEC-A is deferred with no ceiling:** F6-SEC-B is also deferred and
recorded alongside F6-SEC-A. Both will be addressed when the file-size ceiling
or streaming-reader lands.

**If F6-SEC-A is fixed with a size ceiling:** F6-SEC-B may still warrant an
explicit cap as a defense-in-depth measure. The cap at 65535 interfaces is
reasonable for defense-in-depth and adds negligible implementation cost at
that point. Recommended action if F6-SEC-A is fixed: add the explicit cap
(see remediation spec below) as part of the same burst.

**Tracked hardening item to record in STATE.md drift items:**

```
F6-SEC-B [MEDIUM, deferred] — CWE-770 uncapped interface table amplification.
src/reader.rs:1141-1145 pushes InterfaceInfo with no table-size limit. Standalone
risk is lower than F6-SEC-A because file-size bounds the table transitively.
Resolve as part of the F6-SEC-A burst or streaming-reader follow-up. If an
explicit cap is added, use 65535 as the limit and assign E-INP-014 (or the
next available code if E-INP-014 is consumed by SEC-A). BC impact:
BC-2.01.011 PC3 requires a new error condition; BC-2.01.017 error-code table
requires the new code.
```

**F6 gate impact:** Does NOT block F6→F7 transition (same reasoning as SEC-001:
CLI forensic tool, analyst-driven file loading, no automated ingestion path).

---

### Remediation Spec (contingent on SEC-A fix landing or human ordering)

- **Cap constant:** `src/reader.rs`, new top-level const:
  `const MAX_INTERFACE_TABLE_ENTRIES: usize = 65_535;`
- **Guard location:** `src/reader.rs`, in the `IDB_BLOCK_TYPE` arm, at line 1141,
  immediately BEFORE `interfaces.push(...)`:
  ```rust
  if interfaces.len() >= MAX_INTERFACE_TABLE_ENTRIES {
      return Err(anyhow!(
          "pcapng file has too many Interface Description Blocks: \
           limit is {} (E-INP-014: interface table overflow)",
          MAX_INTERFACE_TABLE_ENTRIES
      ));
  }
  ```
  (Replace E-INP-014 with the assigned code; see note on code allocation below.)
- **Error code:** If E-INP-014 is available after SEC-A's ceiling is assigned
  (which uses E-INP-014), then the interface-cap error requires E-INP-015. If
  SEC-A is deferred and no new code is allocated, E-INP-014 is used here.
  The error-taxonomy entry: category INP, severity broken, exit 1.
- **BC impact:** BC-2.01.011 PC3 currently states "Push to interface table
  (BC-2.01.011 PC3 / Invariant 1)." Add a new Precondition or extend PC3:
  "interface table MUST NOT exceed MAX_INTERFACE_TABLE_ENTRIES; if exceeded,
  return E-INP-[NNN] and abort block walk."
  BC-2.01.017 and BC-2.01.018 error-code tables require the new code.
- **No BC-2.01.018 (per-file isolation) changes needed:** The error propagates
  through the existing per-file isolation mechanism exactly as E-INP-011 does.

---

## F6-SEC-C — TOCTOU in `resolve_targets` (CWE-367, LOW)

### Decision: Accept-and-Document (no fix in F6)

**Rationale:** The threat model requires local filesystem write access to the
scanned directory during the narrow window between `is_file()` and `read_magic`
in `src/main.rs:654-656`. This is not a realistic vector for the stated threat
model (attacker supplies crafted capture file via network/email; analyst opens
it with wirerust). The attack requires privilege on the analyst's own workstation
filesystem, at which point the attacker already has sufficient access to cause
far greater harm by other means.

The proposed fix (open the file once, reuse the file descriptor for magic
detection and parsing) has merit for correctness and would also reduce system
call count. However, it requires refactoring `from_pcap_reader` to accept a
`File` rather than a generic `R: Read`, or threading the opened `File` through
a separate code path. This is a non-trivial refactor that touches the public
API boundary and the `BufReader` wrap site (AC-007 per BC-2.01.009). It is
better suited to a dedicated maintenance story than an F6 hardening burst.

**No spec changes required.** Record in drift items:

```
F6-SEC-C [LOW, deferred, maintenance] — CWE-367 TOCTOU in resolve_targets
(src/main.rs:654-656). Three separate syscalls: is_file() / read_magic open /
from_file open. Fix by opening the file once and passing the fd to both magic
detection and parsing. Requires from_pcap_reader to accept File or changing
the public API. Non-blocking for F6-F7. DF-VALIDATION-001 required before
filing a GitHub issue.
```

**F6 gate impact:** None. Does not block F6→F7.

---

## F6-SEC-D — Crate Error-String Disclosure (CWE-209, LOW)

### Decision: Accept-and-Document (no fix)

**Rationale:** For a forensic CLI tool, surfacing crate error strings to the
operator via stderr is intentional and correct. The attacker already knows
what bytes they wrote into the crafted file — the error message reveals
nothing the attacker does not already know. The risk surfaces only if wirerust
is ever exposed as an HTTP service (at which point the error messages must be
sanitized before returning them to the caller), but that is out of scope for
the current product. The existing practice of wrapping crate errors with
`anyhow::Context` strings (e.g., `"pcapng SHB parse failed: {e} (E-INP-010)"`)
is architecturally correct: operator-facing diagnostic, not user-facing API.

**Documentation:** Add to ADR-009 or the module-level doc comment:
"Error messages from `pcap-file` crate errors are surfaced verbatim in anyhow
context chains and are intended for operator (analyst) consumption only. If
wirerust is ever exposed as a network service, these messages must be
sanitized before transmission to avoid CWE-209."

**No spec changes or error taxonomy changes required.**
**F6 gate impact:** None.

---

## F6-SEC-E — `wrapping_sub` in Padding Computation (CWE-191, INFO)

### Decision: Accept — No change warranted

**Rationale:** The scan already verifies (Section 2, SEC-005) that the idiom
is correct: `(4usize.wrapping_sub(captured_len as usize % 4)) % 4` produces
the correct 4-byte alignment padding for all values of `captured_len % 4`,
and the outer `% 4` clamps the result regardless of the intermediate wrapping.
The PC6b guard at lines 501-511 and 586-592 validates the total offset before
any slice indexing, so the intermediate wrapping value never escapes bounds
checking.

Replacing `wrapping_sub` with `(4 - (captured_len as usize % 4)) % 4` (which
would use debug-mode overflow checks to catch mistakes) is a cosmetic change.
In debug builds, `4 - 0` where the left side is `4usize` is not a subtraction
underflow — it is `4 - 0 = 4`, then `4 % 4 = 0`. The `wrapping_sub` is already
unnecessary from an overflow perspective (the `% 4` before it guarantees
the subtrahend is always 0, 1, 2, or 3, so `4 - x` for `x in [0,3]` is
always in `[1,4]`, never underflows). The existing idiom is idiomatic Rust
for this pattern and was reviewed as part of the F5 twin-equivalence proof.

**No change.** No spec update, no error taxonomy impact.
**F6 gate impact:** None.

---

## F6 Gate Summary

| Finding | Code | Severity | Decision | Gate Blocking |
|---------|------|----------|----------|---------------|
| F6-SEC-A | SEC-001 | MEDIUM (CWE-400) | DEFER — human policy decision required | NO — but FLAG for human |
| F6-SEC-B | SEC-002 | MEDIUM (CWE-770) | DEFER — contingent on SEC-A outcome | NO |
| F6-SEC-C | SEC-003 | LOW (CWE-367) | Accept-and-document | NO |
| F6-SEC-D | SEC-004 | LOW (CWE-209) | Accept-and-document | NO |
| F6-SEC-E | SEC-005 | INFO (CWE-191) | Accept — no change | NO |

**F6→F7 is NOT blocked by any finding on technical grounds.**

The only required human interaction is the F6-SEC-A ceiling-value policy
decision (see FLAG section above). That decision determines whether a
size-gate lands before F7 or is tracked as a deferred hardening item
alongside the streaming-reader follow-up.

---

## Artifact Changes Required

### If both findings are deferred (recommended baseline)

1. **STATE.md drift items:** Add F6-SEC-A and F6-SEC-B entries as shown above.
2. **ADR-009:** Add a brief note under Decision 13 referencing the SEC-A/SEC-B
   deferred-hardening items and the streaming-reader follow-up as the
   permanent architectural fix.
3. **src/reader.rs module doc:** The existing "known limitation" sentence at
   lines 23-24 is adequate. No change required.
4. **error-taxonomy.md:** No change (E-INP-014 remains unallocated/next_free).
5. **No BC changes.**

### If SEC-A ceiling fix is ordered by human (F6 implementation burst)

In addition to the above drift-item recording:

1. **src/reader.rs:** Add `MAX_PCAPNG_FILE_BYTES` constant. Add size-gate in
   `from_file` (before BufReader construction, on the pcapng-magic branch only,
   using `fs::metadata(path)?.len()`).
2. **error-taxonomy.md:** Add E-INP-014 entry (size-exceeded, broken, exit 1).
3. **BC-2.01.009:** Add Precondition for size gate; add E-INP-014 to error
   conditions. Update version.
4. **BC-2.01.017:** Add E-INP-014 to error-code table. Update version.
5. **ADR-009:** Add Decision N for interim size-gate policy.
6. **If SEC-B cap is added in same burst:** Add `MAX_INTERFACE_TABLE_ENTRIES`
   constant; add guard in IDB arm; assign E-INP-015 (since E-INP-014 is taken);
   update BC-2.01.011 PC3 and BC-2.01.017.
