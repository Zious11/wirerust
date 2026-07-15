---
document_type: research-validation
producer: research-agent
date: 2026-07-15
finding_id: SR-172-03
subsystem: SS-19
feature_cycle: feature-iec104
related_artifacts:
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - BC-2.19.026
recommendation: EMIT-WITH-DEDUP
---

# SR-172-03 — Malformed APCI Length Handling: Spec-Contradiction Validation

## Purpose

Resolve the ADR-013 ↔ BC-2.19.026 contradiction for the case: **valid start byte `0x68`,
but LEN octet outside the valid range `[4, 253]`.**

- **ADR-013 Decision 3, step 4:** emit a MITRE ATT&CK ICS **T0814 (Denial of Service)**
  finding, then advance past a 2-byte APCI stub.
- **BC-2.19.026 PC4:** advance-only, **NO finding** (silent resync).

Temporary risk noted by the caller: the per-flow findings cap does not land until the next
story, so any per-frame emission is an unbounded flood concern in the interim.

This report validates the underlying facts (per DF-VALIDATION-001 discipline) before the
contradiction is resolved in code/spec.

---

## Q1 — What does IEC 60870-5-104 specify about the APCI LEN bounds and receiver handling of an invalid LEN?

**Bounds — CONFIRMED.** The APDU LEN octet is bounded `4 ≤ LEN ≤ 253`.

- The upper bound is derived directly in the standard text: "the maximum length of the ASDU
  is limited to 249 because the maximum value of the field length of APDU is 253… the APDU
  is defined as 255 octets minus the start and length octet" (EN 60870-5-104:2006; IEC
  60870-5-104:2006 §5.1). `APDU_max = 255 − 2 = 253`.
  - https://standards.iteh.ai/catalog/standards/clc/b6ed7c9e-0feb-4a9f-8a0c-9ce89fb9aa5e/en-60870-5-104-2006
  - http://rfc.nop.hu/iec/IEC%2060870-5-104-2006.pdf
- Vendor interoperability docs repeat 253 as the fixed maximum system parameter (may be
  reduced per system, never increased): Phoenix Contact PLCnext, Eaton Cooper Power.
  - https://www.plcnextstore.com/service/api/files/doc/Interoperability_104RTU_V3018_en-20260106074214429.pdf
  - https://www.eaton.com/content/dam/eaton/products/medium-voltage-power-distribution-control-systems/cooper-power-series-historical-literature/voltage-regulators/interoperability-for-communications-protocol-iec-60870-5-104-r225-70-27.pdf
- The lower bound of **4** (the four CF octets carried by a payload-less S/U frame) is
  stated explicitly in the IDS literature: the APDU length field "belongs to the range
  [4, 253]." (IET rule-based IDS for IEC-104.)
  - https://digital-library.theiet.org/doi/pdf/10.1049/cp.2013.1729

**Prescribed receiver reaction to out-of-range LEN — INCONCLUSIVE / NOT NORMATIVELY
PRESCRIBED.** This is the important nuance and it partially undercuts the premise in the
research question ("active endpoints close the connection — confirm").

- The standard defines the *valid range* and the frame structure, but the accessible
  standard text and companion/vendor documentation do **not** prescribe a single, uniform
  reaction (discard-and-resync vs. TCP reset) when a receiver sees an out-of-range LEN.
  Handling is left as an implementation detail.
- Empirical confirmation that the standard is silent here: a University of Twente security
  assessment of IEC-104 implementations found "it was not specified in the standard whether
  the entire length of the packet had to be received before parsing the message format,"
  and observed vendor-specific divergence — e.g., after receiving a single 2-byte `Lmax`
  frame, one implementation "ignored [all subsequent messages] up to 251 bytes." This
  documents real, non-uniform, implementation-defined handling of length manipulation.
  - https://essay.utwente.nl/fileshare/file/72277/Kerkers_MA_EEMCS.pdf
