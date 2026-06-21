---
document_type: adjudication
finding_ids:
  - F-F5P1-003
  - O-2
status: resolved
produced_by: architect
date: 2026-06-21
feature: story-128-pcapng-perfile-isolation
bc_refs:
  - BC-2.01.009
  - BC-2.01.012
  - BC-2.01.013
adr_refs:
  - ADR-009 Decision 19 (zero-packet notice format)
  - ADR-009 Decision 22 (SPB parse path)
---

# F-F5P1-003 / O-2: Adjudication — Zero-Packet Notice Format Discriminant and SPB/EPB Check-Ordering

---

## Finding F-F5P1-003 (MEDIUM) — Zero-Packet Notice Re-reads File

### Gap Summary

`format_zero_packet_notice` (src/main.rs:61-72) calls `read_magic(path)` to
re-open the file a second time solely to discriminate "pcap file" vs "pcapng
file" in the notice wording. The first open already happened inside
`PcapSource::from_file` → `from_pcap_reader`, which branched on the identical
magic at reader.rs:603-614. Two concrete defects arise:

**(a) Redundant I/O.** Every zero-packet file causes two `open(2)` calls: one
inside `from_file` and one inside `read_magic`. On a 1,000-file directory scan
every zero-packet file pays double I/O cost — avoidable because the format
decision was already made.

**(b) TOCTOU mislabel.** If the file is deleted or replaced between the two
opens, `read_magic` returns `None` and the code defaults to "pcapng file"
(main.rs:71). A classic-pcap file that disappears between reads is then labelled
"pcapng" in the notice, which contradicts BC-2.01.009 PC6 EC-009 wording
symmetry: EC-009 mandates `"notice: <filename>: 0 packets read from pcap file"`
for empty classic-pcap inputs. The default is not just sloppy — it produces a
spec-incorrect message string.

### Decision: OPTION A — Add `is_pcapng: bool` discriminant to `PcapSource`

**Rationale.** The format discriminant is known with certainty at the branch
point inside `from_pcap_reader` (reader.rs:603-614). The pcapng branch is taken
when `magic == PCAPNG_MAGIC`; the classic-pcap branch is taken otherwise.
Carrying this decision forward as a struct field eliminates both defects cleanly:
no second file open, no TOCTOU exposure, no default that contradicts the BC.

Option B (document the TOCTOU default as accepted) is rejected because:
(1) it leaves the spec-incorrect label for a window that is operationally
constructible (deletion during a directory scan is not exotic); (2) the fix is
a two-line struct change plus a three-line formatter change — the cost of
accepting the defect exceeds the cost of fixing it.

### Exact Implementation Guidance

**Step 1 — Add the field to `PcapSource` (src/reader.rs:183-193)**

```rust
#[derive(Debug)]
pub struct PcapSource {
    pub packets: Vec<RawPacket>,
    pub datalink: DataLink,
    /// Total blocks entering the skip arm during pcapng block walk.
    pub skipped_blocks: u32,
    /// Sub-count of `skipped_blocks` that were Obsolete Packet Blocks.
    pub opb_skipped: u32,
    /// True when the source file was identified as pcapng via the magic-byte
    /// probe (BC-2.01.009 PC3); false for classic-pcap. Populated by
    /// `from_pcap_reader` at the branch point (reader.rs:603); consumed by
    /// `format_zero_packet_notice` (main.rs) to choose notice wording
    /// (BC-2.01.009 PC6 "pcap|pcapng" discriminant / Decision 19).
    pub is_pcapng: bool,
}
```

**Step 2 — Populate `is_pcapng` in both branch return sites**

The pcapng branch (reader.rs:697 → `read_pcapng_crate`) returns a `PcapSource`
at reader.rs:1206-1211. Set `is_pcapng: true` there:

```rust
Ok(PcapSource {
    packets,
    datalink: final_datalink,
    skipped_blocks,
    opb_skipped,
    is_pcapng: true,   // ← add this line
})
```

