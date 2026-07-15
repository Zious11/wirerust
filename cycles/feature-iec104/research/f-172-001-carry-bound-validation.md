---
document_type: research-validation
producer: research-agent
date: 2026-07-15
finding_id: F-172-001
subsystem: SS-19
feature_cycle: feature-iec104
related_artifacts:
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - BC-2.19.025
  - src/analyzer/dnp3.rs
recommendation: WALK-FIRST-RESIDUAL-BOUND
---

# F-172-001 — IEC-104 Carry-Buffer Bound Semantics: Adversarial-Finding Validation

## Purpose

Adjudicate the disputed semantics for the per-direction carry buffer
(`MAX_IEC104_CARRY_BYTES = 255`) when `carry.len() + delivery.len()` exceeds the bound:

- **(A) PRE-CHECK-DISCARD-ALL** (current BC-2.19.025): if the sum exceeds 255, discard the
  **entire** delivery before any frame extraction, keep prior carry, emit one T0814. The
  adversary (F-172-001) claims this (i) drops legitimate multi-frame bursts ≥ 256 B
  unparsed, (ii) drops the legitimate split-255-byte-APDU case, and (iii) opens a
  detection-evasion channel: an attacker pads a burst just over the bound to force the whole
  thing dropped unparsed, hiding a real attack frame at the head.
- **(B) WALK-FIRST-RESIDUAL-BOUND** (proposed): concatenate carry+delivery, extract **all**
  complete frames first, then bound only the residual partial-frame carry at 255. A
  spec-conformant partial frame is ≤ 254 B by construction (a 255-byte prefix would be a
  *complete* frame), so the bound is defensive/fail-closed and unreachable for conformant
  traffic.

Validated per DF-VALIDATION-001 before the contradiction is resolved in code/spec.

Note: the ADR-013 Decision 3 frame-walk loop (steps 1–7: prepend carry → walk complete
frames → stash residual) already describes a **walk-first** design. BC-2.19.025's
PRE-CHECK-DISCARD-ALL is the outlier that contradicts the ADR's own parser description, so
this validation also functions as an internal-consistency check.

---

## Q1 — How do established passive monitors bound protocol-layer reassembly / carry buffers, and what do they do when the bound trips?

**Answer: they degrade gracefully — parse what fits, then trim or disable the parser, and
emit an event/metric. Wholesale discard of the incoming delivery before parsing is NOT the
passive-mode behavior in any of the three; it appears only as an explicit inline-IPS *drop*
policy.** This directly contradicts semantics (A).

**Zeek.**
- TCP reassembler signals a missing region with the `content_gap` event rather than silently
  dropping; higher-layer analyzers decide whether to continue or abort, but transport-level
  tracking persists.
- The Files framework caps per-file reassembly buffer growth (`set_reassembly_buffer_size`);
  once the max is reached it stops *extending* the buffer (truncates / trims) — it does not
  discard already-seen bytes, and the connection stays tracked.
- Spicy-based analyzers enforce per-field/per-unit bounds with `&size` / `&max-size`;
  exceeding a bound raises a **parse error** → `analyzer_violation` event, aborting the
  current unit while leaving the raw stream visible to other analyzers.
- Category: (b) parse-what-you-can-then-trim at the stream/file layer; (c) disable-parser-
  for-unit at the Spicy layer. Always with an explicit event.
- Sources: docs.zeek.org `base/frameworks/files/main.zeek`; `base/bif/event.bif.zeek`
  (`content_gap`, `analyzer_confirmation`, `analyzer_violation`);
  docs.zeek.org/projects/spicy `programming/parsing.html` (`&size`, `&max-size`, `&requires`).

**Suricata.**
- `stream.reassembly.depth` (and per-protocol `stream-depth`, e.g. Modbus) caps how many
  bytes are reassembled; beyond the depth it **stops extending** the reassembled view
  (category b) — it does not drop the flow.
