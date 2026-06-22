# Demo Evidence Manifest — STORY-127

**Story:** STORY-127 — pcapng magic-byte glob (resolve_targets Content Detection) and E2E Corpus Wiring
**Epic:** E-19 | **Wave:** 55
**BC:** BC-2.12.011
**Branch:** worktrees/STORY-127
**Recorded:** 2026-06-20
**Toolchain:** VHS 0.11.0

---

## Coverage Summary

| Recording | AC(s) | Behavior | Observable |
|-----------|-------|----------|-----------|
| `AC-004-content-over-extension` | AC-004 | HEADLINE: pcapng-magic .cap detected; wrong-magic .pcap skipped | Packets: 54 from .cap; no error from imposter |
| `AC-001-five-magic-values` | AC-001 | All 5 magic variants detected regardless of extension | Skipped: 5 packets (one per magic; reject.pcap excluded) |
| `AC-002-003-silent-skip` | AC-002, AC-003 | Wrong-magic files and short files silently skipped | Packets: 0, exit 0 (no error, no panic) |
| `AC-005-sorted-output` | AC-005 | Lexicographic sort enforced; .cap detected in sorted order | 71 total packets; a, m, z order despite z,a,m creation order |
| `AC-009-e2e-corpus` | AC-009 | Full reader stack via smb3.pcapng (54 pkts) + synthetic 16-EPB .cap | Sub-case 1: Packets: 54; Sub-case 2: Skipped: 16 |
| `AC-000-test-suite` | All ACs | Full test suite run: 9/9 pass | `test result: ok. 9 passed` |

---

## Recordings

### AC-004-content-over-extension (HEADLINE)

**Demonstrates:** BC-2.12.011 Inv1 / ADR-009 C-2 resolution.

A file named `capture-renamed-as.cap` (pcapng content, `.cap` extension) is detected and
analyzed — producing 54 SMB packets from smb3.pcapng content. A file named
`imposter-wrong-magic.pcap` (wrong magic `[DE AD BE EF]`, `.pcap` extension) is silently
skipped — no error, no reader invocation.

This resolves ADR-009 C-2: the prior extension-based filter excluded `.cap` files entirely,
causing `arp-baseline-16pkt.cap` to be missed.

- `AC-004-content-over-extension.gif`
- `AC-004-content-over-extension.webm`
- `AC-004-content-over-extension.tape` (source)
- `demo-ac004-content-over-extension.sh` (fixture builder)

---

### AC-001-five-magic-values

**Demonstrates:** BC-2.12.011 PC1 + Inv1 + Inv2.

A directory with 5 capture files, each using a non-standard extension that the pre-STORY-127
stub would reject: `1-be.PCAP`, `2-le.CAP`, `3-ns-be.data`, `4-ns-le.txt`, `5-ng.bin`.
A `reject.pcap` with wrong magic `[DE AD BE EF]` is silently excluded.

The discriminating observable: `Skipped: 5 packets (decode errors)` — exactly one decode
error per magic variant. If any of the 5 CAPTURE_MAGICS entries were missing, the count
would drop below 5 and the test would fail.

- `AC-001-five-magic-values.gif`
- `AC-001-five-magic-values.webm`
- `AC-001-five-magic-values.tape` (source)
- `demo-ac001-five-magics.sh` (fixture builder)
- `build-5-magic-fixtures.py` (Python fixture generator)

---

### AC-002-003-silent-skip (ERROR PATHS)

**Demonstrates:** BC-2.12.011 PC2 (wrong magic) + PC3/Inv5 (short file).

Two error-path scenarios:
- **AC-002:** A directory containing only `bad.pcap` with wrong magic `[DE AD BE EF]` →
  `Packets: 0`, exit 0. The file is silently excluded at the magic-probe stage; the reader
  is never invoked.
- **AC-003:** A directory containing only `truncated.pcap` (3 bytes, too short for 4-byte
  probe) → `Packets: 0`, exit 0. No panic, no error.

Both are intentional silent skips, not routing errors.

- `AC-002-003-silent-skip.gif`
- `AC-002-003-silent-skip.webm`
- `AC-002-003-silent-skip.tape` (source)
- `demo-ac002-003-silent-skip.sh` (fixture builder)

---

### AC-005-sorted-output

**Demonstrates:** BC-2.12.011 PC5 + Inv3 (lexicographic sort) + .cap content detection.

Files created in reverse sort order (`z.pcap`, `a.pcap`, `m.cap`) — `ls -lt` shows creation
order z, a, m. Analysis processes them in sorted order a, m, z. The `m.cap` file (pcapng
magic, `.cap` extension) is detected by content and included.

Result: 71 total packets (1 HTTP from a.pcap + 54 SMB from m.cap + 16 HTTP-OOO from z.pcap).

- `AC-005-sorted-output.gif`
- `AC-005-sorted-output.webm`
- `AC-005-sorted-output.tape` (source)
- `demo-ac005-sorted.sh` (fixture builder)

---

### AC-009-e2e-corpus

**Demonstrates:** BC-2.12.011 EC-001..002 — full E2E reader stack corpus wiring.

