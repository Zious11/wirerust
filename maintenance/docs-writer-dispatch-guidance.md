# Docs-Writer Dispatch: Ground-Truth Citation Mandate

**Policy reference:** PG-RA-P3-ARP-REC006-INVERSION-001  
**Finding reference:** F-RA-P3-001 (maint-2026-07-09 Route A adversary Pass 3)  
**Codification story:** STORY-163 AC-163-001  
**Added:** 2026-07-10 (STORY-163 AC-163-001)

---

## Background

During the maint-2026-07-09 Route A documentation sweep, a docs-writer dispatch
produced a claim that VLAN/QinQ/MACsec-tagged ARP frames produce _no_ findings.
This was factually inverted. `src/decoder.rs` D-078/D-078b (strict path lines 196-210,
lax path lines 242-257, VLAN-offset fix lines 291-313) shows that these frames are
correctly handled by both parse paths and do produce findings via `extract_arp_frame`.

The dispatch contained only the one-line sweep recommendation REC-006 ("README §
Known Limitations: add one sentence on MACsec/VLAN ARP offset detection limitation")
and did not name any source file for the writer to Read before drafting behavioral
claims. Finding F-RA-P3-001 (adversary Pass 3, classified HIGH;
`.factory/maintenance/sweep-report-2026-07-09.md:326`) caught the inversion
before merge. This guidance codifies the dispatch requirement to prevent recurrence.

---

## Section 1 — Scope

This guidance applies to any dispatch of a technical-writer, docs-writer, or equivalent
agent (or equivalent instruction to any agent) to produce or modify **user-facing
documentation** from:

- finding summaries (sweep reports, adversarial pass findings, PR review findings)
- sweep recommendations or one-line recommendation lists
- any other compressed description of behavioral properties, API behavior, or code
  capabilities

It applies regardless of document type: README sections, ADR decision blocks,
Known-Limitations entries, CLI help text, or module documentation.

It does **not** apply to pure change-log enumeration (e.g., listing commit SHAs,
story IDs, or PR numbers) where no behavioral claim about code behavior is made. If
even one behavioral claim is present in the draft, the mandate applies to that claim.

---

## Section 2 — Ground-Truth Citation Mandate

Every behavioral claim in the produced documentation text MUST be traceable to a
specific source anchor that the writer actually Read during the task:

- **Code anchor:** `file:line` reference (e.g., `src/decoder.rs:291-313`)
- **Spec anchor:** BC-S.SS.NNN behavioral contract section (e.g., `BC-2.16.015 v1.6 PC-7a`)
- **ADR anchor:** ADR section (e.g., `ADR-007 Decision 3`)
- **VP anchor:** VP-NNN property statement

The dispatch MUST name the expected ground-truth source files explicitly so the writer
can Read them before drafting any behavioral claim. Naming the files in the dispatch
prompt is a prerequisite, not a suggestion.

A behavioral claim is any statement about:
- what the code does or does not do for a given input
- what output or finding is produced (or not produced) in a given scenario
- what edge cases are handled or not handled by the implementation
- what limitations exist in detection, analysis, or protocol support

---

## Section 3 — Inversion-Prevention Rule

One-line finding summaries (from sweep reports, recommendation lists, or adversarial
findings) are sufficient to identify **what topic to document**, but are NOT sufficient
as the sole input for **drafting behavioral claims**. The writer must verify the actual
code or spec behavior against the named file:line anchor before writing.

**Before writing any behavioral claim, the writer MUST:**

1. Identify the source file(s) that contain the implementation or spec for the claimed
   behavior (or confirm from the dispatch's named sources).
2. Read those files at the relevant lines.
3. Confirm that the proposed claim matches what the code or spec actually says.
4. Record the `file:line` anchor alongside the claim before submitting the draft.

A one-line summary that says "ARP detection limitation" cannot be safely paraphrased
without verifying whether the limitation is:

- that the feature is absent (no finding produced for any case), or
- that findings are produced in the common case but absent in one specific edge case, or
- that detection is present but has a bounded accuracy window under certain conditions.

These are three distinct claims that cannot be disambiguated from a one-line summary.
Paraphrasing the summary without verification is the direct cause of factual inversions.
Post-draft verification is performed by the per-story fresh-context adversarial convergence
pass (BC-5.39.001), which MUST open every cited anchor — the writer's self-produced
claims-citation table is an input to that audit, never its substitute.

---

## Section 4 — Verification Template for Orchestrator Dispatches

The orchestrator MUST include the following block verbatim in every docs-remediation
dispatch that involves behavioral claims, substituting the `[ORCHESTRATOR: ...]` placeholder
with the actual named source files — a dispatch that ships the unsubstituted placeholder
is NON-COMPLIANT (the writer would have no named files to Read):

```
## Ground-Truth Citation Requirement (mandatory per PG-RA-P3-ARP-REC006-INVERSION-001)

Before drafting any behavioral claim in the documentation:

1. Named ground-truth source files for this task:
   [ORCHESTRATOR: list files explicitly, e.g., src/decoder.rs, docs/adr/NNNN-*.md]

2. Read each named file at the lines relevant to your claims before drafting.

3. For each behavioral claim in your draft, append a citation in the form:
   > Claim: <one-sentence behavioral statement>
   > Source: <file:line or spec anchor you Read>

4. Do not draft a behavioral claim you cannot cite. If the source file is ambiguous,
   missing, or contradicts the finding summary, STOP and report the discrepancy to
   the orchestrator before producing any documentation output.

5. Submit the claims-citation table alongside the draft output.
```

### Citation Preflight Validation (PG-W73-CITATION-VALIDATOR — STORY-164 AC-164-002)

Before submitting the claims-citation table for review, extract the `file:line` anchors
from the table into a plain text file (one `path:LINE` or `path:LINE-LINE` entry per line,
no pipe characters or markdown) and run `bin/validate-citations` on that file. Any FAIL
result means a cited file or line range does not exist — the anchor MUST be corrected
before proceeding.

```bash
# Example plain-anchor file (one bare path:LINE per line — no # prefix, no table pipes):
src/decoder.rs:196-210
docs/adr/0007-dnp3-stream-dispatch-and-parser-design.md:45

# Run the validator:
bin/validate-citations path/to/anchors.txt
```

A FAIL result indicates a phantom anchor — the cited file does not exist or the cited
line numbers exceed the file's actual line count. This is the mechanical equivalent of
the fabrication caught in F-S163P1-001 (STORY-163 adversarial Pass 1, CRITICAL severity).
Do not proceed with a FAIL result; correct the anchor to an existing file:line reference
before dispatching the evidence artifact.

---

## Section 5 — Concrete Application Example: REC-006 / F-RA-P3-001

### The finding summary (insufficient as sole input)

REC-006 from `.factory/maintenance/sweep-report-2026-07-09.md` (§Route A table):

> "README § Known Limitations: add one sentence on MACsec/VLAN ARP offset detection
> limitation"

This one-line summary names the topic (MACsec/VLAN ARP offset detection) and genre
(Known Limitations entry). It does NOT specify whether the limitation means:
- that no findings are produced for VLAN/QinQ/MACsec-tagged ARP frames, or
- that findings are produced but detection has a specific boundary condition.

### The incorrect draft (produced without ground-truth verification)

The initial docs-writer draft stated that VLAN/QinQ/MACsec-tagged ARP frames produce
_no_ findings — i.e., that the decoder does not detect ARP anomalies in VLAN-tagged
frames at all. This was a factual inversion of the code behavior.

Root cause: the dispatch contained only the REC-006 one-liner. The writer was not
required to cite a file:line anchor for each behavioral claim, and was not told which
source files to Read for verification.

### The ground-truth check (what the verification template requires)

Reading `src/decoder.rs` D-078/D-078b before drafting reveals:

**Strict path (lines 196-210):** VLAN-tagged ARP frames where the inner EtherType is
0x0806 produce `Some(NetSlice::Arp(arp))` from the etherparse strict slicer, with
VLAN extension headers carried in `link_exts`. `extract_arp_frame` is called on this
ARP slice. For valid Ethernet/IPv4 ARP payloads — including those inside VLAN, QinQ,
or MACsec-encapsulated Ethernet frames — the result is `Ok(DecodedFrame::Arp(f))`.
Findings ARE produced.

**Lax path (lines 242-257):** For truncated VLAN-tagged ARP frames (snaplen cut), the
lax parser produces `Some(LaxNetSlice::Arp(arp))` and `extract_arp_frame` is called
under the same logic. D-078b path-independence (confirmed at lines 22, 157) means both
paths emit the same result for valid ARP payloads. Findings ARE produced.

**D-078 VLAN-offset fix (lines 291-313):** For severe truncation where the lax parser
cannot reconstruct the net layer, the code computes the ARP payload offset as
14 bytes (Ethernet2 base) plus `LaxLinkExtSlice::header_len()` summed over all
link-extension headers (`lax.link_exts`). A single 802.1Q VLAN tag adds 4 bytes, QinQ
adds 8, and MACsec adds its variable header length — all handled via
`LaxLinkExtSlice::header_len()` without hardcoding (lines 292-294, 307-313).

**Correct claim:** VLAN/QinQ/MACsec-tagged ARP frames ARE handled by the lax path.
The D-078 fix ensures the correct ARP payload offset is computed via
`LaxLinkExtSlice::header_len()`. Findings are produced for frames with valid
Ethernet/IPv4 ARP payloads. The README Known-Limitations entry correctly describes the
detection boundary — it does not state that findings are absent, because they are not.

### Claims-citation table produced by this verification

| Claim | Source anchor |
|-------|--------------|
| VLAN/QinQ/MACsec-tagged ARP frames reach `extract_arp_frame` on the strict path | `src/decoder.rs:196-210` |
| VLAN/QinQ/MACsec-tagged ARP frames reach `extract_arp_frame` on the lax path | `src/decoder.rs:242-257` |
| Valid Ethernet/IPv4 ARP payloads produce `Ok(DecodedFrame::Arp(f))` — findings generated | `src/decoder.rs:205, 251` |
| `LaxLinkExtSlice::header_len()` computes correct offset for VLAN, QinQ, and MACsec | `src/decoder.rs:291-313` |
| D-078/D-078b path-independence: same result on both strict and lax paths | `src/decoder.rs:22, 157` |

---

## Reference

- **PG-RA-P3-ARP-REC006-INVERSION-001:** Root process-gap (maint-2026-07-09 Route A
  adversary Pass 3, 2026-07-09/10). Direct cause of this guidance document.
- **F-RA-P3-001:** Finding — inverted VLAN/QinQ/MACsec claim, HIGH severity, maint-2026-07-09
  Route A adversary Pass 3. HIGH severity attribution:
  `.factory/maintenance/sweep-report-2026-07-09.md:326`. Finding existence and resolution
  evidence: `.factory/code-delivery/maint-2026-07-09/pr-review.md` §Adversarial Convergence
  Evidence Reviewed (lines 99-101).
- **REC-006:** Original sweep recommendation, one-liner that triggered the inversion.
  `.factory/maintenance/sweep-report-2026-07-09.md` §Route A table (line 202).
- **D-078 / D-078b:** VLAN-offset fix decision references, `src/decoder.rs` lines 22, 157,
  266, 291-313.
- **BC-2.16.009 / BC-2.16.015:** Behavioral contracts for ARP path-independence and VLAN
  offset handling.
- **STORY-163 AC-163-001:** Factory codification story for this guidance.