- Global reassembly **memcap** breaches are handled by configurable *exception policies*
  (`drop-flow`, `drop-packet`, `bypass`, `ignore`); `drop-flow` (category a/c) is an
  IPS-mode choice, not the default and not intrinsic to limit enforcement. Stats are kept for
  every exception state.
- libhtp caps HTTP request/response line+headers at ~18 kB; exceeding it **fails the parser
  and sets an app-layer event** (`http.request_field_too_long` /
  `http.response_field_too_long`, e.g. sid 2221018) and increments an anomaly counter —
  localized parse failure, flow not dropped (category b).
- Sources: docs.suricata.io `configuration/suricata-yaml.html`,
  `configuration/exception-policies.html`; redmine.openinfosecfoundation.org issue 986.

**Snort3.**
- `stream_tcp` reassembles into PDUs and normalizes overlaps (first-copy-seen in inline
  mode). The legacy `stream5` performance option "do not queue large packets in reassembly
  buffer" carries an explicit warning that it **"could result in missed packets"**, and the
  **default is to queue them** — i.e. the vendor treats skip-before-parse as unsafe.
- AppID classifies within a bounded payload window; exceeding it stops AppID for that flow
  (category c) while other inspectors continue.
- Sources: cisco.com Snort-3 Inspector Reference `stream-tcp-inspector`; blog.snort.org
  "Better application logging with Snort3"; docs.snort.org `start/configuration`.