The classic-pcap branch returns a `PcapSource` at reader.rs:652-657. Set
`is_pcapng: false` there:

```rust
Ok(PcapSource {
    packets,
    datalink,
    skipped_blocks: 0,
    opb_skipped: 0,
    is_pcapng: false,  // ← add this line
})
```

**Step 3 — Replace `format_zero_packet_notice` body (src/main.rs:61-72)**

Remove the `read_magic` call entirely. Replace the discriminant logic with a
read of `source.is_pcapng`:

```rust
fn format_zero_packet_notice(path: &std::path::Path, source: &PcapSource) -> String {
    // Discriminant carried from the magic-byte probe in from_pcap_reader —
    // no second file open needed (BC-2.01.009 PC6 / Decision 19 / F-F5P1-003).
    let file_kind = if source.is_pcapng { "pcapng file" } else { "pcap file" };

    let base = format!(
        "notice: {}: 0 packets read from {file_kind}",
        path.display()
    );

    let g = source.skipped_blocks.saturating_sub(source.opb_skipped);
    let n = source.opb_skipped;

    match (g > 0, n > 0) {
        (false, false) => base,
        (true, false) => format!("{base} ({g} block(s) skipped as unsupported)"),
        (false, true) => format!(
            "{base} (includes {n} obsolete Packet Block(s) whose data was not analyzed; \
             re-save with mergecap)"
        ),
        (true, true) => format!(
            "{base} ({g} block(s) skipped as unsupported) (includes {n} obsolete Packet \
             Block(s) whose data was not analyzed; re-save with mergecap)"
        ),
    }
}
```

After this change `read_magic` is still called by `resolve_targets` (main.rs:667)
for directory-scan content detection — do NOT remove it. Only the call inside
`format_zero_packet_notice` is removed.

**Step 4 — Compiler follow-up**

`cargo check` will surface any other `PcapSource { ... }` literal that now needs
`is_pcapng` filled in (e.g., in unit tests that construct a `PcapSource`
directly). Set those to `false` for classic-pcap test fixtures, `true` for
pcapng test fixtures. No test behavioral change is expected; the field only
affects the notice format.

### BC / Spec Version Bump Required?

No BC version bump is required. BC-2.01.009 PC6 already specifies `"pcap|pcapng"`
as the discriminated wording (Decision 19) and says nothing about how the
discriminant is obtained — it places no constraint on implementation mechanism.
The `is_pcapng` field is an internal implementation detail that makes the BC
obligation correctly fulfillable. The observable behaviour (notice wording) was
already mandated; this change makes it reliably correct rather than TOCTOU-fragile.

An ADR-009 amendment note should be added to Decision 19 referencing the
`is_pcapng` carrier field as the canonical discriminant for the
"pcap|pcapng" notice wording, to close the implementation gap that the finding
exposed.

---

## Finding O-2 (LOW) — SPB vs EPB Check-Ordering Asymmetry

### Gap Summary

BC-2.01.012 PC9 mandates a five-step EPB evaluation order:

- (i) body.len() >= 20 → else E-INP-008
- (ii) read interface_id
- (iii) interface table EMPTY → E-INP-009 (before any captured_len work)
- (iv) interface_id OOB on non-empty table → E-INP-010
- (v) captured_len validation → E-INP-008

The SPB arm in reader.rs (lines ~1089-1115) reverses the body-len and empty-table
order:

- (A) empty-table guard (E-INP-009) at ~1096
- (B) body-length guard (E-INP-008) at ~1107

For an SPB that is simultaneously before any IDB AND has a body shorter than 4
bytes (btl=12, body=0), the SPB arm currently fires E-INP-009 (empty-table) while
the EPB arm applied to the same logical condition would fire E-INP-008 (body-too-
short, step i) before reaching the empty-table check.

BC-2.01.013 does not specify an evaluation order between these two guards; it
only mandates that both produce their respective error codes when their respective
conditions are met. The SPB ordering is not a BC-2.01.013 contract violation.

