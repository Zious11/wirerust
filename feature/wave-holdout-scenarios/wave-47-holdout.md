---
document_type: holdout-scenario
version: "1.3"  # F-PB-001: absent-UA → present-but-empty UA trigger; F-C-03: [Uncategorized] → ## Uncategorized; F-O-02: drop HTTP/1.1 from evidence examples; F-E-01: HS-W47-006 re-grounded on real path-traversal emission; F-H-001: --output json/csv → --json/--csv throughout
status: draft
producer: product-owner
timestamp: 2026-06-17T00:00:00Z
waves: [47]
cycle: v0.8.0-finding-collapse
stories: [STORY-118]
feature_id: e18-finding-collapse
github_issue: 259
confirmed_constants:
  collapse_evidence_samples_k: 3
  collapse_key_fields: [category, verdict, confidence, summary]
  default_collapse_on: true
  no_collapse_flag: "--no-collapse"
  json_and_csv_unaffected: true
  mitre_group_bypasses_collapse: true
---

# Wave 47 Holdout Scenarios: Finding-Collapse Terminal Display (v0.8.0)

> **Purpose:** End-to-end integration holdout for the terminal finding-collapse feature
> (issue #259). Validates that repeated identical findings in the terminal output are
> collapsed by default, that the opt-out flag restores one-line-per-finding rendering,
> that JSON/CSV outputs are invariant to the collapse pass, that the K=3 evidence cap
> and positional no-slide window operate correctly, and that the grouped (`--mitre`) path
> remains collapse-free.
>
> **Evaluator note:** These scenarios are BLIND — the evaluator observes only the CLI
> surface and its stdout/stderr output. No source code is read. Pass/fail is determined
> entirely by the EXACT EXPECTED OUTPUT specified in each scenario assertion.
>
> **Information asymmetry invariant:** The evaluator MUST NOT read:
> - `src/reporter/terminal.rs`
> - `src/reporter/json.rs` / `src/reporter/csv.rs`
> - `src/cli.rs` or `src/main.rs`
> - `tests/reporter_terminal_tests.rs`
> - Any implementation PR diff or commit diff for STORY-118
>
> The evaluator observes ONLY the tool's public surface (binary output, exit codes,
> JSON/CSV text) and the holdout assertions below.
>
> **Empty-UA canonical grounding:** The canonical finding tested in HS-W47-001 mirrors
> the exact emission from `src/analyzer/http.rs` (around line 359-371 in v0.7.x). The
> trigger is `parsed.user_agent.as_deref() == Some("")` — a User-Agent header that is
> **PRESENT with an empty value** (wire bytes: `User-Agent:\r\n`). An ABSENT User-Agent
> header (header omitted entirely) returns `None` and is deliberately ignored by the
> analyzer. Each matching request emits one `(Anomaly, Inconclusive, Low, "Empty
> User-Agent header")` finding with `evidence: [<method> <uri>]` (e.g., `"GET /path"`
> — method + space + URI, NO ` HTTP/1.1` version suffix), `source_ip: None`,
> `mitre_techniques: []`. Evidence strings are distinct per request. Timestamps may differ.

---

## Per-Wave Gate Summary

| Wave | Story | Gate Criteria |
|------|-------|---------------|
| 47 | STORY-118 | Default-on collapse, --no-collapse opt-out, K=3 evidence cap, positional no-slide, severity-agnostic, JSON/CSV unaffected, grouped-mode bypass, MITRE from member[0], determinism |

---

## HS-W47-001: Flood Collapse — Empty-User-Agent Flood Collapses to One Annotated Group

**Scope:** STORY-118 (BC-2.11.025 PC-1, BC-2.11.026 PC-1, BC-2.11.027 PC-2)
**Priority:** P0 (must-pass)
**Wave:** 47

### Setup

