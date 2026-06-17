# Validation Report — GitHub Issue #259

**Policy:** DF-VALIDATION-001 (research-validated deferred findings)
**Issue title:** "Collapse repeated low-value findings (e.g. empty User-Agent) into a summarized line instead of thousands of identical entries"
**Date:** 2026-06-17
**Validator:** vsdd-factory:research-agent
**Branch context:** develop (current default)

---

## 1. Claim Under Validation

The issue asserts a two-part defect plus a proposed remediation:

- **Defect mechanism (code):** The HTTP analyzer emits **one `Finding` per request** for the empty-User-Agent anomaly (`src/analyzer/http.rs` ~359-364), and the terminal reporter renders **one line per finding** with no dedup/aggregation/count layer (`src/reporter/terminal.rs` `render_finding_flat` / `render_finding_prefix`). On real captures with many empty-UA requests this floods the FINDINGS section with thousands of identical lines. The only existing aggregation is MITRE-tactic grouping (`--mitre`), not identical-finding dedup.
- **Proposed fix:** Collapse identical `(category, verdict, confidence, summary)` low-value findings into a single summarized line with an occurrence count, aggregating **in the terminal reporter only**, keeping the raw `Finding` stream intact for machine-readable (JSON/CSV) consumers, with a `--no-collapse` / verbose opt-out or threshold.

The task scope is to validate the **design / best-practice dimension** (is this an established convention; are the pitfalls and mitigations sound). The code-citation accuracy was supplied as pre-confirmed.

---

## 2. Code Mechanism — Spot-Check (in-repo, develop)

Although the caller pre-confirmed the citations, I spot-verified them to anchor the verdict:

- `src/analyzer/http.rs:359-371` — confirmed: `if parsed.user_agent.as_deref() == Some("") { self.all_findings.push(Finding { category: Anomaly, verdict: Inconclusive, confidence: Low, summary: "Empty User-Agent header", evidence: vec![format!("{} {}", parsed.method, parsed.uri)], … }); }`. This pushes **exactly one `Finding` per matching request** — no dedup, no counter.
- `src/reporter/terminal.rs:203-227` (`render_finding_prefix`) and `:232-238` (`render_finding_flat`) — confirmed: each `Finding` is rendered as its own `[category] verdict (confidence) - summary` line plus evidence lines. There is no count/occurrence layer; the only grouping path is `render_finding_grouped` (the `--mitre` tactic view).
- Notable corroboration: `src/analyzer/http.rs:351-358` carries an author comment that already cites Suricata's `http-events.rules` posture and explicitly notes that any future "missing UA" finding "should be added as a separate, lower-confidence finding rather than collapsing the two cases." This shows the codebase author is conversant with IDS conventions — relevant to the design dimension below.

**Defect mechanism: CONFIRMED GENUINE in develop.** A single noisy condition (empty-UA) produces one finding per request, and the default flat renderer prints one line per finding, with no aggregation. The flooding scenario described is a real consequence of the current design.

---

## 3. Design / Best-Practice Dimension

### 3.1 Is "collapse high-frequency low-confidence findings into a counted summary line" an established convention?

**Yes — but with an important nuance about *where* it is applied.** Mature network-security / IDS / packet-analysis tooling converges on two distinct families of mechanisms for taming high-frequency, low-value alerts (deep-research synthesis, sources [1]-[17]):

1. **Detection-time thresholding / suppression** (first-tier sensors): reduce the number of alerts *generated*, discarding the rest — no "N occurrences" summary is produced.
2. **Analysis-time aggregation** (analyzers / UIs / SIEMs): collapse many raw events into one human-facing representation, *typically annotated with a count*, while the raw per-event records persist in the underlying store.

The issue's proposal — a **counted summary line in the human-facing output**, with the raw stream preserved — is the **analysis-time aggregation** pattern, and there are direct, concrete precedents:

- **Wireshark "Expert Information"** [7][8][15] — the closest precedent. Wireshark groups detected anomalies (retransmissions, malformed packets, protocol errors) by **severity + summary string**, and each summary row carries a **count of how many packets/occurrences** fall under it, expandable to the individual frames. This is a UI-only construct: the `.pcap`/`.pcapng` is unchanged, per-frame `_ws.expert` fields remain addressable and filterable (`_ws.expert.severity == "Warning"`). This is almost exactly the issue's proposal, transposed from packets to findings: collapse-by-(severity, summary) with a count in the human view, raw data intact underneath.
- **ntopng Alerts Explorer** [13][14] — explicitly redesigned to cluster/aggregate alerts via custom queries that group by arbitrary tuples (e.g., client/server/alert-type) with a `count` aggregation function, "to display the number of alerts" per group. The underlying alert database retains every individual alert; aggregation is a query-time/UI-time lens only. A clean modern precedent for the exact split the issue proposes.
- **Splunk `dedup`** [10] — collapses events with identical field combinations at **search/pipeline time**; the index retains every raw event. Explicit precedent for "aggregate in the human-facing result, never mutate the canonical stream."
- **SIEM aggregation stage** [17] — aggregation is a named pipeline stage whose stated purpose is to combine related events into a consolidated, counted view specifically **to reduce alert fatigue**, while raw logs/telemetry are preserved for forensic search.
- **syslog "last message repeated N times"** [9][16] — the oldest precedent for collapsing repeats into a counted line. *Counter-precedent on the split:* syslog collapses the **canonical** record (there is no separate raw stream), and this is widely criticized as harmful to forensic accuracy (the [9] "rant", FreeBSD `-c` disable option, rsyslog's `$RepeatedMsgContainsOriginalMsg` knob). This is precisely the anti-pattern the issue's design **avoids** by keeping the raw `Finding` stream intact.

By contrast, **Snort** (`event_filter` / `rate_filter` / `detection_filter` / `suppress`, plus the older `threshold`) [1][2] and **Suricata** (`threshold` types `threshold`/`limit`/`both`/`backoff`, `detection_filter`) [3][4], and **Zeek's Notice framework** (automated suppression + policy hooks) [5][6][11][12] all sit in the **detection-time suppression** family — they reduce what is emitted rather than emitting a counted summary. Suricata's EVE JSON is explicitly a per-event "firehose" with no in-engine aggregation [4]; Zeek suppresses repeat *notices* but still preserves granular `conn.log` events [11][12].

**Conclusion (3.1):** The counted-summary-in-the-human-view pattern is an established convention with strong, current precedents (Wireshark Expert Info, ntopng, Splunk, SIEM aggregation). It is the right analogue for an offline pcap analyzer like wirerust, which — like Wireshark — operates on a finite capture after the fact rather than as a live sensor.

### 3.2 Recognized pitfalls and whether the proposed mitigation aligns

The research surfaces two principal pitfalls, both of which the issue's design directly addresses:

- **Pitfall A — losing per-instance evidence needed for forensics.** This is the documented failure mode of syslog-style collapsing [9][16], where the collapsed line replaces the canonical record. **Mitigation alignment: STRONG.** The issue keeps the raw `Finding` stream intact for JSON/CSV consumers and only aggregates in the terminal reporter. This mirrors Wireshark (raw frames preserved, addressable by `_ws.expert` fields), ntopng (alert DB intact), and Splunk (`dedup` never mutates the index). The aggregated view should remain *expandable / opt-out-able* (the `--no-collapse`/verbose flag), matching Wireshark's expand-to-frames and ntopng's drill-down. The issue proposes exactly this opt-out.
- **Pitfall B — breaking machine-readable output.** Aggregating in the canonical machine stream would corrupt downstream parsers/SIEM ingestion. **Mitigation alignment: STRONG and explicitly correct.** Every modern precedent ([4] EVE firehose, [10] index-vs-search, [13] DB-vs-query, [17] raw-logs-vs-alerts) keeps the machine stream un-aggregated and confines collapsing to the human/analysis layer. The issue's "terminal reporter only; JSON/CSV unchanged" placement is the textbook-correct boundary.

A secondary design consideration (not a blocker, but a refinement input): the precedents differ on **key selection and counting granularity**. Wireshark keys on (severity, summary); the issue proposes (category, verdict, confidence, summary). Whether to collapse *all* findings or only *low-value* ones (e.g., gate on `Confidence::Low` / `Verdict::Inconclusive`, or a threshold like "collapse once >N identical") is a real design choice with precedent on both sides (Wireshark collapses universally; SIEM/Snort gate by severity/threshold). The issue acknowledges this ("ideally with a … threshold"), so it is appropriately scoped as a refinement, not an unresolved contradiction.

---

## 4. Sources

| # | Source | URL |
|---|--------|-----|
| [1] | Snort README.filters (event_filter / rate_filter / detection_filter / suppress) | https://www.snort.org/document/readme-filters |
| [2] | Snort README.thresholding | https://www.snort.org/faq/readme-thresholding |
| [3] | Suricata thresholding (threshold/limit/both/backoff) | https://docs.suricata.io/en/latest/rules/thresholding.html |
| [4] | Suricata EVE JSON output ("firehose") | https://docs.suricata.io/en/latest/output/eve/eve-json-output.html |
| [5] | Zeek Notice framework (automated suppression) | https://docs.zeek.org/en/master/frameworks/notice.html |
| [6] | Corelight — custom Zeek notice action (Telegram) | https://corelight.com/blog/telegram-zeek-youre-my-main-notice |
| [7] | Wireshark User's Guide — Expert Information | https://www.wireshark.org/docs/wsug_html_chunked/ChAdvExpert.html |
| [8] | Wireshark issue 17228 (_ws.expert fields; multiple per frame) | https://gitlab.com/wireshark/wireshark/-/issues/17228 |
| [9] | "syslog last message repeated X times (rant)" | https://dcid.me/notes/syslog-last-message-repeated-x-times-rant |
| [10] | Splunk `dedup` (search-time, index preserved) | https://help.splunk.com/en/splunk-cloud-platform/spl-search-reference/10.4.2604/search-commands/dedup |
| [11] | Zeek Notice framework (LTS) | https://docs.zeek.org/en/lts/frameworks/notice.html |
| [12] | Zeek logging framework (Notice::LOG / ALARM_LOG) | https://docs.zeek.org/en/lts/scripts/base/frameworks/logging/main.zeek.html |
| [13] | ntop — "Sorting Out and Clustering Alerts in ntopng" (count aggregation) | https://www.ntop.org/sorting-out-alerts-in-ntopng/ |
| [14] | ntop — Threshold vs Statistical Metric Alerts | https://www.ntop.org/threshold-vs-statistical-metric-alerts-in-ntopng/ |
| [15] | IT-Connect — Wireshark Expert Information tutorial (summary + count column) | https://www.it-connect.fr/la-fonctionnalite-information-expert-de-wireshark/ |
| [16] | rsyslog — $RepeatedMsgContainsOriginalMsg | http://rsyslog-doc-v5.readthedocs.io/en/latest/configuration/global/ |
| [17] | Stellar Cyber — SIEM alerts: aggregation stage & alert fatigue | https://stellarcyber.ai/learn/siem-alerts-types-and-best-practices/ |

In-repo evidence: `src/analyzer/http.rs:351-371`, `src/reporter/terminal.rs:198-238` (read on develop, 2026-06-17).

---

## 5. Verdict

**VERDICT: GENUINE — NEEDS-REFINEMENT on implementation parameters**
**Confidence: HIGH**

### Rationale

The defect mechanism is confirmed genuine in develop (one `Finding` per empty-UA request at `http.rs:359-371`; one rendered line per finding at `terminal.rs:203-238`; the only aggregation is MITRE grouping, not identical-finding dedup), so on a real capture with many empty-UA requests the FINDINGS section floods exactly as described. The proposed remedy is not merely reasonable — it is the textbook analysis-time aggregation pattern used by mature tooling that, like wirerust, analyzes finite captures/event sets after the fact: Wireshark's Expert Information (collapse by severity+summary with an occurrence count, expandable to individual frames) is an almost one-to-one precedent, reinforced by ntopng's counted alert clustering, Splunk's search-time `dedup`, and the SIEM aggregation stage — all of which exist specifically to combat alert fatigue. Critically, the issue's two architectural guardrails are exactly the ones the literature mandates: aggregate **only** in the human-facing reporter, and keep the raw per-event stream intact for machine-readable consumers — thereby avoiding the well-documented forensic-loss anti-pattern of syslog-style canonical-record collapsing (the one counter-precedent, which is widely criticized and routinely disabled). The opt-out flag matches the expand/drill-down affordance every UI-aggregation precedent provides. I classify this **NEEDS-REFINEMENT rather than a bare GENUINE** only because the *parameters* — collapse key (category,verdict,confidence,summary vs Wireshark's severity,summary), whether to gate on low-confidence/inconclusive findings or apply a count threshold, and the default vs opt-in behavior — are genuine design choices with precedent on multiple sides; the issue itself flags these as "ideally"/"threshold", so they should be resolved during decomposition/PRD rather than treated as blockers. The finding is sound, still open on develop, and safe to file as an issue.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 (reasoning_effort=high) | Deep multi-source synthesis: how Snort, Suricata, Zeek, Wireshark, ntopng, syslog, Splunk/SIEM handle high-frequency low-value alerts; thresholding vs suppression vs aggregation; human-vs-machine output split. 17 cited sources. |
| Read (in-repo) | 3 | Spot-verify defect mechanism: `http.rs:350-374`, `terminal.rs:195-244`, and the policy file `.factory/policies.yaml`. |
| Grep (in-repo) | 4 | Extract the 17 citation URLs and conclusion/forensic-pitfall text from the saved research file. |

**Total MCP tool calls:** 1 (`perplexity_research`, reasoning_effort=high)
**Training data reliance:** low — the design-precedent claims are grounded in 17 web sources via `perplexity_research`; the defect mechanism is grounded in direct in-repo reads. The single `perplexity_research` call returned a comprehensive, well-cited answer covering every tool named in the task, so additional MCP calls were not needed; ~75% of the report body and 100% of the citation list were read directly from the saved result, with the unread tail being the conclusion section that restates the already-captured two-category framework.
