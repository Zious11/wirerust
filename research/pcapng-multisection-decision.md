# pcapng Multi-Section Decision: REJECT vs SUPPORT (resolves F-06 INCONCLUSIVE)

**Date:** 2026-06-19
**Agent:** vsdd-factory:research-agent
**Gates:** F2 -> F3 decision for the pcapng-reader cycle (SS-01)
**Resolves:** Finding F-06 ("`pcap-file` 2.0.0 likely does NOT reset interface state per section") from `.factory/research/pcapng-spec-completeness-validation.md`, which was flagged INCONCLUSIVE pending a runtime/source determination.
**Decision under review:** ADR-009 Decision 7 (single-section only; second SHB rejected with `E-INP-012`) and BC-2.01.010 AC-002/EC-006.

---

## RECOMMENDATION

**KEEP REJECT (ADR-009 Decision 7 as written) for this cycle.** Reject is the correct, low-risk posture, but for a *different and stronger reason* than the one ADR-009/F-06 currently cite. The cited rationale ("pcap-file 2.0.0 accumulates interfaces and would mis-attribute") is **factually wrong** — see Finding 1. Reject remains correct because:

1. Multi-section pcapng is **rare** in real-world workflows and **absent from wirerust's entire intended corpus** (Finding 2) — so reject blocks zero target files (Finding 5).
2. Rejecting with a clear `E-INP-012` error is strictly safer than the alternative failure modes, and is **better-behaved than several peer tools** (Wireshark historically errored or mis-attributed; gopacket/scapy support is uncertain) — so reject is a defensible, even conservative, posture for a metadata analyzer (Finding 4).
3. The actual cost-to-SUPPORT is genuinely low now that the parser behavior is known (Finding 3), so SUPPORT is a cheap *future* enhancement — but it buys nothing for the current corpus and adds test surface for a path no target file exercises. Defer it.

**Required correction to ADR-009 and F-06:** The rationale text must be corrected. `pcap-file` 2.0.0 **does** reset the interface table per section (confirmed from source — Finding 1). The justification for reject is "rare + not in corpus + clear-error-beats-silent-edge-case + scope discipline," NOT "the parser would silently mis-attribute." The mis-attribution claim is false and should not be relied upon. See "Required Edits" at the end.

**One-line summary:** Reject is right; the *reason* in the ADR is wrong and must be fixed. pcap-file 2.0.0's per-section reset behavior is **RESOLVED (not inconclusive): it correctly clears interfaces on each SHB.**

---

## Findings

### Finding 1 — pcap-file 2.0.0 RESETS the interface table per section. RESOLVED (was INCONCLUSIVE). HIGH importance.

The F-06 claim — that `pcap-file` 2.0.0 "tracks a single 'current' section and accumulates IDBs in one growing interface list with no visible per-section reset" — was derived from API *shape* (the singular `section()` / "all the current" `interfaces()` doc strings) and was explicitly flagged as unverified by runtime test. **Reading the actual 2.0.0 source settles it the opposite way.**

The `PcapNgParser` state machine (docs.rs rendered source for pcap-file 2.0.0, `src/pcapng/parser.rs`) handles blocks in `next_raw_block_inner` as follows:

- On `Block::SectionHeader`: it replaces `self.section` with the new SHB **and executes `self.interfaces.clear();`** (parser.rs line ~101). The interface table is emptied at every section boundary.
- On `Block::InterfaceDescription`: `self.interfaces.push(interface);` (parser.rs line ~105) — interfaces are appended *into the freshly-cleared, section-local vector*.
- `packet_interface(&self, packet) -> Option<&InterfaceDescriptionBlock>` returns `self.interfaces.get(packet.interface_id as usize)` — i.e., it resolves an EPB's `interface_id` against the **current section's** interface table, which is exactly the per-section-scoped lookup the pcapng spec requires.
- `next_block()` does **not** error or stop at a second SHB. It returns the `SectionHeader` block normally and continues. `PcapNgReader::next_block()` delegates to the parser (`self.reader.parse_with(|src| self.parser.next_block(src))`) and surfaces every block — including `Block::SectionHeader` — to the caller without filtering.

