---
document_type: holdout-scenario
level: ops
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-076.md
  - .factory/stories/STORY-077.md
  - .factory/stories/STORY-078.md
  - .factory/stories/STORY-079.md
  - .factory/stories/STORY-080.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.023.md
  - .factory/specs/behavioral-contracts/ss-11/BC-2.11.024.md
input-hash: "bfce575"
traces_to: .factory/stories/STORY-076.md
id: "HS-083"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-8"
behavioral_contracts:
  - BC-2.11.024
  - BC-2.11.023
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: CSV Optional Fields Use Empty Strings for None; Direction Is CamelCase Debug

## Scenario

A security toolchain integrator imports wirerust CSV output into a data pipeline that detects
missing fields by checking for empty strings — not sentinel strings like "null", "N/A",
or "-". The tool must never write those sentinels.

1. A finding with `mitre_techniques = vec![]` (empty Vec), `source_ip = None`, `direction = None`,
   and `timestamp = None` is rendered via CsvReporter.
2. The CSV row for this finding has columns 6, 7, 8, and 9 all as empty strings `""`.
   Column 6 (`mitre_techniques`) is empty because `vec![].join(";")` produces `""`.
   Columns 7–9 (`source_ip`, `direction`, `timestamp`) are empty because
   `unwrap_or_default()` on `None` produces `""`.
3. None of the four columns 6–9 (one Vec-backed, three Option-backed) contain the string `"null"`, `"N/A"`, or `"-"`.
4. A separate finding with `direction = Some(ClientToServer)` produces column 8 as
   `"ClientToServer"` (CamelCase, not `"client_to_server"` or `"client-to-server"`).
5. A separate finding with `direction = Some(ServerToClient)` produces column 8 as
   `"ServerToClient"`.
6. A finding with `source_ip = Some(2001:db8::1)` produces column 7 as `"2001:db8::1"`.
7. A finding with `timestamp = Some(datetime)` produces column 9 as an RFC 3339 string.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.024 | Postcondition 1: empty mitre_techniques Vec → empty string | Column 6 of all-empty/None finding is `""` (join of empty vec) |
| BC-2.11.024 | Postcondition 2: None source_ip → empty string | Column 7 of all-None finding is `""` |
| BC-2.11.024 | Postcondition 3: direction → Debug CamelCase | ClientToServer and ServerToClient encoding |
| BC-2.11.024 | Postcondition 4: None timestamp → empty string | Column 9 of all-None finding is `""` |
| BC-2.11.024 | Invariant 4: no sentinel values for None | No "null", "N/A", "-" in any optional column |
| BC-2.11.023 | Postcondition 2/3: summary/analyzer_summaries not in output | No extra rows or values from those |

## Verification Approach

Invoke `CsvReporter::render` with:
1. A Finding where `mitre_techniques = vec![]` (empty Vec) and all three Option fields
   (`source_ip`, `direction`, `timestamp`) are None.
2. A Finding where `direction = Some(ClientToServer)`.
3. A Finding where `source_ip = Some(IpAddr::V6([0x20, 0x01, 0x0d, 0xb8, ...]))`.
4. A Finding where `timestamp = Some(DateTime<Utc>)` set to a known UTC time.

Parse the output CSV and verify:
- For finding 1: assert columns 6-9 are each `""` (empty); assert none contain "null", "N/A", "-".
- For finding 2: assert column 8 = `"ClientToServer"`.
- For finding 3: assert column 7 = `"2001:db8::1"`.
- For finding 4: assert column 9 matches RFC 3339 format pattern.

## Evaluation Rubric

- **Functional correctness** (weight: 0.4): All None fields map to empty strings; all Some fields encode correctly.
- **Edge case handling** (weight: 0.25): Direction uses Debug format (CamelCase), not lowercase or display; IPv6 compact format.
- **Error quality** (weight: 0.1): Valid RFC 4180 output in all cases.
- **Performance** (weight: 0.05): Completes immediately.
- **Data integrity** (weight: 0.2): Columns 1-5 (non-optional) are unaffected; neutralize_csv_injection is still applied to optional field derived values.

## Edge Conditions

- `source_ip = Some(127.0.0.1)` produces `"127.0.0.1"` (IPv4 dotted notation, compact).
- `mitre_techniques = vec!["=HYPERLINK(...)"]` — the singleton is joined to `"=HYPERLINK(...)"` then neutralized to `"'=HYPERLINK(...)"` via csv_injection guard.
- All four columns 6–9 are empty in the same row when `mitre_techniques = vec![]` and the three Option fields are None: a four-column sequence of empty strings.
  Column 6 is empty via `vec![].join(";") == ""`; columns 7–9 via `unwrap_or_default()` on None.
- Summary and AnalysisSummary are non-empty but must not appear in the CSV output.

## Failure Guidance

"HOLDOUT LOW: HS-083 (satisfaction: 0.XX) -- CSV optional fields used sentinel values ('null', 'N/A') instead of empty strings for None, or Direction was not in CamelCase Debug format, breaking the downstream parser contract."