### Decision: DO NOT ALIGN — Document the SPB ordering as accepted behavior

**Rationale.**

**(1) No behavioral specification violation.** BC-2.01.013 does not define an
evaluation order between the empty-table guard (AC-001 / PC5 / EC-006) and the
body-too-short guard (AC-004a / PC6 / EC-008). The BC is silent on which fires
first when both conditions are true. The EPB precedence in BC-2.01.012 PC9 is an
EPB-specific postcondition, not a sibling discipline that extends by implication
to SPB.

**(2) The overlap is a single constructible case: btl=12, empty table.** btl=12
is the only constructible SPB body-too-short window (body=0 < 4 bytes). A file
that is both btl=12 AND before any IDB presents an empty-table condition
(E-INP-009 under current SPB ordering) vs a body-too-short condition (E-INP-008
under EPB-aligned ordering). Either error correctly rejects the block. No silent
pass-through exists under either ordering. The distinction is which error string
the caller receives — this is an error-case quality concern, not a safety or
correctness concern.

**(3) The SPB ordering has an independent semantic rationale.** The SPB empty-table
guard (E-INP-009) fires before body decode because it is a structural violation at
the file level — no IDB means the pcapng section is malformed for any block
referencing interface 0. The EPB precedence (body-len first) was introduced in
BC-2.01.012 v1.7 specifically to ensure `interface_id` can be safely read from the
body before the table check. That rationale does not apply to SPB: SPB has no
`interface_id` field and always binds to interface 0. Checking the table early for
SPB is semantically valid and slightly more informative to callers.

**(4) Cost/risk of alignment.** Aligning SPB to EPB ordering would mean checking
body.len() >= 4 before the empty-table guard. This is a behavior change that would
require a BC-2.01.013 version bump, a holdout update (HS-107 currently validates
the btl=12/body=0 case as E-INP-008, which is the same outcome under both
orderings — but the empty-table precondition is not combined in HS-107's Case F
fixture). The risk of test regression for no observable external benefit justifies
the non-alignment decision.

### Accepted Behavior Statement for BC-2.01.013

The following note is to be added to BC-2.01.013 as a sub-item under Postcondition
5 (or as a new Postcondition 5a note) in the next version update:

> **SPB check ordering (sibling-discipline note):** Unlike BC-2.01.012 PC9 (EPB),
> BC-2.01.013 does not mandate a specific evaluation order between the empty-table
> guard (PC5 / E-INP-009) and the body-too-short guard (PC6 / E-INP-008). The
> implementation checks the empty-table condition (reader.rs SPB arm) before the
> body.len() >= 4 check. This ordering is accepted: SPB has no `interface_id` field
> (it always uses interface 0), so the empty-table check does not depend on reading
> any body field. For the single constructible overlap case (btl=12, body=0, empty
> table), the implementation yields E-INP-009; EPB-aligned ordering would yield
> E-INP-008. Both correctly reject the block. The asymmetry is accepted and
> documented; no implementation change is required.

### BC / Spec Version Bump Required?

A minor BC-2.01.013 version bump (v1.9 → v2.0 or a v1.9 patch) is recommended
to add the accepted-behavior note described above. This is documentation-only; no
normative constraint changes. The version bump prevents a future adversarial pass
from re-raising O-2 as an open finding.

No implementation change to reader.rs is required.

---

## Summary Table

| Finding | Severity | Decision | Impl Change | BC Bump |
|---------|----------|----------|-------------|---------|
| F-F5P1-003 | MEDIUM | Option A: add `is_pcapng: bool` to `PcapSource`; remove `read_magic` call from notice formatter | reader.rs: add field + populate in both branch returns; main.rs: replace `read_magic` call with `source.is_pcapng` | No (note ADR-009 Decision 19) |
| O-2 | LOW | Accept SPB ordering; document as accepted in BC-2.01.013 | None | Yes (BC-2.01.013 minor bump to add accepted-behavior note) |
