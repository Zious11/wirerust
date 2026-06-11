# DNP3 Analyzer (Feature #8) — F2 Scope & Threshold Validation

**Feature:** wirerust #8 — DNP3 analyzer (TCP-only, port 20000)
**Date:** 2026-06-10
**Purpose:** Pre-human-gate research validation of two F2 spec decisions: (Q1) is the v1
detection SCOPE complete/appropriate, and (Q2) are the 3 detection THRESHOLD defaults sound.
**Companion:** `.factory/research/dnp3-research.md` (protocol byte-layout + MITRE-mapping
reference). This document does **not** re-derive protocol facts established there; it validates
*scope* and *thresholds* against how real DNP3 security tooling and the ICS-security literature
scope detection.

> **Confidence legend (same as `dnp3-research.md`):** [VERIFIED] = confirmed against a primary
> source quoted/linked below; [TOOL] = confirmed against the named tool's own docs/source;
> [JUDGMENT] = defensible engineering call where sources are environment-dependent or silent;
> [UNVERIFIED] = could not source — do not rely on without confirming.

> ⚠️ **Methodology caveat (must read).** The two `perplexity_research` (`sonar-deep-research`)
> passes returned large, fluent syntheses that included **fabricated specifics** — invented
> Suricata SIDs (e.g. "SID 2027785"), invented Snort preprocessor event numbers, a non-existent
> "Dirt-spiders" malware, precise-but-unsourced percentages, and at least one **wrong function-code
> mapping** (claiming DIRECT_OPERATE=0x03/FC14 and WARM_RESTART=0xC4). Those are **NOT** carried
> into the verdicts below. Every load-bearing claim here is re-grounded against a primary source
> (a quoted vendor device profile, the icsnpp-dnp3 repo, the Crain/Sistrunk S4x14 material, or
> `dnp3-research.md`'s already-verified protocol table). Where the deep-research synthesis is the
> *only* source for a claim, it is marked [UNVERIFIED].

---

## QUESTION 1 — Is the v1 detection SCOPE complete and appropriate?

### VERDICT: **Scope is appropriate for v1 but has gaps. Conditionally complete — ADD two detections, ELEVATE one deferral.**

The chosen detections are the right high-value ones and align with what real DNP3 tooling
emits. But two commonly-recognized, high-value DNP3 attack patterns are **missing**, and one
"defer" item (CRC) needs a one-line caveat because of *how* the Crain-Sistrunk attacks work.

#### (a) Are the chosen detections the right high-value ones? — YES.

The v1 set maps cleanly onto what the reference open-source tool actually surfaces and what the
literature treats as high-value:

| v1 detection | Corroboration |
|---|---|
| Unauthorized control cmd (SELECT/OPERATE/DIRECT_OPERATE/DIRECT_OPERATE_NR over threshold) → T1692.001 | icsnpp-dnp3 dedicates an entire extension log (`dnp3_control.log`) to exactly the SELECT/OPERATE/RESPONSE Control-Relay-Output-Block / Pattern-Control-Block path — i.e. the tool authors judged control commands the single most detection-worthy DNP3 event class. [TOOL] |
| COLD_RESTART/WARM_RESTART → T0814 | Universally treated as a cheap high-value single-packet DoS signal; FC 0x0D/0x0E per `dnp3-research.md` §3.2. [VERIFIED] |
| WRITE (0x02) → T0836 | A parameter/config write is the canonical "Modify Parameter" signal. [VERIFIED] |
| Inferred block-command (control req without response) → T1691.001 | icsnpp-dnp3 logs a control-block `status_code` enum that explicitly includes `Timeout`, `No Select`, and `Not Authorized` — the tool exposes exactly the request/response-outcome data this inference is built on. [TOOL] |
| Derived loss-of-control (correlated multi-event) → T0827 | Correctly modeled as a derived/correlated impact finding per `dnp3-research.md` §6. [VERIFIED] |
| Broadcast-destination anomaly (0xFFFD/E/F) | Crain/Sistrunk explicitly note many implementations **accept broadcast** for the crash frames "which doesn't require any attacker knowledge about the link endpoint configurations" — broadcast is a real, attacker-relevant abuse channel, so flagging it is justified. [VERIFIED] |
| Unsolicited-response anomaly (UNS / FC 0x82) | **Strongly validated.** Crain/Sistrunk found a *disproportionate* number of application-layer vulns "associated with the unsolicited response functions," because RESPONSE/UNSOLICITED_RESPONSE are the most object-overloaded (highest-attack-surface) function codes in IEEE 1815-2012's Parsing Guideline Tables. Watching FC 0x82 is squarely on-target. [VERIFIED] |

**Conclusion (a):** the seven chosen detections are the correct high-value set. No chosen
detection is misguided or low-value.

#### (b) Are IMPORTANT, commonly-detected DNP3 attack patterns MISSING? — YES, two.

**GAP 1 — Unsolicited-reporting *enable/disable* abuse (FC 0x14 ENABLE_UNSOLICITED / 0x15
DISABLE_UNSOLICITED). [VERIFIED gap, JUDGMENT on severity]**
The v1 scope watches *unsolicited responses* (0x82) but **not** the control-plane functions that
turn unsolicited reporting on/off. `DISABLE_UNSOLICITED` (0x15) is the classic **alarm-suppression /
event-blinding** primitive: an attacker silences the outstation's event reporting so the master
stops seeing state changes — an Inhibit-Response-Function move conceptually adjacent to the
block-command the spec already covers, and a recognized ICS abuse pattern. `ENABLE_UNSOLICITED`
(0x14) is the enabling half (and, per `dnp3-research.md` §5.1, observing whether an enable was
ever seen is *already a planned input* to the unsolicited-anomaly logic). Both are single-FC
matches (near-zero implementation cost) and both are in the verified FC table (`dnp3-research.md`
§3.2: 0x14/0x15). **Recommendation: ADD detection of DISABLE_UNSOLICITED (0x15) — and ideally
ENABLE_UNSOLICITED (0x14) — from an unexpected/non-allowlisted source, mapping DISABLE_UNSOLICITED
to T0814 (DoS / Inhibit Response Function) or T1691.x.** This is the single most defensible
addition; it is cheap, high-value, and directly supported by the same body of evidence (unsolicited
functions = highest DNP3 attack/abuse surface) that already justifies the 0x82 detection.

**GAP 2 — Malformed-DNP3 / structural-anomaly detection (the Crain-Sistrunk core). [VERIFIED gap,
JUDGMENT on v1 scoping]**
The entire Crain/Sistrunk "Project Robus, Master Serial Killer" (S4x14) result — ~28-30 vulns,
16+ ICS-CERT advisories — is about **malformed frames that crash masters and outstations**: ASDUs
too short to hold a valid object header, object counts set to 0xFFFF with no bytes following
(infinite loop), transport-header-only frames (unhandled exception), control objects appearing
where unexpected (buffer overrun). Critically, **these frames carry a *valid* CRC** — "ASDUs that
are too short to contain a valid object header could be delivered in a frame with a correct lower-
layer CRC value to cause an unhandled exception." A passive analyzer that only parses well-formed
frames is **blind to the single most-documented DNP3 attack class of the last decade.** [VERIFIED]

A *full* malformed-frame detector (object-count-vs-payload consistency, qualifier validation,
short-ASDU checks) is legitimately large and is **defensible to defer from v1** — but the v1
parser already computes the structural facts needed for a **cheap subset**: `dnp3-research.md` §1.1
mandates rejecting `LENGTH < 5`, and §4 framing math derives the expected frame length. **At
minimum, v1 should *emit a low-confidence malformed-DNP3 anomaly finding* when the parser's own
sanity gates trip (LENGTH<5, frame-length/`num_data_blocks` mismatch, sync-but-implausible-FC),
rather than silently dropping the frame.** This is the exact failure mode the deep-research pass
flagged for icsnpp-dnp3/Suricata/Snort: they parse-and-discard malformed frames without raising a
security event, leaving a documented blind spot. wirerust can cheaply do better by surfacing what
it already detects. **Recommendation: ADD a "malformed/structurally-invalid DNP3 frame" anomaly
(low/medium confidence) wired to the parser's existing reject paths → T0814 (these frames are DoS
crash vectors). Defer *deep* object-level malformation analysis to v2.**

