---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-06-18T00:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-11
capability: CAP-11
lifecycle_status: active
introduced: v0.9.0
modified:
  - "v1.1 2026-06-18: R2-2 — correct introduced version: v0.10.0 → v0.9.0 (canonical per ADR-0003 §Semver, design-note §7, BC-INDEX:273 'BCs 030-034: grouped-collapse (greenfield, STORY-119, v0.9.0)'; D-110 bundles into unreleased 0.9.0)."
  - "v1.2 2026-06-18: F2 adversarial round-3 fix (NIT) — correct BC-INDEX line reference in v1.1 stanza: ':269' (CsvReporter BCs 020-024 note) → ':271' (grouped-collapse v0.9.0 line)."
  - "v1.3 2026-06-18: F2 adversarial round-4 fix — correct BC-INDEX line citation in v1.1 and v1.2 stanzas: verified live BC-INDEX shows grouped-collapse v0.9.0 entry at line 273 (not 271); v1.2 ':271' was itself wrong. v1.1 stanza now cites ':273' verbatim with entry content; v1.2 stanza retained as audit trail of the round-3 intermediate error."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.030: `--mitre` Default Maps to {Grouped, Collapsed}; `--mitre --no-collapse` Maps to {Grouped, Expanded} — CLI-to-Render Mode Mapping Contract

## Description

STORY-119 makes collapse default-on symmetrically across both flat and grouped rendering
modes. When `--mitre` is passed without `--no-collapse`, the resulting `FindingsRender` is
`{ grouping: Grouping::Grouped, collapse: Collapse::Collapsed }` — i.e., grouped-collapse is
the new default for MITRE-grouped output. When `--mitre` AND `--no-collapse` are both passed,
the resulting `FindingsRender` is `{ grouping: Grouping::Grouped, collapse: Collapse::Expanded
}` — suffix-free, name-expanded MITRE lines, one line per finding. This is the pre-STORY-119
`--mitre` behavior; it is now preserved only via the explicit `--no-collapse` opt-out.

This BC documents the CLI-to-`FindingsRender`-struct wiring at the `TerminalReporter`
construction site in `src/main.rs::run_analyze`. It is the grouped-mode analogue of the
flat-mode wiring already in BC-2.11.028, broadened to cover both axes of the two-dimensional
`FindingsRender` struct-of-enums (D-110, STORY-119 F1 gate).

## Preconditions

1. STORY-119 F4 implementation is complete: `FindingsRender` is a struct with orthogonal
   `grouping: Grouping` and `collapse: Collapse` fields (D-110 approved type model).
2. The `TerminalReporter` construction site in `src/main.rs::run_analyze` uses the new
   struct literal form (see Architecture Anchors).
3. `show_mitre_grouping: bool` and `collapse_findings: bool` are the in-scope bool params
   inside `run_analyze`; they are derived at the `main()` call site from `*mitre` and
   `collapse_findings_from_flag(*no_collapse)` respectively (unchanged resolution logic).
4. The user invokes `wirerust analyze <pcap>` with some combination of `--mitre` and/or
   `--no-collapse` flags.

## Postconditions

1. When `--mitre` is absent (regardless of `--no-collapse`): `render.grouping == Grouping::Flat`.
   The Grouping axis is fully determined by `--mitre`.
2. When `--mitre` is present and `--no-collapse` is absent (the new default):
   `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`.
   The output is organized into tactic-bucket headers; within each bucket, findings are
   collapsed by `(category, verdict, confidence, summary)` key per BC-2.11.031; per-bucket
   `(xN)` suffix rule applies.
3. When `--mitre` is present and `--no-collapse` is also present:
   `render == FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }`.
   The output is organized into tactic-bucket headers; each finding is rendered individually
   (one display line per finding); no `(xN)` suffix appears anywhere.
4. When neither `--mitre` nor `--no-collapse` is present (the default terminal output):
   `render == FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }`.
   Unchanged from pre-STORY-119 behavior.
5. When `--no-collapse` is present but `--mitre` is absent:
   `render == FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }`.
   Unchanged from pre-STORY-119 behavior.
6. The `run_summary` construction site is unaffected: it always produces
   `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }` (inert value;
   `run_summary` renders no FINDINGS section).

## Invariants

1. The Grouping axis is determined exclusively by `show_mitre_grouping` (derived from `--mitre`).
   The Collapse axis is determined exclusively by `collapse_findings` (derived from `!no_collapse`).
   The two axes are fully orthogonal; no combination is illegal under the D-110 struct model.
2. The `collapse_findings_from_flag` function remains `fn collapse_findings_from_flag(no_collapse: bool) -> bool { !no_collapse }` — unchanged from STORY-118.
3. The `run_analyze` function signature is unchanged by STORY-119: it still receives
   `show_mitre_grouping: bool` and `collapse_findings: bool` as separate params.
4. `--no-collapse` acts as a dual-scope opt-out: it suppresses collapse in flat mode
   (producing `{Flat, Expanded}`) AND in grouped mode (producing `{Grouped, Expanded}`).
   There is no flag that independently controls the two axes; `--no-collapse` always affects
   the Collapse axis universally.