A capture file contains 50 HTTP requests, each carrying a User-Agent header that is
**present but empty** (wire bytes: `User-Agent:\r\n` — value is the empty string, which
produces `parsed.user_agent.as_deref() == Some("")` at the analyzer). The analyzer emits
50 `(Anomaly, Inconclusive, Low, "Empty User-Agent header")` findings, one per request,
each with a distinct evidence string (e.g., `"GET /req/0001"` through `"GET /req/0050"`)
in `method + space + URI` format. All four collapse-key fields are identical across all
50 findings: category=Anomaly, verdict=Inconclusive, confidence=Low,
summary="Empty User-Agent header".

**Abstractly:** A finding-set with 50 identical-key findings and distinct evidence URIs.

### Command

```
wirerust analyze --http <pcap>
```

(default output: terminal, no flags overriding collapse behavior)

### Expected Assertions

1. The FINDINGS section of the terminal output contains exactly ONE display group for
   the key `(Anomaly, Inconclusive, Low, "Empty User-Agent header")`. There is no second
   occurrence of the string "Empty User-Agent header" as a standalone finding header line.
2. That one group header line contains the exact substring `"(x50)"` — specifically the
   format ` (x50)` (space, open-paren, x, integer, close-paren).
3. Exactly 3 evidence lines appear under that header, each prefixed `    > `. No fourth
   evidence line appears. The 3 evidence lines are from the first 3 findings in emission
   order (e.g., `GET /req/0001`, `GET /req/0002`, `GET /req/0003`).
4. No other FINDINGS count suffix ` (xN)` appears elsewhere in the terminal output.
5. The tool exits with code 0.

### Evaluation Rubric

- **Collapse fires (weight 0.5):** Single group header with `(x50)` in terminal output.
- **Evidence cap (weight 0.3):** Exactly 3 evidence lines rendered; no 4th.
- **Exit code (weight 0.1):** Exit 0.
- **No suffix pollution (weight 0.1):** No other ` (xN)` suffixes on unrelated findings.

### Failure Guidance

"HOLDOUT LOW: HS-W47-001 (satisfaction: 0.XX) — flood collapse did not fire; terminal
showed N individual lines instead of one collapsed group with (x50) suffix, or evidence
cap was not enforced."

---

## HS-W47-002: --no-collapse Restores One-Line-Per-Finding

**Scope:** STORY-118 (BC-2.11.028 PC-2, BC-2.11.026 Inv-2)
**Priority:** P0 (must-pass)
**Wave:** 47

### Setup

Same input as HS-W47-001: 50 identical-key `(Anomaly, Inconclusive, Low, "Empty
User-Agent header")` findings, each with a distinct evidence URI.

### Command

```
wirerust analyze --http --no-collapse <pcap>
```

### Expected Assertions

1. The FINDINGS section contains exactly 50 individual finding header lines, each
   containing the text "Empty User-Agent header".
2. No ` (xN)` suffix appears anywhere in the FINDINGS section output — not ` (x50)`,
   not ` (x1)`, not any variant.
3. Every finding's full evidence is rendered (1 evidence line per finding, none elided).
4. The terminal output is byte-identical to what would have been emitted by a v0.7.x
   (pre-collapse) run against the same capture.
5. The tool exits with code 0.

### Evaluation Rubric

- **No collapse suffix (weight 0.5):** Zero occurrences of ` (x` pattern in FINDINGS output.
- **Correct finding count (weight 0.3):** Exactly 50 individual header lines.
- **Full evidence (weight 0.1):** Evidence rendered for each finding.
- **Exit code (weight 0.1):** Exit 0.

### Failure Guidance

"HOLDOUT LOW: HS-W47-002 (satisfaction: 0.XX) — --no-collapse flag had no effect;
terminal still showed collapsed output with (x50) suffix instead of 50 individual lines."

---

## HS-W47-003: Singleton (N=1) Unchanged — No (xN) Suffix, Full Evidence

**Scope:** STORY-118 (BC-2.11.026 PC-2, BC-2.11.027 Inv-6, BC-2.11.029 PC-3)
**Priority:** P0 (must-pass)
**Wave:** 47

### Setup