**Considered and judged correctly-deferred (NOT must-add for v1):**
- **STOP_APPL (0x12):** real disruptive management FC (verified in `dnp3-research.md` §3.2 and the
  Chipkin quick reference: "12 Stop application"). It is a legitimate "stop the application
  process" primitive an attacker could abuse. **However**, it is rare, often used legitimately in
  maintenance, and lower-signal than restart. **Optional add** (trivial single-FC match → T0814 if
  v1 wants completeness of the disruptive-management set: COLD_RESTART/WARM_RESTART/STOP_APPL),
  but **not a blocking gap.** [JUDGMENT]
- **IIN (Internal Indication) bit anomalies:** Zeek's *default* `dnp3.log` already surfaces the
  response `iin` field [TOOL], and IIN bits (DEVICE_RESTART IIN1.7, EVENT_BUFFER_OVERFLOW IIN2.3,
  CONFIG_CORRUPT IIN2.5, FUNC_NOT_SUPPORTED IIN2.0, etc.) are genuinely useful corroborating
  signals — e.g. an unexpected DEVICE_RESTART IIN bit independently corroborates a restart event.
  But IIN-bit *manipulation* detection is a richer behavioral feature (which combinations, against
  which baseline) and is **defensibly deferred to v2.** [JUDGMENT] Recommend noting IIN parsing as
  a planned v2 enrichment so the restart/loss-of-control correlation can later cross-check the
  outstation's own IIN1.7 DEVICE_RESTART bit.

