# TLS Handshake-Reassembly Overflow Policy + Per-Message Cap — Research & Recommendation

**Topic:** Overflow policy and per-message size cap for wirerust's new TLS handshake-message reassembler (ClientHello/ServerHello fragmented across TLS records, RFC 5246 §6.2.1 / RFC 8446 §5.1).
**Decision drivers:** evasion-resistance, bounded memory, correctness on legitimate large/ECH/PQ ClientHellos.
**Author:** vsdd-factory research-agent
**Date:** 2026-06-29
**Related prior research:** `.factory/research/TLS-CLIENTHELLO-FRAG-001-validation.md` (confirms the reassembly gap; recommends a bounded reassembler).
**Code context verified in-tree:** `src/analyzer/tls.rs` — `MAX_BUF = 65_536` (L30), `MAX_RECORD_PAYLOAD = 18_432` (L34). The existing per-record overflow handler at L689–698 already implements **clear-and-recover** (`state.client_buf.clear()` / `server_buf.clear()`, then `return`), and the byte-copy loop at L820–833 silently stops appending once `MAX_BUF` is reached (saturating). There is currently **no** sticky abandon flag for buffer overflow.

---

## TL;DR Recommendation

- **Overflow policy: Policy A — clear-and-recover** (with a counter + bounded recovery). **Do NOT adopt Policy B "sticky abandon-direction."**
- **Per-message reassembly cap: 64 KiB (65,536 bytes)** for the handshake-reassembly buffer — i.e. reuse `MAX_BUF`. Raise the *per-message body guard* from `MAX_RECORD_PAYLOAD` (18,432) to the same 64 KiB so legitimate large ClientHellos are not silently dropped. Do **not** use the 16 MiB theoretical max; do **not** keep 16–18 KiB as the message cap.
- **Strongest single evidence — overflow policy:** Ptacek & Newsham's foundational evasion analysis shows that any mechanism which lets an attacker *desynchronize* the analyzer's view of a flow from the endpoint's view creates a pass-through evasion channel. A sticky-abandon policy is exactly such a desynchronizer — one crafted oversized fragment permanently blinds the analyzer for that flow while the endpoints keep talking. (Ptacek & Newsham 1998.)
- **Strongest single evidence — per-message cap:** Go's `crypto/tls` hard-caps a single reassembled handshake message at **`maxHandshake = 65536`** bytes and *aborts the handshake* on exceed — a real, shipping production TLS stack that legitimate peers must already stay under. 64 KiB is therefore a cap no benign ClientHello/ServerHello will hit, while staying far below the 16 MiB protocol ceiling. (Go `src/crypto/tls/conn.go`.)

---

## Evidence quality legend

- **[Documented]** — stated in official docs, source, RFC, or peer-reviewed/published work (URL given).
- **[Inference]** — my reasoning from documented mechanisms; not a quoted result.
- **[Inconclusive]** — sources thin or conflicting; flagged as such.

---

## Q1 — How do mature analyzers bound TLS handshake reassembly memory, and what do they do on limit-exceeded?

**Summary finding:** Across Suricata, Zeek, and Wireshark/tshark, the consistent design philosophy is that **reassembly/memory limits curtail *inspection scope* (or mark data unreassembled), but do NOT permanently blacklist the flow.** Suricata is the only one with documented machinery that can *drop or bypass* a flow — and even that is (a) a configurable policy, not a hardcoded sticky abandon, and (b) tied to a *global* memcap, not a per-flow handshake overflow. No mainstream analyzer documents a "one overflow → ignore this direction forever" behavior. [Documented for the limit knobs; the absence of sticky-abandon is a strong negative finding across the three docs sets.]

### Suricata
- Two distinct limit classes [Documented]:
  - **Depth** — `stream.reassembly.depth` (global) and per-protocol `stream-depth` (e.g. under `app-layer.protocols.tls`). `0` = unlimited. When depth is reached, Suricata **stops reassembling beyond that many bytes for that direction but keeps tracking the flow** — detection coverage is curtailed, the flow is not deleted.
    - https://docs.suricata.io/en/latest/configuration/suricata-yaml.html
    - https://docs.suricata.io/en/latest/devguide/internals/engines/stream/inspection_raw_data.html
  - **Memcap (global resource limit)** — when the stream memcap is hit, Suricata applies a configurable **exception policy** (`ignore` / `bypass` / `drop-packet` / `drop-flow`). This is a *global* memory-pressure response, configurable and operator-chosen — not a per-flow "this handshake overflowed" sticky flag.
    - https://docs.suricata.io/en/latest/configuration/exception-policies.html
    - https://docs.suricata.io/en/suricata-6.0.20/configuration/exception-policies.html
