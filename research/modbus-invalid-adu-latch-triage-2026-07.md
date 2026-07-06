# Triage: Modbus "invalid-ADU latch" silent-failure claim (DF-VALIDATION-001)

- **Date:** 2026-07-06
- **Mode:** Read-only validation. No code changed, no GitHub issue filed.
- **Target:** develop HEAD (`a85c6f7`), `src/analyzer/modbus.rs:1121-1135`
- **Prior audit claim:** When an invalid Modbus ADU is detected, code sets
  `is_non_modbus = true` and increments `parse_errors` but emits NO Finding,
  whereas the DNP3 analyzer emits a T0814-equivalent Finding for the analogous
  condition — a genuinely-reachable silent quarantine of a flow.

## Verdict summary

**NOT a genuine observability gap. Claim is REJECTED as stated.**

The claim rests on a false analogy. The Modbus latch is (a) **not silent** — it
increments `parse_errors`, which is surfaced in the summary, terminal, and JSON
outputs — and (b) the DNP3 code it is compared against does **not** emit a
T0814 Finding for the analogous condition either. The correct DNP3 analogue
(the `is_non_dnp3` initial desync bail) is likewise silent-and-latched with no
Finding. DNP3's T0814 malformed-frame anomaly fires for a *different* condition
(≥3 malformed frames on an **established** flow within a 300s window), which has
no bearing on this quarantine latch.

## 1. Is the site real and reachable? (YES — reachable, not dead code)

`src/analyzer/modbus.rs:1121-1135`, inside the ADU walk loop in
`on_data` → the `!is_valid_modbus_adu(&header)` arm:

```rust
if !is_valid_modbus_adu(&header) {
    flow.is_non_modbus = true;
    flow.carry_c2s.clear();
    flow.carry_s2c.clear();
    self.parse_errors += 1;
    break;
}
```

**Concrete triggering traffic.** `is_valid_modbus_adu` is the 3-point validity
gate (BC-2.14.003/004). This arm latches when a **fully-parsed MBAP header**
(≥8 bytes present, so `parse_mbap_header` returned `Some`) fails validity:

1. `protocol_id != 0x0000` — a non-Modbus/TCP stream on port 502 (well-specified
   Modbus deviation), OR
2. `header.length` outside `[2, 254]` — a structurally-impossible Modbus PDU
   length that no conforming device would emit.

This is plainly reachable: any port-502 flow whose first well-formed 8-byte
header carries a bad protocol_id or an out-of-range length hits it. It is the
documented `F-DELTA-003` fix path, exercised by tests. **Not dead code.**

Note the sibling latch sites at `1110-1112` (partial-header carry-cap overflow)
and `1162-1164` (partial-ADU carry-cap overflow) are documented UNREACHABLE in
the current clear-then-stash structure (RULING-MODBUS-SIBLING-001 addendum) and
retained as defensive future-proofing. Those are not the site under audit.

Once latched, the desync bail at `modbus.rs:1046-1049` short-circuits every
subsequent `on_data` for the flow (no parsing, no further `parse_errors`
inflation) — matching BC-2.14.003 / Decision 6 desync policy.

## 2. When it latches, is anything surfaced? (YES — `parse_errors`, not silent)

The `self.parse_errors += 1` increment at line 1134 is **observable end-to-end**:

- `ModbusAnalyzer::summarize` (`modbus.rs:939-983`, BC-2.14.021) emits a
  six-key `AnalysisSummary.detail`, including `"parse_errors"` (line 956-959).
- Terminal reporter `src/reporter/terminal.rs:269-275` iterates
  `asummary.detail` and renders **every** key/value pair per analyzer — so
  `parse_errors` appears in TTY output.
- JSON reporter `src/reporter/json.rs` serializes the same `AnalysisSummary`
  under the `analyzers` array — `parse_errors` appears in machine output.

So the *event* is surfaced as a lifetime parse-error counter increment visible
in both terminal and JSON summaries. It is **not** fully silent. What is not
emitted is a per-flow *Finding* — which is a deliberate design choice, not an
omission (see §4).

## 3. DNP3 comparison — does DNP3 emit a Finding for the analogous condition? (NO)

The prior audit conflated two distinct DNP3 mechanisms.

**(a) The true analogue — DNP3 initial desync bail (`is_non_dnp3`).**
`src/analyzer/dnp3.rs:404-435`. On the first delivery, if the bytes do not begin
with the sync word `[0x05, 0x64]`, DNP3 does:

```rust
flow.is_non_dnp3 = true;
return;
```

This is the direct structural mirror of the Modbus `is_non_modbus` latch: a
flow-level "this is not my protocol" quarantine. It emits **NO Finding**, and
notably does **not even increment `parse_errors`** (it is a cleaner no-op than
the Modbus path, which at least counts the event). So for the genuinely
analogous condition, DNP3 is *less* observable than Modbus, not more.