#### (c) Is deferring self-address / reserved-range / CRC-validation / multi-fragment / UDP defensible for v1? — MOSTLY YES, with ONE caveat.

| Deferral | Verdict | Rationale |
|---|---|---|
| **Self-address (0xFFFC) detection** | **Defensible defer.** [JUDGMENT] | Niche feature; exact reserved value still [UNVERIFIED] against IEEE 1815 (`dnp3-research.md` §4). Low prevalence, low signal for v1. |
| **Reserved-address-range detection** | **Defensible defer.** [JUDGMENT] | Lower bound of the reserved block is [UNVERIFIED] (`dnp3-research.md` §4); deferring avoids encoding an unverified boundary. The high-value addressing signal (broadcast 0xFFFD–F) is *already* in v1. |
| **CRC-16/DNP validation (structure-only skip)** | **Defensible defer — but add a one-line caveat.** [VERIFIED caveat] | CRC validation is correctly deferred for *integrity*. **Caveat:** do NOT let "we skip CRC" create the assumption that malformed payloads are rare — Crain/Sistrunk frames carry *correct* CRCs, so CRC validation would **not** have caught them anyway. The defensive value is in GAP 2 (structural/object sanity), not CRC. Document this so a future reader doesn't mistake "CRC deferred" for "malformed-frame coverage deferred for the same reason." |
| **App-layer multi-fragment reassembly (FIR=1 first-fragment only)** | **Defensible defer.** [JUDGMENT] | Matches `dnp3-research.md` §2: parse FC from the FIR=1 first segment. Control/restart/write FCs all appear in the first fragment, so the high-value detections are unaffected. Note the residual blind spot: an attacker who splits a malicious APDU across fragments evades first-fragment-only parsing — acceptable for v1, flag for v2. |
| **DNP3-over-UDP** | **Defensible defer, but note the reference tool covers it.** [TOOL] | The v1 "TCP-only, port 20000" scope is reasonable. **Note for the record:** Zeek/icsnpp-dnp3 registers BOTH `20000/tcp` and `20000/udp` (verified in zeek `dnp3/main.zeek`: `const ports = { 20000/tcp , 20000/udp }`). DNP3-over-UDP is real and the canonical tool handles it, so UDP is a known v2 item, not a non-existent concern. |

**Q1 bottom line:** **Scope has gaps: [DISABLE_UNSOLICITED (0x15) abuse], [malformed/structural
DNP3 anomaly from existing reject paths].** Both are cheap and high-value. STOP_APPL (0x12) and
IIN-bit enrichment are optional/v2. All five named deferrals are defensible; the CRC deferral
needs the documented caveat above so it isn't conflated with malformed-frame coverage.

---

## QUESTION 2 — Are the 3 detection THRESHOLD defaults sound?

### Threshold 1 — `--dnp3-direct-operate-threshold = 10 control-class FCs / 60s / flow` (T1692.001)