- **Sub-case 1:** `smb3.pcapng` (committed fixture, 15692 bytes) → pcapng reader via
  STORY-123 probe + STORY-125 EPB parse → `Packets: 54` SMB packets.
- **Sub-case 2:** Synthetic 16-EPB pcapng written to a `.cap` extension tempfile →
  content detection routes to pcapng reader → `Skipped: 16 packets (decode errors)` from
  empty-payload EPBs. This is the synthetic fallback for the F-5 deferred authentic
  `arp-baseline-16pkt.cap` fixture (gitignored; not present in CI).

- `AC-009-e2e-corpus.gif`
- `AC-009-e2e-corpus.webm`
- `AC-009-e2e-corpus.tape` (source)
- `demo-ac009-e2e.sh` (fixture builder)

---

### AC-000-test-suite

**Demonstrates:** All 9 acceptance criteria via the formal test suite.

`cargo test --test bc_2_12_011_story127_tests` — 9/9 pass:

```
test story_127::test_BC_2_12_011_all_5_magic_values_accepted ... ok
test story_127::test_BC_2_12_011_cap_extension_pcapng_magic_accepted ... ok
test story_127::test_BC_2_12_011_e2e_corpus_pcapng_reader_stack ... ok
test story_127::test_BC_2_12_011_empty_directory ... ok
test story_127::test_BC_2_12_011_io_error_on_probe_silently_skipped ... ok
test story_127::test_BC_2_12_011_non_magic_silently_skipped ... ok
test story_127::test_BC_2_12_011_short_file_skipped ... ok
test story_127::test_BC_2_12_011_sorted_output ... ok
test story_127::test_BC_2_12_011_subdir_skipped ... ok
test result: ok. 9 passed
```

- `AC-000-test-suite.gif`
- `AC-000-test-suite.webm`
- `AC-000-test-suite.tape` (source)

---

## AC Coverage Map

| AC | Behavior Tested | Recording | Visually Demoable? |
|----|----------------|-----------|-------------------|
| AC-001 | All 5 magic values accepted by content | `AC-001-five-magic-values` | Yes — Skipped: 5 |
| AC-002 | Wrong-magic file silently skipped | `AC-002-003-silent-skip` | Yes — Packets: 0, exit 0 |
| AC-003 | Short (<4 byte) file silently skipped | `AC-002-003-silent-skip` | Yes — Packets: 0, exit 0 |
| AC-004 | .cap extension + pcapng magic accepted (C-2 fix) | `AC-004-content-over-extension` | Yes — Packets: 54 |
| AC-005 | Sorted lexicographic output + .cap in sort | `AC-005-sorted-output` | Yes — 71 total pkts |
| AC-006 | Empty directory returns 0 packets, exit 0 | `AC-000-test-suite` | Via test only (trivial path) |
| AC-007 | I/O error on magic probe silently skipped (Unix chmod 000) | `AC-000-test-suite` | Via test only (requires root-restricted file) |
| AC-008 | Subdirectories skipped (is_file() check) | `AC-000-test-suite` | Via test only (trivial guard) |
| AC-009 | E2E corpus: smb3.pcapng + 16-EPB .cap | `AC-009-e2e-corpus` | Yes — Packets: 54, Skipped: 16 |

### Notes on test-only ACs

- **AC-006** (empty directory): trivially demonstrated — no directory entry to iterate,
  always returns 0 packets. Pinned in the test suite as a regression guard.
- **AC-007** (I/O error silent skip): requires `chmod 000` on a file, which is a Unix-only
  permission test. Cannot be meaningfully shown in a VHS terminal without elevated setup.
  The test `test_BC_2_12_011_io_error_on_probe_silently_skipped` (marked `#[cfg(unix)]`)
  covers this path fully.
- **AC-008** (subdirectory skipped): the `is_file()` guard is a single-line Rust check.
  Visual demo would show an empty result, indistinguishable from AC-006. The test covers it.

---

## Artifact Inventory

```
/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-127/
├── demo-manifest.md                          # this file
├── AC-000-test-suite.gif                     # test suite: 9/9 pass
├── AC-000-test-suite.webm
├── AC-000-test-suite.tape
├── AC-001-five-magic-values.gif              # all 5 magics detected
├── AC-001-five-magic-values.webm
├── AC-001-five-magic-values.tape
├── demo-ac001-five-magics.sh
├── build-5-magic-fixtures.py
├── AC-002-003-silent-skip.gif               # error paths: silent skip
├── AC-002-003-silent-skip.webm
├── AC-002-003-silent-skip.tape
├── demo-ac002-003-silent-skip.sh
├── AC-004-content-over-extension.gif        # HEADLINE: .cap detected
├── AC-004-content-over-extension.webm
├── AC-004-content-over-extension.tape
├── demo-ac004-content-over-extension.sh
├── AC-005-sorted-output.gif                 # sorted + mixed extensions
├── AC-005-sorted-output.webm
├── AC-005-sorted-output.tape
├── demo-ac005-sorted.sh
├── AC-009-e2e-corpus.gif                    # E2E: smb3 + 16-EPB .cap
├── AC-009-e2e-corpus.webm
├── AC-009-e2e-corpus.tape
└── demo-ac009-e2e.sh
```
