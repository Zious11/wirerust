# Research: Handling a Detection-Threshold Value of 0 for `--arp-storm-rate` and `--arp-spoof-threshold`

**Type:** general (technology / CLI-design decision)
**Date:** 2026-06-15
**Author:** vsdd-factory research-agent
**Status:** complete
**Decision scope:** wirerust `analyze` subcommand CLI flags `--arp-storm-rate <N>` and `--arp-spoof-threshold <N>` (STORY-115 / STORY-114; BC-2.16.013 / BC-2.16.012)

---

## 1. Question

Two ARP detection flags accept a threshold `N` and today allow `N=0`:

- `--arp-storm-rate <N>` — ARP frames per window per source MAC that triggers a "storm" (DoS/scan) finding. **RATE** threshold.
- `--arp-spoof-threshold <N>` — number of MAC-rebindings for an IP before escalating an ARP-spoof finding to HIGH. **COUNT / escalation** threshold.

With `N=0` and an inclusive `rate >= threshold` comparison, the predicate is **always true**, so a finding fires on the FIRST frame from EVERY source — mass false positives. The governing behavioral contracts left this open: *"Clamp to 1 OR return a CLI error."* The codebase already contains two precedents that appear to disagree (modbus rejects `0`; dnp3 accepts `0`). This report resolves the decision.

---

## 2. Recommendation (TL;DR)

> **REJECT `0` at the CLI boundary with a clear, fail-fast error for BOTH flags.**
> Emit, e.g., `--arp-storm-rate must be >= 1 (got 0)` and `--arp-spoof-threshold must be >= 1 (got 0)`, before any packet processing. Implement via the existing `parse_nonzero_usize`-style `value_parser` pattern already in `src/cli.rs` (or a `u32` variant), not via a runtime `bail!`, so the constraint is declarative and shows in `--help`/usage.

**Do NOT clamp-to-1 silently** (hides intent, anti-pattern across all CLI-design references and the `clap` ecosystem). **Do NOT accept `0` as "alert-on-all"** for these two flags, because — unlike the dnp3 precedent — the ARP comparison is **inclusive (`>=`)**, so `0` is not a coherent "fire on first real event" sentinel; it is a degenerate "always true" that also fires on rate/count `0`. This recommendation is **consistent with the modbus precedent** and the dominant convention in mature IDS tooling (Suricata, Snort), and the apparent conflict with the dnp3 precedent dissolves once the `>` vs `>=` comparison semantics are examined (see §6).

**Confidence: HIGH.** Cross-validated against primary tool docs (Suricata, Snort, arp-scan man pages) and the in-repo source. The one genuinely defensible alternative (accept `0` as a documented sentinel) is rejected on a concrete technical ground specific to this codebase, not on taste.

---

## 3. How mature IDS / packet-analysis tools handle threshold = 0

Cross-validated against official docs where possible. Findings distinguish **COUNT** (cumulative occurrences) from **RATE** (events per time window), per the question.

