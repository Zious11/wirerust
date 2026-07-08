---
document_type: maintenance-sweep-output
sweep: pattern-consistency
sweep_id: maint-2026-07-08 / Sweep-3
producer: code-reviewer
develop_head: b642c0fdabfd6ae9f9ea8d1680b50662c5654e93
date: 2026-07-08
---

# Pattern Consistency Findings — maint-2026-07-08

Run: **maint-2026-07-08**, Sweep 3.
Scope: `src/` + `tests/`, codebase at `b642c0f` (develop).
Clippy result: **CLEAN** (`cargo clippy --all-targets -- -D warnings`, 0 warnings).

---

## Summary Table

| ID | Description | Severity | Classification |
|----|-------------|----------|----------------|
| PF-001 | Counter discipline: plain `+=` on u64/u32 diagnostic counters across all analyzer modules | MEDIUM | MANUAL |
| PF-002 | dnp3.rs free-function naming: 4 functions lack the `dnp3_` module prefix | LOW | FIXABLE-AUTO |
| PF-003 | enip.rs `check_t0814` lacks `enip_` module prefix used by peer functions | LOW | MANUAL |
| PF-004 | Trait gap: `Dnp3Analyzer` and `EnipAnalyzer` do not implement `StreamHandler`/`StreamAnalyzer` | LOW | ARCH-REVIEW |
| PF-005 | Error handling style: all analyzers consistent — no finding | — | INFO (CLEAN) |
| PF-006 | Clippy gate | — | INFO (CLEAN) |
| PF-007 | PG-HELP-PROVENANCE-CLI-DOC-001 (factory IDs in clap `///` doc-comments) | — | INFO (CLEAN) |
| PF-008 | Wave-71 CR-001 (MINOR + 3 NITs): no standalone gate-level document; per-story NITs verified present | LOW | INFO |

---

## PF-001 — Counter Discipline (MANUAL)

**Description:**
The house style established by SEC-003/SEC-004/SEC-007 and the open item REBIND-COUNT-SATURATING-001
is `saturating_add` for all diagnostic counter increments. This style is consistently applied in
`src/dispatcher.rs`, `src/reader.rs`, `src/reassembly/flow.rs`, and `src/reassembly/segment.rs`.
However, all six protocol analyzer modules and `src/reassembly/lifecycle.rs` use plain `+=`.

The known open item REBIND-COUNT-SATURATING-001 calls out only `src/analyzer/arp.rs:856`
(`entry.rebind_count`, type `u32`). This scan enumerates all additional sites.

**Sites by file (plain `+=` on diagnostic counters, excluding cursor/index variables):**

### `src/analyzer/dns.rs` — 2 sites

| Line | Expression | Field type |
|------|-----------|------------|
| 72 | `self.query_count += 1` | `u64` |
| 74 | `self.response_count += 1` | `u64` |

### `src/analyzer/arp.rs` — 5 sites (line 856 is the known REBIND-COUNT-SATURATING-001 item)

| Line | Expression | Field type |
|------|-----------|------------|
| 467 | `self.frames_analyzed += 1` | `u64` |
| 469 | `self.request_count += 1` | `u64` |
| 470 | `self.reply_count += 1` | `u64` |
| 471 | `self.other_opcode_count += 1` | `u64` |
| 856 | `entry.rebind_count += 1` | `u32` (REBIND-COUNT-SATURATING-001) |

### `src/analyzer/tls.rs` — 6 sites

| Line | Expression | Field type |
|------|-----------|------------|
| 486 | `self.handshakes_seen += 1` | `u64` |
| 501 | `self.parse_errors += 1` | `u64` |
| 710 | `self.parse_errors += 1` | `u64` |
| 951 | `self.parse_errors += 1` | `u64` |
| 1009 | `self.parse_errors += 1` | `u64` |
| 1010 | `self.truncated_records += 1` | `u64` |

### `src/analyzer/enip.rs` — 4 sites

| Line | Expression | Field type |
|------|-----------|------------|
| 868 | `flow.malformed_in_window += 1` | `u64` |
| 894 | `flow.malformed_in_window += 1` | `u64` |
| 957 | `flow.malformed_in_window += 1` | `u64` |
| 1346 | `flow.write_count_in_window += 1` | `u64` |