**Answer to Question 1:** Behavior is **(a) — it correctly resets the interface table per section.** It does NOT accumulate (b), does NOT error (c), and does NOT silently stop at the boundary (d). The mis-attribution-class bug that F-06 feared is **not present** in pcap-file 2.0.0.

This means wirerust, if it simply consumed `pcap-file`'s reader and used `packet_interface()` (or indexed `interfaces()` per the *current* state) to resolve EPB interface IDs, would already get section-local interface resolution for free — the hard part is done by the crate.

> **Confidence:** HIGH. This is read from the actual rendered source of the 2.0.0 release on docs.rs (`/2.0.0/src/...`), version-confirmed, not from "latest" or from inference. The two earlier deep-research syntheses both reached only "most plausible interpretation / strongly implied" and explicitly recommended confirming against the real source — which this finding does. The earlier F-06 conclusion should be regarded as superseded.

*Sources: docs.rs/pcap-file/2.0.0 rendered source `src/pcap_file/pcapng/parser.rs.html` (lines ~101 `self.interfaces.clear();`, ~105 `self.interfaces.push(interface);`, `packet_interface()` body) and `src/pcap_file/pcapng/reader.rs.html` (`next_block` delegation). Verified 2026-06-19.*

---

### Finding 2 — Multi-section pcapng is RARE; produced almost only by raw byte concatenation. Question 2.

The pcapng spec makes multi-section files first-class (a file may contain multiple SHBs; `cat a.pcapng b.pcapng > c.pcapng` is a spec-valid way to merge), but the *tool ecosystem overwhelmingly produces single-section files*:

| Producer | Multi-section output? | Notes |
|----------|----------------------|-------|
| Wireshark / dumpcap / tshark | **No** | One SHB per saved file. Multi-interface complexity is expressed via multiple IDBs *within one section*, never via multiple sections. Rotation (`-b`) yields multiple single-section *files*, not multi-section files. |
| `mergecap` | **No** | Performs a *logical* packet-level merge and re-serializes into a new single-section pcapng. Flattens multi-section input into one section. |
| `editcap` / `reordercap` | **No** | Read, re-serialize, write single-section output. |
| `cat`/`tail` raw concatenation | **Yes** | The principal real-world source of multi-section files. Valid per spec but a niche power-user technique; community guidance (Wireshark Q&A) explicitly prefers `mergecap` over raw concatenation. |
| Custom/distributed capture aggregators | Rare | Theoretically possible (one section per sensor); not visibly common in public tooling. |

**Public corpora:** Multi-section files are rare and, where they appear, are typically *test/bug-repro artifacts* rather than ordinary traces. The Wireshark sample-captures wiki calls out a multi-section file specifically *because Wireshark could not read it* — i.e., it is flagged as an exception. PacketLife / malware-traffic-analysis corpora are predominantly single-section pcap or simple single-section pcapng.

**Verdict: RARE.** For wirerust's target users (analysts consuming captures from mainstream tools and public corpora), multi-section is an edge case, not a routine input.

*Sources: pcapng.com; WinPcap/IETF pcapng drafts (`draft-tuexen-opsawg-pcapng-03/04`); Wireshark man pages for `dumpcap`, `mergecap`, `editcap`; Netresec "HowTo handle PcapNG files"; Wireshark sample-captures wiki; Zeek issue #864 (multi-section via concatenation); Packet-Foo "trouble with multiple capture interfaces". Perplexity deep-research, 2026-06-19.*

---

### Finding 3 — Cost-to-SUPPORT is LOW, because the per-section mapping IS exposed at the block level. Question 3.

Because Finding 1 establishes that `pcap-file` 2.0.0 (a) yields `Block::SectionHeader` to the consumer and (b) already maintains a correct section-local interface table internally, wirerust has **two cheap SUPPORT paths** if it ever wants them:

**Option SUPPORT-1 (lean on the crate):** Stop rejecting on the second SHB. Continue consuming blocks. Resolve each EPB's interface via the crate's current `interfaces()` state (or `packet_interface()`), which is already section-correct. Because wirerust collapses everything to a single `PcapSource.datalink`, the only *additional* logic needed is to extend the existing multi-IDB link-type-agreement check (BC-2.01.018 / ADR-009 Decision 3) to also hold *across* sections (or to re-affirm it per section and require global agreement). Estimated added code: small — on the order of removing the reject branch plus ~10-30 LOC to keep the single-`DataLink` invariant honest across a section boundary.

**Option SUPPORT-2 (track it yourself):** If wirerust wanted full ownership, it can watch for `Block::SectionHeader` in the stream and reset its *own* `Vec<InterfaceDescriptionBlock>` — the section boundary is fully visible to the consumer, so wirerust is not blocked by anything hidden in the reader. Slightly more code than SUPPORT-1, but still modest (~30-60 LOC) and entirely within `reader.rs`.

**Test burden either way:** The crafted 2-section fixture and assertions that BC-2.01.010 already specifies for the *reject* case (SHB1 + IDB + EPB + SHB2 + IDB + EPB) would be repurposed to assert *correct per-section attribution* instead of `Err`. Add 1-2 fixtures with deliberately *different* interface layouts per section (e.g., section 1 = Ethernet iface 0, section 2 = a different linktype iface 0) to prove section-local resolution. This is the same fixture investment the reject path needs, redirected.

**Why it is still not worth doing now:** SUPPORT delivers value only on inputs that (per Finding 2) are rare and (per Finding 5) absent from the corpus, while adding a cross-section `DataLink`-agreement decision that interacts with the deliberately-deferred per-packet-DataLink scope boundary. The cost is low but the benefit this cycle is zero. Defer to a future cycle gated on an actual user requirement.

*Sources: same pcap-file 2.0.0 source as Finding 1 (`Block::SectionHeader` exposure, `packet_interface()`); ADR-009 Decision 3 / BC-2.01.018 (existing single-DataLink agreement model). 2026-06-19.*

---

### Finding 4 — Peer tools handle multi-section UNEVENLY; reject is a defensible posture. Question 4.

| Tool | Multi-section read behavior |
|------|---------------------------|
| **Wireshark / tshark** (wiretap) | Historically **rejected** multi-section files outright ("Wireshark doesn't support files with multiple Section Header Blocks ... so it cannot read it"; 2016 bug report: "multi-section PCAPng files are not supported"). Later versions attempt to read but had a **mis-attribution bug** — packets in later sections used the *first section's* interfaces (GitLab #16531). Full robust support is recent and was historically incomplete. |
| **libpcap / tcpdump** | Designed to stream through sections; resets section-specific state internally and presents a continuous packet stream. The most robust baseline. Constrained to pcapng version 1.0. |
| **Zeek** (via libpcap) | Parses multi-section, but *higher-level analysis* can misbehave (excess/doubled flows with overlapping timestamps across concatenated sections) — Zeek issue #864. |
| **scapy `PcapNgReader`** | Reads as a flat packet stream; per-section semantics **uncertain/limited**; may misbehave when sections differ in interface layout. |
| **Go gopacket (`pcapgo`)** | Multi-section support **limited/uncertain**; common design rejects or processes only the first section. |

