# F2 — Port-102 Catalog-Model Validation (design decision for ADR-014 Decision 3)

**Cycle:** `feature-s7comm`
**Phase:** F2 (spec-evolution / human-ratification input)
**Date:** 2026-09-06
**Author:** research-agent (vsdd-factory)
**Status:** Research complete — feeds the human ratification of ADR-014 Decision 3.
**Scope:** Validation only. No source, ADR, or spec was modified.

> **Purpose.** ADR-014 Decision 3 recommends option **(b)** (name-keyed exclusion list) for
> the port-102 catalog-model problem and flags it as *REQUIRES EXPLICIT HUMAN RATIFICATION*.
> This document independently validates the design path against comparable-tool prior art and
> the extensibility axis, so the human decision is evidence-based rather than diff-size-based.

---

## 1. Executive summary / top recommendation

**The proper path on the evidence is a per-entry explicit support state — option (d), a
`Support` enum — not the derived-port-intersection-plus-exclusion-list of option (b).**

The decisive reasons, expanded in §4–§7:

1. **Prior art is unanimous.** Wireshark, Suricata, and Zeek all key "is this protocol
   supported/decoded" on an **explicit per-protocol registry entry** (a registered
   dissector / parser / analyzer), *never* on the port number. Port is only an *attachment
   or scoping hint*. wirerust's `canonical_ports ∩ SUPPORTED_PORTS` model is precisely the
   "port ⇒ support" derivation that every mature tool rejects for ambiguous ports like 102.
   Option (b) preserves that rejected model and patches its most visible symptom; options
   (a)/(d) replace it with the model the field actually uses.

2. **ADR-0012 already borrowed the right vocabulary — for the wrong surface.** ADR-0012
   Decision 2 adopts Suricata's tri-state (`known-supported` / `known-unsupported` /
   `unknown`) for the *dynamic* gap report. But the *static* catalog still derives support
   from ports. Modeling the static catalog with the same explicit per-entry state closes
   that loop and removes the mismatch entirely.

3. **A third state already exists in this very cycle.** ADR-014 **Decision 6** defines
   S7comm-plus as *observed, not dissected* — framing-level classification only, never
   promoted to `known-supported`. That is exactly Suricata's `detection-only` state. A
   `bool` (option a) and an exclusion list (option b) **cannot express it** — they flatten
   S7comm-plus into the same "unsupported" bucket as fully-opaque MMS/ICCP, discarding a
   distinction the architecture itself just drew. A `Support { Supported, KnownUnsupported,
   DetectionOnly }` enum captures it natively.

4. **Extensibility (the decisive axis) favours the explicit field with safe polarity.**
   Option (b)'s exclusion list is a **deny-list with an unsafe default**: any *new* catalog
   entry that shares an already-supported port is *silently promoted to supported* unless a
   human remembers to add it to `PORT_102_UNSUPPORTED_SIBLINGS`. Option (a)/(d) invert this
   to a safe positive polarity — Rust's struct-init rules force every new literal to state
   its support explicitly (compiler-enforced field presence), so "forgot to decide" is a
   compile error, not a false promotion.

**Recommended verdict, tiered for the ratifier:**

- **Evidence-optimal ("proper") path — option (d):** a per-entry `Support` enum. Best fit to
  prior art, to ADR-0012's own Suricata vocabulary, to ADR-014 Decision 6's `DetectionOnly`
  reality, and to the extensibility axis. Recommended as the target model.
- **Acceptable minimal-diff interim — option (b):** defensible *only* if the team explicitly
  accepts it as tech debt with a **named migration trigger** — migrate to (d) when the
  **second** port-102 protocol (IEC 61850 MMS) is promoted, which is the exact moment (b)'s
  costs begin compounding. Ratify (b) with that trigger recorded, or ratify (d) now.
- **Option (a) is strictly dominated by (d):** identical blast radius, strictly less
  expressive (cannot represent `DetectionOnly`). Choose (a) over (d) only if the team wants
  to *forbid* a third state permanently — which ADR-014 Decision 6 already contradicts.