A capture file that produces exactly ONE unique finding: one HTTP request carrying a
User-Agent header that is **present but empty** (`User-Agent:\r\n`). The finding is
`(Anomaly, Inconclusive, Low, "Empty User-Agent header")` with evidence
`["GET /single"]` and no MITRE techniques.

**Abstractly:** A finding-set containing exactly one finding (no repetition).

### Command

```
wirerust analyze --http <pcap>
```

### Expected Assertions

1. The FINDINGS section contains exactly one header line for that finding.
2. That header line does NOT contain any `(x1)` or `(xN)` suffix in any form. The line
   matches the exact pre-v0.8.0 format:
   `  [Anomaly] INCONCLUSIVE (LOW) - Empty User-Agent header`
3. The single evidence line `    > GET /single` is rendered below the header.
4. The overall FINDINGS section output is byte-identical to what v0.7.x would have
   produced for the same single-finding input.
5. The tool exits with code 0.

### Evaluation Rubric

- **No suffix (weight 0.6):** No `(x` substring appears in the FINDINGS section.
- **Evidence rendered (weight 0.2):** Evidence line present and untruncated.
- **Exit code (weight 0.1):** Exit 0.
- **Byte-identity (weight 0.1):** Header format matches pre-v0.8.0 exactly.

### Failure Guidance

"HOLDOUT LOW: HS-W47-003 (satisfaction: 0.XX) — singleton finding rendered with a
` (x1)` suffix when it should have rendered without any count annotation."

---

## HS-W47-004: K=3 Evidence Cap — N=5 Group Shows Exactly 3 Evidence Lines, First K Positional

**Scope:** STORY-118 (BC-2.11.027 PC-2, Inv-2)
**Priority:** P0 (must-pass)
**Wave:** 47

### Setup

A finding-set of 5 identical-key findings where each finding has exactly one evidence
line, distinct per finding: `evidence[0]="GET /path/a"`,
`evidence[1]="GET /path/b"`, ..., `evidence[4]="GET /path/e"`.
The findings appear in the input slice in the order a, b, c, d, e (alphabetical by path).

**Abstractly:** A 5-member collapse group where every member has exactly 1 evidence line.

### Command

```
wirerust analyze --http <pcap>
```

### Expected Assertions

1. The FINDINGS section contains exactly one collapsed group with a `(x5)` suffix.
2. Exactly 3 evidence lines appear under that header, prefixed `    > `:
   - Line 1: `    > GET /path/a`
   - Line 2: `    > GET /path/b`
   - Line 3: `    > GET /path/c`
3. Evidence lines for `/path/d` and `/path/e` do NOT appear in the terminal output.
4. A fourth evidence line (of any form) does NOT appear.

### Evaluation Rubric

- **Cap fires (weight 0.5):** Exactly 3 evidence lines rendered.
- **Positional order (weight 0.3):** First 3 paths in input order (a, b, c), not d or e.
- **No 4th line (weight 0.2):** No evidence line for paths d or e.

### Failure Guidance

"HOLDOUT LOW: HS-W47-004 (satisfaction: 0.XX) — K=3 cap not enforced; terminal showed
4 or more evidence lines for a 5-member group."

---

## HS-W47-005: Empty First Member — Window Does Not Slide; Total Evidence = 2

**Scope:** STORY-118 (BC-2.11.027 PC-2, Inv-2 no-slide rule)
**Priority:** P0 (must-pass)
**Wave:** 47

### Setup

A finding-set of 5 identical-key findings arranged so that:
- `members[0].evidence = []` (empty — no evidence)
- `members[1].evidence = ["GET /b"]`
- `members[2].evidence = ["GET /c"]`
- `members[3].evidence = ["GET /d"]`
- `members[4].evidence = ["GET /e"]`

The positional window inspects the first min(5, 3) = 3 members: member[0], member[1],
member[2]. Member[0] contributes 0 lines (empty vec). Member[1] contributes 1 line.
Member[2] contributes 1 line. The window does NOT slide past member[0] to reach member[3].
Members[3] and members[4] are never inspected.

