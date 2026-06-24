## PR Review — #306 (STORY-129, BC-2.11.035) — per-finding `mitre_attack` JSON enrichment

**Verdict: APPROVE**

Independent fresh-eyes review of the diff, PR description, and demo evidence. Reviewed every
changed file. All 10 CI checks green (Test, Clippy, Format, Deny, Audit, gates). No blocking
findings, no warnings, no nits.

### What I verified

**Contract correctness (all confirmed against the diff):**

- **Additive / non-breaking.** `FindingJsonDto` uses `#[serde(flatten)] inner: &'a Finding`, so
  every existing finding key (including `mitre_techniques`) is serialized verbatim; `mitre_attack`
  is the only new key. The sole production wiring change is one line in `json.rs`
  (`"findings": findings_dto`). CSV and terminal reporters are untouched. (AC-8/AC-9/AC-10)
- **Order-preserving.** `mitre_techniques.iter().map(...).collect()` preserves declaration order;
  AC-4 asserts index 0/1 for `T1692.001`,`T0836`.
- **Unknown ID keeps `id`+`reference`, omits optionals.** The `None => (None, None, None)` arm plus
  `#[serde(skip_serializing_if = "Option::is_none")]` suppresses the keys entirely (not null).
  AC-2 asserts absence, not null.
- **Empty omits field.** `#[serde(skip_serializing_if = "Vec::is_empty")]` on `mitre_attack`. AC-3.
- **Duplicates preserved.** 1:1 map, no dedup; AC-5 asserts len 3 with repeated `T1046`.
- **Sub-technique dot preserved.** `id.clone()` verbatim + `format!(".../{id}/")`; AC-6 on `T1071.001`.
- **17-variant `MitreTactic` → TA-id mapping.** Exhaustive `match` (no wildcard) over all variants;
  Enterprise values canonical (TA0043, TA0042, TA0001–TA0011 incl. C2=TA0011/Exfil=TA0010,
  Impact=TA0040); ICS = TA0107/TA0106/TA0105. The drift guard added to `vp007_catalog_drift_guard`
  asserts every seeded ID maps through `technique_tactic_id`, plus a `T9999` None canary.

**Consistency safety.** `tactic_name` derives from `tactic.to_string()` while `tactic_id` derives
from `technique_tactic_id(id)`. Both resolve through `technique_tactic(id)`
(= `technique_info(id).map(|(_, t)| t)`), so the name and the id cannot describe different tactics —
there is no divergence path. The exhaustive match means a future `MitreTactic` variant fails to
compile rather than silently producing a `None` tactic_id.

**Rust idiom / safety.** No `unwrap`/`panic`/`expect` on the production path (`.expect()` appears
only in tests). The `'a` lifetime on the DTO correctly borrows `&Finding` within the serialize call.
Visibility is well-scoped: `pub(crate)` DTO types and `pub(crate) mod json_dto` keep this out of the
public API; `technique_tactic_id` is `pub`, consistent with its sibling `technique_tactic`.

**Test quality.** 13 tests, non-tautological — they assert concrete catalog values (names, TA-ids,
synthesized URLs), not just structural shape. EC coverage is strong: EC-008 mixed-batch per-finding
independence, EC-009 Enterprise sub-technique (`T1557.002` → TA0006), EC-010 ICS lateral movement
(`T0830` → TA0008), and the catalog drift guard.

**Demo evidence.** `docs/demo-evidence/STORY-129/evidence-report.md` present with a coverage map.
GIF + WebM recordings for the JSON success path (AC-1/4/7/8) and the CSV non-breaking path (AC-9).
The unrecorded ACs (AC-2/3/5/10) require synthetic inputs with no fixture pcap and are explicitly
mapped to their covering unit tests — a reasonable and clearly-documented decision, not a gap.

**Regression risk: minimal.** The flatten wrapper preserves the prior JSON shape; the only behavioral
delta is the additive `mitre_attack` key on findings that carry techniques.

### Findings

None. (No rubber-stamp: the verification above is what I checked, file-by-file, against the stated
contract and the test assertions.)

### Scope / size

Production code delta is small and focused (`json_dto.rs` new + 38-line `mitre.rs` addition + 1-line
`json.rs` wire + module export). The 891 additions are dominated by 13 tests and binary demo assets.
Conventional commit / semantic PR title (`feat:`) passes CI. Closes #64.