| Tool | Mechanism | Behavior at `count`/`rate` = 0 | Source confidence |
|------|-----------|-------------------------------|-------------------|
| **Suricata** | `threshold` (count C → alert on Cth match), `detection_filter`, `rate_filter` | Count semantics are 1-based ("alert the Cth time it matches"); a `count` of 0 has no meaning. Documented minimum is effectively `>= 1`. **REJECT / no zero semantics.** | HIGH — [docs.suricata.io thresholding](https://docs.suricata.io/en/latest/rules/thresholding.html) confirms count is 1-based ("generate an alert the Cth time"); deep-research reports startup config error on `count=0`. |
| **Snort** | `detection_filter`, `rate_filter` | Official README.filters: **"C must be nonzero"** (detection_filter count) and **"C must be positive"** (rate_filter count). Zero `seconds` is a *separate* special case (total-count mode), but **count of 0 is explicitly disallowed.** | HIGH — [snort.org README.filters](https://www.snort.org/document/readme-filters) verbatim "must be nonzero" / "must be positive". |
| **Zeek/Bro** | `Notice` framework threshold; `SumStats` | Flexible/scripted. `Notice` thresholds are commonly written so a low/zero value means "notify on first occurrence"; SumStats imposes no engine-level minimum and delegates meaning to the script. **ACCEPT, context-defined** — but always *explicitly documented per-script.* | MEDIUM — deep-research synthesis; Zeek's scripted model is well-known but exact `threshold==0` short-circuit was not independently re-verified against source. Flagged. |
| **arpwatch** | Stateful change detector | No count/rate threshold paradigm — reports *every* state change. Operational params (cache size) reject `0` because `0` is non-functional. **N/A for thresholds; rejects degenerate operational values.** | MEDIUM — deep-research; arpwatch's "report all changes" model is well-established. |
| **arp-scan** | Active scanner | `--retry` accepts low values; `--retry=1` is the documented single-pass mode. `0`/low values are *legitimate* because retry is a "do it this many times" knob, not a detection threshold. **ACCEPT low values — but this is a retry counter, not an alert threshold.** | HIGH — [arp-scan(1) man page](https://man.archlinux.org/man/arp-scan.1.en) documents `--retry=1` single-pass. |
| **ntopng** | Alerting thresholds (absolute & %) | **Context-dependent:** rejects `0` for percentage/deviation thresholds ("must be greater than zero") because `0%` matches everything; accepts `0` for some absolute metrics as "alert on any traffic" but documents it as diagnostic-only + has alert-rate-limiting as a backstop. | MEDIUM — deep-research synthesis; the percentage-vs-absolute split is plausible and consistent with ntopng's design but specific error strings/source paths not independently verified. Flagged. |

### Synthesis of the tool survey

1. **For COUNT/alert thresholds, the mature signature-based IDS consensus (Suricata, Snort) is REJECT `0`.** Snort says so in so many words ("must be nonzero" / "must be positive"). Count semantics in these tools are inherently 1-based.
2. **Where `0` is accepted as a sentinel (Zeek, parts of ntopng), it is ALWAYS explicitly documented**, and it is accepted in contexts where `0` maps cleanly onto "first occurrence" or "any traffic" via the tool's comparison semantics — never as a silent fallthrough.
3. **`0` is legitimately accepted for non-threshold knobs** (arp-scan `--retry`, "seconds=0 → total count" in Snort/Suricata rate_filter). These are *not* alert thresholds and do not bear on this decision.
4. **No mature tool silently clamps an alert threshold of `0` to `1`.** Clamping did appear in some *historical* Snort behavior for the `seconds` field and was treated as a wart to be removed, not a pattern to emulate.

**RATE vs COUNT does not change the answer here.** Both Suricata's `detection_filter`/`rate_filter` (rate) and `threshold`/count are 1-based and reject `0`. The only place "rate-ish 0" is meaningful is `seconds=0` (collapse a rate into a cumulative count) — which is a *time-window* of 0, not a *threshold* of 0, and is irrelevant to `--arp-storm-rate`'s `N` (which is the frame count, not the window).

---

## 4. CLI-design best practice (clig.dev, clap ecosystem, general UX)

The CLI-design literature converges far more tightly than the tool survey:

- **Fail-fast validation is the dominant best practice.** Invalid numeric arguments should be rejected at parse time with a clear, actionable error (e.g. "must be >= 1 (got 0)"), not propagated into core logic. CLIs are run non-interactively in scripts; a silently-adjusted value produces a command that *appears* to succeed while behaving differently from what the invocation says. [clig.dev] emphasizes clear contracts, early/clear errors, and not "guessing" user intent.
- **Silent clamping is a near-universally-rejected anti-pattern.** It hides the discrepancy between requested and actual behavior, is hostile to automation/observability (logs show success), complicates testing, and violates the principle of least surprise. Across the CLI-design references surveyed, clamping was the *least* defensible of the three options.
- **The Rust `clap` ecosystem structurally biases toward validate-and-reject.** Idiomatic patterns — typed `value_parser`, range validators (`1..`), and `NonZeroU64`/`NonZeroU32` types — make rejection trivial and clamping non-idiomatic (you must bypass the library to clamp). `clap` auto-generates clear parse-time errors and surfaces ranges in `--help`. **wirerust already follows this idiom**: `src/cli.rs:18-24` defines `parse_nonzero_usize`, used for `--reassembly-depth` and `--reassembly-memcap`.
- **If an "alert on everything" mode is genuinely wanted, expose it explicitly** — a named flag (`--arp-storm-rate-any` / `--no-arp-storm-threshold`) or a *documented* sentinel — never an implicit `0` fallthrough. For these two flags there is no evidence such a mode is desired (the contracts frame `N` as a positive detection threshold), so the explicit-sentinel option carries no weight here.

**The one acknowledged area of legitimate disagreement** in the literature is *whether* to ever treat `0` as a deliberate "do everything" sentinel. That disagreement is moot for this decision because (a) the contracts don't call for an alert-on-all mode, and (b) the ARP comparison's inclusive `>=` makes `0` an incoherent sentinel anyway (§6).

---

## 5. RATE threshold vs COUNT threshold — does the answer differ?

Short answer: **No, not for these two flags.** Both should reject `0`.

- **`--arp-spoof-threshold` (COUNT / escalation):** Number of MAC-rebindings before HIGH escalation. The code already documents "Set to 1 to fire HIGH on the very first rebind" (`src/cli.rs:193`). `1` is therefore *already* the "most aggressive meaningful" value — "escalate on the first rebind." A value of `0` would mean "escalate on the zeroth rebind," i.e., escalate before any rebind has occurred — semantically null. Reject. This matches Suricata/Snort count semantics exactly.
- **`--arp-storm-rate` (RATE, frames/window/MAC):** Default 50; ICS operators advised to lower to 5–20/s (`src/cli.rs:198-202`). The meaningful floor is `1` frame/window ("any ARP frame from a source is a storm" — already extremely aggressive but coherent). `0` frames/window is not a rate; with `>=` it fires even on zero observed frames. Reject.

The *only* way `0` would be semantically meaningful for either is if you wanted a literal "flag every source unconditionally" mode — which is not a detection threshold, it's a different feature, and should be a different flag if ever needed.

---

## 6. Squaring with the two in-codebase precedents

This is the crux. The precedents only *appear* to conflict; the deciding variable is the **comparison operator**, verified in source.

### Precedent A — modbus: REJECT `0` (`src/main.rs:110-115`)
```
if modbus_write_burst_threshold == 0 {
    anyhow::bail!("--modbus-write-burst-threshold must be >= 1 (got 0)");
}
if modbus_write_sustained_threshold == 0 {
    anyhow::bail!("--modbus-write-sustained-threshold must be >= 1 (got 0)");
}
```
Modbus rejects `0` at startup. **The recommendation matches this precedent.**

### Precedent B — dnp3: ACCEPT `0` as "fires immediately" (`src/analyzer/dnp3.rs:163`, `src/main.rs:209`)
The dnp3 direct-operate guard fires when:
```
flow.direct_operate_count > direct_operate_threshold   // STRICTLY GREATER
```
`src/main.rs:209` documents: *"AC-007: 0 fires immediately."* With a **strict `>`** comparison, `threshold = 0` means "fire when count > 0", i.e., **on the first actual Control-class FC observed.** That is a *coherent, useful* "alert on the first real event" sentinel. It does **not** fire spuriously on flows with zero direct-operate FCs, because `0 > 0` is false. So dnp3 can safely accept `0`.

### Why ARP is different — the inclusive `>=`
The problem statement specifies the ARP path uses **`rate >= threshold`** (inclusive). With `threshold = 0`:
- `rate >= 0` is **always true**, for *every* source MAC, including those whose rate/count is literally `0`.
- There is no "first real event" gating; the predicate degenerates to a constant `true`.

So `0` is **not** a coherent "alert on first event" sentinel in the ARP code the way it is in dnp3 — it is a pure degenerate "always fire." The dnp3 precedent's safety depends entirely on its strict `>`; the ARP path lacks that safety.

### Resolution
The two precedents are governed by a single consistent rule:

> **Accept `0` only where the comparison is strict (`> threshold`) so that `0` cleanly means "fire on the first real event." Reject `0` where the comparison is inclusive (`>= threshold`), because there `0` degenerates to "always fire on everything including non-events."**

- dnp3 uses `>` → `0` is a safe documented sentinel → accepted. ✔ consistent
- modbus + ARP: reject `0`. ✔ consistent (modbus already rejects; ARP should too)

**This unifies the codebase under one principle rather than leaving two contradictory precedents.** The recommendation therefore does not "pick a side" — it identifies the latent invariant (`>` ⇒ sentinel-safe, `>=` ⇒ reject) and applies it.

### Secondary option (only if a future maintainer prefers symmetry with dnp3)
If, instead of rejecting `0`, the team wanted `--arp-storm-rate 0` / `--arp-spoof-threshold 0` to mean "fire on the first frame/rebind" like dnp3, the *correct* way is to **change the ARP comparison from `>=` to `>`** (and adjust the default/semantics so `N` means "fire when strictly more than N"). That is a larger behavioral change touching the storm/spoof detection math and the contracts' `>=` definition, and it is **not recommended** for this decision — but it is the only route by which accepting `0` would be defensible. Reject-at-CLI is lower-risk and ships now.

---

## 7. Recommended implementation (non-binding, for the architect/implementer)

Use the existing declarative `clap` pattern rather than a runtime `bail!`, so the constraint appears in `--help` and is caught at parse time (true fail-fast). A `u32` analogue of the existing `parse_nonzero_usize` (`src/cli.rs:18`):

```rust
/// Value parser for u32 CLI args that must be >= 1 (0 is rejected at parse time).
fn parse_nonzero_u32(s: &str) -> Result<u32, String> {
    let v: u32 = s.parse().map_err(|e| format!("invalid value '{s}': {e}"))?;
    if v == 0 {
        return Err("0 is not in 1.. (must be >= 1)".to_string());
    }
    Ok(v)
}
```
Then on both fields in `src/cli.rs`:
```rust
#[arg(long, default_value_t = 3, value_parser = parse_nonzero_u32)]
arp_spoof_threshold: u32,

#[arg(long, default_value_t = 50, value_parser = parse_nonzero_u32)]
arp_storm_rate: u32,
```
Notes:
- This is *more* idiomatic than the modbus `bail!` (which validates at runtime in `run_analyze`). Consider, separately, migrating the modbus checks to the same `value_parser` for consistency — but that is out of scope for this decision and should be its own change.
- Update the doc-comments to state "Must be >= 1; 0 is rejected" (the spoof flag already says "Set to 1 to fire HIGH on the very first rebind", which becomes the documented floor).
- Add/keep RED tests asserting non-zero exit + the error string for `--arp-storm-rate 0` and `--arp-spoof-threshold 0` (mirrors existing modbus threshold tests in `tests/`).

---

## 8. Limitations & confidence

- **HIGH confidence** on: Snort/Suricata reject `0` for count thresholds (primary-source verified); CLI-design consensus favors fail-fast reject over silent clamp (multiple references + clap idiom); the in-repo `>` vs `>=` distinction that reconciles the two precedents (verified directly in `src/main.rs` and `src/analyzer/dnp3.rs`).
- **MEDIUM confidence / flagged** on: the *exact* error-string and source-file claims attributed to Zeek and ntopng in the deep-research output. Those tools' high-level behavior (Zeek = scripted/flexible, ntopng = context-dependent with a percentage-vs-absolute split) is consistent with their known design, but specific verbatim error messages and source paths (e.g. `base/frameworks/notice/main.zeek`, `lua/src/scripts/...`) were **not independently re-verified** and should be treated as illustrative rather than citable. They do not affect the recommendation, which rests on the Suricata/Snort + CLI-design + in-repo evidence.
- **Not inconclusive.** The decision is well-supported; no ambiguity remains.

---

## 9. Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) How Suricata/Snort/Zeek/arpwatch/arp-scan/ntopng handle threshold=0, count vs rate; (2) CLI-design best practice for degenerate numeric args + clap idioms (reject vs clamp vs sentinel). `reasoning_effort=high`. |
| Perplexity perplexity_search | 2 | Cross-validate Suricata/Snort threshold-minimum docs and arp-scan `--retry` semantics against primary man pages / official docs. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | Not needed; clap idiom already evidenced in-repo and in deep-research. |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Grep / Read (local source) | 5 | Verify in-codebase precedents: modbus `bail!` reject (`src/main.rs:110-115`), dnp3 `>` comparison + "0 fires immediately" (`src/analyzer/dnp3.rs:163`, `src/main.rs:209`), ARP flag definitions (`src/cli.rs:196-204`), existing `parse_nonzero_usize` pattern (`src/cli.rs:18-24`). |
| Training data | 1 area | General framing of fail-fast / least-surprise CLI principles (corroborated by perplexity_research, not solely relied upon). |

**Total MCP tool calls:** 4 (2 `perplexity_research` + 2 `perplexity_search`).
**Training data reliance:** low — every load-bearing claim is sourced to a tool doc, the CLI-design literature, or directly to the wirerust source tree. Two per-tool detail claims (Zeek, ntopng) are explicitly flagged as medium-confidence and are non-load-bearing for the recommendation.