**Abstractly:** A 5-member group where the first member has empty evidence.

### Command

```
wirerust analyze --http <pcap>
```

### Expected Assertions

1. The FINDINGS section contains exactly one collapsed group with a `(x5)` suffix.
2. Exactly 2 evidence lines appear under the header:
   - `    > GET /b`
   - `    > GET /c`
3. Evidence from `/d` and `/e` does NOT appear. No "slide" from member[0]'s empty vec
   to member[3] occurs.
4. No blank `    > ` line appears for member[0]'s empty evidence.
5. A third evidence line does NOT appear.

### Evaluation Rubric

- **Correct count (weight 0.5):** Exactly 2 evidence lines (not 3 from d/e via slide).
- **No slide (weight 0.3):** Lines b and c appear; lines d and e do NOT.
- **No blank evidence line (weight 0.2):** No empty `    > ` for the empty-evidence member.

### Failure Guidance

"HOLDOUT LOW: HS-W47-005 (satisfaction: 0.XX) — evidence window slid past the
empty-evidence first member; terminal showed evidence from member[3] (path /d)
instead of stopping at 2 lines."

---

## HS-W47-006: Severity-Agnostic Collapse — Likely/High Identical Findings Collapse

**Scope:** STORY-118 (BC-2.11.025 PC-7, EC-014)
**Priority:** P0 (must-pass)
**Wave:** 47

### Setup

A capture file contains 3 HTTP requests each with URI `/../../etc/passwd`, triggering
the path-traversal detector at `src/analyzer/http.rs:200-217`. Each request produces a
finding with the identical 4-field collapse key:
- `category: Reconnaissance`
- `verdict: Likely`
- `confidence: High`
- `summary: "Path traversal in URI: /../../etc/passwd"`

Each finding's evidence is `"URI: /../../etc/passwd"` (all three are identical — evidence
identity does not prevent collapse, only the 4-field key governs grouping per BC-2.11.025).
Each finding carries `mitre_techniques: ["T1083"]`.

**Abstractly:** A 3-member Likely/High collapse group — the highest-severity finding the
HTTP analyzer emits — confirming collapse is severity-agnostic.

### Command

```
wirerust analyze --http <pcap>
```

### Expected Assertions

1. The FINDINGS section contains exactly one collapsed group with a `(x3)` suffix.
2. The header line contains the substring `"(x3)"`.
3. The header is rendered in red bold color (if the terminal supports ANSI and --no-color
   is not set): the Likely+High color ladder branch fires (terminal.rs:212:
   `Likely + High → red().bold()`).
4. The `(x3)` suffix is visually inside the red-bold color span (the suffix is colorized
   with the rest of the header, not appended after the color reset).
5. Alternatively, if the evaluator runs with `--no-color`: the header line reads
   `  [Reconnaissance] LIKELY (HIGH) - Path traversal in URI: /../../etc/passwd (x3)`
   with no ANSI codes, confirming the suffix is part of the pre-colorization string.

### Evaluation Rubric

- **Collapse fires for high-severity (weight 0.5):** `(x3)` suffix on a Likely/High group.
- **Color positioning (weight 0.3):** Suffix inside color span (use `--no-color` to verify
  suffix is present without color if needed).
- **Correct count (weight 0.2):** Exactly ` (x3)` not ` (x2)` or other count.

### Failure Guidance

"HOLDOUT LOW: HS-W47-006 (satisfaction: 0.XX) — severity-agnostic collapse did not fire
for Likely/High path-traversal findings; either no collapse occurred, or the (x3) suffix
appeared outside the red-bold color span."

---

## HS-W47-007: JSON Output Unaffected — N=1000 Identical Findings, Terminal Collapses, JSON Has 1000 Objects

**Scope:** STORY-118 (BC-2.11.029 PC-1, Inv-1/3)
**Priority:** P0 (must-pass)
**Wave:** 47

### Setup