**Calibration:** A wirerust-style *metadata analyzer* that **rejects multi-section with a clear, specific error** is squarely within the range of peer behavior — it is *better* than the silent mis-attribution Wireshark exhibited (#16531) and at least as safe as gopacket/scapy's uncertain handling. Notably, the Wireshark #16531 mis-attribution bug is the *exact* failure mode F-06 feared from pcap-file — but it was Wireshark, not pcap-file, that had it. Reject-with-clear-error is a principled choice for a tool whose job is correctness of extracted metadata.

*Sources: Wireshark sample-captures wiki; Wireshark-bugs mailing list 2016-02; GitLab wireshark issue #16531 (later-section packets use first section's interfaces); IETF pcap/pcapng slides (libpcap reads pcapng); Zeek issue #864; inferred design patterns for scapy/gopacket (flagged uncertain). Perplexity deep-research, 2026-06-19.*

---

### Finding 5 — REJECT is ZERO-COST for wirerust's target corpus and workflows. Question 5.

- **Intended unlock corpus is 100% single-section.** `arp-baseline-16pkt.cap`, `smb3.pcapng`, Wireshark `dump.pcapng`, `tls12-dsb.pcapng` are all Wireshark/dumpcap/PacketLife output — single SHB by construction (Finding 2; corroborated by the completeness-validation report). Reject rejects **none** of them.
- **Mainstream analyst workflows don't yield multi-section files.** Interactive Wireshark capture, dumpcap rotation, and `mergecap`-based merging all produce single-section output (Finding 2). An analyst would have to *deliberately* `cat` pcapng files together to trip the reject path.
- **Failure is graceful and actionable.** When reject does fire, it is a clear per-file `E-INP-012` ("multi-section pcapng not supported"), not a panic or a silent wrong answer. In directory-mode, ensure (consistent with E-INP-005 and the F-11 recommendation) this is a *per-file* error that does not abort the whole run — so one concatenated file doesn't block a batch.
- **Cheap self-service remediation exists.** A user who hits this can run `mergecap -w out.pcapng in.pcapng` (or `editcap`) to flatten to single-section, which is the ecosystem-recommended path anyway. The reject message should hint at this.

**Verdict:** Reject is genuinely **zero-cost** for the target users this cycle. The residual cost is purely the rare power-user who hand-concatenates pcapng and feeds it directly to wirerust — and they have a one-command fix.

*Sources: completeness-validation report (corpus single-section); Finding 2 producer analysis; ADR-009 Consequences. 2026-06-19.*

---

## pcap-file 2.0.0 behavior: RESOLVED (no runtime probe required)

The question that gated this decision is **no longer inconclusive.** Source inspection of the 2.0.0 release is definitive: **per-section interface reset IS implemented** (`self.interfaces.clear()` on every `Block::SectionHeader`; section-local `packet_interface()` lookup; `SectionHeader` exposed to the consumer; no error/stop at the boundary).

**If F3 nonetheless wants a belt-and-suspenders runtime probe** (recommended as a one-time regression guard if SUPPORT is ever adopted, optional under REJECT), the minimal probe is:

> **Probe fixture:** Craft a 2-section pcapng byte buffer:
> `SHB1 + IDB(linktype=ETHERNET) + EPB(interface_id=0, distinctive payload A) + SHB2 + IDB(linktype=RAW or distinct) + EPB(interface_id=0, distinctive payload B)`.
> **Assertion under SUPPORT:** Feed to `PcapNgParser`/`PcapNgReader`; after the second SHB, assert `interfaces().len() == 1` (reset occurred, not accumulated to 2) and that `packet_interface()` for payload-B's EPB resolves to section 2's IDB (the distinct linktype), NOT section 1's. If `interfaces().len() == 2` or payload B resolves to ETHERNET, the reset failed (it does not — but the probe guards against a future crate regression).
> **Assertion under REJECT (current policy):** Feed to wirerust's reader; assert it returns `Err` mapping to `E-INP-012` after consuming section 1 and before yielding any packet from section 2 — exactly the assertion BC-2.01.010 AC-002 / Canonical Test Vector already specifies.

The reject-side assertion is already in the BC and should ship regardless. The support-side probe is only needed if/when SUPPORT is adopted.

---

## Cost-to-support estimate (summary)

| | REJECT (current) | SUPPORT (future) |
|---|---|---|
| Added code | ~0 (reject branch + E-INP-012, already specified) | Low: SUPPORT-1 ~10-30 LOC (drop reject; extend cross-section linktype-agreement); SUPPORT-2 ~30-60 LOC (own interface Vec keyed on `Block::SectionHeader`) |
| Test burden | 1 crafted 2-section fixture asserting `E-INP-012` (already in BC-2.01.010) | Repurpose that fixture to assert per-section attribution + add 1-2 fixtures with *differing* per-section interface layouts |
| Corpus value this cycle | Unblocks 100% of intended corpus | +0 over reject (no corpus file is multi-section) |
| Risk | Clear error on a rare input; zero silent-corruption risk | Must keep single-`DataLink` invariant honest across sections; touches the deferred per-packet-DataLink boundary |
| Recommendation | **Adopt now** | **Defer** until a real multi-section user requirement appears |

---

## Required edits (correctness fixes flowing from this research)

These correct factual errors; they do not change the REJECT outcome.

1. **ADR-009 Decision 7 rationale (clause 1)** currently states pcap-file 2.0.0 "accumulates IDBs in a single growing interface list with no visible per-section reset ... attempting to read a multi-section file ... would silently mis-attribute packets." **This is factually incorrect** (Finding 1). Replace with: "pcap-file 2.0.0 *does* reset the interface table per section (`interfaces.clear()` on each SHB, confirmed from source 2026-06-19), so silent mis-attribution is NOT a risk. wirerust nonetheless rejects multi-section because such files are rare, absent from the intended corpus, and a clear `E-INP-012` error is the tight-scope choice; full support is a cheap future enhancement deferred for lack of a current requirement." Keep `E-INP-012` and the reject decision unchanged.

2. **Completeness-validation F-06** should be annotated as **superseded by this report**: the "likely does NOT reset" conclusion was source/shape inference; direct source inspection shows it DOES reset. F-04 ("interface-index reset is correctly specified") is unaffected and is in fact *delivered* by the crate, not merely specified.

3. **ADR-009 Consequences / BC-2.01.010** text that justifies reject via "pcap-file 2.0.0's unverified per-section reset behavior" should drop "unverified" — the behavior is verified (and correct). The reject justification becomes "rarity + corpus + scope discipline + clear-error-over-edge-case," not "parser can't be trusted."

4. **E-INP-012 message** should hint at remediation, e.g.: `"multi-section pcapng not supported (second Section Header Block at block #<seq>); flatten with 'mergecap -w out.pcapng <file>' or 'editcap' and retry"`. Confirm directory-mode treats it as a per-file (not per-run) error (consistent with E-INP-005 / F-11).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) pcap-file 2.0.0 multi-section state-machine behavior (reset vs accumulate vs error vs stop), `interfaces()`/`section()` semantics, `Block::SectionHeader` exposure; (2) real-world prevalence of multi-section pcapng + which tools produce them + peer-tool read support (Wireshark/tshark, libpcap/tcpdump, scapy, gopacket, Zeek). |
| WebFetch | 4 | docs.rs/pcap-file/**2.0.0** rendered source: `parser.rs.html` (the decisive `interfaces.clear()` on SectionHeader, `interfaces.push()` on IDB, `packet_interface()` body, no-error-on-2nd-SHB) and `reader.rs.html` (delegation + SectionHeader exposure); `struct.PcapNgParser.html` (accessor doc strings). Version-confirmed 2.0.0. |
| WebSearch | 1 | Locate the correct source path for the pcap-file parser state machine (GitHub tag 404'd; redirected to docs.rs rendered source). |
| Read (local) | 4 | Context files: completeness-validation report (F-06), ADR-009, BC-2.01.010, Cargo.toml (confirmed `pcap-file = "2"`). |
| Training data | 0 areas | None relied upon for findings; every load-bearing claim is sourced to the 2.0.0 source or live deep-research with citations. |

**Total MCP tool calls:** 2 `perplexity_research` (PRIMARY) + 4 WebFetch + 1 WebSearch = 7 external calls.
**Training data reliance:** low. The decisive Finding 1 is from the *actual rendered 2.0.0 source* on docs.rs (not inference, not training data, not "latest"); the two deep-research syntheses were treated as leads only and were explicitly *overruled* on the reset question by direct source reading, because both could reach only "most plausible interpretation." Prevalence and peer-tool findings are from cited Wireshark/IETF/Netresec/Zeek/GitLab sources via deep research.

**Note on the F-06 reversal:** This is a case where the prior deep-research's hedged "likely does NOT reset" was wrong, and going to the primary source corrected it. The earlier report itself flagged this item as the one that "should be empirically settled before F3 locks the multi-section AC" — that settlement is this document.
