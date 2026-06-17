# Research: Larger / higher-rate ARP captures for the v0.7.0 ARP Security Analyzer e2e corpus

- **Date:** 2026-06-16
- **Type:** general (technology / fixture sourcing)
- **Requested by:** ARP e2e corpus extension (extend `tests/fixtures/E2E-PCAPS.md` ARP table)
- **Goal:** Find publicly downloadable PCAPs that (P1) exercise the **shipped default** `--arp-storm-rate 50` (sustained >50 ARP req/sec → D3 fires at default), (P2) stress D1/D12 at larger scale than the current 14 MB `arpspoof.pcap`, and (P3) provide heavy background ARP for false-positive-rate stress.

---

## TL;DR / headline finding

**There is no publicly documented PCAP with a confirmed sustained ARP request rate above 50/sec.** This is a real, well-corroborated gap in the public dataset landscape, not a search miss — two independent deep-research passes plus targeted GitHub repo probes all converge on the same conclusion. Public network datasets annotate IP/transport-layer phenomena (TCP/UDP/HTTP floods, Modbus function codes) and treat ARP as unlabelled link-layer background. The only explicitly-labelled "ARP storm" public capture is the very file we already ship (`arp-storm.pcap`, ~10–20 req/sec), which is below the default threshold by design.

**Consequences for the three priorities:**

- **P1 (D3 at default 50 fps):** No real public capture verifiably satisfies this. The **best primary validator is a synthetic generator** (`mk_arp_storm_pcap.py`), mirroring the existing `mk_modbus_large_pcap.py` pattern already in the repo. This is the recommended path and is fully deterministic, redistributable, and rate-controllable. See "Best primary validator" below.
- **P2 (larger D1/D12 MITM):** The **University of Coimbra ICS_PCAPS** dataset (`tjcruz-dei/ICS_PCAPS`) contains a real ettercap-style ARP-based MITM scenario at far larger scale than our 14 MB `arpspoof.pcap`. Strong, verified-live candidate — but the ARP rate is *inferred* low (maintenance-rate poisoning), so it stresses D1/D12, **not** D3.
- **P3 (background ARP):** The same ICS_PCAPS DoS captures (ICMP/SYN/Modbus flooding over 0.5 h windows) and the existing 4SICS captures provide large mixed traffic with incidental ARP for false-positive stress.

---

## Candidate table (drop-in rows for `tests/fixtures/E2E-PCAPS.md` ARP table)

D-code legend: **D1** spoof · **D2** GARP · **D3** storm · **D11** malformed · **D12** L2/L3 MAC mismatch.
"Rate basis" is **STATED** (source asserts it) or **INFERRED** (must be measured/derived).