A finding-set (or capture with HTTP flood) that produces exactly 1000 identical-key
`(Anomaly, Inconclusive, Low, "Empty User-Agent header")` findings. The terminal
collapse feature is enabled (default).

**Abstractly:** A large identical-key group, checked against JSON output.

### Command

```
wirerust analyze --http --json <pcap>
```

### Expected Assertions

1. The JSON output is a valid JSON object parseable by `jq` without error.
2. The `findings` array in the JSON output contains exactly 1000 objects.
3. Every object in the `findings` array has the fields: `category`, `verdict`,
   `confidence`, `summary`. All 1000 objects have identical values for these four fields.
4. No "count" field, "collapsed" field, or any aggregation artifact appears in any JSON
   finding object. The JSON schema is unmodified.
5. The tool exits with code 0.

Additional verification (if terminal output is also captured separately):
6. Running the same command without `--json` (terminal mode) produces exactly 1
   collapsed group header with `(x1000)` — confirming the collapse applies to terminal
   but not JSON.

### Evaluation Rubric

- **JSON finding count (weight 0.6):** Exactly 1000 objects in `findings` array.
- **No aggregation artifact (weight 0.2):** No `count` or `collapsed` field in JSON objects.
- **Valid JSON (weight 0.1):** Parseable by `jq '.findings | length'` returning `1000`.
- **Exit code (weight 0.1):** Exit 0.

### Failure Guidance

"HOLDOUT LOW: HS-W47-007 (satisfaction: 0.XX) — JSON output had fewer than 1000 finding
objects (collapse was incorrectly applied upstream of the reporter dispatch), or the
findings array contained aggregation artifacts."

---

## HS-W47-008: CSV Output Unaffected — N=5 Identical Findings Produce 5 CSV Rows

**Scope:** STORY-118 (BC-2.11.029 PC-2)
**Priority:** P0 (must-pass)
**Wave:** 47

### Setup

A finding-set with exactly 5 identical-key `(Anomaly, Inconclusive, Low, "Empty UA")`
findings, each with one distinct evidence URI.

### Command

```
wirerust analyze --http --csv <pcap>
```

### Expected Assertions

1. The CSV output contains exactly one header row plus exactly 5 data rows (6 lines total
   if no trailing newline issues, or 7 with a trailing newline — the point is exactly 5
   data rows).
2. Each data row contains the same `category`, `verdict`, `confidence`, `summary` values.
3. No "count" column or aggregation artifact appears in any row. The CSV column schema
   is the standard 9-column schema (unchanged from pre-v0.8.0).
4. The `evidence` column of each row contains the per-finding evidence (not aggregated).
5. The tool exits with code 0.

### Evaluation Rubric

- **CSV row count (weight 0.6):** Exactly 5 data rows.
- **No aggregation artifact (weight 0.2):** Standard 9-column schema; no extra count column.
- **Per-row evidence (weight 0.1):** Evidence column populated per individual finding.
- **Exit code (weight 0.1):** Exit 0.

### Failure Guidance

"HOLDOUT LOW: HS-W47-008 (satisfaction: 0.XX) — CSV output had fewer than 5 data rows,
suggesting collapse was incorrectly applied to the CSV reporter."

---

## HS-W47-009: Grouped Mode (--mitre) Bypasses Collapse — No (xN) Suffix in Grouped Output

**Scope:** STORY-118 (BC-2.11.025 Inv-5, BC-2.11.026 EC-007/EC-009)
**Priority:** P0 (must-pass)
**Wave:** 47

### Setup

A finding-set with 100 identical-key `(Anomaly, Inconclusive, Low, "Empty User-Agent
header")` findings. Collapse is enabled by default.

**Abstractly:** A 100-member identical-key group rendered in grouped (MITRE) mode.

### Command

```
wirerust analyze --http --mitre <pcap>
```

### Expected Assertions

1. The FINDINGS section contains exactly 100 individual finding lines (one per finding).
2. No ` (xN)` suffix of any form appears on any line in the FINDINGS section — not
   ` (x100)`, not ` (x2)`, not any variant.