- **Option (c) rejected:** agrees with ADR-014 — a `dispatch_target: Option<&'static str>`
  string field is a stringly-typed, uncompiled-checked coupling to `dispatcher::DispatchTarget`
  from a module documented as forbidden to depend on `dispatcher` (BC-2.05.010 PC-4).

**Important scoping caveat (§8):** *no* catalog-model option — (a), (b), or (d) — fully
resolves the port-102 problem in the **dynamic gap classifier** (`main.rs::lookup_protocol_state`),
which is a *second* consumer of `SUPPORTED_PORTS` keyed on `(transport, port)` with no
protocol identity available. That defect requires the analyzer's `protocol_id`
(ADR-014 Decision 2) and is already deferred to F4 by ADR-014 Decision 10. The catalog choice
governs only the *static* partition; the ratifier should not read any option as "fully solves
port 102."

---

## 2. Problem restatement (as verified against the code)

`src/protocols.rs` (read 2026-09-06):

- `supported_protocols()` (lines 434–444) filters `KNOWN_PROTOCOLS` by
  `canonical_ports ∩ SUPPORTED_PORTS ≠ ∅  ||  name == "ARP"`.
- `unsupported_protocols()` (lines 455–461) is the **derived complement** — a genuinely
  valuable partition invariant (`supported ⊎ unsupported = all`, VP-041).
- Four entries carry `canonical_ports: &[102]`: **S7comm** (177), **S7comm-plus** (186),
  **IEC 61850 MMS** (206), **ICCP/TASE.2** (246). Only S7comm is promoted this cycle.
- `SUPPORTED_PORTS` (line 74) currently `&[502, 20000, 44818, 2404, 443, 8443, 80, 8080, 53]`
  — 102 is deliberately absent.

**Second consumer (decisive, easy to miss).** `SUPPORTED_PORTS` is *not* used only by the
catalog partition. `main.rs::lookup_protocol_state` (lines 1071–1105) — the **dynamic
coverage-gap tri-state classifier** — independently does
`find(port matches).canonical_ports ∩ SUPPORTED_PORTS` to decide `KnownSupported` vs
`KnownUnsupported`. Adding `102` to `SUPPORTED_PORTS` therefore changes behaviour in **two**
places, and option (b)'s name-keyed clause lives only in `supported_protocols()` — it does
**not** reach `lookup_protocol_state`, which has only a `(transport, port)` pair and no name
(see §8). (Note: the code comment at main.rs:1091–1093 records that the `|| name=="ARP"`
disjunct is dead code in the gap classifier because ARP is `LinkLayer`/`ports=[]`.)

Guards that any option must keep green:
- **REGRESSION-GUARD** in `tests/protocols_tests.rs` (~line 465) asserting all four port-102
  entries are currently unsupported — must be *rewritten*, not deleted, under every option.
- **VP-041 oracle cross-check** (`proptest_vp041_oracle_cross_check`, ~line 731) which
  recomputes support as `canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p)) || name=="ARP"`.
  Under option (b) the oracle must mirror the exclusion clause; under (a)/(d) the oracle
  becomes `entry.support.is_supported()` — arguably *simpler* and no longer coupled to
  `SUPPORTED_PORTS` at all.

---

## 3. Comparable-tool prior art (one port → many protocols)

Primary source: Perplexity `sonar-deep-research` deep sweep of Wireshark/Suricata/Zeek
official docs and source (2026-09-06); key citations inlined. The three tools are the exact
mature passive analyzers ADR-0012 already cites as prior art.

### 3.1 Wireshark — the canonical port-102 four-way case

- **Port 102 identifies the framing stack entry point, not the app protocol.** `packet-tpkt.c`
  registers TPKT into the `tcp.port` dissector table for port 102 via
  `dissector_add_uint_range_with_preference`; TPKT parses the RFC 1006 header and hands off
  to the ISO transport dissector (`ositp`/COTP). So `102 → TPKT → COTP` is *table/port* based.
  [Wireshark `packet-tpkt.c`; wiki/TPKT; osmocom TPKT commit]