| # | Suggested filename | Direct download URL | Source | ARP behaviors (D-codes) | Approx pkt count / ARP rate | Rate basis | License / attribution | Verified-live? (probe result) |
|---|--------------------|---------------------|--------|--------------------------|-----------------------------|-----------|------------------------|-------------------------------|
| 1 | *(synthetic)* `arp-storm-fast.pcap` | n/a — generate via `tests/fixtures/mk_arp_storm_pcap.py` (new) | synthetic, repo-local generator (mirrors `mk_modbus_large_pcap.py`) | **D3** (primary); can also seed D1/D12 variants | tunable; e.g. 6 000 ARP req over 60 s = **100 req/sec** sustained | STATED (you set it) | repo-authored, fully redistributable | n/a (generated locally; deterministic checksum) |
| 2 | `ics-mitm-arp.pcap` (extract from `captures1_v2.zip` → `mitm/eth2dump-mitm-change-15m-0,5h_1.pcap`) | `https://github.com/tjcruz-dei/ICS_PCAPS/releases/download/MODBUSTCP%231/captures1_v2.zip` | [tjcruz-dei/ICS_PCAPS](https://github.com/tjcruz-dei/ICS_PCAPS) (Univ. of Coimbra, ATENA H2020 "Modbus TCP SCADA #1") | **D1** spoof + **D12** MAC mismatch (ettercap-style ARP MITM, on-the-fly Modbus change); likely **D2** GARP during poison warm-up | 30-min capture, 15-min active MITM; per-file pkt count **not published** (must `capinfos`). ARP rate **inferred low** (maintenance poisoning, few/sec) | INFERRED (no source states tool or ARP rate) | Academic dataset, freely downloadable; **credit Univ. of Coimbra / T. Cruz et al.**; keep-local-only (no explicit redistribution license) | **YES** — `…/MODBUSTCP%231/captures1_v2.zip` returns **302** to `release-assets.githubusercontent.com` CDN; GitHub API reports asset size **669,680,240 B (~670 MB)**, `application/octet-stream` |
| 3 | `ics-dos-background.pcap` (extract from `captures2.zip` or `captures3.zip` → `pingFloodDDoS/` or `tcpSYNFloodDDoS/`) | `https://github.com/tjcruz-dei/ICS_PCAPS/releases/download/MODBUSTCP%231/captures2.zip` (≈194 MB) · `…/captures3.zip` (≈224 MB) | same ICS_PCAPS release | Background ARP under IP-layer DoS (ICMP/SYN/Modbus-query flooding) — exercises **false-positive rate** (analyzer must NOT fire D1/D2/D3 on benign ARP amid heavy IP flood) | multi-hundred-MB; ARP is incidental, **rate not documented** | INFERRED | same as #2 — credit Univ. of Coimbra; keep-local-only | **YES** (same release; API sizes 194,810,610 B and 224,051,839 B; URLs 302 to GitHub CDN) |
| 4 | `arp-storm.pcap` *(already shipped — reference baseline)* | `https://gitlab.com/wireshark/wireshark/-/wikis/uploads/__moin_import__/attachments/SampleCaptures/arp-storm.pcap` | [Wireshark SampleCaptures](https://wiki.wireshark.org/SampleCaptures) | **D3** at `--arp-storm-rate 10` only; silent at default 50 | 622 ARP req; "more than 20 req/sec" (wiki) but ~10/sec average (our analysis) | STATED (wiki) / our measured average | Wireshark Foundation public sample | **YES** — re-probed: **200/HTTP OK**, `application/octet-stream`, **46.2 KB** (matches our committed sha256) |

### Candidates investigated and REJECTED (negative results — recorded so this is not re-searched)

| Source | Why rejected for our needs |
|--------|----------------------------|
| `StopDDoS/packet-captures` (and `wqrld.net/captures.zip` mirror) | All captures are L3/L4 amplification/reflection (DNS/SNMP/memcached/UDP/TCP-SYN). **Zero ARP/L2 attack captures.** Probed file list directly — no ARP. |
| `researcher111/ARP-pcap-files` | Repo contains **only** `arpspoof.pcap` (our existing 14 MB file). No larger siblings. |
| Wireshark SampleCaptures (rest of page) | Only `arp-storm.pcap` + its pcapng variant `220703_arp-storm.pcapng` (same packets, larger metadata) are ARP-storm-labelled; nothing >50/sec. |
| CIC-DDoS2019, UNSW-NB15 | DDoS/intrusion datasets; ARP present but **unlabelled and not high-rate** (targets known by IP, no ARP discovery flood). Useful only as injectable background, not as direct ARP-storm source. |
| `arptools/arpflood`, ettercap, bettercap, ARPFloodTool repos | **Tools** that generate ARP floods — they ship no pre-recorded flood PCAPs. (Relevant only as generators for the synthetic route.) |
| CyberTalents "ARP Storm" CTF challenge | Behind login; no direct file URL; rate undocumented. Not a public direct download. |
| Netresec MITM PCAPs (Marczak Turkey/Egypt) | In-path/injection attacks, not ARP floods; ARP rate undocumented. |

---

## Best primary validator for "D3 storm fires at the default 50 fps threshold"

**Winner: a synthetic generator `tests/fixtures/mk_arp_storm_pcap.py` (NEW), not a downloaded capture.**

Rationale (this is the load-bearing recommendation):

1. **No real public capture satisfies the requirement.** Confirmed by two independent `perplexity_research` deep passes (high + medium effort) and direct repo probes. Spending effort hunting further has diminishing returns — the gap is structural (ARP is link-layer background; privacy concerns discourage sharing raw ARP/MAC data; "ARP storm" sample captures are pedagogical and deliberately mild).
2. **The repo already establishes the pattern.** `tests/fixtures/mk_modbus_large_pcap.py` is a tracked, deterministic Python generator whose output (`modbus-large.pcap`) is regenerated and checksum-verified by `bin/fetch-e2e-pcaps`. A sibling `mk_arp_storm_pcap.py` fits the existing architecture exactly — no new tooling, no storage-backend question, no licensing risk.
3. **It directly and precisely exercises the default.** Emit e.g. 6 000 ARP requests evenly over 60 s of pcap timestamps (100 req/sec) from one source MAC — sustained, well above 50 fps, so D3 fires at the **shipped default** `--arp-storm-rate 50`. The rate is STATED (you control it), so the validation assertion is exact and stable, not "approximately, if the link was busy."
4. **Fully redistributable + deterministic.** Repo-authored output can live committed or gitignored-and-regenerated; deterministic byte output gives a stable sha256 for the index — consistent with the gitignored-local-samples + checksum policy.

**Suggested generator spec (to hand to the implementer):**
- Output `tests/fixtures/local-samples/arp-storm-fast.pcap` (classic pcap magic `d4 c3 b2 a1`, Ethernet linktype — wirerust does not yet read pcapng, per the `arp-baseline-16pkt.cap` note).
- N = 6 000 broadcast ARP requests (op=1), single source MAC `02:00:00:00:00:01` (locally-administered), sweeping sender→target IPs across a /24 so it reads as a scan-storm.
- Timestamps: uniform 60 s span → 100 req/sec sustained → crosses default 50 fps with margin.
- Deterministic (no RNG, or fixed seed) so the sha256 is stable.
- Optionally emit sibling severities (`arp-storm-60fps.pcap` just over threshold, `arp-storm-40fps.pcap` just under) to test threshold edges and confirm the default stays silent below 50 — the negative test that `arp-storm.pcap` currently provides.

**Best REAL-capture candidate (for the other priorities):** ICS_PCAPS `captures1_v2.zip` → `mitm/` (Candidate #2). It is the strongest real, verified-live download for **P2** (larger-scale D1/D12 MITM than our 14 MB `arpspoof.pcap`) — a genuine ettercap-style ARP MITM in a Modbus testbed. Caveat to encode in the index: its ARP rate is **inferred low**, so it must be filed under D1/D12, **not** D3. Per-file packet/ARP counts are **not published** by the dataset — run `capinfos` / `tshark -Y arp` after extraction and record measured values before adding the row.

---

## Licensing constraints (consistent with gitignored-local-samples policy)

- **Synthetic `arp-storm-fast.pcap` (Candidate #1):** repo-authored → fully redistributable. Safe to commit OR regenerate-on-fetch. No attribution constraint.
- **ICS_PCAPS (#2, #3):** public academic dataset, freely downloadable, but **no explicit redistribution license** in the repo. Treat as **keep-local-only** (download via `bin/fetch-e2e-pcaps`, gitignored under `tests/fixtures/local-samples/`, never committed) — identical handling to `arpspoof.pcap` / 4SICS. **Credit: University of Coimbra / T. Cruz, P. Abreu et al. (ATENA H2020, "Modbus TCP SCADA #1").**
- **`arp-storm.pcap` (#4):** already in the index; Wireshark Foundation public sample; keep-local-only as currently handled.

---

## How to add the winner(s) to `bin/fetch-e2e-pcaps` + `E2E-PCAPS.md`

### A. Synthetic D3 validator (recommended first)

1. Author `tests/fixtures/mk_arp_storm_pcap.py` (deterministic generator, spec above). Run it once; capture the resulting sha256 (`shasum -a 256 …`).
2. In `bin/fetch-e2e-pcaps`, add a **second synthetic block** alongside the existing Modbus one:
   ```bash
   ARP_SYNTH_NAME="arp-storm-fast.pcap"
   ARP_SYNTH_SHA="<sha256-from-step-1>"
   ARP_SYNTH_GEN="$ROOT/tests/fixtures/mk_arp_storm_pcap.py"
   # …then mirror the existing "regenerate the synthetic capture" + verify block.
   ```
3. Add a row to the ARP table in `tests/fixtures/E2E-PCAPS.md` (Validates: "**D3 storm fires at the DEFAULT `--arp-storm-rate 50`** — 6 000 ARP req over 60 s = 100 req/sec sustained; synthetic, deterministic"). Note in the "Direct download URLs" section that this file is generator-produced, not downloaded.

### B. ICS_PCAPS MITM (real, larger D1/D12 — local-only)

1. `bin/fetch-e2e-pcaps` downloads `.pcap` files directly; ICS_PCAPS ships **nested ZIPs**, so this needs an extract step (download `captures1_v2.zip`, `unzip` the `mitm/` member, then checksum the extracted `.pcap`). Either extend the script with an unzip helper or document the manual extraction in `E2E-PCAPS.md`.
2. After extracting `eth2dump-mitm-change-15m-0,5h_1.pcap`, **measure before documenting**: `capinfos <file>` (pkt count, duration) and `tshark -r <file> -Y arp -T fields -e frame.time_epoch | …` (per-second ARP rate). Run `target/release/wirerust analyze --arp <file>` and record actual D1/D12 finding counts.
3. Add the row with measured values (size, sha256, real pkt count, measured ARP rate) and the Coimbra attribution. Keep gitignored under `local-samples/`.

> Per CLAUDE.md `DF-VALIDATION-001`: this research validates the sourcing finding; any follow-up "no public >50 fps capture exists" drift note destined for a GitHub issue is now research-backed by this report.

---

## Sources

- Wireshark SampleCaptures (`arp-storm.pcap`, "more than 20 ARP req/sec", pcapng variant): <https://wiki.wireshark.org/SampleCaptures> ; direct file probed live at the GitLab wiki mirror.
- `tjcruz-dei/ICS_PCAPS` (Univ. of Coimbra, ATENA H2020 "Modbus TCP SCADA #1"; release assets `captures1_v2.zip`/`captures2.zip`/`captures3.zip`; `mitm/`, `pingFloodDDoS/`, `tcpSYNFloodDDoS/`, `modbusQueryFlooding/`; naming `eth2dump-mitm-change-15m-0,5h_1.pcap`): <https://github.com/tjcruz-dei/ICS_PCAPS/releases> (asset sizes/URLs from GitHub API; download URL probed live → 302 to GitHub release CDN).
- IMPACT Cyber Trust "ICS PCAPS" entry (dataset description: small-scale Modbus/TCP process automation, ML research).
- Frazão, Abreu, Cruz, Simões, Monteiro — "Denial of Service Attacks: Detecting the Frailties of Machine Learning Algorithms in the Classification Process" (associated with ICS_PCAPS DoS scenarios; **does not report ARP rates**).
- "Effective Anomaly Detection for Process Control Traffic in Absence of Ground Truth" (uses Modbus TCP SCADA #1; flow features only, **no ARP-rate annotation**).
- `StopDDoS/packet-captures` + `wqrld.net/captures.zip` (probed file list — all L3/L4 amplification, **no ARP**): <https://github.com/StopDDoS/packet-captures>
- `researcher111/ARP-pcap-files` (only `arpspoof.pcap`): <https://github.com/researcher111/ARP-pcap-files>
- bettercap `arp.spoof.interval` (default 1000 ms → ~1 ARP/sec; set <20 ms → >50/sec) — documents *how* to generate, no pre-recorded high-rate PCAP: <https://www.bettercap.org/>
- `arptools/arpflood`, ettercap, `aryapratama88/ARPFloodTool` (ARP-flood generators; ship no flood PCAPs).
- Netresec public PCAP index (no documented >50/sec ARP capture): <https://www.netresec.com/?page=PcapFiles>
- CIC-DDoS2019: <https://www.unb.ca/cic/datasets/ddos-2019.html> ; UNSW-NB15: <https://research.unsw.edu.au/projects/unsw-nb15-dataset> (both IP-layer-labelled, ARP unlabelled — background-only).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (high effort) broad survey of public ARP-storm/flood/DoS captures across Wireshark/Netresec/CIC/UNSW/GitHub/CTF — established the "no >50/sec public capture" finding; (medium effort) deep dive on ICS_PCAPS MITM tooling/rates + ettercap/bettercap high-rate ARP configurability |
| Perplexity perplexity_search | 2 | raw ranked URLs for "arp flood pcap" and "arp spoofing pcap dataset" GitHub repos — surfaced ICS_PCAPS, StopDDoS, ARPFloodTool leads the deep pass under-weighted |
| Perplexity perplexity_ask | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | — |
| Tavily | 0 | — |
| WebFetch | 5 | probed ICS_PCAPS releases page + GitHub API (asset URLs/sizes), StopDDoS file list, researcher111 repo, and **live-probed** ICS_PCAPS `captures1_v2.zip` (302→CDN) + Wireshark `arp-storm.pcap` (200, 46.2 KB) |
| WebSearch | 0 | — |
| Training data | 1 area | wirerust analyzer D-code semantics + repo fixture conventions, taken from the loaded `E2E-PCAPS.md` / `bin/fetch-e2e-pcaps` (not external knowledge) |

**Total MCP tool calls:** 4 (2 `perplexity_research`, 2 `perplexity_search`) + 5 WebFetch probes.
**Training data reliance:** low — every external claim is web-sourced; download URLs were live-probed (HTTP status + size recorded); rates are explicitly tagged STATED vs INFERRED. The central negative finding ("no public >50/sec ARP capture") is independently corroborated by two deep-research passes plus direct repo probing.
