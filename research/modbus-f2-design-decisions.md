# Modbus TCP Analyzer (Feature #7) — F2 Spec-Gate Design Decisions

**Type:** general (technology / detection-engineering)
**Date:** 2026-06-09
**Status:** complete
**Purpose:** Evidence-backed recommendations for three open Modbus analyzer design decisions, so a human can make an informed F2 spec-gate call.
**Related prior research:** `.factory/research/modbus-tcp-research.md` (domain research, feature #7).

> **Confidence legend:** [VERIFIED] = stated in an authoritative source cited below. [INFERRED] = reasoned synthesis across sources, not a direct quote. [JUDGMENT] = genuine design judgment call where evidence under-determines the answer.

---

## DECISION 1 — Write-burst threshold: single 1s-window vs dual-window

**Question:** Is a single configurable `--modbus-write-threshold` (max write-FCs per 1-second window per target, default 10) adequate for detecting anomalous Modbus write rates (MITRE T0806 Brute Force I/O, T0855 Unauthorized Command Message), or is a dual criterion (sustained-rate + burst) needed?

### What real tools and the literature actually do

| Tool / source | Model used | Window(s) | Notes |
|---|---|---|---|
| Digital Bond Quickdraw (Snort/Suricata) | Fixed-window counter threshold | ~1–5 s, threshold ~5–20 writes/window [INFERRED from community reports] | Fixed-window counting only; **documented blind spot to "low-and-slow"** writes that stay just under threshold [community discussion, Snort/Suricata ICS rules]. |
| Suricata `modbus` keyword + `threshold`/`detection_filter` | Fixed-window count-in-window | Configurable | Counter-based, not sliding/statistical. |
| Zeek + ICSNPP-Modbus (CISA) | Rich logging → external analytics (Z-score / MAD / STL) | Configurable rolling windows | No native rate-limit algorithm; rolling statistical baselining done downstream (e.g. TimescaleDB `PARTITION BY device_id`). |
| Academic timing-based SCADA IDS (DiVA 2019) | Statistical window + sequential threshold | **W=5 s, N=4 consecutive samples** | >99% detection / <1.4% FPR. Demonstrates *longer windows + sequential/sustained criterion* outperform single fixed-count. |
| Dragos | Dual-window: short burst + long sustained baseline | ~1–3 s burst **and** 15–30 min sustained | Explicitly combines short burst detection with long sustained-rate baselining; adaptive thresholds + process-state awareness. |
| Claroty CTD | Fixed thresholds (critical points) + statistical baselining (dynamic points) | Mixed | Per-state ("partitioned") baselines. |
| Nozomi Networks | ML statistical baseline | 2–4 wk learning | Device-specific baselines. |
| Cisco Cyber Vision | Dual: short burst (1–5 s) + long trend (24–48 h) | Dual | Mirrors Dragos's dual-window pattern. |

**Convergent finding [VERIFIED across multiple sources]:** Mature commercial ICS platforms (Dragos, Cisco Cyber Vision) and the academic literature use a **dual-horizon model — short burst window AND a longer sustained-rate criterion** — precisely because a single fixed window misses one of the two attack shapes. The open-source rulesets (Quickdraw/Suricata) that *do* use a single fixed window are **documented as having a low-and-slow blind spot.** This directly confirms the original research's dual-criterion instinct.

### Realistic baseline write rates (is default=10/s defensible?)

[VERIFIED / INFERRED from traffic studies and vendor docs]:
- Typical Modbus poll cadence default ~2000 ms (Wachendorff); operators often poll faster (e.g. 10×/s) when feasible.
- **Writes are the minority of traffic** — sources estimate reads are 70–90% of Modbus transactions, writes 10–30%.
- Steady-state **write** rates per device: **~0.1–5 writes/sec**, most continuous processes **< 1/s**; legitimate bursts during startup/shutdown/changeover **5–15/s for 10–60 s**.

**Implication:** A flat default of **10 writes/sec is at the *high end* of legitimate burst activity** — it will pass most steady-state traffic, but it sits *inside* the legitimate-transition band (5–15/s). It is defensible as a conservative **burst** ceiling (low false-positive risk) but it is **not** a sensitive sustained-rate detector. The "8 writes/sec for 30s" scenario the question raises is a **real blind spot** under a single 10/s window: every 1-s bucket stays under 10, so nothing fires, yet 8/s sustained for 30s is ~240 writes well outside normal steady-state.

### RECOMMENDATION — Decision 1

**Adopt a dual-window model. [JUDGMENT, evidence-backed]** A single 1-s window is the one model the literature explicitly flags as having a low-and-slow gap, and every mature commercial tool surveyed uses two horizons.

Concrete, defensible, minimal design (avoids the heavyweight statistical/ML baselining that the commercial tools layer on, which is out of scope for a stateless passive analyzer):

- **Burst criterion (keep current):** `--modbus-write-burst` = max write-FCs in any 1 s window per target. **Default 20.** (Catches flooding/brute-force; 20/s is above the legitimate transition band's typical peak, minimizing FP.)
- **Sustained criterion (add):** `--modbus-write-sustained` = max *average* write-FCs/sec maintained over a `--modbus-write-sustained-window` of **≥ N seconds**. **Default 10 writes/sec over 2 s** (i.e. >20 writes accumulated across a 2 s sliding window), extensible to longer N. This is exactly the original research's ">10/s sustained over ≥2s OR >20 in any 1s" dual criterion, now expressible because it is two params, not one u32.

This maps cleanly: burst → T0806 Brute Force I/O (rapid repetitive I/O changes); sustained → the "manipulating a single value an excessive number of times" language MITRE uses for T0806 detection.

**If the F2 gate wants to minimize scope:** the single-window option is *acceptable but should raise the default to ~20/s* (a pure burst/flood detector) and the spec MUST explicitly document the low-and-slow blind spot as an accepted limitation. Shipping a single window at default 10 is the worst of both — too low to be a clean burst ceiling, too coarse to catch sustained. **Do not ship single-window @ 10.**

**Defensible defaults summary:** burst **20/s**; sustained **10/s over 2 s** (configurable window).

---

## DECISION 2 — MITRE ATT&CK for ICS mapping of Modbus recon FCs

**Question:** Current spec maps Report Server ID (0x11), Read Device Identification (0x2B/MEI 0x0E), and arguably Read Exception Status (0x07) to **T0846 Remote System Discovery**. Is that correct?

### Finding [VERIFIED against attack.mitre.org]

The MITRE ATT&CK for ICS **Discovery tactic (TA0102)** has five techniques:
- **T0840** Network Connection Enumeration
- **T0842** Network Sniffing
- **T0846** Remote System Discovery
- **T0887** Wireless Sniffing
- **T0888** Remote System Information Discovery

The critical distinction:
- **T0846 Remote System Discovery** = discovering *that systems exist* — "get a listing of other systems by IP address, hostname, or other logical identifier." MITRE's detection examples are network-scan tools (`ping.exe`, `tracert.exe`). This answers *"what systems are on the network?"*
- **T0888 Remote System Information Discovery** = "get **detailed information about remote systems and their peripherals, such as make/model, role, and configuration**" used "to aid in targeting and shaping follow-on behaviors." MITRE's detection guidance explicitly says to monitor **"for anomalies related to discovery related ICS functions … or for functions being sent to many outstations."** This answers *"what is this device?"*

Modbus **0x11 (Report Server ID)** returns device ID + run status; **0x2B/MEI 0x0E (Read Device Identification)** returns **vendor name, product code, firmware/version** — i.e. make/model/configuration. That is **textbook T0888**, not T0846.

**Corroboration [VERIFIED]:**
- Broadcom IPS ships a signature "TCP MODBUS – Read Device Identification" describing 0x2B as returning vendor/product/version — the exact T0888 information class.
- ICS reconnaissance literature classifies 0x11/0x17/0x2B as **"device identification attacks" / fingerprinting**, a distinct phase *after* address scanning. Address-scanning is the T0846-like phase; the FC-level identification is the T0888 phase.
- Digital Bond Quickdraw carries Modbus recon signatures; the community treats 0x11/0x2B as device-identification/fingerprinting indicators.

### On 0x07 Read Exception Status

[VERIFIED]: 0x07 returns a **single status byte** — no make/model/firmware. The reconnaissance literature does **not** list it among the primary device-ID function codes (0x11, 0x17, 0x2B). Its standalone reconnaissance value is low.

### RECOMMENDATION — Decision 2

**Switch the primary mapping from T0846 to T0888 Remote System Information Discovery (tactic TA0102 Discovery). [VERIFIED]** This is a correctness fix, not a judgment call — the current T0846 mapping is a documented common misattribution.

- **0x2B / MEI 0x0E (Read Device Identification)** → **T0888** (highest-fidelity recon indicator; vendor/product/firmware).
- **0x11 (Report Server ID)** → **T0888** (device-ID + status).
- **0x07 (Read Exception Status)** → **do NOT map as a standalone T0888/discovery indicator.** [JUDGMENT, evidence-backed] Too low-signal — a single status byte. If you want coverage, treat it only as a *low-weight corroborating signal* inside a multi-FC scan sequence (e.g. many distinct FCs probed against one target), never as a standalone finding. Recommended: **exclude it from the recon finding set in F2**; revisit only if sequence-aware scan detection is added.

**Optional nuance [JUDGMENT]:** If you also detect the *enumeration pattern* (one source sweeping many slave addresses / many FCs), that pattern is closer to T0846 (system listing) — so the analyzer could legitimately emit **T0846 for the address-sweep behavior** and **T0888 for the per-device identification FCs**. But for the per-FC mapping that this decision is about, the answer is unambiguously **T0888**.

**Exact IDs to put in the spec:**
- `T0888` — Remote System Information Discovery — Tactic `TA0102` Discovery (for 0x11, 0x2B).
- (Retire `T0846` as the mapping for these two FCs.)

---

## DECISION 3 — Multi-technique co-emission on a single event

**Question:** A single Modbus write PDU can match T0855 (Unauthorized Command Message), T0836 (Modify Parameter), and T0835 (Manipulate I/O Image). Current spec caps to most-specific write-technique per PDU + T0855 once per burst, to avoid finding-amplification exhausting a 10k finding cap. Is cap-to-most-specific right, or emit-all, or one-alert-N-tags?

### Finding [VERIFIED across detection-engineering sources]

The clear industry consensus for **one observable → multiple techniques** is: **emit ONE finding/alert tagged with MULTIPLE technique IDs.** Not N separate alerts; not silent suppression of the secondary techniques.

- **Sigma** (de-facto cross-platform detection standard): the `tags` field is explicitly designed to carry **multiple `attack.tXXXX` technique tags on a single rule/detection.** SigmaHQ examples show multiple ATT&CK tags per rule.
- **Elastic Common Schema / Elastic Security**: `threat.technique` fields are **multi-valued by design** "to classify events according to … MITRE ATT&CK," precisely to capture events spanning multiple techniques. Elastic = explicit industry signal for **multi-tagged single alert.**
- **MITRE's own usage**: techniques are tags/labels on observed behavior, designed to co-occur.

So the *attribution* model is settled: **one finding, N technique tags.**

### Is the amplification / detector-DoS concern real? [VERIFIED — yes]

Alert amplification and alert fatigue are well-documented, and mature tools mitigate it with **rate-limiting / suppression / aggregation — separately from attribution:**

- **Suricata** `threshold` keyword: four modes (`threshold`, `limit`, `both`, `backoff`) to cap alerts per rule per time window.
- **Snort** `rate_filter` / `event_filter`: throttle alerts when a rate is exceeded (`track`, `count`, `strobe`).
- **Elastic Security** *alert suppression*: "groups related events and creates a single representative alert instead of one alert per event" (per-rule-execution or per-time-period).
- **Splunk** *alert suppression groups* / event correlation: "aggregates, deduplicates, and analyzes alerts … cut through noise."

**Key architectural lesson [VERIFIED]:** mature tools **separate two concerns**:
1. **Attribution** → one alert, many technique tags (never collapse the tags).
2. **Volume control** → suppress/aggregate/rate-limit *the number of alert instances* over a window.

The current spec conflates them: it controls volume by *throwing away technique granularity* (cap-to-most-specific). That **does risk losing analytically useful signal** — and the sources confirm analysts value seeing the full set of co-occurring techniques to understand attack scope. An analyst genuinely may want to see both "unauthorized command" (T0855) and "modify parameter" (T0836) on the same event.

### RECOMMENDATION — Decision 3

**Adopt one-finding / multi-tag, with separate burst-level aggregation for volume control. [VERIFIED model + JUDGMENT on the cap]** Specifically:

1. **Per write PDU that matches multiple techniques: emit ONE finding carrying ALL applicable technique IDs as tags** (e.g. `techniques: [T0855, T0836]`, plus T0835 for coil writes). Do **not** cap to most-specific — keep the multi-tag set. This is the Sigma/Elastic-aligned norm and preserves analyst signal.
2. **Solve the 10k-cap / flood-DoS problem with aggregation, not by dropping tags.** During a detected write burst, **emit one aggregated burst finding** (with count, target, time window, and the union of technique tags) rather than one finding per PDU. This is exactly Elastic "one representative alert per time period" / Suricata `limit` / Splunk suppression-group behavior. It caps finding count by ~the burst-event count, not the PDU count — directly defeating the detector-DoS without information loss.
3. **Keep T0835 vs T0836 specificity as a tag, not a filter.** Register write → tag T0836; coil write → tag T0835. Emit whichever applies; if a multi-write PDU spans both register and coil semantics, tag both. The "register > coil" *priority* the current spec uses is fine as a **display/sort priority**, but should not *suppress* the other tag.

**Net:** the current spec's *instinct* (cap to avoid amplification) is right about the problem but wrong about the mechanism. Switch the mechanism from **tag-suppression** to **event-aggregation + multi-tagging.** This keeps the 10k cap safe (aggregation bounds count) AND keeps full technique signal (multi-tag).

**If F2 wants the absolute minimum change:** keep "T0855 once per burst event" (that is already aggregation, which is correct), but for the write-technique, **emit one finding per burst event carrying BOTH T0836 and T0835 tags as applicable** instead of picking one. That single change converts cap-to-most-specific into one-alert-multi-tag with negligible added cost.

---

## Recommendations Summary (action-ready)

- **Decision 1 (write-burst):** Ship a **dual-window** detector, not single-window. Defaults: **burst = 20 writes/sec in any 1s window**; **sustained = 10 writes/sec averaged over a ≥2s window** (configurable). Rationale: every mature ICS tool (Dragos, Cisco Cyber Vision) and the academic literature use two horizons; single-window has a *documented* low-and-slow blind spot; measured baselines (~0.1–5 writes/s steady, 5–15/s legit transitions) make a flat 10/s neither a clean burst ceiling nor a sustained detector. If forced to single-window, raise default to ~20/s and document the blind spot. **Do not ship single-window @ 10.**

- **Decision 2 (MITRE mapping):** **Change T0846 → T0888 Remote System Information Discovery (TA0102 Discovery)** for **0x11** and **0x2B/MEI 0x0E** — verified against attack.mitre.org; the current T0846 mapping is a known misattribution. **Drop 0x07 Read Exception Status** as a standalone recon indicator (single status byte, too low-signal); use it at most as a low-weight corroborator in a scan sequence. Optionally keep T0846 only for the *address-sweep/enumeration* behavior, not for the device-ID FCs.

- **Decision 3 (co-emission):** **Switch from cap-to-most-specific to one-finding / multi-tag + burst aggregation.** Emit a single finding carrying ALL applicable technique tags (T0855 + T0836/T0835), and control volume by **aggregating per burst event** (Elastic/Suricata/Splunk pattern) rather than by discarding technique tags. This defeats the 10k-cap detector-DoS without losing the analyst-relevant signal that the cap currently sacrifices.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | (1) Modbus write-rate anomaly detection models, baselines, single vs dual window across Quickdraw/Suricata/Zeek/Dragos/Claroty/Nozomi/Cisco + academic SCADA-IDS; (2) MITRE ATT&CK for ICS mapping of recon FCs 0x11/0x2B/0x07 (T0846 vs T0888 vs T0840) verified against attack.mitre.org; (3) multi-technique co-emission best practice across Sigma/Elastic/Suricata/Snort/Splunk + alert-amplification mitigation. All run at `reasoning_effort: high`. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — |
| Tavily | 0 | — |
| WebFetch | 0 | (attempted on local file, not applicable) |
| WebSearch | 0 | — |
| Training data | 1 area | MITRE technique ID/tactic structure cross-checked against model knowledge; all load-bearing claims (T0888 description, FC semantics, tool threshold mechanisms) sourced from the Perplexity-cited authoritative pages (attack.mitre.org, modbus.org, Quickdraw, Suricata/Snort docs, Elastic/Splunk docs, Broadcom signature DB, DiVA academic paper). |

**Total MCP tool calls:** 3 (all `perplexity_research`, the mandated primary tool).
**Training data reliance:** low — every recommendation is anchored to a source cited in the three deep-research passes (attack.mitre.org T0846/T0888/TA0102 pages, Modbus spec, Digital Bond Quickdraw, Suricata/Snort/Zeek ICSNPP docs, Dragos/Claroty/Nozomi/Cisco vendor docs, Sigma/Elastic/Splunk detection docs, Broadcom IPS signature, DiVA 2019 SCADA-IDS paper).

### Key sources (verified)
- MITRE ATT&CK for ICS: T0846, T0888, T0840, TA0102 (attack.mitre.org); T0806, T0855.
- Modbus Application Protocol Spec (modbus.org); SimplyModbus exception-code reference.
- Digital Bond Quickdraw-Snort/Suricata rulesets (github.com/digitalbond).
- Suricata `modbus` keyword + `threshold` docs; Snort `rate_filter`/Modbus README; CISA ICSNPP-Modbus (Zeek).
- Dragos, Claroty CTD, Nozomi Networks, Cisco Cyber Vision public platform docs.
- Sigma/SigmaHQ tags spec; Elastic Common Schema `threat.technique`; Elastic Security alert suppression; Splunk suppression groups / event correlation.
- Broadcom IPS signature "TCP MODBUS – Read Device Identification" (asid=20672).
- DiVA 2019 timing-based SCADA anomaly-detection study (W=5, N=4); ICS reconnaissance / "exploiting Modbus" literature.