**RECOMMENDATION: CONFIRM 10/60s as the default, but document it as deliberately *lax* (a
burst/flood detector, not a precision unauthorized-command detector). Optionally expose a tighter
secondary default (~5/60s) for transmission profiles.** [JUDGMENT, grounded in vendor + literature]

**Evidence:**
- Real control-command rates are **far below** 10/min. Mechanical reality dominates: circuit
  breakers, tap changers, and protective relays require seconds-to-minutes between operations to
  avoid mechanical/thermal damage; protective-relay operating times are ~200 ms with deliberate
  time delays. Even the busiest legitimate control environment — distribution FLISR (automated
  fault location/isolation/restoration) — issues control commands in short bursts that *rarely
  exceed ~5/min*, with long idle gaps otherwise. [UNVERIFIED — this specific 0.5–5/min figure
  comes from the deep-research synthesis citing a ScienceDirect FLISR traffic study
  (S1874548223000252) and the DNP3 IDS dataset (Zenodo 7348493); I did not independently open
  those, so treat the *exact* numbers as indicative, not pinned.]
- Implication: **10 control FCs in 60s on a single flow is genuinely anomalous** in steady-state
  SCADA — it exceeds typical mechanical cycling limits. So 10/60s will *not* fire on normal
  operation, which is the right false-positive posture. The risk is the opposite: at 10/60s the
  threshold is **lax** — it only catches a fairly aggressive burst and would miss a low-and-slow
  unauthorized operator issuing, say, 3 unauthorized OPERATEs over a minute.
- This is acceptable **because the threshold is not the only gate**: per `dnp3-research.md` §5/§8,
  the high-value unauthorized-command signal is *control FC from a non-allowlisted SOURCE*, which
  should fire at **count = 1** regardless of rate. The 10/60s threshold is correctly a *secondary*
  volume/flood guard for the allowlisted-but-abnormally-busy case, not the primary authz check.

**Verdict:** **CONFIRM 10/60s** as a sane, low-FP default for the *rate/flood* arm. Make the
documentation explicit that (i) unauthorized-source control should fire at count=1, independent
of this threshold; (ii) 10/60s is deliberately conservative against FP; (iii) operators in
quiet transmission environments may tighten to ~5/60s. Do **not** raise it above 10.

### Threshold 2 — Correlation window = 300s; block-command pattern = 3 timed-out control requests / 300s (T1691.001); request/response timeout = 10s

**RECOMMENDATION: CONFIRM all three values (300s window, 3 timeouts, 10s timeout). They are
well-aligned with real DNP3 timing.** [VERIFIED on the 10s value; JUDGMENT on 3-and-300s]

**Evidence — the 10s request/response timeout is strongly corroborated as the dominant default:**
- ABB REC615/RER615 DNP3 manual: SBO "CROB select timeout" **default = 10 seconds.** [VERIFIED]
- DNP3 device profile (Scribd device-profile table): "Select / Operate Arm Timeout: **10s**";
  "Unsolicited response retry delay: 10s." [VERIFIED]
- ICDN FC22 DNP3 device profile: "Application SBO Timeout … range 1–9999 s, **default 10s**."
  [VERIFIED]
- GE Vernova DNP3 manual: "SELECT BEFORE OPERATE arm timeout … configurable from zero to **64
  seconds**" (10s sits comfortably inside). [VERIFIED]
- Siemens TIM device profile: "Timeout waiting for Complete Application Layer Response (ms): **30**"
  — i.e. some app-layer response timeouts are configured higher (30s); and Siemens TIM "Max. time
  between Select and Operate" **default = 1s** (a conservative outlier). [VERIFIED]
- Synthesis: app-layer/SBO timeouts cluster at **5–30s, with 10s the single most common SBO/arm
  default.** A **10s request/response timeout is the correct, evidence-backed pick** — it matches
  the modal SBO arm timeout, so a control request unanswered for 10s genuinely exceeds the device's
  own select-validity window. [VERIFIED]

**Evidence — "3 timed-out within 300s" reliably discriminates blocking from packet loss:**
- DNP3 masters retry. Siemens TIM: command frames are "repeated a **maximum of three times** before
  they are discarded by the master." [VERIFIED] DNP Users Group even publishes guidance on
  "Maximum Application Layer Retries for Control Select Messages" (AN2022-001), confirming retries
  are a standard, bounded mechanism. [VERIFIED — title; exact default not pinned]
