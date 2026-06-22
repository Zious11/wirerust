# Demo Evidence Manifest — STORY-128

**Story:** STORY-128 — pcapng main.rs Per-File Error Isolation Loop (Catch-and-Continue)
**Epic:** E-19 | **Wave:** 56
**Coverage:** BC-2.01.018 AC-002 (per-file isolation) + BC-2.01.009 PC6 (zero-packet notice format)
**Test suite:** 22 tests — all passing (`cargo test --test bc_2_01_018_story128_tests`)
**Binary recorded:** wirerust v0.9.2 (worktree `STORY-128`, debug build)

---

## Recordings

### AC-001: Per-File Error Isolation (BC-2.01.018 AC-002 / ADR-009 Decision 12)

**Files:**
- `AC-001-per-file-isolation.gif` — VHS recording (204 KB)
- `AC-001-per-file-isolation.webm` — VHS recording (266 KB)
- `AC-001-per-file-isolation.tape` — VHS script source

**What it demonstrates:**

Directory contains two files in alphabetic order:
- `a-conflict.pcapng` — ETHERNET + LINUX_SLL IDBs → E-INP-011 conflict error (bad, sorts FIRST)
- `b-valid.pcapng` — SHB + ETHERNET IDB + 1 EPB (good)

The recording shows:
1. `ls /tmp/demo-128-iso/` — both files visible
2. `wirerust analyze /tmp/demo-128-iso/ --no-color` — the bad file emits a per-file error to
   stderr (`error: .../a-conflict.pcapng: ... E-INP-011 ...`), then the good file is processed
   and its packet counted (`Skipped: 1 packets` from the 0-byte-payload EPB decode error)
3. `echo exit:$?` — exit code 1 (any_error=true from the conflict file)

**Key isolation guarantee shown:** one bad file does NOT abort the batch. The good file runs
despite the bad file being processed first.

**Covered acceptance criteria:**
- BC-2.01.018 AC-002: batch completes, per-file error emitted, good output present
- STORY-128 AC-001: bad-file-first ordering → good files still processed
- STORY-128 AC-002: E-INP-011 specifically does not abort the batch
- STORY-128 EC-001: order independence (bad file first is the strongest discriminator)

---

### AC-002: Zero-Packet Notice Variants (BC-2.01.009 PC6 / ADR-009 Decision 19)

**Files:**
- `AC-002-zero-packet-notice.gif` — VHS recording (381 KB)
- `AC-002-zero-packet-notice.webm` — VHS recording (529 KB)
- `AC-002-zero-packet-notice.tape` — VHS script source

**What it demonstrates (three cases in one recording):**

**Case 1 — SHB-only pcapng (bare notice):**
```
notice: /tmp/demo-128-notice/shb-only.pcapng: 0 packets read from pcapng file
```
No parenthetical. skipped_blocks=0, opb_skipped=0 → both gates false → bare format.

**Case 2 — OPB-bearing pcapng (notice + OPB clause):**
```
notice: .../opb-bearing.pcapng: 0 packets read from pcapng file (includes 1 obsolete Packet Block(s) whose data was not analyzed; re-save with mergecap)
```
The OPB clause appears because opb_skipped=1. The "re-save with mergecap" remediation hint
is the headline data-integrity feature: users are told their OPB data was skipped and how to
recover it.

**Case 3 — Unknown-type skip blocks (notice + generic segment):**
```
notice: .../unknown-skip.pcapng: 0 packets read from pcapng file (2 block(s) skipped as unsupported)
```
G = skipped_blocks - opb_skipped = 2 - 0 = 2 → generic segment with count 2.

**Covered acceptance criteria:**
- BC-2.01.009 PC6 EC-007 (HS-108 Case D): OPB clause with count and mergecap hint
- BC-2.01.009 PC6 (HS-108 Case B): generic skip segment with block count
- BC-2.01.009 EC-010 / HS-108 Case F: bare notice when both counters are zero
- ADR-009 Decision 19: zero-packet notice format with independently-gated segments

---

### AC-003: Test Suite Evidence (22 tests passing)

**Files:**
- `AC-003-tests-passing.gif` — VHS recording (264 KB)
- `AC-003-tests-passing.webm` — VHS recording (282 KB)
- `AC-003-tests-passing.tape` — VHS script source

**What it demonstrates:**

`cargo test --test bc_2_01_018_story128_tests 2>&1 | tail -30` running in the STORY-128
worktree. The tail of the output shows all 22 test names with `... ok` and the final line:
```
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
```