5. No `Default` trait is derived on `FindingsRender`, `Grouping`, or `Collapse` — consistent
   with the deliberate omission in STORY-120 (ADR-0003). All construction sites select both
   axes explicitly.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--mitre` alone (no `--no-collapse`) | `render = {Grouped, Collapsed}` — new default since STORY-119 |
| EC-002 | `--mitre --no-collapse` | `render = {Grouped, Expanded}` — suffix-free; pre-STORY-119 `--mitre` behavior preserved via opt-out |
| EC-003 | No flags (default) | `render = {Flat, Collapsed}` — unchanged from v0.8.0/v0.9.0 |
| EC-004 | `--no-collapse` without `--mitre` | `render = {Flat, Expanded}` — unchanged from v0.8.0/v0.9.0 |
| EC-005 | `run_summary` construction site | Always produces `{Flat, Collapsed}`; inert (no FINDINGS section rendered by summary) |
| EC-006 | Both `--mitre` and `--no-collapse` on `run_summary` | `run_summary` ignores these flags; the `render` field is hardcoded to `{Flat, Collapsed}` at the summary construction site; the field is structurally present but irrelevant |
| EC-007 | Existing test helpers using `FindingsRender::Grouped` (pre-STORY-119 enum) | After F4 migration, all such sites are updated to `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` per the D-110 migration map — preserving suffix-free semantics |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| CLI: `wirerust analyze cap.pcap --mitre` (no `--no-collapse`), repeat-key findings in tactic bucket | `render = {Grouped, Collapsed}`; output contains tactic bucket headers and `(xN)` suffix for N≥2 groups within buckets | happy-path (new grouped-collapse default) |
| CLI: `wirerust analyze cap.pcap --mitre --no-collapse`, repeat-key findings in tactic bucket | `render = {Grouped, Expanded}`; output contains tactic bucket headers; no `(xN)` suffix on any line; one finding per line | happy-path (grouped-expanded opt-out) |
| CLI: `wirerust analyze cap.pcap` (neither flag), repeat-key findings | `render = {Flat, Collapsed}`; flat FINDINGS section with `(xN)` suffixes for collapsed groups | happy-path (flat-collapsed default, unchanged) |
| CLI: `wirerust analyze cap.pcap --no-collapse` (no `--mitre`), repeat-key findings | `render = {Flat, Expanded}`; one line per finding, no suffixes | happy-path (flat-expanded opt-out, unchanged) |
| `TerminalReporter { ..., render: FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed } }` constructed directly in test | Dispatches to `render_findings_grouped_collapsed`; grouped-collapse behavior exercised | unit test construction |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | `--mitre` alone routes to `{Grouped, Collapsed}` | unit: test_BC_2_11_030_mitre_alone_maps_to_grouped_collapsed |
| — | `--mitre --no-collapse` routes to `{Grouped, Expanded}` | unit: test_BC_2_11_030_mitre_no_collapse_maps_to_grouped_expanded |
| — | flat-mode routing unchanged (EC-003, EC-004) | unit: test_BC_2_11_030_flat_routing_unchanged |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per domain/capabilities/cap-11-reporting-output.md — this BC defines the CLI-to-render-mode mapping contract that determines which rendering path the terminal output capability uses for grouped mode; the flag wiring is the public interface of the Reporting capability for `--mitre` users |
| L2 Domain Invariants | INV-4 (Raw-Data/Display-Layer Separation — CLI flag wiring affects only terminal rendering; JSON/CSV reporters receive the unmodified findings slice regardless of `--mitre` or `--no-collapse` values) |
| Architecture Module | SS-11 (src/main.rs::run_analyze construction site + src/reporter/terminal.rs dispatch) |
| Stories | STORY-119 |
| Issue | #259 (Collapse repeated low-value findings — grouped-mode extension) |
| ADR | ADR-0003 (Binding Rule 5 revised, STORY-119; grouped-mode collapse subsection) |

## Related BCs

- BC-2.11.028 — supersedes (for grouped-mode wiring; BC-2.11.028 documents flat-mode wiring; this BC adds the grouped-mode dimension introduced by STORY-119)
- BC-2.11.031 — composes with (per-bucket count suffix rule that applies when `{Grouped, Collapsed}`)
- BC-2.11.032 — composes with (per-bucket evidence sampling that applies when `{Grouped, Collapsed}`)
- BC-2.11.033 — composes with (tactic-bucket ordering invariant under grouped-collapse)
- BC-2.11.034 — composes with (MITRE line format for collapsed groups within buckets)
- BC-2.11.013 — depends on (tactic-bucket structure and ordering; grouped-collapse preserves it)

## Architecture Anchors

- `src/main.rs::run_analyze` — **F4-pending insertion target:** `render: FindingsRender { grouping: if show_mitre_grouping { Grouping::Grouped } else { Grouping::Flat }, collapse: if collapse_findings { Collapse::Collapsed } else { Collapse::Expanded } }` at the `TerminalReporter` construction site; replaces the v0.9.0 three-arm if-expression (F-pending: not yet implemented)
- `src/reporter/terminal.rs:~202-224` (v0.9.0 `match self.render` three-arm dispatch) — **F4-pending replacement:** becomes a `match (self.render.grouping, self.render.collapse)` four-arm tuple dispatch; new arm `(Grouping::Grouped, Collapse::Collapsed)` routes to `render_findings_grouped_collapsed` (F-pending: function does not yet exist)
- `src/reporter/terminal.rs:100-111` — current `FindingsRender` enum (v0.9.0 three-variant enum); **F4-pending replacement** with `Grouping` enum + `Collapse` enum + `FindingsRender` struct per D-110

## Story Anchor

STORY-119

## VP Anchors

- — (VPs to be authored by test-writer; see Verification Properties above)

---

### Greenfield Sections

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (CLI parsing is I/O; this BC governs the downstream wiring effect) |
| **Global state access** | none |
| **Deterministic** | yes — flag values fully determine render mode |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure (TerminalReporter is pure; CLI parsing is effectful upstream) |