3. The output is byte-identical to what would have been rendered by `--mitre` in v0.7.x
   (grouped path is structurally unmodified by STORY-118).
4. Tactic section headers (e.g., `## Uncategorized` — the `## <Tactic>` markdown-style
   header emitted by `render_findings_grouped`) appear as usual in the grouped output.
5. The tool exits with code 0.

### Evaluation Rubric

- **No collapse suffix (weight 0.5):** Zero occurrences of ` (x` substring in FINDINGS output.
- **Correct individual count (weight 0.3):** 100 individual finding lines present.
- **Grouped structure intact (weight 0.1):** Tactic headers and grouped format unmodified.
- **Exit code (weight 0.1):** Exit 0.

### Failure Guidance

"HOLDOUT LOW: HS-W47-009 (satisfaction: 0.XX) — grouped mode (--mitre) showed ` (xN)`
suffixes when it should be structurally suffix-free; collapse incorrectly applied to the
grouped render path."

---

## HS-W47-010: MITRE Line Sources group_members[0] — Divergent mitre_techniques Across Group

**Scope:** STORY-118 (BC-2.11.026 PC-7, BC-2.11.017 PC-6)
**Priority:** P0 (must-pass)
**Wave:** 47

### Setup

A finding-set of 3 identical-key findings (same category/verdict/confidence/summary) where:
- `members[0].mitre_techniques = ["T1036"]`
- `members[1].mitre_techniques = []`
- `members[2].mitre_techniques = ["T1059"]`

All three share the same 4-field collapse key. Collapse is enabled (default).

**Abstractly:** A 3-member collapse group with divergent MITRE technique arrays.

### Command

```
wirerust analyze --http <pcap>
```

### Expected Assertions

1. The collapsed group header shows `(x3)`.
2. A MITRE line `    MITRE: T1036` appears under the header (from member[0]'s
   `mitre_techniques`).
3. No MITRE line for `T1059` appears in the terminal output for this group. Member[2]'s
   `mitre_techniques` are elided from terminal output.
4. No second MITRE line (e.g., `    MITRE: T1059`) appears for this collapsed group.

Additionally, when run with `--json`:
5. All 3 JSON finding objects preserve their individual `mitre_techniques` values:
   object[0] has `["T1036"]`, object[1] has `[]` (or no key), object[2] has `["T1059"]`.
   The MITRE elision is terminal-display-layer only.

### Evaluation Rubric

- **Correct representative MITRE (weight 0.5):** Only T1036 appears in terminal for group.
- **No T1059 in terminal (weight 0.3):** T1059 does NOT appear in terminal FINDINGS for
  this collapsed group.
- **JSON MITRE preserved (weight 0.2):** All 3 JSON objects carry their original
  `mitre_techniques` values.

### Failure Guidance

"HOLDOUT LOW: HS-W47-010 (satisfaction: 0.XX) — collapsed MITRE line used wrong member's
techniques (not group_members[0]), or member[2]'s T1059 appeared in terminal output."

---

## HS-W47-011: Determinism — Same Input Produces Byte-Identical Output on Repeated Runs

**Scope:** STORY-118 (BC-2.11.025 PC-9, Inv-7)
**Priority:** P0 (must-pass)
**Wave:** 47

### Setup

A finding-set with 5 groups of 3 findings each, interleaved in the input slice
(e.g., order A, B, C, A, D, B, E, C, A, D, B, E, C, D, E — where A–E are distinct
collapse keys). Each group has 3 members. Collapse is enabled.

**Abstractly:** A multi-group input with interleaved members; run twice identically.

### Command (run twice with the same input)

```
wirerust analyze --http <pcap> > out1.txt
wirerust analyze --http <pcap> > out2.txt
diff out1.txt out2.txt
```

### Expected Assertions

1. `diff out1.txt out2.txt` produces zero output (files are identical).
2. The 5 collapsed groups appear in the terminal output in first-occurrence order: group A
   first (first input occurrence = index 0), group B second (first input occurrence = index 1),
   group C third, group D fourth, group E fifth.