- **TLS-specific:** Suricata can *bypass encrypted traffic* once stream depth is filled, but this is explicitly an `app-layer.protocols.tls` config option whose default changed in 8.x — it concerns the *encrypted bulk after the handshake*, not the handshake reassembly itself. [Documented]
  - https://docs.suricata.io/en/suricata-8.0.3/upgrade.html
- **The exact default `app-layer.protocols.tls.stream-depth` value was NOT found verbatim** in the docs surveyed. [Inconclusive] The `0 = unlimited` convention is documented for other protocols (Modbus) and is the general Suricata pattern, but treat the specific TLS default as unverified.

### Zeek/Bro
- Zeek separates **connection tracking** (core `Conn` analyzer) from **protocol analyzers** (attached via Dynamic Protocol Detection). On a protocol violation (`analyzer_violation` / DPD logic), Zeek **disables/removes just that analyzer for the connection and keeps processing the connection** — it does not drop the flow, and may attach a different analyzer later. [Documented at the framework level: https://docs.zeek.org]
  - **Caveat:** the survey could not quote the exact `analyzer_violation`/`skip_further_processing` semantics for the SSL/TLS analyzer specifically; the framework-level behavior is documented, the TLS-analyzer-specific text is [Inference] from that framework. Note that "remove the TLS analyzer on violation" *is* a form of per-flow analyzer abandonment — but it is triggered by a *protocol violation*, not by merely exceeding a reassembly byte budget, and Zeek 2.5.5+ raises a "weird" event on fragment overlap/inconsistency rather than silently abandoning. [Documented: https://old.zeek.org/manual/2.5.5/install/changes.html]

### Wireshark / tshark
- TCP reassembly governed by the "Allow subdissector to reassemble TCP streams" preference and a `tcp.desegment_max_bytes` cap; the TLS dissector has "Reassemble TLS records spanning multiple TCP segments." When reassembly fails or a cap/missing-segment is hit, Wireshark **marks data `[unreassembled]` / "Malformed Packet" but never drops the flow** — consistent with a passive analyzer. [Documented behavior class; exact pref strings from Wireshark User's Guide / TLS dissector docs at https://www.wireshark.org/docs — the survey relied on the manual rather than a quoted snippet, so treat exact pref names as well-known-but-not-freshly-quoted.]
- **No TLS-handshake-specific size cap** beyond the generic TCP reassembly byte cap was found. [Inconclusive / likely absent]

### JA3 / JA4 reference implementations
- **[Inconclusive]** The survey did not surface explicit, documented reassembly-size bounds in the Salesforce `ja3` or FoxIO `ja4` reference code. In practice these tools operate on whatever ClientHello bytes the host parser/Zeek/Suricata hands them; they inherit the host's reassembly bound rather than defining their own. Prior research (`TLS-CLIENTHELLO-FRAG-001-validation.md`) already established that JA3/JA4 require a *complete* ClientHello parse, so they depend on the upstream reassembler being correct and bounded.

**Bottom line for Q1:** The de-facto industry pattern for a *passive* analyzer is **"limit curtails inspection, flow survives"** (Wireshark mark-malformed; Suricata depth-reached-but-keep-flow). Permanent per-flow blacklisting on a byte-budget overflow is **not** a documented norm — and where Suricata *can* drop/bypass, it is an operator-configured global-memcap response, not a silent hardcoded sticky flag.

---

## Q2 — Which overflow policy is more evasion-resistant? Can sticky-abandon be poisoned?

**Finding: Policy A (clear-and-recover) is more evasion-resistant. Policy B (sticky abandon-direction) is directly exploitable as a per-flow blinding primitive.** The general principle is **[Documented]** in the evasion literature; the specific A-vs-B framing for a TLS handshake reassembler is **[Inference]** that follows tightly from that literature (no source names this exact tradeoff, so it is reasoning, not a citation).

### Documented principles
- **Ptacek & Newsham (1998), "Insertion, Evasion, and Denial of Service: Eluding Network Intrusion Detection."** Core thesis: a passive analyzer is evaded whenever an attacker can make the analyzer's reconstructed view *diverge* from the endpoint's view. Once desynchronized, "subsequent attack traffic may pass undetected because the IDS believes the connection is in a different state than the host." They explicitly flag resource limits as an inherent DoS/evasion surface for passive analyzers.
  - https://users.ece.cmu.edu/~adrian/731-sp04/readings/Ptacek-Newsham-ids98.pdf
- **Handley & Paxson (2001), traffic normalization** — passive NIDS face unavoidable ambiguity; the robust fix is to remove ambiguity, not to give up on a flow. https://www.usenix.org/legacy/events/sec01/full_papers/handley/handley.pdf
- **State-exhaustion as a known class:** low-volume crafted traffic can deplete per-flow/state resources of stateful inspection devices. https://fastnetmon.com/2025/08/12/understanding-transport-and-state-exhaustion-ddos-attacks/ ; DPI pipelines explicitly list "state table exhaustion" as an evasion pitfall: https://devsecopsschool.com/blog/dpi/
- **Concrete precedent that an analyzer can be made to ignore real data in a flow:** Suricata CVE-2019-18792 — a fake FIN overlapping a data segment caused Suricata to ignore the data while the client processed it. Demonstrates that per-flow desynchronization → silent miss is a *real, shipped* failure mode, not theoretical. https://redmine.openinfosecfoundation.org/issues/3324
- **Record-fragmentation is already a live, server-tolerated evasion vector** (relevant because the attacker controls fragmentation, hence controls how easily they can drive a reassembly buffer): https://upb-syssec.github.io/blog/2023/record-fragmentation/

### Applying this to A vs B
- **Policy B (sticky abandon) is a deliberate, attacker-triggerable desynchronizer.** [Inference, strongly supported] An attacker who knows or guesses the cap sends one oversized/over-fragmented garbage "handshake" in the client→server direction at flow start. The buffer overflows, the sticky flag is set, and **for the entire remaining lifetime of that flow the analyzer ignores that direction** — including any later, well-formed ClientHello the attacker then sends. The endpoints complete a normal handshake; wirerust emits no SNI, no JA3, no findings. This is precisely the Ptacek/Newsham desynchronization-to-evasion pattern, made trivial and *permanent* per flow. The cost to the attacker is a single crafted record; the payoff is total per-flow blinding. **This is the worst property a passive analyzer can have: silent, attacker-controlled, permanent under-reporting.**
- **Policy A (clear-and-recover) denies the attacker permanence.** [Inference, strongly supported] After overflow the buffer is cleared and a later well-formed handshake record re-populates it and parses normally. The attacker cannot convert a transient overflow into a durable blind spot; to keep wirerust blind they would have to *continuously* overflow the buffer, which (a) is far more conspicuous, (b) is itself observable/countable, and (c) still does not guarantee the real handshake is missed if it arrives in a clean window.
- **Does Policy A open a *different* evasion (forced repeated clears)?** Marginally, and far less severe. [Inference] An attacker could in principle interleave garbage fragments with the real ClientHello to force a clear at exactly the wrong moment. But this requires precise, continuous interleaving rather than one shot, is fragile against the real handshake landing in a clean drain, and — critically — is **bounded and recoverable** rather than permanent. The asymmetry strongly favors A: B gives a guaranteed permanent win for one packet; A gives at most a fragile, transient, repeatable-effort miss.
- **Mitigation to keep with Policy A:** increment a per-flow/per-direction `reassembly_overflow` counter on each clear (mirroring the existing `truncated_records` counter) so overflows are *observable telemetry*, not silent. Repeated overflows on one flow are themselves a suspicious signal. This converts the residual Policy-A risk into a detectable event — exactly the "signal the anomaly, keep inspecting" posture Wireshark/Zeek embody.

**Note — the named-class caveat (honesty flag):** "resource-limit poisoning to blind a single flow via sticky abandon" is **not a formally named/published evasion class** in the literature surveyed. The *building blocks* (desynchronization-as-evasion, state exhaustion, fragmentation control, real Suricata desync CVEs) are all documented; the synthesis into "sticky abandon is exploitable, clear-and-recover is safer" is my reasoning. It is well-grounded reasoning, but it is reasoning — not a quotable result. Treat the *direction* of the recommendation as high-confidence and the *absence of a CVE/paper naming it* as expected (it is a design-choice anti-pattern, not a vulnerability in a specific product).

---

## Q3 — Realistic max size of a legitimate ClientHello (ECH + post-quantum) in 2024–2026

**Finding:** A real-world ClientHello in 2024–2026, including ECH and post-quantum hybrid key shares, is **roughly 1.5–2.5 KiB** — an order of magnitude below the 16,384-byte TLS record limit. [Documented]

- Post-quantum hybrid key shares: ML-KEM-768 (Kyber-768) key share ≈ **1,184 bytes**; the X25519+ML-KEM-768 / X25519+Kyber768 hybrid adds on the order of ~1.2 KiB to the ClientHello. Resulting Chrome/Cloudflare PQ ClientHellos are commonly reported at **~1.5–2.5 KiB**. [Documented — from the PQ-deployment sources surveyed; concrete keyshare size is a known ML-KEM-768 parameter.]
- ECH adds further extensions but does not push typical ClientHellos past a single 16,384-byte record. **ClientHello rarely needs *record*-layer fragmentation today**, though it can exceed a single ~1500-byte MTU (IP/TCP-level segmentation is common). [Documented]
- **Why a cap still matters despite small typical sizes:** (a) deliberate fragmentation (the evasion case) can spread a small ClientHello across many tiny records, inflating *buffered* bytes if the reassembler is naive; (b) the *handshake* reassembler may also accumulate ServerHello + Certificate-adjacent control messages; and (c) future extension growth. The cap protects memory and bounds adversarial input — it is not sized to typical ClientHellos, it is sized to "comfortably above any legitimate control-message handshake while well below the protocol ceiling."

**Practical cap used by real analyzers/stacks:** the survey's conclusion was that **64 KiB–256 KiB is the practical range** real systems use for a per-handshake-message bound; the full 16 MiB is "purely theoretical in mainstream use" and operationally impractical. [Documented as the survey synthesis, corroborated by the stack-specific caps in Q4.]

---

## Q4 — Standards/RFC guidance on max handshake message size and oversize handling

- **Handshake length field is 24-bit.** A single TLS handshake message can be up to **2^24 − 1 = 16,777,215 bytes (≈16 MiB)**. Confirmed for TLS 1.3 (RFC 8446 §4 Handshake struct, `uint24 length`) and TLS 1.2 (RFC 5246). [Documented]
  - https://datatracker.ietf.org/doc/html/rfc8446
  - https://datatracker.ietf.org/doc/html/rfc5246
- **The RFCs do NOT mandate a smaller cap** and give little explicit guidance on how implementations SHOULD treat oversize/over-fragmented handshakes. RFC 5246 implementation guidance does warn implementers to correctly handle handshake messages fragmented across multiple records and their corner cases. [Documented]
- **Real TLS stacks impose far tighter caps than 16 MiB** and treat exceed as a fatal/abort condition, NOT as something to buffer indefinitely. This is the most decision-relevant finding:
  - **Go `crypto/tls`: `maxHandshake = 65536` bytes (64 KiB).** On exceed it returns `"tls: handshake message of length %d bytes exceeds maximum of %d bytes"` and **aborts the handshake**. Source: `src/crypto/tls/conn.go`. [Documented — verified via focused lookup]
    - https://github.com/golang/go/issues/35153 (constant + behavior)
    - https://github.com/golang/go/blob/master/src/crypto/tls/conn.go
  - **JDK / Java TLS: default max handshake message size 32,768 bytes (32 KiB), tunable** via `jdk.tls.maxHandshakeMessageSize`. Exceed → connection error. [Documented]
    - https://support.pingidentity.com/s/article/The-size-of-the-handshake-message-n-exceeds-the-maximum-allowed-size-32768-connection-errors-in-PingAM-and-PingDS
  - **BoringSSL:** internal `kMaxMessageLen` — "the default maximum message size for handshakes which do not accept peer certificate chains" — i.e. ClientHello/ServerHello/control messages are capped on the order of **tens of KiB**, with a separate, higher (but still far-below-16-MiB) cap for certificate-bearing messages. Exact numeric value not quoted in the source snippet surveyed. [Documented (the design); numeric value Inconclusive]
  - **OpenSSL:** record plaintext default `SSL3_RT_MAX_PLAIN_LENGTH = 16384` (`max_send_fragment`); handshake message assembly is bounded by memory rather than a small hard constant, so it *can* in theory approach the 16 MiB ceiling, but mainstream deployments never approach megabyte handshakes. [Documented]
  - **Real anecdote of a genuinely large handshake:** a certificate-bearing handshake of ~184 KiB was observed tripping Go's 64 KiB limit — but this is a **Certificate message**, not a ClientHello, and is explicitly exceptional. [Documented from the survey] This matters: the messages wirerust cares about for SNI/JA3 (ClientHello, ServerHello) are *control* messages, the category every stack keeps small.

**Synthesis:** The protocol ceiling is 16 MiB, but the *de-facto interoperable maximum* for a ClientHello/ServerHello is set by the strictest mainstream stack — Go at 64 KiB and JDK at 32 KiB. Any legitimate peer that wants to talk to Go-based or Java-based servers must already keep its handshake control messages under 64 KiB. **A passive analyzer that caps at 64 KiB will never truncate a handshake that a Go server would have accepted.** Capping below ~32 KiB risks dropping handshakes that JDK/Go would accept (over-blinding); capping at 16 MiB is an unbounded-memory liability and matches no real stack's control-message behavior.

---

## Q5 — Recommendation for wirerust

### Overflow policy: **Policy A — clear-and-recover (with counter).** Reject Policy B.

Rationale:
1. **Evasion-resistance (primary driver).** Policy B is a one-packet, permanent, attacker-controlled blinding primitive — the textbook desynchronization-to-evasion failure (Ptacek & Newsham; Suricata CVE-2019-18792 shows the failure mode is real). Policy A denies permanence; the residual "forced repeated clear" risk is fragile, bounded, and observable. The asymmetry is large and one-directional.
2. **Industry alignment.** Passive analyzers (Wireshark) and stateful ones (Suricata at depth, Zeek per-analyzer) overwhelmingly favor "curtail inspection / mark anomaly / keep the flow" over "blacklist the flow forever." wirerust is a passive, Wireshark-like tool; "keep inspecting, signal the anomaly" is the matching posture.
3. **Consistency with existing code.** `src/analyzer/tls.rs` L689–698 already does clear-and-recover for the `MAX_RECORD_PAYLOAD` over-size case (`client_buf.clear()` / `server_buf.clear()` then `return`). Adopting Policy A for the new handshake-reassembly buffer keeps one coherent overflow discipline rather than introducing a contradictory sticky path.
4. **Snaplen-truncation safety (from prior research cand-05).** Clear-and-recover with "incomplete at flow close ⇒ no finding, not an error" avoids the false-positive storm that a sticky/abandon-with-error path could create on truncated captures.

**Concrete shape:** On handshake-reassembly-buffer overflow → clear that direction's handshake buffer, increment a `handshake_reassembly_overflows` counter (mirror the existing `truncated_records` field at L313–318), and continue. Do **not** emit a malformed-packet finding merely for overflow (avoids FP on truncation); the counter is telemetry. A later well-formed handshake re-populates and parses normally.

### Per-message cap: **64 KiB (65,536 bytes) = reuse `MAX_BUF`.** Raise the per-message body guard from 18,432 → 65,536.

Rationale:
1. **64 KiB is the strictest real-world interoperable ceiling** (Go `maxHandshake = 65536`). No ClientHello/ServerHello that a Go server accepts can exceed it; therefore wirerust at 64 KiB never truncates a legitimate, internet-viable handshake. This is the single strongest, most concrete anchor.
2. **The current 18,432 (`MAX_RECORD_PAYLOAD`) is too low as a *message* cap.** It is correct as a single-*record* payload bound (TLS 1.2 ciphertext max), but a handshake *message* may legally be reassembled from several records and can legitimately exceed one record's payload. Using 18 KiB as the per-message guard would silently drop genuinely large-but-legal ClientHellos/ServerHellos (the exact silent-drop failure called out in the task). JDK already allows 32 KiB by default — an 18 KiB cap would under-cut a mainstream stack.
3. **64 KiB, not 16 MiB.** The 16 MiB protocol ceiling matches no mainstream control-message stack and is a memory-exhaustion liability for a passive tool that may track many concurrent flows (`MAX_MAP_ENTRIES = 50_000`). 50k flows × 2 directions × 16 MiB is absurd; × 64 KiB is bounded and already the established `MAX_BUF` budget. 64 KiB keeps per-flow memory predictable and equal to the existing record-buffer budget.
4. **64 KiB, not 256 KiB.** 256 KiB would only be needed to capture pathological multi-hundred-KiB *Certificate* chains (the 184 KiB anecdote) — which are (a) not ClientHello/ServerHello, (b) not what SNI/JA3 extraction needs, and (c) not accepted by Go anyway. For the SNI/JA3 mission, the relevant messages are control messages that every stack keeps well under 64 KiB. Choosing 64 KiB keeps the cap aligned to the data wirerust actually parses and to its existing `MAX_BUF`.

**Caveat / honest limit:** If wirerust later wants to reassemble and inspect full *Certificate* messages (large chains), 64 KiB would truncate some legitimate cert handshakes (Go's own limit cuts them off too, so wirerust would be in good company, but the 184 KiB anecdote shows real chains exist). That is a *separate* feature decision; for the current SNI/JA3-on-ClientHello/ServerHello scope, 64 KiB is correct. Document the cap with a comment pointing at Go's `maxHandshake` as the rationale so a future cert-reassembly story can revisit it deliberately rather than by accident.

### Net change set (advisory — research only, not an implementation)
- Add a per-direction handshake-reassembly buffer bounded at `MAX_BUF` (64 KiB).
- On overflow: clear that direction's handshake buffer + increment a new `handshake_reassembly_overflows` counter; continue (Policy A). No finding emitted.
- Replace the `payload_len > MAX_RECORD_PAYLOAD` *handshake-message* guard with a 64 KiB message-body guard (keep `MAX_RECORD_PAYLOAD` for the genuine single-record payload sanity check if that distinct check is still wanted, but do not let it abandon a multi-record handshake message that is ≤ 64 KiB total).
- Preserve "incomplete at flow close ⇒ no finding, not an error."

---

## Conflicts / inconclusive items (explicit)

- **Suricata default `app-layer.protocols.tls.stream-depth`** — exact value not verified. [Inconclusive] Does not affect the recommendation.
- **BoringSSL `kMaxMessageLen` numeric value** — design documented, number not quoted. [Inconclusive] Go (64 KiB) and JDK (32 KiB) are the verified anchors.
- **JA3/JA4 reference-impl internal caps** — no documented bound found; they inherit upstream reassembly bounds. [Inconclusive]
- **"Sticky-abandon poisoning" as a named class** — not formally named/published; the recommendation rests on documented evasion *principles* (Ptacek/Newsham, real Suricata desync CVE) applied via reasoning. [Inference, high-confidence direction]
- **Zeek SSL-analyzer-specific violation handling** — framework behavior documented; the TLS-analyzer-specific text is inference from the framework, and note Zeek *can* remove an analyzer on a protocol violation (a violation, not a byte-budget overflow). This is not a counterexample to Policy A: Zeek abandons on *malformed protocol*, not on *legitimate-but-large*, and still raises observable events.

---

## Cited sources

Primary / standards:
- RFC 8446 (TLS 1.3) §4/§5.1 — 24-bit handshake length, coalesce/fragment: https://datatracker.ietf.org/doc/html/rfc8446
- RFC 5246 (TLS 1.2) §6.2.1 — fragmentation rule: https://datatracker.ietf.org/doc/html/rfc5246

Analyzer behavior (limits/overflow):
- Suricata stream/raw-data inspection: https://docs.suricata.io/en/latest/devguide/internals/engines/stream/inspection_raw_data.html
- Suricata YAML config: https://docs.suricata.io/en/latest/configuration/suricata-yaml.html
- Suricata exception policies (memcap → drop/bypass/ignore): https://docs.suricata.io/en/latest/configuration/exception-policies.html ; https://docs.suricata.io/en/suricata-6.0.20/configuration/exception-policies.html
- Suricata 8.x TLS encrypted-bypass + stream depth: https://docs.suricata.io/en/suricata-8.0.3/upgrade.html
- Suricata CVE-2019-18792 (fake-FIN overlap → ignored data; per-flow desync precedent): https://redmine.openinfosecfoundation.org/issues/3324
- Zeek docs (analyzer framework / DPD): https://docs.zeek.org
- Zeek 2.5.5 fragment-overlap "weird" events: https://old.zeek.org/manual/2.5.5/install/changes.html
- Wireshark docs (TCP/TLS reassembly prefs, malformed marking): https://www.wireshark.org/docs

Evasion literature:
- Ptacek & Newsham 1998 (Insertion/Evasion/DoS): https://users.ece.cmu.edu/~adrian/731-sp04/readings/Ptacek-Newsham-ids98.pdf
- Handley & Paxson 2001 (traffic normalization): https://www.usenix.org/legacy/events/sec01/full_papers/handley/handley.pdf
- Overlapping OS/NIDS reassembly (2025 preprint, all NIDS still evadable): https://arxiv.org/html/2504.21618v1
- State-exhaustion DDoS overview: https://fastnetmon.com/2025/08/12/understanding-transport-and-state-exhaustion-ddos-attacks/
- DPI state-table-exhaustion pitfall: https://devsecopsschool.com/blog/dpi/
- TLS record-fragmentation circumvention (GFW): https://upb-syssec.github.io/blog/2023/record-fragmentation/

Per-message size caps (real stacks):
- Go `crypto/tls` maxHandshake = 65536 (+ abort behavior): https://github.com/golang/go/issues/35153 ; https://github.com/golang/go/blob/master/src/crypto/tls/conn.go
- JDK default 32768 (`jdk.tls.maxHandshakeMessageSize`): https://support.pingidentity.com/s/article/The-size-of-the-handshake-message-n-exceeds-the-maximum-allowed-size-32768-connection-errors-in-PingAM-and-PingDS

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 4 | (1) initial broad sweep (returned oversized; re-split); (2) analyzer reassembly limits/overflow behavior — Suricata/Zeek/Wireshark, effort=medium; (3) evasion literature + sticky-abandon poisoning, effort=medium; (4) ClientHello PQ/ECH sizes + real-stack caps + analyzer cap, effort=high |
| Perplexity perplexity_ask | 1 | Verified exact Go `crypto/tls maxHandshake = 65536` value + abort-on-exceed behavior |
| Read | 1 | Read large persisted research result (chunked); prior research file TLS-CLIENTHELLO-FRAG-001-validation.md |
| Grep | 4 | Verified in-tree MAX_BUF/MAX_RECORD_PAYLOAD constants and current overflow/clear behavior in src/analyzer/tls.rs; extracted sections from persisted oversized research outputs |
| Glob | 1 | Surveyed existing .factory/research for prior context |
| Training data | 1 area | TLS handshake header structure (1-byte type + uint24 length) — cross-checked against RFC 8446 §4 |

**Total MCP tool calls:** 5 (4 perplexity_research, 1 perplexity_ask).
**Training data reliance:** low — every load-bearing number (24-bit length / 16 MiB ceiling, Go 65536, JDK 32768, ML-KEM-768 ~1184 B, PQ ClientHello ~1.5–2.5 KiB, OpenSSL 16384 record) is sourced; analyzer overflow semantics sourced to official docs; evasion conclusion grounded in named papers + a real Suricata CVE.
**Inconclusive items explicitly flagged:** Suricata TLS stream-depth default; BoringSSL kMaxMessageLen numeric value; JA3/JA4 internal caps; "sticky-abandon poisoning" is reasoned (not a named published class) — see §Q2 caveat and the Conflicts section.

**Deviation note (per agent mandate):** The initial single `perplexity_research` call returned a result exceeding the token cap and could not be fully read via the single-giant-line file. Rather than rely on a partial read, the topic was decomposed into three focused `perplexity_research` calls (each fully readable) plus one `perplexity_ask` to nail the load-bearing Go constant. This satisfies the PRIMARY-tool default with full source grounding.