- Therefore: a *single* unanswered control request is well within normal retry/loss behavior and
  must NOT fire. Requiring **3 distinct timed-out control requests** means the analyzer has
  effectively observed the master exhaust (or repeatedly hit) its retry budget across multiple
  command attempts — strongly consistent with sustained blocking rather than a transient drop.
  This matches `dnp3-research.md` §5.2's explicit requirement for a *sustained* pattern, not a
  single missing response, and §8.3's warning that single request-without-response is "not always
  blocking" (packet loss, capture gaps, DIRECT_OPERATE_NR by design). [VERIFIED reasoning chain]
- 300s window: at a 10s timeout, 3 timeouts fit comfortably inside 300s even with retry backoff
  and normal inter-command spacing, while 300s is short enough that the 3 events are plausibly part
  of one attack episode rather than 3 unrelated daily blips. [JUDGMENT — no single source pins
  "300s"; it is a sound engineering choice consistent with the timing above.]

**One required guard (from `dnp3-research.md` §8.3): EXCLUDE DIRECT_OPERATE_NR (0x06) from the
timed-out-request count.** 0x06 is no-response *by design*; counting its (expected) missing
response toward the block-command pattern would manufacture false positives. Confirm the spec's
correlation key counts only response-expecting control FCs (SELECT/OPERATE/DIRECT_OPERATE).
[VERIFIED requirement]

**Verdict:** **CONFIRM 300s / 3-timeouts / 10s-timeout.** The 10s timeout is the modal real-world
SBO value; "3" sits just past the typical 3-retry budget so it cleanly separates blocking from
loss; 300s is a sound correlation horizon. Add the DIRECT_OPERATE_NR exclusion guard.

### Threshold 3 — T0827 (Loss of Control) guard = ≥3 combined restart+block events / 300s

**RECOMMENDATION: CONFIRM ≥3 combined impact-events as the guard. It is a sound, appropriately
conservative false-positive guard for a high-severity derived finding.** [JUDGMENT — correct in
principle; no protocol source dictates the exact count.]

**Evidence / reasoning:**
- T0827 Loss of Control is an **Impact-tactic outcome**, not a single-packet method
  (`dnp3-research.md` §6). A high-severity "operators have lost control" finding emitted from one
  event would be both semantically wrong and FP-prone. Requiring **multiple correlated
  impact-events** before asserting the *outcome* is exactly the right design — it makes T0827 a
  derived/correlated finding, which is what `dnp3-research.md` §6 and §5.2 explicitly recommend.
  [VERIFIED design principle]
- Each contributing event (a restart, or a sustained block-command pattern) is *already itself* a
  guarded finding (restart = real disruptive FC; block = already gated behind "3 timeouts/300s"
  per Threshold 2). So the ≥3 combined guard compounds two layers of conservatism — appropriate
  for the highest-severity emission. A lower bar (≥1 or ≥2) would risk labeling a single noisy
  restart or one blocking episode as full "loss of control." [JUDGMENT]
- 300s window shared with Threshold 2 keeps the correlation horizon consistent — the combined
  events must cluster in time to plausibly represent one loss-of-control episode. [JUDGMENT]
- Minor note: ensure the "3 combined events" are *distinct* impact events, not e.g. the same block
  pattern double-counted with its constituent timeouts; otherwise the guard can be satisfied by a
  single underlying incident. Recommend the spec define the combined-event set as
  {distinct restart event, distinct sustained-block finding} so the ≥3 is genuinely multi-event.
  [JUDGMENT]

**Verdict:** **CONFIRM ≥3 combined restart+block events / 300s.** Requiring 3 correlated
impact-events before a high-severity Loss-of-Control emission is a sound FP guard. Add the
"distinct events" clarification.

---

## Summary of Recommendations

**Q1 — Scope verdict: COMPLETE FOR v1 WITH GAPS.**
- **MUST-ADD (2):**
  1. **DISABLE_UNSOLICITED (0x15)** abuse detection (and ENABLE_UNSOLICITED 0x14) from
     unexpected source → T0814 / Inhibit-Response-Function. Cheap single-FC match; alarm-suppression
     primitive; same high-attack-surface unsolicited-function family the v1 0x82 detection already
     targets.
  2. **Malformed/structural DNP3 anomaly** surfaced from the parser's *existing* reject paths
     (LENGTH<5, frame-length/block-count mismatch, sync-with-implausible-FC) → low/med-confidence
     T0814. This is the only coverage for the Crain-Sistrunk malformed-frame crash class, which is
     the most-documented DNP3 attack family of the decade. Defer *deep* object-level malformation
     to v2.
