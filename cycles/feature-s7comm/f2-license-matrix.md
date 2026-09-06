# F2 License Matrix — S7comm Prior-Art Clean-Room Gate

**Feature cycle:** `feature-s7comm`
**Gate:** F2 (BLOCKING prerequisite) — mirrors ADR-013 Decision 7 (IEC-104 licensing constraint)
**Date:** 2026-09-06
**Author:** research-agent (jaredbrichards@gmail.com)
**Status:** COMPLETE — clean-room path CONFIRMED VIABLE

## Purpose

wirerust is **MIT OR Apache-2.0** dual-licensed. Policy (per ADR-013 Decision 7, applied
here to S7comm) is a HARD, immutable, PR-review-checked constraint: **GPL / LGPL / copyleft /
proprietary prior art MUST NOT be copied, referenced structurally, or used as an implementation
template.** The S7comm dissector must be clean-roomed from open specifications and permissively
licensed / freely usable design references only.

This document records the license (with SPDX identifier and a cited source) for each candidate
S7comm / ISO-on-TCP prior-art source, and issues a per-source USABLE / BANNED ruling.

---

## Bottom Line (read this first)

**The S7comm feature CAN proceed clean-room — YES**, by exactly the same route IEC-104 took:

- **Lower layers (TPKT / COTP):** implement from **RFC 1006** (IETF, freely usable) + **ITU-T
  X.224 = ISO/IEC 8073** (COTP; official free-download PDF from ITU). These are the authoritative
  open specifications and are safe to use as the primary implementation reference.
- **S7comm classic (`0x32`):** there is **NO official public Siemens specification** — it is a
  proprietary, reverse-engineered protocol. Implementation is inherently from community
  reverse-engineering knowledge. Use **behavioral/field descriptions only** from permissive or
  free-to-read documentation (Wireshark S7comm **wiki page** — prose, not code; academic papers;
  the awesome-industrial-protocols catalog). Do **NOT** read or transcribe the GPL Wireshark
  dissector **source code** or the LGPL Snap7 **source code**.