### `src/analyzer/dnp3.rs` — 25 sites

| Line | Expression | Field type |
|------|-----------|------------|
| 496 | `flow.parse_errors += 1` | `u64` |
| 497 | `flow.malformed_in_window += 1` | `u64` |
| 562 | `flow.parse_errors += 1` | `u64` |
| 563 | `flow.malformed_in_window += 1` | `u64` |
| 595 | `flow.parse_errors += 1` | `u64` |
| 596 | `flow.malformed_in_window += 1` | `u64` |
| 661 | `flow.parse_errors += 1` | `u64` |
| 662 | `flow.malformed_in_window += 1` | `u64` |
| 703 | `flow.parse_errors += 1` | `u64` |
| 704 | `flow.malformed_in_window += 1` | `u64` |
| 750 | `flow.parse_errors += 1` | `u64` |
| 751 | `flow.malformed_in_window += 1` | `u64` |
| 766 | `flow.frame_count += 1` | `u64` |
| 786 | `self.master_addrs_dropped += 1` | `u64` |
| 1020 | `flow.direct_operate_count += 1` | `u32` |
| 1063 | `*dropped_findings += 1` | `u64` (via `&mut u64` param) |
| 1122 | `*dropped_findings += 1` | `u64` |
| 1127 | `flow.restart_event_count += 1` | `u64` |
| 1173 | `*dropped_findings += 1` | `u64` |
| 1223 | `flow.block_event_count += 1` | `u64` |
| 1258 | `*dropped_findings += 1` | `u64` |
| 1390 | `*dropped_findings += 1` | `u64` |
| 1452 | `*dropped_findings += 1` | `u64` |
| 1490 | `*dropped_findings += 1` | `u64` |
| 1616 | `*dropped_findings += 1` | `u64` |
| 1681 | `*dropped_findings += 1` | `u64` |
| 1720 | `*dropped_findings += 1` | `u64` |
| 1792 | `*dropped_findings += 1` | `u64` |

### `src/reassembly/lifecycle.rs` — 3 sites

| Line | Expression | Field type |
|------|-----------|------------|
| 166 | `self.stats.evictions += 1` | `u64` |
| 183 | `self.stats.dropped_findings += 1` | `u64` |
| 213 | `self.stats.dropped_findings += 1` | `u64` |

### `src/dispatcher.rs` — 1 site

| Line | Expression | Field type |
|------|-----------|------------|
| 477 | `self.unclassified_flows += 1` | `u64` |

Note: The rest of `dispatcher.rs` (lines 371, 494, 715) already uses `saturating_add`. The line 477
site is the lone outlier in an otherwise compliant file. The surrounding comment explicitly preserves
the gating logic — the plain `+=` is not accidentally placed but was written before the saturating
discipline was enforced.

**Also in `src/main.rs`:**

| Line | Expression | Field type |
|------|-----------|------------|
| 442 | `arp_analyzer.malformed_frames += 1` | `u64` (`ArpAnalyzer::malformed_frames`) |
| 451 | `total_decode_errors += 1` | local `u64` var |
| 837 | `total_decode_errors += 1` | local `u64` var |

**Total: ~48 plain `+=` sites on diagnostic counters** (excluding cursor/index usize vars and
HashMap entry value increments).

**Classification:** MANUAL — The change is mechanical (each `x += 1` → `x = x.saturating_add(1)`)
but the scope (6 files, ~48 sites) warrants a dedicated PR with full test run. Some window counters
(`malformed_in_window`, `write_count_in_window`) are reset per detection window; saturating semantics
are still correct there (capped at u64::MAX rather than silently wrapping). No behavioral change
expected in practice.

---

## PF-002 — dnp3.rs Free-Function Naming Inconsistency (FIXABLE-AUTO)

**Description:**
`src/analyzer/dnp3.rs` has two groups of public free functions. Four carry a `dnp3_` module prefix;
four do not:

| Function | Has `dnp3_` prefix? |
|----------|-------------------|
| `parse_dnp3_dl_header` | Yes |
| `is_valid_dnp3_frame_header` | Yes |
| `classify_dnp3_fc` | Yes |
| `compute_dnp3_frame_len` | Yes |
| `transport_is_fir` | **No** |
| `has_user_data` | **No** |
| `is_broadcast_destination` | **No** |
| `is_master_frame` | **No** |

The un-prefixed names are ambiguous in isolation (`has_user_data` — which protocol? which field?)
and break the module-prefix convention the file otherwise follows. The four functions are defined
at lines 2052, 2065, 2085, 2102 respectively.

**Classification:** FIXABLE-AUTO — rename to `dnp3_transport_is_fir`, `dnp3_has_user_data`,
`dnp3_is_broadcast_destination`, `dnp3_is_master_frame`; update all call sites in the same file
and in any tests. Requires a mechanical PR.

---

## PF-003 — enip.rs `check_t0814` Naming (MANUAL)

**Description:**
`src/analyzer/enip.rs:447` exports `pub fn check_t0814(...)`. All other public free functions
in the same file use the `enip_` prefix: `parse_enip_header`, `classify_enip_command`,
`is_valid_enip_frame`. The T0814 suffix is the MITRE ATT&CK for ICS threat-tag reference and
is domain-specific, but the missing prefix makes the function opaque when read without module
context.

**Classification:** MANUAL — `check_t0814` may be intentionally named for the threat-tag
reference (paralleling how Modbus uses the same pattern). Evaluate whether `enip_check_t0814`
or `check_enip_t0814` better fits the domain convention before renaming. No auto-rename.

---

## PF-004 — Trait Implementation Gap: EnipAnalyzer, Dnp3Analyzer (ARCH-REVIEW)

**Description:**
The stream-protocol trait hierarchy has a structural split:

| Module | Implements `StreamHandler` | Implements `StreamAnalyzer` |
|--------|--------------------------|----------------------------|
| `HttpAnalyzer` | Yes | Yes |
| `ModbusAnalyzer` | Yes | Yes |
| `TlsAnalyzer` | Yes | Yes |
| `Dnp3Analyzer` | **No** | **No** |
| `EnipAnalyzer` | **No** | **No** |

`Dnp3Analyzer` and `EnipAnalyzer` expose `on_data`, `on_flow_close`, and `summarize` as
bare `impl` methods. The `StreamDispatcher` holds them as concrete `Option<Dnp3Analyzer>`
and `Option<EnipAnalyzer>` fields accessed through typed accessor methods
(`dnp3_analyzer()`, `take_dnp3_analyzer()`, `enip_analyzer()`, `take_enip_analyzer()`),
rather than as `dyn StreamAnalyzer` trait objects.

This is an intentional structural choice recorded in ADR-007 and ADR-010 (the concrete-field
dispatch pattern). It is not a bug. It does mean the two newer analyzers cannot be used
polymorphically alongside the older three, and adding a third concrete analyzer would require
additional dispatcher fields and accessor methods rather than pushing a new item into a
`Vec<Box<dyn StreamAnalyzer>>`.

**Classification:** ARCH-REVIEW — document the decision boundary explicitly. If the
factory ever needs 4+ ICS protocol analyzers with the same dispatch pattern, re-evaluate
whether a unified trait-object approach is cheaper than N concrete fields. No code change
required now; flag for the next ADR revision.

---

## PF-005 — Error Handling Style (INFO — CLEAN)

All seven protocol analyzer modules (`dns`, `arp`, `http`, `modbus`, `tls`, `enip`, `dnp3`) follow
an identical error-handling pattern:

- Trait methods (`analyze`, `on_data`, `on_flow_close`) return `Vec<Finding>` or `()` — no `Result`.
- Protocol parse errors are handled inline: increment an error counter, continue or return empty.
- No `anyhow`, `thiserror`, `bail!`, or `ensure!` in any analyzer code path.
- `unwrap_or(default)` / `unwrap_or_else(...)` with safe defaults in production paths.
- `.expect("invariant message")` is used in test code (inside `#[cfg(test)]` modules) where
  panicking on assertion failure is correct behavior.