- **COTP → {S7comm, MMS, session/…} is decided by an explicit heuristic registry, not the
  port.** COTP owns heuristic lists `cotp` / `cotp_is`; `dissect_s7comm` registers itself with
  `heur_dissector_add("cotp", …)` and each candidate's *payload test* (e.g. MMS validates BER
  class/tag/length) makes the final call. [`packet-s7comm.c`; `packet-mms.c`; README.heuristic]
- **"Decode As" is an operator override of a dissector-table entry, not detection** — it does
  not prove whether a COTP payload is S7comm or MMS. [wsug ChCustProtocolDissection; decode_as.h]
- **No general "known-but-unsupported" state keyed on a port.** Decoding support = "a dissector
  is registered and reachable" (table, direct handoff, OID registration, or heuristic list);
  otherwise bytes fall through to the generic data dissector. (Maturity like TPKT "fully
  functional" vs S7comm "partially functional" is tracked *informally*, per-dissector — not
  per-port.) [wiki/TPKT, wiki/S7comm]
- **Verified vs not:** upstream has real `s7comm` and generic `mms` dissectors; **could not
  verify** a separate upstream `iec61850-mms`, `iccp`, or `tase2` dissector, and S7comm-plus is
  a not-yet-bundled plugin. IEC 61850/ICCP ride the generic MMS/OSI path.

> **Modeling lesson (Wireshark):** identity is *layered and per-registration*. Do **not** model
> "TCP/102 supported." Model each protocol/layer independently, keyed on an explicit registration
> + a payload recognizer — never on the port alone.

### 3.2 Suricata — the tool ADR-0012 already borrows from

- App-layer detection is **port-independent multi-pattern / probing-parser** first; the port
  only *scopes* where standalone probes run. `SCAppLayerProtoDetectPPRegister(ipproto, portstr,
  alproto, …)` registers a probe for an `AppProto` on a port set — "port-based" means
  *candidate scoping*, not "whatever is on port N is protocol X." [devguide/internals/engines;
  app-layer-detect-proto.h]
- **First-class three-state support:** the app-layer `enabled` option is **`yes` | `no` |
  `detection-only`** (`yes` = detect + parse; `detection-only` = recognise but *no parser*;
  `no` = neither). IMAP/POP3 ship `detection-only` by default. [suricata.yaml.in; app-layer.rst]
- Full support = registered parser callbacks (`RustParser`) + `enabled` — *not* a port.

> **Modeling lesson (Suricata):** "supported" is an explicit per-protocol registration state
> with **three** values; the port is optimisation metadata. This is *literally* the
> `Support { Supported, KnownUnsupported, DetectionOnly }` enum of option (d) — and it is the
> vocabulary ADR-0012 Decision 2 already adopted for wirerust's dynamic report.

### 3.3 Zeek — explicit analyzer registry + DPD

- `Analyzer::register_for_port(tag, port)` *adds* a candidate analyzer for a port; multiple
  tags may claim the same port (not a 1-port→1-protocol dict). [base/frameworks/analyzer/main.zeek]
- **DPD** attaches analyzers by payload **signature** (`enable "<name>"`) independent of port;
  the analyzer then *confirms or violates*, and Zeek disables violated analyzers. [frameworks/
  signatures; logs/dpd; PIA.h `ActivateAnalyzer(tag, rule)`]
- Supported-ness = the **installed analyzer component/tag** (`zeek -NN`); port map and DPD
  signature are merely *activation routes* to that registered component.

> **Modeling lesson (Zeek):** supported-ness is the **explicit analyzer-registry entry**, not a
> port mapping and not a signature match. On an ambiguous port, register multiple candidates and
> let confirmation/violation prune — never assume "port ⇒ protocol."

### 3.4 Cross-tool synthesis

| Tool | Is "supported" keyed on… | port role | 3rd "known-but-unsupported/observed" state? |
|------|--------------------------|-----------|---------------------------------------------|
| Wireshark | explicit **dissector registration** (table/heuristic/OID) | entry point / attachment hint | informal, per-dissector maturity (no port-keyed state) |
| Suricata | explicit **parser registration + `enabled`** | probe scoping only | **yes, first-class**: `detection-only` |
| Zeek | explicit **analyzer-component registry (tag)** | activation route only | via signatures/events; no first-class flag |