3. Each group shows `(x3)`.
4. The tool exits with code 0 on both runs.

### Evaluation Rubric

- **Byte-identical output (weight 0.6):** diff returns exit 0 (no differences).
- **First-occurrence order (weight 0.3):** Groups A, B, C, D, E in that order.
- **Correct counts (weight 0.1):** Each group shows `(x3)`.

### Failure Guidance

"HOLDOUT LOW: HS-W47-011 (satisfaction: 0.XX) — repeated runs produced different group
ordering, indicating a non-deterministic accumulator (e.g., HashMap) was used instead of
the required Vec accumulator."

---

## HS-W47-012: Real-World Corpus — Known-Good HTTP Traffic (Low False-Positive Rate for Collapse)

**Scope:** STORY-118 (BC-2.11.025, BC-2.11.029; regression guard)
**Priority:** P0 (must-pass)
**Wave:** 47

### Corpus

**Source:** Any publicly available well-maintained HTTP pcap with normal browser traffic
(e.g., a Wireshark sample HTTP capture, or the standard HTTP wiki.wireshark.org sample
file `http.cap`). A capture with varied requests from different user agents, some with
present User-Agent headers, some with standard headers.

**Expected finding profile (pre-collapse):** Few or zero empty-UA findings. Possibly some
other HTTP anomaly findings (path traversal, header anomalies) but at low frequency, each
likely unique.

### Command

```
wirerust analyze --http <pcap>
```

### Expected Assertions

1. The tool completes without panic, crash, or non-zero exit code.
2. If any collapsed group appears (rare for clean traffic): the `(xN)` suffix is
   syntactically correct.
3. If no collapsed groups appear (expected for clean traffic): the FINDINGS section
   renders normally with no `(xN)` suffixes anywhere.
4. The JSON output (`--json`) contains the same number of finding objects as would
   be expected from the known finding count of the corpus (no findings are lost due to
   collapse being inadvertently applied upstream).
5. The overall structure of the terminal output (PROTOCOLS, SERVICES, FINDINGS sections)
   is intact and correct.

### Evaluation Rubric

- **No crash / no panic (weight 0.4):** Tool completes, exit 0.
- **Output structure intact (weight 0.3):** Expected sections present; no garbled output.
- **JSON finding count consistent (weight 0.2):** JSON and terminal agree on finding count
  (modulo collapse grouping in terminal).
- **Syntactically correct suffixes (weight 0.1):** Any `(xN)` suffixes present are
  well-formed.

### Failure Guidance

"HOLDOUT LOW: HS-W47-012 (satisfaction: 0.XX) — known-good corpus produced a crash or
panic, or the collapse feature corrupted the output structure on clean traffic."

---

## HS-W47-013: Real-World Corpus — Known-Problematic HTTP Traffic (Empty-UA Flood Detected)

**Scope:** STORY-118 (BC-2.11.025 canonical test vector, BC-2.11.027 PC-2; real-world)
**Priority:** P0 (must-pass)
**Wave:** 47

### Corpus

**Source:** A crafted or captured pcap where many HTTP requests carry a User-Agent header
that is **present but empty** (wire bytes: `User-Agent:\r\n` — the value is the empty
string `""`). This is the `Some("")` trigger at `http.rs:359`; requests that omit the
header entirely (`None`) do NOT trigger this finding and must NOT be used. Suitable
sources include:
- A synthetically generated pcap (e.g., via scapy or a raw socket harness) with 20–500
  HTTP/1.1 requests each including `User-Agent:\r\n` in the header block
- A capture from any HTTP client configured to send an empty User-Agent value
  (not `--no-user-agent` / omit, but explicitly `User-Agent:` with empty value)

**Expected finding profile:** 20+ identical `(Anomaly, Inconclusive, Low, "Empty
User-Agent header")` findings, each with a distinct evidence string (`"GET <uri>"` format).

### Command

```
wirerust analyze --http <pcap>
```

### Expected Assertions