- **OPTIONAL/v2:** STOP_APPL (0x12) single-FC match → T0814; IIN-bit parsing/enrichment (default
  Zeek already logs `iin`) to corroborate restart/loss-of-control.
- **Deferrals (self-address, reserved-range, CRC-validation, multi-fragment reassembly, UDP): all
  defensible.** One required documentation caveat: note that CRC validation would NOT have caught
  the Crain-Sistrunk frames (they carry valid CRCs) — so "CRC deferred" must not be read as
  "malformed-frame coverage deferred," which is why MUST-ADD #2 exists.

**Q2 — Threshold recommendations:**
1. **Direct-operate threshold 10/60s → CONFIRM** (as a deliberately-lax flood guard). Document that
   unauthorized-*source* control fires at count=1 independent of this rate; optionally offer ~5/60s
   for quiet transmission profiles. Do not raise above 10.
2. **300s window / 3 timed-out control requests / 10s req-resp timeout → CONFIRM all three.** 10s is
   the modal real-world SBO arm timeout (ABB, ICDN, device profiles all default 10s); "3" sits just
   past the standard 3-retry budget (Siemens TIM "maximum of three times"), cleanly separating
   blocking from packet loss. **Add guard: exclude DIRECT_OPERATE_NR (0x06) from the timed-out
   count** (no-response by design).
3. **Loss-of-Control guard ≥3 combined restart+block events / 300s → CONFIRM.** Sound FP guard for a
   high-severity derived Impact finding. **Add clarification:** the ≥3 must be *distinct* impact
   events (a restart event + a sustained-block finding), not a single incident double-counted.

---

## Sources