**Tests covered by the 22-test suite:**
| Test | AC/BC |
|------|-------|
| `test_BC_2_01_018_per_file_isolation_continues_on_error` | AC-001 |
| `test_BC_2_01_018_einp011_does_not_abort_batch` | AC-002 |
| `test_BC_2_01_018_any_reader_error_isolated` | AC-003 (E-INP-008) |
| `test_BC_2_01_018_zero_packet_notice_not_suppressed_by_isolation` | AC-004 |
| `test_BC_2_01_018_order_independence_bad_file_first` | EC-001 |
| `test_BC_2_01_018_order_independence_bad_file_last` | EC-001 |
| `test_BC_2_01_018_order_independence_bad_file_middle` | EC-001 |
| `test_BC_2_01_018_all_good_batch_exit_zero` | EC-002 |
| `test_BC_2_01_018_all_bad_batch_no_panic_exit_one` | EC-003 |
| `test_BC_2_01_018_invariant1_reader_fail_closed_preserved` | Invariant 1 |
| `test_BC_2_01_018_summary_subcommand_per_file_isolation` | run_summary isolation |
| `test_BC_2_01_018_zero_packet_notice_decision19_lone_valid_file` | Decision 19 standalone |
| `test_BC_2_01_009_pc6_opb_clause_analyze` | PC6 OPB clause (analyze) |
| `test_BC_2_01_009_pc6_opb_clause_summary` | PC6 OPB clause (summary) |
| `test_BC_2_01_009_pc6_generic_skip_segment_analyze` | PC6 generic segment (analyze) |
| `test_BC_2_01_009_pc6_generic_skip_segment_summary` | PC6 generic segment (summary) |
| `test_BC_2_01_009_pc6_both_segments_nrb_plus_opb_analyze` | PC6 both segments (analyze) |
| `test_BC_2_01_009_pc6_both_segments_nrb_plus_opb_summary` | PC6 both segments (summary) |
| `test_BC_2_01_009_pc6_neither_segment_shb_only_analyze` | PC6 gate regression (analyze) |
| `test_BC_2_01_009_pc6_neither_segment_shb_only_summary` | PC6 gate regression (summary) |
| `test_BC_2_01_009_pc6_classic_pcap_wording_analyze` | PC6 classic-pcap wording (analyze) |
| `test_BC_2_01_009_pc6_classic_pcap_wording_summary` | PC6 classic-pcap wording (summary) |

---

## Coverage Map

| Acceptance Criterion | Demo | Recording |
|---------------------|------|-----------|
| BC-2.01.018 AC-002: per-file isolation + batch completion | AC-001 | `AC-001-per-file-isolation.gif` |
| BC-2.01.018 AC-002: bad-file-first ordering | AC-001 | `AC-001-per-file-isolation.gif` |
| BC-2.01.009 PC6 bare notice (SHB-only, Decision 19) | AC-002 Case 1 | `AC-002-zero-packet-notice.gif` |
| BC-2.01.009 PC6 OPB clause (includes N obsolete + mergecap) | AC-002 Case 2 | `AC-002-zero-packet-notice.gif` |
| BC-2.01.009 PC6 generic skip segment (N block(s) skipped) | AC-002 Case 3 | `AC-002-zero-packet-notice.gif` |
| 22-test suite (all AC/EC/PC6 variants) | AC-003 | `AC-003-tests-passing.gif` |

## AC Not Visually Demoed (with reason)

**Classic-pcap wording (BC-2.01.009 PC6 EC-009):** The `test_BC_2_01_009_pc6_classic_pcap_wording_*` tests (both analyze and summary) are covered by AC-003 (test suite). A separate VHS demo was omitted because the visual distinction (`"pcap file"` vs `"pcapng file"` in a single notice line) is small and the test evidence in AC-003 is unambiguous. If a visual demo is needed, the fixture is `empty_classic_pcap_bytes()` (24-byte global header) fed to `wirerust analyze`.

**run_summary isolation path:** Covered by `test_BC_2_01_018_summary_subcommand_per_file_isolation` in AC-003. Omitted from a separate VHS tape because the behavior is identical to AC-001 with `summary` replacing `analyze` — the test evidence is sufficient.

**Both-segments case (HS-108 Case E, 2 NRBs + 1 OPB):** Covered by AC-003 tests. The visual output is a superset of Cases 2 and 3 shown in AC-002 and is fully exercised by `test_BC_2_01_009_pc6_both_segments_nrb_plus_opb_*`.

---

## Tooling

- **Recording tool:** VHS 0.11.0 (`/opt/homebrew/bin/vhs`)
- **Font:** Menlo (system font, `/System/Library/Fonts/Menlo.ttc`)
- **Theme:** Dracula
- **Binary:** wirerust 0.9.2 (worktree `STORY-128`, `target/debug/wirerust`)
- **Fixture generation:** inline Python 3 (stdlib only, no dependencies)