This pattern is consistent across ALL modules, including the newest (`enip`, `dnp3`). No drift found.

---

## PF-006 — Clippy (INFO — CLEAN)

`cargo clippy --all-targets -- -D warnings` at `b642c0f`: **0 warnings, 0 errors.**

---

## PF-007 — PG-HELP-PROVENANCE-CLI-DOC-001 (INFO — CLEAN)

Grep for `///` doc-comment lines in `src/cli.rs` containing `BC-`, `VP-`, `SS-`, `ADR-`, `STORY-`:
**0 matches.**

All factory ID references in `src/` are in `//` inline comments (not visible in `--help` output),
not in `///` doc-comments on clap-attributed fields. Specifically:
- `src/cli.rs` — `//` references to BC-2.11.028, BC-2.14.023/024, BC-2.15.010/017/021,
  BC-2.16.011/012/013, BC-2.17.020/023/026 are internal traceability anchors on struct fields,
  not doc-comment text.
- `src/decoder.rs`, `src/dispatcher.rs`, `src/protocols.rs` — `///` doc-comments on non-clap
  types/functions do reference BC-/VP-/ADR- IDs, which is acceptable: they are internal API
  documentation, not help text.

PG-HELP-PROVENANCE-CLI-DOC-001: **no violation found.**

---

## PF-008 — Wave-71 CR-001 (MINOR + 3 NITs) (INFO)

**Finding source:** gate-summary.md Dimension (c): "Code Review | APPROVE | CR-001 MINOR + 3 NITs;
all routed to maintenance/debt; 0 BLOCKING."

**Gate-level code review artifact:** No standalone document was written to
`cycles/wave-71/wave-gate/`. The directory contains only `gate-summary.md` and `demo-evidence/`.
The 1 MINOR finding is not described in any stored file. This is a gap in the factory artifact
protocol (wave-gate code review results should be persisted in a dedicated file).

**3 NITs — verification (all still present):**

The per-story PR reviews for wave-71 contain the following NITs (the gate reviewer's "3 NITs"
appears to correspond to a subset of these four):

| Source file | Location | NIT description | Still present? |
|------------|----------|-----------------|----------------|
| `code-delivery/STORY-156/pr-review.md:127` | `src/analyzer/arp.rs` `mod bc_2_16_016` | AC-004 test reproduces the 10,001-iteration D1 setup from AC-003 sibling; documented standalone justification. Consider shared fixture helper if a third pin lands. | **Yes** — test exists at `src/analyzer/arp.rs`, module `bc_2_16_016`. No action required. |
| `code-delivery/STORY-156/pr-review.md:128` | `docs/demo-evidence/STORY-156/` | Error-path demo requires manual code injection/restore protocol. Acceptable as-is; protocol documented in `evidence-report.md`. | **Yes** — demo evidence committed; accepted as-is. No source change needed. |
| `code-delivery/STORY-157/pr-review.md:70` | `bin/compute-input-hash` | `_INPUTS_INLINE_EMPTY_RE` does not match `inputs: [] # trailing comment`. Not in AC set; unusual in practice. | **Yes** — regex still present in `bin/compute-input-hash`. Not in AC scope; acceptable. |
| `code-delivery/STORY-157/pr-review.md:71` | `bin/compute-input-hash` | `path.find(" #")` treats first ` #` as comment start; a path containing literal ` #` would be mis-truncated. YAML inline-comment convention; no such paths exist. | **Yes** — `path.find(" #")` still present. Acceptable per YAML convention. |

**1 MINOR — verification:**
The MINOR finding has no stored text. It was not captured in any per-story PR review (all three
wave-71 PR reviews concluded with 0 MINOR or BLOCKING findings at the individual-story level).
The MINOR may have originated in the gate-level aggregate code review pass, which was not
written to disk.

**Recommended follow-up:** Amend the factory artifact protocol to require that gate-level code
review output be written to `cycles/wave-NNN/wave-gate/code-review.md` before the gate is
closed (DF-VALIDATION-001-gated before filing as an issue; flag for STORY-158 or next sweep).