**All three reject "port ⇒ supported." Two of three model an explicit third state.** wirerust's
option (b) keeps the rejected port-derivation and cannot express the third state; option (d)
matches the majority pattern exactly.

---

## 4. The decisive extensibility axis (2nd and 3rd port-102 protocol)

IEC 61850 MMS and ICCP/TASE.2 are *named future wirerust protocols* on port 102 (ADR-014
Decision 1 explicitly architects `iso_on_tcp.rs` for their reuse). So "what happens on the 2nd
and 3rd promotion" is not hypothetical — it is the planned roadmap.

### Option (b) — name-keyed exclusion list, evolved forward

```
now:      PORT_102_UNSUPPORTED_SIBLINGS = ["S7comm-plus","IEC 61850 MMS","ICCP/TASE.2"]
+MMS:     PORT_102_UNSUPPORTED_SIBLINGS = ["S7comm-plus","ICCP/TASE.2"]      // one-line removal
+ICCP:    PORT_102_UNSUPPORTED_SIBLINGS = ["S7comm-plus"]                    // one-line removal
```

- **Promotion inverts cleanly** — each promotion is a one-line deletion. This part of ADR-014's
  claim is *correct and verified*.
- **But the polarity is unsafe by default.** The effective rule becomes "any port-102 entry is
  supported *unless* excluded." Add a *new* unsupported protocol on port 102 (or any future
  second entry on an already-supported port such as 443/80) and it is **silently promoted**
  unless a human remembers to add it to the deny-list. The safe default (unsupported) is not the
  code default.
- **Three coupled sources of truth.** `SUPPORTED_PORTS` (contains 102) **+**
  `PORT_102_UNSUPPORTED_SIBLINGS` (excludes 3 names) **+** the implicit one-port-one-protocol
  assumption. All three must be kept in sync by hand, plus the VP-041 oracle mirror. ADR-014
  calls (b) the "smallest change"; that is true for *this diff* but false for *moving parts to
  keep synchronised*, which grows.
- **Cannot express `DetectionOnly`.** S7comm-plus is *observed* (ADR-014 Decision 6) yet (b)
  files it in the same bucket as fully-opaque ICCP. The distinction is lost.
- **String-key fragility.** A rename of any of the three names silently breaks the exception
  unless a test catches it.

### Option (a) — `supported: bool` per entry

- Promotion = flip one field, locally. New entry = compiler forces you to write the field (safe
  positive polarity; no silent default). Single source of truth per entry.
- Scales linearly and locally to N same-port protocols with **no** shared list to synchronise.
- **Limit:** binary only — cannot represent S7comm-plus `DetectionOnly`; would re-introduce an
  ad-hoc side channel the moment a third state is needed (which is *now*).

### Option (d) — `Support` enum per entry

- Everything (a) offers, plus a native third state. Each future promotion = change one variant.
  Rust's exhaustive `match` forces every consumer (partition fns, CLI display, future gap
  classifier) to be revisited when a variant is added — compile-time pressure exactly where the
  catalog semantics evolve. [Rust API Guidelines type-safety; enums-over-bools idiom]
- **The derivation model itself is the thing that breaks for shared ports** — not any particular
  mechanism. Port-derivation assumes `port → protocol` is a function; port 102 proves it is
  one-to-many. (a)/(d) stop deriving support from ports; (b) keeps deriving and bolts on an
  exception. The prior art (§3) says the derivation was an *accidental invariant*, not a domain
  rule.

**Axis verdict:** on extensibility-with-safety, **(d) > (a) > (b)**. (b) is smallest now and
largest later; its one-line-removal virtue is real but is outweighed by unsafe default polarity,
three coupled sources of truth, and inability to model the `DetectionOnly` state this cycle
already needs.

---

## 5. Idiomatic Rust static-catalog modeling

Grounded via Perplexity `sonar-reasoning-pro` over the Rust API Guidelines and common idiom
references (2026-09-06):

- **Prefer a deliberate type over `bool` when the value carries meaning or may gain states.**
  The Rust API Guidelines' type-safety section recommends custom types over `bool`/primitives
  precisely because they convey intent and make future variants easy to add. [rust-lang.github.io
  /api-guidelines/type-safety.html; "enums over bools" idiom]