| # | Source | Used for | Confidence |
|---|--------|----------|------------|
| [1] | Crain & Sistrunk, *"Project Robus, Master Serial Killer"* S4x14 (2014) — via langsec.org DNP3 ICSS2016 slides (`langsec.org/dnp3/dnp3-icss2016-slides.pdf`) | Malformed-frame crash vectors (short ASDU, 0xFFFF object count infinite loop, transport-only frame, unexpected control objects); broadcast-accepted crash frames; "30 CVEs 2013-2014" | [VERIFIED] |
| [2] | Crain & Bratus, *"Bolt-On Security Extensions … DNP3 SAv5"* (Dartmouth/Automatak, IEEE S&P) (`cs.dartmouth.edu/~sergey/langsec/papers/crain-bratus-bolt-on-dnp3sa.pdf`) | >80% of vulns in application layer; disproportionate share in **unsolicited response** functions; valid-CRC malformed frames; single-frame master/outstation crash | [VERIFIED] |
| [3] | Dale Peterson, *"Why Crain/Sistrunk Vulns Are A Big Deal"* (2013) (`dale-peterson.com/2013/10/16/...`) | Master-crash-via-crafted-response attack model; unsolicited-response means no need to wait for a poll; no firewall stops it | [VERIFIED] |
| [4] | DarkReading/stewilliams.com coverage of Project Robus (2014) | ~28 flaws, 16 ICS-CERT advisories, Cooper/Cybectec pulled product | [VERIFIED] |
| [5] | cisagov/icsnpp-dnp3 README + `scripts/consts.zeek` (`github.com/cisagov/icsnpp-dnp3`) | `dnp3_control.log` (7 fns) / `dnp3_objects.log` (2 fns); control `status_code` enum incl. Timeout/No Select/Not Authorized; SELECT-OPERATE-RESPONSE focus | [TOOL] |
| [6] | zeek `scripts/base/protocols/dnp3/main.zeek` (`github.com/zeek/zeek`) | Default `dnp3.log` logs `fc_request`/`fc_reply`/`iin`; **ports = 20000/tcp AND 20000/udp** | [TOOL] |
| [7] | Chipkin *DNP3 Quick Reference* (`cdn.chipkin.com/.../DNP3QuickReference.pdf`) | FC table incl. 12=Stop application, 14=Enable unsolicited, 15=Disable unsolicited; IIN bit map (IIN1.7 Device restart, IIN2.3 Event buffer overflow, etc.) | [VERIFIED] |
| [8] | ABB REC615/RER615 DNP3 Communication Protocol Manual (`techdoc.relays.protection-control.abb`) | SBO CROB select timeout **default 10s** | [VERIFIED] |
| [9] | DNP3 Device Profile table (Scribd 754635609) | Select/Operate Arm Timeout **10s**; Data Link/App Confirm fixed 15s; Unsolicited retry delay 10s | [VERIFIED] |
| [10] | ICDN FC22 DNP3 device profile (`icdnteam.com/.../FC22_DNP3_Proflie_V1.1...pdf`) | Application SBO Timeout range 1–9999s **default 10s**; app confirm 0–60s default 5s | [VERIFIED] |
| [11] | GE Vernova DNP3 Manual (`gevernova.com/.../m6xxd_en_m_b.pdf`) | SBO arm timeout configurable **0–64s** | [VERIFIED] |
| [12] | Siemens TIM 1531/4R DNP3 device profile + SIMATIC NET DNP3 docs (`cache.industry.siemens.com`, `docs.tia.siemens.cloud`) | App-layer response timeout 30; Select-Operate default 1s; **"repeated a maximum of three times"** retry; DIRECT_OPERATE ignores SBO timer | [VERIFIED] |
| [13] | DNP Users Group AN2022-001 (Device Profile) (`dnp.org/FastFind/New-DNP3-Device-Profile-Guide...`) | "Maximum Application Layer Retries for Control Select Messages" — retries are standard/bounded | [VERIFIED — title] |
| [14] | Schneider Geo SCADA DNP3 Driver Guide — Config Corrupt Alarm (`tprojects.schneider-electric.com/.../ConfigurationCorruptAlarm.htm`) | CONFIG_CORRUPT IIN bit operational meaning (IIN enrichment value) | [VERIFIED] |
| [15] | `dnp3-research.md` (this repo) | All protocol byte-layout, FC hex table, MITRE-ICS v19.1 mappings, addressing, FP considerations | [VERIFIED — prior pass] |
| [16] | ScienceDirect S1874548223000252 (FLISR traffic study); Zenodo 7348493 (DNP3 IDS dataset) | Indicative normal control-command rates (0.5–5/min) — **cited by deep-research synthesis, not independently opened** | [UNVERIFIED] |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Real-tool DNP3 detection scope (Zeek/icsnpp-dnp3, Suricata, Snort, Dragos/Nozomi/Claroty) + Crain-Sistrunk gap analysis + deferral defensibility — `reasoning_effort: high`. (2) DNP3 SCADA control-command rates, SBO/arm-timeout values, retry/packet-loss-vs-blocking discrimination for threshold validation — `high`. |
| Perplexity perplexity_search | 3 | Primary-source grounding to corroborate/correct the deep-research syntheses: (a) Crain-Sistrunk/Project Robus primary material; (b) icsnpp-dnp3 actual log/event set + Zeek default dnp3 script; (c) DNP3 SBO/select-operate-arm timeout device-profile defaults. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | N/A (protocol/threat-tooling research, not a software-library API). |
| Tavily | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | 1 area | Cross-check only on well-known ICS facts; every load-bearing claim is grounded to a primary source above or to the prior-verified `dnp3-research.md`. |

**Total MCP tool calls:** 5 (2 `perplexity_research` high-effort + 3 `perplexity_search`).
**Training data reliance:** low.
**Critical methodology note:** the 2 deep-research passes contained **fabricated specifics**
(invented Suricata SIDs, invented Snort event numbers, a fictional "Dirt-spiders" malware,
unsourced percentages, and a wrong FC mapping). These were detected by cross-checking against
the 3 `perplexity_search` primary-source passes and the prior-verified `dnp3-research.md`, and
were **excluded** from all verdicts. Claims sourced only to the deep-research synthesis (the
indicative control-command rates, source [16]) are explicitly marked **[UNVERIFIED]**. The
threshold verdicts that matter (10s SBO timeout, 3-retry budget, 10s arm-timeout dominance) rest
on quoted vendor device profiles, not on the synthesis.