- IEC-104's only explicit connection-teardown mechanism is the U-format `Reset_Process`
  (C_RP_NC_1) control command, which is documented for *station initialization*, not as a
  mandated response to malformed length.
  - https://www.prosoft-technology.com/knowledge-base/Protocols/IEC-60870/In-the-IEC-60870-5-104-protocol-what-is-the-purpose-of-the-RESET_PROCESS-command-C_RP_NC_1-and-what-does-it-do

**Q1 verdict:** Bounds `[4, 253]` are authoritative. The claim that "active endpoints close
the connection on invalid LEN" is **NOT** a normative requirement — some stacks close, some
discard/resync, some hang (see Q3). For a *passive monitor* this means the anomaly is real
and worth recording, but there is no single standard behavior to mirror.

---

## Q2 — How do established passive ICS monitors treat `0x68` + out-of-range LEN?

Consistent precedent across every open-source monitor whose code/expert-info is inspectable:
**they flag it as a protocol anomaly and continue** (log-and-skip with an event), rather than
silently dropping or tearing down the flow.

- **Snort 3 IEC104 service inspector — EMITS an alert.** The inspector raises a dedicated
  detection event `IEC104_BAD_LENGTH` (in `GID_IEC104`) when the LEN octet is below the
  minimum APCI length, then continues; on a subsequent full decode failure it *resets the
  flow state* rather than emitting more alerts. This is a direct precedent for **emit-once
  then continue**, not silent skip.
  - https://github.com/snort3/snort3/tree/master/src/service_inspectors/iec104
  - Source: `src/service_inspectors/iec104/iec104.cc` (event `IEC104_BAD_LENGTH`,
    `queue_event(GID_IEC104, IEC104_BAD_LENGTH)`; `if (!Iec104Decode(...)) iec104fd->reset()`).