- **Permissive design references** that ARE safe to consult: `cisagov/icsnpp-s7comm`
  (BSD-3-Clause), `kprovost/libs7comm` (BSD-2-Clause), `python-snap7` (MIT). Use as design
  reference only — no verbatim code copy (consistent with ADR-013's "design reference only,
  no code copy" wording for the MIT/Apache IEC-104 references).

**BANNED (strong/weak copyleft — MUST NOT be copied, structurally referenced, or templated):**
Wireshark `packet-s7comm.c` / `packet-s7comm_plus.c` (**GPL-2.0-or-later**), **Snap7** upstream
(**LGPL-3.0-or-later**), **libnodave** (**LGPL-2.0-or-later**). The C source of these MUST NOT be
opened as an implementation template; their existence and high-level behavior are common
knowledge, but their expressive code (constants, lookup tables, logic structure) is off-limits.

**IP / clean-room implication of "no official spec":** Because S7comm classic has no published
Siemens standard, the implementation is unavoidably derived from reverse-engineered community
knowledge. This is legally **lower-risk than it sounds**: reimplementing observed wire formats /
state behavior for **interoperability** is distinct from copying copyrighted source expression.
The defensible posture is (1) lower layers from RFC 1006 + X.224, (2) S7comm fields from
free-to-read *prose/behavioral* descriptions and permissive references, (3) zero lines lifted
from any GPL/LGPL implementation, (4) describe the result as an "independent implementation
interoperable with Siemens S7 devices," never "official/Siemens-certified." Do not use leaked
specs, decompiled TIA Portal, or NDA material. (Patents/trademarks/export are out of scope for
this copyright-focused gate but flagged for any future commercial-release review; classic S7comm
interoperability is materially lower-risk than bypassing S7CommPlus auth/TLS.)

---

## Per-Source License Matrix

### 1. Dissectors / analyzers (prior-art parsers)

| Source | License | SPDX | Safe to reference? | Notes |
|--------|---------|------|--------------------|-------|
| Wireshark `packet-s7comm.c` (epan/dissectors, upstream `wireshark/wireshark`) | GNU GPL v2 or later | `GPL-2.0-or-later` | **BANNED (code)** | SPDX header **verified in-file**: `SPDX-License-Identifier: GPL-2.0-or-later`; "Copyright 1998 Gerald Combs", author Thomas Wiens 2014. Strong copyleft — source MUST NOT be copied, transcribed, or used as a structural template. See src [A]. |
| Wireshark `packet-s7comm_plus.c` | GNU GPL v2 or later | `GPL-2.0-or-later` | **BANNED (code)** | **Correction:** not in the official Wireshark tree; it is a separately maintained plugin at `moki-ics/s7commwireshark`. File header states "version 2 … or (at your option) any later version." Same BANNED status. S7CommPlus is out of MVP scope anyway. See src [B]. |
| CISA/INL **ICSNPP-S7comm** Zeek analyzer (`cisagov/icsnpp-s7comm`) | BSD 3-Clause "New/Revised" | `BSD-3-Clause` | **USABLE (design ref only)** | LICENSE.txt **verified**: "Copyright (c) 2023, Battelle Energy Alliance, LLC" — BSD 3-Clause. Permissive; safe as a design/field reference. No verbatim copy of Zeek/Spicy code into Rust (attribution + design-reference discipline, mirroring ADR-013's MIT/Apache references). See src [C]. |

### 2. Client / communication libraries (C / Python prior art)

| Source | License | SPDX | Safe to reference? | Notes |
|--------|---------|------|--------------------|-------|
| **libnodave** (Thomas Hergenhahn, SourceForge) | GNU LGPL v2 or later | `LGPL-2.0-or-later` | **BANNED (code)** | SourceForge metadata = "GNU Library or Lesser GPL v2.0"; `nodave.c` header says "version 2, or … any later version" → `-or-later` (not LGPL-2.1). Weak copyleft, still copyleft — source MUST NOT be copied/templated. See src [D]. |
| **libs7comm** (`kprovost/libs7comm`) | BSD 2-Clause "Simplified" | `BSD-2-Clause` | **USABLE (design ref only)** | GitHub identifies repo license as BSD-2-Clause. Permissive. NOTE: its own README says it was "inspired by libnodave" and references Snap7/Wireshark — so treat it as a *design* reference, do not launder LGPL/GPL structure through it. See src [E]. |
| **Snap7** (upstream C++ lib, snap7.sourceforge.net) | GNU LGPL v3 or later | `LGPL-3.0-or-later` | **BANNED (code)** | SourceForge metadata = LGPLv3; source notices say "version 3 … or any later version". Relevant to CISA advisory **AA26-231A** (Snap7 is the referenced S7 tooling). Weak copyleft — source/refman code MUST NOT be copied or templated. Its **documentation prose** may inform behavioral understanding but treat cautiously; prefer neutral specs. See src [F]. |
| **python-snap7** (`gijzelaerr/python-snap7`) | MIT | `MIT` | **USABLE (design ref only)** | LICENSE file = standard MIT text. Permissive — the Python wrapper is safe as a design/behavior reference. (It links the LGPL native Snap7 at runtime, but the Python source itself is MIT.) See src [G]. |

### 3. crates.io Rust crates (search: `s7`, `s7comm`, `snap7`, `iso-on-tcp`, `cotp`, `tpkt`)

Client/active libraries and FFI bindings are listed for completeness; **none should be added to
`Cargo.toml`/`Cargo.lock`** — wirerust is a passive dissector and needs no S7 client, and the
IEC-104 precedent (ADR-013 Decision 7) is that the parser is original Rust with zero external
implementation deps. License column is the crates.io metadata `license` field.

| Crate | Latest ver | License (SPDX) | Maintenance | Client vs passive | Safe to reference? |
|-------|-----------|----------------|-------------|-------------------|--------------------|
| `s7` | 0.1.9 | **non-standard** (crates.io `license` = "non-standard"; described as BSD-flavored custom) | Dormant (last 2019-10-20) | CLIENT (PLC read/write) | **AVOID** — non-standard license string is not a clean permissive grant; do not depend on or template. Verified via crates.io API [H]. |
| `s7-comm` | 0.1.2 | **non-standard** (crates.io `license` = "non-standard") | Dormant (last 2023-06-12) | Protocol codec building block | **AVOID** — unverifiable/custom license; not a clean grant. Verified via crates.io API [I]. |
| `s7-client` | 0.1.2 | not a standard SPDX grant | Dormant (~2023-06) | CLIENT | AVOID — same family as `s7-comm`/`tpkt`/`copt`; unclear license. [J] |
| `rust7` | 0.1.2 | `MIT` | Maintained (2025-08) | CLIENT (native Rust Snap7-style) | Permissive; design ref only if ever needed. Not a dep — wirerust is passive. [J] |
| `turbos7` | 0.2.1 | `MIT OR Apache-2.0` | Active (2026) | CLIENT | Permissive, dual-license matches wirerust; design ref only. [J] |
| `snap7-client` / `snap7-cli` / `snap7-server` / `snap7-proto` / `snap7-partner` / `snap7-opcua-gateway` | 0.1.7 (proto 0.1.0) | `MIT` | Active (2026) | CLIENT / server / codec | Permissive; `snap7-proto` (TPKT/COTP/S7 framing codec) is the most relevant as a *design* reference. Not a dep. [J] |
| `s7commplus` | 0.1.0 | not independently recoverable | Active (2026) | CLIENT (S7CommPlus) | S7CommPlus is out of MVP scope; verify license before any use. [J] |
| `snap7-sys` | 0.1.5 | `MIT` | Dormant (2023-12) | FFI bindings to LGPL Snap7 | MIT wrapper but links LGPL native lib — irrelevant (no FFI in a passive dissector). [J] |
| `rust-snap7` | 1.142.3 | `MulanPSL-2.0` | Maintained (2025) | FFI bindings | MulanPSL-2.0 + bundles LGPL Snap7 — **AVOID**. [J] |
| `snap7-rs` | 1.142.1 | `MulanPSL-2.0` | Dormant (2023); reported security issues | FFI bindings (static-links Snap7) | **AVOID** — copyleft-adjacent bundling + security concerns. [J] |
| `tpkt` | 0.1.0 | non-standard | Dormant (2023-05) | TPKT framing (active stack) | AVOID — unclear license; RFC 1006 is the clean source anyway. [J] |
| `copt` | 0.1.0 | non-standard | Dormant (2023-05) | COTP codec (name = transposed "COTP") | AVOID — unclear license; X.224 is the clean source. [J] |
| `rusty-cotp` / `rusty-tpkt` | 1.2.0 | not independently recoverable | Active (2026) | COTP / TPKT stack | Verify license before any reference; prefer implementing from RFC 1006 + X.224 directly. [J] |
| `packet_parser` | 10.4.0 | `MIT` | Active | **PASSIVE** dissector (recognizes S7comm/COTP/TPKT) | The closest analog (passive). MIT — permissive; design ref only, no verbatim copy. [J] |
| `cotp` (the crate) | — | — | — | **FALSE POSITIVE** — a TOTP/HOTP authenticator, unrelated to COTP | Ignore. [J] |

> **Ruling for §3:** No crate is adopted as a dependency. `wirerust` implements an original,
> passive Rust S7comm/COTP/TPKT parser (ADR-013 Decision 7 precedent: "original Rust parser only;
> zero lines borrowed"). Permissive crates (MIT / `MIT OR Apache-2.0`) may be consulted as
> *design references* only; `non-standard`/`MulanPSL-2.0`/unrecoverable-license crates are AVOID.

### 4. Open specifications (clean-room reference basis)

| Source | License / usage status | SPDX-ish | Safe to reference? | Notes |
|--------|------------------------|----------|--------------------|-------|
| **RFC 1006** — ISO Transport Service on top of TCP (defines **TPKT**) | IETF RFC, Internet Standard **STD 35**; freely downloadable/copyable/distributable under IETF Trust terms (text copyrighted, not public domain; implementing the wire protocol ≠ copying prose) | n/a (IETF) | **USABLE — PRIMARY** | Authoritative open spec for TPKT framing. Implement from packet formats; preserve notices only if reproducing RFC text/code components. See src [K]. |
| **ISO/IEC 8073 = ITU-T X.224** — COTP (Connection-Oriented Transport Protocol) | ISO edition is a paid/licensed standard, BUT the technically **identical** text is published as **ITU-T X.224** with an **official free-download PDF** from ITU (ITU confirms X.224 ≡ ISO/IEC 8073:1997) | n/a (ITU-T) | **USABLE — PRIMARY** | Use the free X.224 PDF as the clean-room COTP reference. "Free download" ≠ public domain — do not republish substantial standard text/tables/diagrams; implement the specified behavior. See src [L]. |
| **S7comm classic (`0x32`)** protocol description | **NO official public Siemens specification exists** — proprietary; all public documentation is community reverse-engineering | n/a | **USABLE with discipline** (prose/behavioral only) | Layering is TCP → TPKT → COTP → S7comm. Free-to-read *behavioral* sources: Wireshark **S7comm wiki page** (prose), academic papers (Kleinmann & Wool 2014), Orange-Cyberdefense `awesome-industrial-protocols`. **Do NOT** transcribe the GPL Wireshark dissector *source* or LGPL Snap7 *source*. Document as independently derived interoperable behavior. See src [M]. |

---

## Clean-Room Procedure (recommended, mirrors IEC-104)

1. **Lower layers:** implement TPKT + COTP from **RFC 1006** and **ITU-T X.224** only.
2. **S7comm classic fields:** derive from free-to-read **prose/behavioral** descriptions
   (Wireshark wiki page, academic papers) and permissive references (icsnpp-s7comm BSD,
   python-snap7 MIT) — **design reference only, no verbatim code**.
3. **Hard bans:** never open GPL Wireshark dissector source, LGPL Snap7 source, or LGPL
   libnodave source as an implementation template; never copy constants, lookup tables, or
   logic structure from them.
4. **No external S7 dependency** in `Cargo.toml`/`Cargo.lock` — original Rust parser only
   (ADR-013 Decision 7 precedent).
5. **Provenance:** record which official spec editions were consulted; keep the implementation,
   comments, and docs independently written; describe output as "interoperable with Siemens S7,"
   not "official/certified."

## Unverifiable / flagged items

- crates.io `license` field returns literally **"non-standard"** for `s7` and `s7-comm`
  (verified via crates.io API) — meaning the author supplied a custom/non-SPDX license string.
  These are treated as **AVOID** (not a clean permissive grant). Do not rely on them.
- Several 2026-era crates (`rusty-cotp`, `rusty-tpkt`, `s7commplus`, `snap7-client` family,
  `scadaver`, `casket`, `packrat-tui`) have licenses not independently recoverable from the
  rendered pages in this pass; **flagged** — verify the crates.io metadata directly before any
  reference. None are needed (implement from RFC 1006 + X.224), so this does not block the gate.
- S7CommPlus (`0x72`) sources are noted but **out of MVP scope**; its dissectors are GPL and its
  security involves auth/TLS bypass with higher IP/legal risk — defer.

---

## Source citations

- **[A]** Wireshark `packet-s7comm.c` — SPDX header verified in-file (`GPL-2.0-or-later`):
  https://github.com/wireshark/wireshark/blob/master/epan/dissectors/packet-s7comm.c
  (raw fetched 2026-09-06). Wireshark overall license: https://github.com/wireshark/wireshark/blob/master/COPYING
- **[B]** `packet-s7comm_plus.c` (external plugin, GPL-2.0-or-later):
  https://github.com/moki-ics/s7commwireshark/blob/master/src/s7comm_plus/packet-s7comm_plus.c
- **[C]** ICSNPP-S7comm LICENSE.txt verified (BSD-3-Clause, "Copyright (c) 2023, Battelle Energy Alliance, LLC"):
  https://github.com/cisagov/icsnpp-s7comm/blob/main/LICENSE.txt
- **[D]** libnodave — SourceForge project page (LGPL v2.0): https://sourceforge.net/projects/libnodave/ ;
  `nodave.c` "version 2 or later" header (mirror): https://github.com/jogibear9988/libnodave/blob/master/libnodave/nodave.c ;
  SPDX `LGPL-2.0-or-later`: https://spdx.org/licenses/LGPL-2.0-or-later.html
- **[E]** libs7comm LICENSE (BSD-2-Clause): https://github.com/kprovost/libs7comm/blob/master/LICENSE
- **[F]** Snap7 — SourceForge project page (LGPLv3): https://sourceforge.net/projects/snap7/ ;
  LGPL-3.0 text mirror: https://github.com/gijzelaerr/snap7-debian/blob/master/lgpl-3.0.txt ;
  SPDX `LGPL-3.0-or-later`: https://spdx.org/licenses/LGPL-3.0-or-later.html
- **[G]** python-snap7 LICENSE (MIT): https://github.com/gijzelaerr/python-snap7/blob/master/LICENSE
- **[H]** `s7` crate — crates.io API verified: license "non-standard", v0.1.9, updated 2019-10-20:
  https://crates.io/api/v1/crates/s7 (https://crates.io/crates/s7)
- **[I]** `s7-comm` crate — crates.io API verified: license "non-standard", v0.1.2, updated 2023-06-12:
  https://crates.io/api/v1/crates/s7-comm (https://crates.io/crates/s7-comm)
- **[J]** crates.io registry search (`s7`, `s7comm`, `snap7`, `iso-on-tcp`, `cotp`, `tpkt`) —
  aggregated via Perplexity deep research over crates.io / lib.rs / docs.rs (2026-09-06); per-crate
  crates.io pages at https://crates.io/crates/<name>. License fields marked "not independently
  recoverable" are flagged for direct re-verification.
- **[K]** RFC 1006 — https://www.rfc-editor.org/info/rfc1006/ ; IETF Trust legal provisions:
  https://trustee.ietf.org/documents/trust-legal-provisions/
- **[L]** ITU-T X.224 (= ISO/IEC 8073:1997) free PDF:
  https://www.itu.int/rec/T-REC-X.224 ; ISO catalogue: https://www.iso.org/standard/24077.html
- **[M]** S7comm proprietary/reverse-engineered status — Wireshark S7comm wiki:
  https://wiki.wireshark.org/S7comm ; Hampel-Soft S7 communication notes ("reverse-engineered"):
  https://dokuwiki.hampel-soft.com/kb/production/s7-communication ;
  Kleinmann & Wool 2014, "Accurate Modeling of the Siemens S7 SCADA Protocol":
  https://research.ibm.com/haifa/Workshops/security2014/present/Avishai_Wool_AccurateModelingoftheSiemensS7SCADAProtocol-v5.pdf ;
  Orange-Cyberdefense catalog: https://github.com/Orange-Cyberdefense/awesome-industrial-protocols/blob/main/protocols/s7comm.md

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) Licenses of Wireshark s7comm dissectors, ICSNPP-S7comm, libnodave, libs7comm, Snap7, python-snap7 with SPDX + source URLs; (2) crates.io S7/COTP/TPKT crate inventory + metadata, and RFC 1006 / X.224 / S7comm open-spec clean-room analysis |
| WebFetch | 4 | Direct verification: Wireshark `packet-s7comm.c` SPDX header (raw), cisagov `LICENSE.txt`, crates.io API for `s7` and `s7-comm` license/version/date fields |
| Read | 1 | ADR-013 (IEC-104) Decision 7 precedent for the licensing-constraint pattern |
| Glob | 2 | Locate ADR-013 and any prior license artifacts |
| Training data | 1 area | SPDX identifier semantics (GPL/LGPL/BSD/MIT distinctions) — cross-checked against cited sources, not sole basis for any determination |

**Total MCP tool calls:** 2 (both `perplexity_research`, high depth) + 4 WebFetch verifications
**Training data reliance:** low — every license determination is backed by a cited registry/repo
source; the four most load-bearing (GPL Wireshark ban, BSD ICSNPP, non-standard `s7`/`s7-comm`
crates) were independently verified via direct WebFetch.