**Cross-cutting conclusion.** All three favor graceful degradation + explicit signaling over
abrupt discard, and *reserve* wholesale drop for configured inline-IPS exception policies.
For a *passive* analyzer (wirerust's role) the correct analog is parse-what-fits + emit, i.e.
semantics (B). No surveyed monitor discards the *incoming delivery before parsing complete
frames already in the buffer*.

---

## Q2 — Is "discard entire delivery on buffer-bound pre-check before parsing" a documented false-positive / evasion source?

**Answer: the exact algorithm is not documented verbatim, but it is analyzable as an extreme
and insecure form of bounded reassembly that creates a clear evasion gap — and primarily a
FALSE-NEGATIVE (missed-detection) channel, which is worse than a false positive.** The
adversary's finding is CONFIRMED in substance.

- **Ptacek & Newsham (1998), "Insertion, Evasion, and Denial of Service: Eluding Network
  Intrusion Detection"** — foundational taxonomy. *Evasion* = data the host processes but the
  IDS does not see. Discarding a delivery the host will happily reassemble is textbook
  evasion: the monitor sees strictly less than the endpoint.
- **Moura et al. (2025), overlap-based reassembly study** — restates the same insertion/
  evasion dichotomy and shows all tested NIDS remain vulnerable to reassembly-divergence
  attacks; the core risk (host buffer ≠ monitor buffer) is unchanged after 25+ years.
- **Applied to (A):** an attacker places a malicious APDU at the head of a burst and appends
  benign padding so `carry+delivery` clears 256 B. Semantics (A) drops the *entire* delivery
  — including the head frame that fully fit — so the attack frame is never parsed, while the
  IEC-104 endpoint (which does its own TCP reassembly) processes all of it. This is *more*
  powerful than classic tail-beyond-depth evasion because it discards both the early and late
  bytes. It also produces the two legitimate-traffic false-drops the finding names (multi-
  frame bursts ≥ 256 B; the split-255-B APDU that only *looks* oversized mid-reassembly).
- **Documented analog:** Snort's own warning that not-queuing large packets "could result in
  missed packets" is a vendor admission that skip-before-parse is a false-negative source.
- **Design guidance from the literature to avoid the gap:** (i) *incremental inspection* —
  parse the portion already buffered rather than waiting on / discarding a whole delivery;
  (ii) *explicit gap/anomaly representation* rather than silent drop; (iii) *safe-by-default*
  — discard-whole-burst should never be a default and arguably should not be offered.
- Sources: Ptacek & Newsham 1998 (insecure.org/stf/secnet_ids/secnet_ids.html); Moura et al.
  2025 (overlapping-data NIDS study); Handley & Paxson, "Network Intrusion Detection: Evasion,
  Traffic Normalization, and End-to-End Protocol Semantics" (USENIX Security 2001).

---

## Q3 — For fixed-max-frame protocols (IEC-104 255 B, DNP3 292 B), is bounding the *residual partial-frame* buffer at exactly max-frame-size the established pattern?

**Answer: YES. Bounding the per-frame residual at the protocol max is the prevailing pattern,
and a residual that exceeds one max frame while still owned by a single in-progress frame is
treated as malformed / desynchronized — drop and resync by scanning for the next start byte.
The bound applies to the RESIDUAL, not to the aggregate delivery.**

- **Critical distinction (the crux of A vs B):** implementations separate the *aggregate
  stream buffer* (may legitimately hold many frames' worth of bytes across one delivery) from
  the *per-frame residual buffer* (the incomplete-frame tail, bounded at one max frame). The
  bound belongs on the residual. Semantics (A) mistakenly applies a per-frame-sized bound to
  the aggregate carry+delivery, which is why it drops legitimate multi-frame bursts.
- **Wireshark** IEC-104 (`packet-iec104.c`) and DNP3 (`packet-dnp.c`) dissectors rely on
  tvbuff bounds + the LEN field; overlong/inconsistent input is marked *malformed*, surplus
  bytes are handed to the next frame — never accumulated past one frame.
- **Zeek/Spicy** (`&max-size`) makes an oversized single unit a parse error by construction;
  the ICSNPP `IEC60870_5_104` Spicy analyzer parses the APDU as `start(0x68) + len +
  payload &max-size=253`, so a residual > one max frame is structurally impossible — the
  parser errors and resyncs first.
- **Resynchronization** across serial/industrial parsers and Spicy is uniformly *drop-and-
  rescan / skip-forward scan for the next start delimiter* (0x68 for IEC-104). This matches
  ADR-013 Decision 3 step 3 (1-byte advance to the next 0x68 candidate) and SR-172-03.
- Sources: Triangle MicroWorks DNP3 overview; Beckhoff IEC-104 APDU docs (LEN ≤ 253, ≤ 255
  total); lib60870 user guide; ICSNPP `IEC60870_5_104` (github.com/cisagov/icsnpp);
  docs.zeek.org/projects/spicy parsing docs; FlowFuse binary-parsing guide.

### Internal precedent — wirerust's own DNP3 analyzer (`src/analyzer/dnp3.rs`)

**wirerust's DNP3 analyzer already rejects PRE-CHECK-DISCARD-ALL on exactly the evasion
grounds F-172-001 raises.** In `Dnp3Analyzer::on_data` (Step 2, accumulate-with-cap):

- It appends up to the remaining capacity of `MAX_DNP3_FRAME_LEN = 292`, discards only the
  **excess bytes beyond the cap** (not the whole delivery), increments `parse_errors` once,
  performs an **inline byte-walk resync** to the next `[0x05,0x64]` (or clears), then
  **falls through to the frame-walk** — the code comment is explicit: *"Do NOT return early …
  Do NOT clear+return — that silently discards a recoverable head frame (F-B-002 detection-
  evasion DoS)."*
- The frame-walk (Step 3) is a `while` loop that consumes **every** complete frame from the
  head of the carry, draining `frame_len` per iteration, and leaves only the sub-frame
  residual — the residual is naturally ≤ one max frame because complete frames are drained.
- Malformed/overflow events increment `parse_errors` + `malformed_in_window` and route
  through `check_malformed_anomaly`, which is **threshold-gated (3-in-300 s) and one-shot
  deduplicated** (`malformed_anomaly_emitted`) — it does not emit per-byte or per-frame.

This is directly on point: the sibling analyzer (a) never discards a whole delivery, (b)
preserves recoverable head frames, (c) resyncs by start-byte scan, and (d) emits a
threshold+dedup anomaly rather than flooding. Adopting (A) for IEC-104 would make SS-19
internally inconsistent with SS-15 (DNP3) and re-introduce the F-B-002 evasion DoS that DNP3
was explicitly hardened against.

---

## Q4 — When the bound IS exceeded (only possible for non-conformant / attack traffic under B), what is the standard reaction?

**Answer: clear the residual carry for that direction and resync (fresh start, NOT a flow-
wide desync latch), emit a deduplicated anomaly, and keep the flow tracked. Do not drop the
flow, do not permanently disable the analyzer.**

- **Carry:** clear the offending direction's residual and byte-walk-forward to the next
  0x68 (drop-and-rescan). This is a fresh-start resync, not the `is_non_dnp3`-style permanent
  desync latch — a single overflow must not blind the analyzer to the rest of the flow.
- **Finding:** emit T0814 (Anomaly/DoS) with **per-direction dedup** (one finding per flow
  direction), consistent with ADR-013 Decision 3 step 4's ratified EMIT-WITH-DEDUP for
  malformed-LEN (SR-172-03) and with DNP3's threshold+one-shot `check_malformed_anomaly`.
  Per-frame/per-byte emission is rejected — it floods on junk traffic (same rationale as the
  bad-start-byte silent-resync in ADR-013 step 3).
- **Flow tracking:** preserved. Passive monitors keep transport-level visibility even when a
  parser bound trips (Zeek `content_gap`, Suricata stats, Snort default-queue). Dropping the
  flow is an inline-IPS-only behavior with no analog in a passive analyzer.
- **Reachability note:** under (B) this branch is *unreachable for spec-conformant traffic*
  (a 255-byte residual prefix is a complete frame and would already have been walked off), so
  the bound is a defensive fail-closed guard. That is the intended posture — it fires only on
  genuinely malformed/desynchronized or adversarial input, exactly when an anomaly finding is
  warranted.

---

## Inconclusive / caveats

- The IEC 60870-5-104 standard does not *normatively prescribe* a receiver reaction to
  oversized/desynchronized framing (carried over from SR-172-03); the resync-and-continue
  reaction is best-practice convergence across implementations, not a spec mandate.
- Spicy/ICSNPP internal buffering specifics are inferred from the declarative `&max-size`
  model and public analyzer structure, not from line-level source audit of ICSNPP.
- Moura et al. 2025 is cited for the persistence of reassembly-divergence evasion generally;
  it does not test this exact discard-whole-delivery algorithm (no monitor implements it).

---

## RECOMMENDATION

**WALK-FIRST-RESIDUAL-BOUND** (semantics B).

**Rationale.** (1) It is the only option consistent with ADR-013 Decision 3's own walk-first
loop description — (A) contradicts the ADR. (2) It matches wirerust's own DNP3 precedent,
which was explicitly hardened *away* from clear-and-discard on evasion-DoS grounds (F-B-002);
adopting (A) would re-open that hole and split SS-19 from SS-15. (3) It eliminates the two
legitimate false-drops (multi-frame bursts ≥ 256 B; split-255-B APDU) and closes the
Ptacek/Newsham-class evasion channel the finding identifies. (4) It aligns with how Zeek,
Suricata, and Snort3 all bound buffers (parse-what-fits + emit, never discard-before-parse in
passive mode) and with the universal fixed-frame pattern of bounding the *residual*, not the
aggregate. BC-2.19.025 should be revised from PRE-CHECK-DISCARD-ALL to WALK-FIRST-RESIDUAL-
BOUND.

**Recommended bound-trip reaction.** Clear the offending direction's residual carry and
resync forward to the next 0x68 (drop-and-rescan; fresh start, no permanent desync latch);
emit **one T0814 anomaly per flow direction with a per-direction dedup flag** (EMIT-WITH-
DEDUP, matching Decision 3 step 4 and DNP3's threshold/one-shot pattern); keep the flow
tracked and the analyzer active. Under (B) this path is unreachable for conformant traffic,
so the 255-byte bound stands as a defensive fail-closed guard that fires only on
malformed/adversarial input.