- **Wireshark IEC-104 dissector — EMITS expert info.** Defines explicit expert-info fields
  `iec104.apdu_invalid_len` ("Invalid ApduLen") and `iec104.apdu_min_len` ("APDU less than
  bytes"). (GPL — behavioral precedent only, no code copy, per ADR-013 Decision 7.)
  - https://www.wireshark.org/docs/dfref/i/iec60870_asdu.html
- **Zeek (Spicy-based iec104 analyzers)** — Spicy parsers raise a *parse error* on
  malformed input (e.g. `&size`/`&max-size` violations), which Zeek surfaces as an analyzer
  violation → **weird event**, then disables the analyzer for that connection (parsing stops
  for the flow; the TCP flow itself is not torn down). So the anomaly is recorded as a weird,
  not silently swallowed.
  - https://github.com/cert-lv/spicy-iec104
  - https://github.com/georgemakrakis/zeek-iec104
  - https://docs.zeek.org/projects/spicy/en/latest/programming/parsing.html (parse error on `&size`/`&max-size`)
- **Suricata** — has community IEC-104 parser/keyword support; treats malformed frames as
  app-layer/decoder anomaly events. (General app-layer-event mechanism; no IEC-104-specific
  silent-drop path documented.)
  - https://forum.suricata.io/t/additional-industrial-control-protocol-parsers/1287
- **Malcolm** — aggregates Zeek + Suricata; inherits their weird/anomaly-event behavior
  (no separate IEC-104 handling).
  - https://github.com/cisagov/Malcolm
- **Commercial OT NSM (Nozomi, Claroty, Dragos)** — **INCONCLUSIVE.** These are closed
  products; no public documentation was found describing their byte-level handling of an
  out-of-range IEC-104 LEN. Marketing material describes general "protocol anomaly / DPI"
  detection, but the specific behavior cannot be verified. Flagged as inconclusive.

**Q2 verdict:** The dominant, verifiable precedent is **emit-a-protocol-anomaly-and-continue**
(Snort `IEC104_BAD_LENGTH`, Wireshark expert info, Zeek weird). None silently resync with no
signal. This favors ADR-013's "emit" over BC-2.19.026's "silent."

---

## Q3 — Is a malformed LEN field a documented attack/DoS vector? CVEs / advisories / research? Does T0814 apply?

**YES — malformed IEC-104 length/frame handling is a documented, real-world DoS vector, and
one CVE is almost an exact match for this case.**

- **CVE-2023-5768 (Hitachi Energy RTU500/RTU520 HCI IEC-104) — near-exact match.**
  "Incomplete or wrong received APDU frame layout may cause blocking on link layer. Error
  reason was an endless blocking when reading incoming frames on link layer with **wrong
  length information of APDU** or delayed reception of data octets." A malformed APDU LEN
  on a real RTU produced an endless-blocking DoS on the communication link. This is precisely
  the `0x68` + bad-LEN scenario, and its impact is denial of service.
  - https://www.cvedetails.com/cve/CVE-2023-5768/
- **CVE-2026-1773 (Hitachi Energy, IEC-104) — DoS on invalid U-format frame**, CWE-184
  (Incomplete List of Disallowed Inputs). Remote, unauthenticated, over port 2404. This is
  already cited in ADR-013 Decision 5 for the non-canonical U-format angle and is the same
  malformed-frame → DoS family.
  - https://github.com/advisories/GHSA-q2vg-xgjr-32v3
  - https://www.sentinelone.com/vulnerability-database/cve-2026-1773/
- **Fuzzing research explicitly targets malformed APCI length as a fault class:**
  - "Fuzzing Framework for IEC 60870-5-104 Protocol" (ACM). https://dl.acm.org/doi/10.1145/3569966.3570026
  - Step Function I/O **Aegis** IEC-104 fuzzer has a dedicated `apci` procedure that "sends
    malformed I, U, or S frames with **disallowed lengths**" and a `random-frame` procedure
    prepending `0x68`. Notably: "You will almost certainly need to run each test case in its
    own TCP session" — i.e., malformed-length frames routinely kill/hang the session under
    test, confirming device-level DoS impact. https://docs.stepfunc.io/aegis/protocols/iec104/
  - University of Twente assessment (length-manipulation causing message-ignore behavior),
    and a commercial IEC-104 fuzzing/security-testing writeup listing "Parsing and Length
    Validation Errors → Crashes, Memory corruption" as a top vulnerability class.
    https://essay.utwente.nl/fileshare/file/72277/Kerkers_MA_EEMCS.pdf ,
    https://cytal.co.uk/protocols/iec-60870-5-104/

**T0814 (Denial of Service) applicability — SUPPORTED.** MITRE ATT&CK for ICS does not carry
an IEC-104-length-specific technique, but T0814 "Denial of Service" is the correct mapping:
the documented real-world impact of malformed IEC-104 length/frame handling (CVE-2023-5768
endless link blocking; CVE-2026-1773 service disruption; Aegis session kills) is exactly a
denial-of-service condition on the controlled station. ADR-013's choice of T0814 for the
malformed-LEN case is **evidence-grounded**, not arbitrary. (Note: CWE-184 is the more precise
weakness class for the U-format sibling; T0814 is the right ATT&CK technique for the observable
effect.)

**Q3 verdict:** Malformed LEN is a genuine DoS vector with a matching CVE. This argues
*against* BC-2.19.026's NO-FINDING (silently discarding a security-relevant anomaly), and *for*
recording it — the finding has real detection value.

---

## Q4 — Do monitors that alert on malformed frames rate-limit/dedup per flow, or alert per occurrence?

**Established monitors rate-limit / dedup — they do NOT alert per malformed frame.** This is
the decisive evidence for the flood concern.

- **Zeek weird framework — dedups by sampling.** Repeated identical weirds are throttled:
  `weird_sampling_threshold` (default **25**) events pass, then the weird enters a sampled
  phase emitting only 1-in-`weird_sampling_rate` (default **1000**), with state reset after
  `weird_sampling_duration` (default **24 s**). The notice framework adds further
  identifier-based suppression (`suppress_for`). Zeek deliberately avoids per-occurrence
  weird floods.
  - https://docs.zeek.org/en/lts/scripts/base/init-bare.zeek.html
  - https://docs.zeek.org/en/v8.0.4/frameworks/notice.html
- **Snort 3 IEC104 inspector — emit-then-stop.** On decode failure after a bad-length event,
  it resets flow state (`iec104fd->reset()`) rather than re-alerting per frame; operators
  further apply `event_filter`/`detection_filter` for rate governance.
  - https://github.com/snort3/snort3/tree/master/src/service_inspectors/iec104
- **Zeek/Spicy analyzer-violation path — one-shot per flow.** After a parse error the
  analyzer is disabled for that connection, so at most one violation/weird is generated per
  flow direction — structurally the same as "one finding per flow direction."
  - https://docs.zeek.org/projects/spicy/en/latest/programming/parsing.html

**Q4 verdict:** Best-practice monitors emit at most once per flow (or heavily sample), never
once per malformed frame. This directly maps to the proposed `EMIT-WITH-DEDUP` shape and
neutralizes the temporary flood risk even before the per-flow findings cap lands.

---

## Inconclusive / Flagged Areas

- **Commercial OT NSM (Nozomi / Claroty / Dragos)** byte-level malformed-LEN handling —
  no public source; behavior unverifiable. Marked inconclusive (Q2).
- **Standard-mandated TCP teardown on invalid LEN** — the premise "active endpoints close the
  connection" is **not** normatively confirmed; handling is implementation-defined (Q1).
  The passive-monitor decision should therefore not assume a single endpoint behavior to mirror.
- **Exact Zeek `weird_sampling_duration` default** — reported as 24 s in the reporter-BIF docs;
  cross-doc versions vary slightly. Value is illustrative of the dedup principle, not
  load-bearing for the recommendation.

---

## RECOMMENDATION

**EMIT-WITH-DEDUP** — emit exactly one T0814 (Denial of Service) finding **per flow direction**
(first out-of-range LEN observed on that direction), then advance past the 2-byte APCI stub and
resync silently for all subsequent malformed frames on that direction.

**Rationale:** The evidence resolves the contradiction in favor of a *middle path* that both
ADR-013 and BC-2.19.026 approximate but neither states cleanly. Malformed APCI length is a
verified DoS vector — CVE-2023-5768 shows a wrong APDU LEN causing endless link-layer blocking
on a real Hitachi RTU, and CVE-2026-1773 / the ACM+Aegis fuzzers confirm malformed IEC-104
frames crash or hang production stacks — so silently discarding it (BC-2.19.026 NO-FINDING)
throws away a genuinely security-relevant signal, and T0814 is the evidence-grounded technique
for the observed denial-of-service effect. At the same time, every inspectable production monitor
that flags this (Snort's `IEC104_BAD_LENGTH` emit-then-reset, Zeek's threshold+sampling weird
dedup, Spicy's one-violation-then-disable-per-connection) rate-limits rather than alerting per
frame, so unbounded per-frame emission (ADR-013 as literally written, dangerous while the per-flow
cap is absent) is neither necessary nor consistent with precedent. One finding per flow direction
preserves the detection value, matches the "at most once per flow" industry norm, and makes the
interim absence of the per-flow findings cap a non-issue.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source synthesis of IEC-104 LEN bounds, standard receiver-handling, monitor behavior, and attack-vector landscape (`reasoning_effort: high`) |
| Perplexity perplexity_search | 2 | Raw ranked URLs: Zeek/Spicy iec104 weird handling; IEC-104 malformed-length CVEs/fuzzing/DoS |
| Perplexity perplexity_ask | 1 | Factual lookup: Zeek weird_sampling_threshold/rate/duration defaults (dedup mechanism) |
| WebFetch | 2 | Snort3 `iec104.cc` source (IEC104_BAD_LENGTH validation/event); Snort3 iec104 directory |
| Read | 1 | ADR-013 (local artifact under contradiction) |
| Training data | 1 area | MITRE ATT&CK ICS T0814 technique semantics (cross-checked against CVE impact descriptions) |

**Total MCP tool calls:** 4 (1 research + 2 search + 1 ask) plus 2 WebFetch.
**Training data reliance:** low — every load-bearing claim (LEN bounds, CVE-2023-5768,
CVE-2026-1773, Snort `IEC104_BAD_LENGTH`, Wireshark expert info, Zeek weird sampling defaults,
Aegis/ACM fuzzers) is web-sourced with a cited URL. T0814 mapping is model knowledge validated
against the cited CVE impact language.