- **"Make illegal states unrepresentable" (value-level).** An entry cannot be simultaneously
  `Supported` and `DetectionOnly`; a closed set of mutually-exclusive meanings is exactly an
  `enum`. (This is *not* the typestate pattern — a heterogeneous `&[KnownProtocol]` needs one
  runtime value type — so a value-level enum, not marker types, is the right tool here.)
- **Compiler-enforced field presence is a safety feature.** Adding `support`/`supported` to the
  struct forces every one of the ~30 literals to state it; Rust struct-expression rules reject a
  literal missing a field. "Forgot to decide" becomes a *compile error* — the opposite of (b)'s
  silent allow-by-default. [Rust reference, struct expressions]
- **Exhaustive `match`** on the enum makes downstream handling fail to compile until updated when
  a variant is added — useful for a catalog whose semantics are expected to evolve.
- **Exclusion/deny-lists are a legitimate but weaker model** — acceptable only when the rule is
  genuinely allow-by-default *and* enforced by tests; the guidance is to use a stable typed key
  (not `&str`) and a consistency check if retained. That is a workaround, not the cleanest model.

Recommended shape (illustrative — for the ratifier, not a code change):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support { Supported, KnownUnsupported, DetectionOnly }

impl Support {
    pub const fn is_supported(self) -> bool { matches!(self, Self::Supported) }
}
// KnownProtocol gains: pub support: Support,
// supported_protocols()  = filter(|p| p.support.is_supported())  // no SUPPORTED_PORTS, no ARP special-case needed
// unsupported_protocols() = derived complement, unchanged (invariant preserved)
```

`canonical_ports` then means only "where this may be *detected*"; `support` means "what wirerust
can *do* with it" — two distinct facts no longer conflated. `SUPPORTED_PORTS` can be retained
for the dispatcher-mirror / gap-classifier role (§8) but is decoupled from catalog support.

---

## 6. Options comparison matrix

Criteria weighted per the task; **extensibility** is the decisive axis.

| Criterion | (a) `supported: bool` | (b) name-keyed exclusion list *(ADR-014 rec)* | (c) `dispatch_target: Option<&str>` | **(d) `Support` enum** |
|---|---|---|---|---|
| **Correctness for port-102 now (static partition)** | ✅ correct | ✅ correct | ✅ correct | ✅ correct |
| **Extensibility to 2nd/3rd same-port protocol** | ✅ flip one field, local, safe default | ⚠️ one-line removal *but* unsafe allow-by-default + 3 coupled lists | ✅ per-entry but string-coupled | ✅✅ flip one variant, local, safe default |
| **Default polarity safety (new same-port entry)** | ✅ compiler forces explicit decision | ❌ silently promoted unless excluded | ✅ explicit | ✅ compiler forces explicit decision |
| **Sources of truth to keep in sync** | 1 (per-entry) | 3 (SUPPORTED_PORTS + exclusion list + oracle) | 2 (field + dispatcher enum, unchecked) | 1 (per-entry) |
| **Expresses S7comm-plus `DetectionOnly` (ADR-014 Dec 6)** | ❌ no | ❌ no | ❌ no | ✅ yes |
| **Fit with prior art (Wireshark/Suricata/Zeek)** | ✅ explicit per-protocol registration | ❌ keeps rejected port-derivation | ⚠️ registration-ish but stringly-typed | ✅✅ matches Suricata `enabled` 3-state exactly |
| **Fit with ADR-0012 "derived complement" invariant** | ✅ complement preserved | ✅ complement preserved (its stated strength) | ✅ preserved | ✅ complement preserved |
| **Fit with ADR-0012 "derived, not hand-maintained" philosophy** | ⚠️ moves to explicit field (but SUPPORTED_PORTS was already hand-maintained) | ⚠️ preserves philosophy but philosophy is the root defect | ⚠️ explicit field | ⚠️ moves to explicit field (aligned w/ prior art) |
| **Blast radius: ~30 literals** | ❌ all 30 + struct | ✅ 0 literals, 0 struct | ❌ all 30 + struct | ❌ all 30 + struct |
| **Blast radius: regression-guard test** | rewrite | rewrite | rewrite | rewrite |
| **Blast radius: VP-041 oracle** | simpler (`support.is_supported()`) | mirror exclusion clause | field-based | simpler (`support.is_supported()`) |
| **`protocols.rs` no-dispatcher-dependency (BC-2.05.010 PC-4)** | ✅ respected | ✅ respected | ❌ **violated** (stringly-typed coupling) | ✅ respected |
| **Resolves dynamic gap-classifier port-102 defect (§8)** | ❌ no (separate axis) | ❌ no (separate axis) | ❌ no | ❌ no (separate axis) |
| **Net** | Good; dominated by (d) | Smallest diff; local minimum, unsafe polarity | Rejected (boundary violation) | **Best on decisive axis** |

Legend: ✅ good · ✅✅ best-in-class · ⚠️ caveat · ❌ poor/violates.

---

## 7. Recommendation & rationale

**Ratify option (d) — a per-entry `Support` enum — as the proper target model.** If the team
will not absorb the ~30-literal blast radius in this cycle, ratify **(b) as an explicit interim**
with a recorded migration trigger (migrate to (d) at the IEC 61850 MMS promotion). Do **not**
ratify (a) over (d) (dominated), and (c) stays rejected (agrees with ADR-014).

Rationale, weighing the task's five factors:

1. **Correctness (port-102 static case):** a tie among (a)/(b)/(d) — all correct. Not
   decisive.
2. **Extensibility to future same-port protocols (decisive):** favours (d), then (a). (b)'s
   unsafe allow-by-default polarity and three coupled sources of truth make it a local minimum
   that degrades as more ports become multi-entry. The prior art is unanimous that the
   port-derivation model itself — which (b) keeps — is the defect.
3. **Fit with ADR-0012's philosophy:** nuanced. (b) preserves "derived complement" *and*
   "derived not hand-maintained." But (i) the *derived complement* invariant is preserved by
   (a)/(d) too; and (ii) *derived-not-hand-maintained* is already only partly true —
   `SUPPORTED_PORTS` is explicitly hand-maintained ("documented convention, not compile-time
   enforcement", ADR-0012 Decision 5). (a)/(d) consolidate that hand-maintenance into one
   compiler-checked field per entry instead of two-to-three coupled lists — arguably *more*
   faithful to the invariant's spirit (single source of truth) even as it drops the literal
   port-derivation mechanism. And ADR-0012 Decision 2 *already* committed the project to
   Suricata's explicit tri-state vocabulary — (d) simply applies it consistently.
4. **Blast radius:** the one axis where (b) clearly wins (0 literals, 0 struct vs 30 + struct).
   This is the entire strength of ADR-014's recommendation and should be weighed honestly — but
   against a one-time cost, not a recurring one. VP-041's oracle actually gets *simpler* under
   (a)/(d).
5. **No-dispatcher-dependency:** (a)/(b)/(d) all respect it; only (c) violates it. Not
   decisive between the front-runners.

**Where this agrees and disagrees with ADR-014 Decision 3:** it **agrees** that (c) is wrong
(boundary violation) and that (a) as-specified is unattractive. It **disagrees** with the
headline recommendation of (b): ADR-014 optimises for *this diff's* blast radius; the
evidence (prior art + the decisive extensibility axis + the `DetectionOnly` state ADR-014's own
Decision 6 introduces) optimises for the *model*, and points to (d). ADR-014's own Consequences
section concedes (b) "still touches an invariant three other artifacts assert … and requires all
three to be updated in lockstep at F4" — i.e. (b) is not as cheap as the diff count suggests.

---

## 8. Critical caveat — the dynamic gap classifier is a separate, unsolved axis

`main.rs::lookup_protocol_state` (lines 1071–1105) is a **second consumer** of `SUPPORTED_PORTS`,
keyed on `(transport, port)`:

```rust
match KNOWN_PROTOCOLS.iter().find(|p| p.transport == t && p.canonical_ports.contains(&port)) {
    Some(p) if p.canonical_ports.iter().any(|cp| SUPPORTED_PORTS.contains(cp)) => KnownSupported, // "BUG signal"
    Some(_) => KnownUnsupported,
    None    => Unknown,
}
```

Once `102 ∈ SUPPORTED_PORTS`, **every** unclassified TCP/102 gap flow matches the first port-102
entry by declaration order (S7comm) and is classified `KnownSupported` — the "dissector should
have fired, this is a BUG" state. But a genuine MMS/ICCP/S7comm-plus gap on 102 is *not* a bug;
it is a real `KnownUnsupported` gap. **No catalog-model option fixes this**, because:

- Option (b)'s name clause lives only in `supported_protocols()`; `lookup_protocol_state` has no
  name, only a port.
- Even a per-entry flag (a/d) does not help `find()` here — with four entries on port 102 it
  returns the *first* by declaration order regardless of each entry's flag.

Correctly disambiguating a raw port-102 gap flow requires the **analyzer-supplied `protocol_id`**
(ADR-014 Decision 2's `0x32`/`0x72`/other branch), not a catalog property. ADR-014 **Decision 10**
already flags this as an F4 consequence. This document confirms it is a genuinely **orthogonal
axis** and the ratifier must not treat any catalog option as resolving it. A per-entry `Support`
enum (d) does, however, make the eventual F4 refactor cleaner: the gap classifier can key on
`(protocol identity from analyzer) → entry.support` instead of re-deriving from ports.

---

## 9. Confidence & unverifiable items

- **High confidence:** the three tools' modeling lessons (§3) — corroborated by official docs
  *and* source for each. The `SUPPORTED_PORTS` double-use and gap-classifier behaviour (§2, §8)
  — read directly from `src/main.rs` and `src/protocols.rs` in this repo.
- **High confidence:** Rust idiom guidance (§5) — Rust API Guidelines type-safety section +
  widely-cited "enums over bools" idiom.
- **Medium / flagged:** exact upstream Wireshark coverage of IEC 61850-MMS-specific, ICCP/TASE.2,
  and S7comm-plus dissectors — Perplexity **could not verify** dedicated upstream dissectors for
  these (generic `mms`/OSI path + not-yet-bundled S7comm-plus plugin). This does not affect the
  modeling lesson (which is about *how* support is keyed, not *which* protocols ship), but the
  ratifier should not cite Wireshark as having first-class IEC 61850/ICCP support.
- **Not independently re-verified here:** the precise current text of the REGRESSION-GUARD
  assertion and the VP-041 oracle beyond the grep-confirmed line locations
  (`tests/protocols_tests.rs` ~465, ~731). Both were located and their logic read via grep;
  a full re-read was out of scope for a model-choice validation but is recommended at F4
  implementation time.
- **Opinion vs sourced:** the tiered verdict (§1, §7) is this agent's engineering judgment;
  the prior-art lessons and Rust idiom guidance underpinning it are sourced.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source sweep of Wireshark/Suricata/Zeek official docs + source for how each keys "supported" for one-port-many-protocols (port-102 ISO-on-TCP), and whether a third "known-but-unsupported/detection-only" state exists |
| Perplexity perplexity_reason | 1 | Synthesis over gathered evidence: idiomatic Rust modeling of a derived-boolean-with-exceptions compile-time catalog (bool vs exclusion-list vs `Support` enum), grounded in Rust API Guidelines |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — |
| Tavily (any) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Read (local) | 4 | `src/protocols.rs`, ADR-0012, ADR-014, `src/main.rs` gap classifier |
| Grep (local) | 2 | `SUPPORTED_PORTS`/`PORT_102`/VP-041 usage sweep (found the main.rs second-consumer) |
| Training data | 1 area | Rust struct-init / enum-exhaustiveness mechanics (cross-checked against the sourced Rust API Guidelines reasoning call — flagged, not sole basis) |

**Total MCP tool calls:** 2 (1 `perplexity_research` PRIMARY + 1 `perplexity_reason`).
**Training data reliance:** low — the prior-art lessons and Rust idiom guidance are web-sourced;
training data used only for uncontroversial Rust language mechanics that the reasoning call
independently corroborated. Every design claim is grounded in either the cited tool docs/source
or direct reads of this repository's code.