1. The FINDINGS section shows exactly one collapsed group for the empty-UA findings.
2. The `(xN)` suffix on that group reflects the actual count (e.g., `(x20)` for a 20-
   request corpus; the exact number must match the corpus size).
3. Exactly K=3 evidence lines appear under the group header (if N>3; if N≤3, up to N
   lines appear).
4. The overall noise reduction is measurable: instead of 20+ lines, exactly one header
   line with a count annotation is visible — the core UX goal of issue #259 is achieved.
5. Running the same command with `--json` produces N individual JSON objects (N =
   the actual number of requests with a present-but-empty User-Agent header), confirming
   forensic completeness.

### Evaluation Rubric

- **UX goal achieved (weight 0.4):** One collapsed header instead of N lines.
- **Correct count in suffix (weight 0.3):** `(xN)` matches corpus finding count.
- **Evidence cap enforced (weight 0.1):** At most 3 evidence lines.
- **JSON completeness (weight 0.2):** JSON has N objects (no loss of forensic data).

### Failure Guidance

"HOLDOUT LOW: HS-W47-013 (satisfaction: 0.XX) — known-problematic empty-UA flood corpus
showed N individual lines instead of one collapsed group, defeating the feature's UX goal."

---

## Wave 47 Holdout Summary

| HS ID | Title | Priority | BCs |
|-------|-------|----------|-----|
| HS-W47-001 | Flood Collapse — Empty-UA Flood Collapses to One Annotated Group | P0 | BC-2.11.025 PC-1, BC-2.11.026 PC-1, BC-2.11.027 PC-2 |
| HS-W47-002 | --no-collapse Restores One-Line-Per-Finding | P0 | BC-2.11.028 PC-2, BC-2.11.026 Inv-2 |
| HS-W47-003 | Singleton (N=1) Unchanged — No (xN) Suffix, Full Evidence | P0 | BC-2.11.026 PC-2, BC-2.11.027 Inv-6, BC-2.11.029 PC-3 |
| HS-W47-004 | K=3 Evidence Cap — N=5 Group Shows Exactly 3 Evidence Lines | P0 | BC-2.11.027 PC-2, Inv-2 |
| HS-W47-005 | Empty First Member — Window Does Not Slide; Total Evidence = 2 | P0 | BC-2.11.027 PC-2, Inv-2 no-slide |
| HS-W47-006 | Severity-Agnostic Collapse — Likely/High Identical Findings Collapse | P0 | BC-2.11.025 PC-7, EC-014 |
| HS-W47-007 | JSON Output Unaffected — N=1000 Identical Findings, Terminal Collapses, JSON Has 1000 Objects | P0 | BC-2.11.029 PC-1, Inv-1/3 |
| HS-W47-008 | CSV Output Unaffected — N=5 Identical Findings Produce 5 CSV Rows | P0 | BC-2.11.029 PC-2 |
| HS-W47-009 | Grouped Mode (--mitre) Bypasses Collapse — No (xN) Suffix in Grouped Output | P0 | BC-2.11.025 Inv-5, BC-2.11.026 EC-007/EC-009 |
| HS-W47-010 | MITRE Line Sources group_members[0] — Divergent mitre_techniques Across Group | P0 | BC-2.11.026 PC-7, BC-2.11.017 PC-6 |
| HS-W47-011 | Determinism — Same Input Produces Byte-Identical Output on Repeated Runs | P0 | BC-2.11.025 PC-9, Inv-7 |
| HS-W47-012 | Real-World Corpus — Known-Good HTTP Traffic (Low False-Positive Rate for Collapse) | P0 | BC-2.11.025, BC-2.11.029; regression guard |
| HS-W47-013 | Real-World Corpus — Known-Problematic HTTP Traffic (Empty-UA Flood Detected) | P0 | BC-2.11.025 canonical vector, BC-2.11.027 PC-2 |

**Total wave-47 holdout scenarios: 13**
**P0 must-pass: 13**
**P1 nice-to-have: 0**