**(b) The T0814 the audit cited — a DIFFERENT condition.**
`Dnp3Analyzer::check_malformed_anomaly` (`dnp3.rs:1650-1704`, MITRE T0814,
BC-2.15.024) pushes ONE low-confidence Anomaly Finding only when
`malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD` (=3) within a 300s
correlation window on an **established** DNP3 flow (frames that pass the initial
sync gate but then fail structural checks: sync-loss resync at `dnp3.rs:634-642`,
invalid LENGTH at `675-703`, carry overflow at `478-497`). Its evidence string
is `"malformed_in_window={count} ... threshold=3"` and it is one-shot per window.

This is a *sustained-crash-probe correlation* signal ("possible Crain-Sistrunk
crash-probe"), not a single-frame protocol-quarantine event. Modbus has no
established-flow malformed-frame stream to correlate here — the invalid-ADU gate
fires on the header and immediately quarantines the whole flow. The two are not
the same condition; the T0814 comparison is a category error.

(The other DNP3 T0814 site, `detect_restart_split` at `dnp3.rs:1056-1069`, is a
restart-command detection — entirely unrelated.)

## 4. Final verdict, severity, and recommendation

**Verdict: ACCEPTABLE — reject the finding as a silent-failure bug.**

- The event is surfaced via `parse_errors` in summary/terminal/JSON. Not silent.
- The DNP3 code the audit invoked does NOT emit a Finding for the analogous
  condition; the true DNP3 analogue (`is_non_dnp3`) is itself Finding-less (and
  counter-less). The premise "DNP3 emits a T0814-equivalent for this" is false.
- Emitting a per-quarantine Finding would be *inconsistent* with the established
  cross-analyzer convention (Modbus non-Modbus latch, DNP3 non-DNP3 bail, and by
  extension ENIP RULING-137-002) that protocol-mismatch quarantine is a
  counter-level event, not a security Finding. A "not my protocol on this port"
  event is an operational/observability signal, not a threat detection.

**Severity if one insisted on treating it as a gap: INFORMATIONAL (P4) at most.**
An analyst reading the Modbus summary sees `parse_errors > 0`, which correctly
signals malformed/non-Modbus traffic on the port. No analyst is *misled* into
believing a clean flow was analyzed — the counter is the honest signal.

**Optional, non-blocking enhancement (NOT a required fix):** If richer
observability is desired, the most consistent improvement would be a *per-flow*
diagnostic counter (e.g. `flows_quarantined_non_modbus`) added to the Modbus
summary detail — mirroring a possible parallel enhancement to DNP3's silent
`is_non_dnp3` bail (which today lacks even a counter). This would improve BOTH
analyzers symmetrically and preserve the "no security Finding for protocol
mismatch" convention. It should be scoped as a small observability story, not a
bug fix, and applied to Modbus and DNP3 together to avoid re-introducing the
very asymmetry this audit mistakenly perceived. A diagnostic *Finding* is NOT
recommended — it would break cross-analyzer consistency and risk alert noise on
benign port-502 misuse.

**Action:** Do NOT file a silent-failure bug issue. If the observability
enhancement is deemed worthwhile, file it as a low-priority `test`/`feat`
observability story covering Modbus + DNP3 symmetrically, referencing this note.

## Evidence index

| Claim | Source |
|-------|--------|
| Modbus invalid-ADU latch + parse_errors increment | `src/analyzer/modbus.rs:1121-1136` |
| Modbus desync short-circuit on subsequent calls | `src/analyzer/modbus.rs:1046-1049` |
| Modbus parse_errors in six-key summary | `src/analyzer/modbus.rs:956-959` (BC-2.14.021) |
| Summary detail rendered in terminal | `src/reporter/terminal.rs:269-275` |
| Summary detail rendered in JSON | `src/reporter/json.rs` (analyzers array) |
| DNP3 true analogue: is_non_dnp3 silent bail (no Finding, no counter) | `src/analyzer/dnp3.rs:404-435` |
| DNP3 T0814 malformed-anomaly (DIFFERENT condition: ≥3-in-300s established flow) | `src/analyzer/dnp3.rs:1650-1704` (BC-2.15.024) |
| DNP3 T0814 restart detection (unrelated) | `src/analyzer/dnp3.rs:1056-1069` (BC-2.15.011) |

## Note on methodology

This triage was resolved entirely by reading the source and behavioral-contract
references on develop HEAD; the claim is falsified by direct code inspection
(the cited DNP3 comparison does not hold), so no external MCP research was
required to reach a confident verdict. External research would not change a
determination grounded in the repository's own code and BC references. This is a
DF-VALIDATION-001 code-triage, not a technology/domain research report.
