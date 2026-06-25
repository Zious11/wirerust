# Demo Evidence Report — STORY-132

**Story:** CPF Item Walk, CIP Header Parse, and CIP Request Path Extraction
**Story ID:** STORY-132
**Wave:** 59
**Product type:** Pure-core library (no CLI/UI surface — parse functions only)
**Recording tool:** VHS 0.11.0 (terminal recordings of `cargo test --test enip_analyzer_tests`)
**Recorded:** 2026-06-25
**Test result at recording time:** 19 passed / 0 failed / 0 ignored (mod cpf_cip)

---

## AC Coverage Map

| AC | Title | Test filter used | Artifact (GIF) | Artifact (WebM) | Tape |
|----|-------|-----------------|---------------|----------------|------|
| AC-132-001 | `parse_cpf_items` walks CPF items and returns typed item list | `cpf_cip::test_parse_cpf_items` | `AC-001-002-cpf-walk.gif` | `AC-001-002-cpf-walk.webm` | `AC-001-002-cpf-walk.tape` |
| AC-132-002 | `CpfItem` carries type_id and data slice reference | `cpf_cip::test_cpf_item` | `AC-001-002-cpf-walk.gif` | `AC-001-002-cpf-walk.webm` | `AC-001-002-cpf-walk.tape` |
| AC-132-003 | `parse_cip_header` parses CIP header from 0x00B2 item data | `cpf_cip::test_parse_cip_header` | `AC-003-004-cip-header-parse.gif` | `AC-003-004-cip-header-parse.webm` | `AC-003-004-cip-header-parse.tape` |
| AC-132-004 | CIP header parse is 0x00B2-only; 0x00B1 items are skipped | `cpf_cip::test_cip_parse` | `AC-003-004-cip-header-parse.gif` | `AC-003-004-cip-header-parse.webm` | `AC-003-004-cip-header-parse.tape` |
| AC-132-005 | `parse_cip_request_path` extracts Class, Instance, Attribute segments | `cpf_cip::test_parse_cip_path` | `AC-005-cip-request-path.gif` | `AC-005-cip-request-path.webm` | `AC-005-cip-request-path.tape` |
| AC-132-006 | F-P9-002 fuzz obligation stub exists | _structural code review — no automated test_ | — (see note) | — | — |
| AC-132-007 | `classify_cip_service` maps all 15 variants + VP-032 Sub-D Kani harnesses | `cpf_cip::test_classify_cip_service` | `AC-007-classify-and-full-suite.gif` | `AC-007-classify-and-full-suite.webm` | `AC-007-classify-and-full-suite.tape` |

> **AC-132-006 note:** This AC is structural — verified by code review of `/// # Fuzz Obligation
> (F-P9-002)` doc comments on `parse_cpf_items` and `parse_cip_header` in `src/analyzer/enip.rs`.
> No automated test exists for it; the obligation is tracked in the source comment, not a
> unit test. No VHS recording is produced.

> **AC-132-007 VP-032 Sub-D note:** Kani harnesses
> `vp032_cip_service_classification_totality` and `vp032_cip_service_request_partition` in
> `#[cfg(kani)] mod kani_proofs` require `cargo kani` — not exercised by `cargo test`.
> The `AC-007-classify-and-full-suite.*` recording demonstrates the 3/3 classify unit tests
> green (named codes, response-bit, Unknown). Kani proof verification is tracked separately.

---

## Recordings Detail

### AC-001-002-cpf-walk

Demonstrates `parse_cpf_items` walking a CPF byte buffer and returning a `Vec<CpfItem>`,
plus `CpfItem`'s two-field struct shape (`type_id: u16`, `data: Vec<u8>`).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests cpf_cip::test_parse_cpf_items` and
  `cpf_cip::test_cpf_item_type_ids` with grep filter
- Tests cover: single 0x00B2 item (item_count=1, data 4 bytes), two items (0x00B2 + 0x0000),
  empty (item_count=0 → vec![]), truncated (declared length 5 but only 3 bytes remain →
  stops iteration, returns partial list), type_id 0x00B1/0x00B2/0x0000 all stored correctly
- All 5 tests pass green

**Tests in recording:**
- `test_parse_cpf_items_single_item`
- `test_parse_cpf_items_two_items`
- `test_parse_cpf_items_empty`
- `test_parse_cpf_items_truncated`
- `test_cpf_item_type_ids`

---

### AC-003-004-cip-header-parse

Demonstrates `parse_cip_header` decoding `service` and `request_path` from `item_data`
(0x00B2 item bytes), and the F-P9-001 gate that skips 0x00B1 Connected Data Items.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests cpf_cip::test_parse_cip_header` and
  `cpf_cip::test_cip_parse` with grep filter
- Tests cover: request parse (service=0x0E, path=[0x20,0x04]), response parse (service=0x8E
  with 0x80-bit set), too-short input (1 byte → None), truncated path (request_path_size=3
  but only 4 bytes available → None), 0x00B1 items stored but not passed to parse_cip_header,
  0x00B2 items processed normally
- All 6 tests pass green

**Tests in recording:**
- `test_parse_cip_header_request`
- `test_parse_cip_header_response`
- `test_parse_cip_header_too_short`
- `test_parse_cip_header_truncated_path`
- `test_cip_parse_skips_0x00b1_items`
- `test_cip_parse_processes_0x00b2_items`

---

### AC-005-cip-request-path

Demonstrates `parse_cip_request_path` extracting `CipPathSegment::Class(u8)`,
`Instance(u8)`, and `Attribute(u8)` from path bytes using exact-match segment codes.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests cpf_cip::test_parse_cip_path` with grep filter
- Tests cover: empty path (vec![]), Class-only path ([0x20, 0x04] → Class(4)),
  full triple ([0x20,0x04, 0x24,0x01, 0x30,0x03] → Class(4)+Instance(1)+Attribute(3)),
  unrecognized segment type (0x40 skipped; Class(1) after it extracted),
  odd-length path (1 byte — cursor+2 > 1, returns vec![]; no panic)
- All 5 tests pass green

**Tests in recording:**
- `test_parse_cip_path_empty`
- `test_parse_cip_path_class_only`
- `test_parse_cip_path_class_instance_attr`
- `test_parse_cip_path_unrecognized_skip`
- `test_parse_cip_path_odd_length_safe`

---

### AC-007-classify-and-full-suite

Demonstrates `classify_cip_service` mapping all 15 `CipServiceClass` variants correctly,
then shows the complete cpf_cip suite at 19/19 green.

**What the recording shows:**
- First run: `cargo test --test enip_analyzer_tests cpf_cip::test_classify_cip_service`
  with grep — 3 classify tests green (named codes: all 13 verified; response-bit: 0x81,
  0x8E, 0xFF → Response; unknown: 0x7F, 0x06, 0x00 → Unknown)
- Second run: `cargo test --test enip_analyzer_tests cpf_cip` — all 19 tests across
  AC-001..005 and AC-007 pass in a single view

**Tests in recording (classify):**
- `test_classify_cip_service_named_codes`
- `test_classify_cip_service_response_bit`
- `test_classify_cip_service_unknown`

**Full suite (19 tests):**
- All tests in `mod cpf_cip`

---

## Artifacts

```
docs/demo-evidence/STORY-132/
  AC-001-002-cpf-walk.tape                   (VHS script)
  AC-001-002-cpf-walk.gif                    ( 96 KB)
  AC-001-002-cpf-walk.webm                   ( 86 KB)
  AC-003-004-cip-header-parse.tape           (VHS script)
  AC-003-004-cip-header-parse.gif            ( 99 KB)
  AC-003-004-cip-header-parse.webm           (118 KB)
  AC-005-cip-request-path.tape               (VHS script)
  AC-005-cip-request-path.gif                (118 KB)
  AC-005-cip-request-path.webm               ( 93 KB)
  AC-007-classify-and-full-suite.tape        (VHS script)
  AC-007-classify-and-full-suite.gif         (235 KB)
  AC-007-classify-and-full-suite.webm        (302 KB)
  evidence-report.md                         (this file)
```

## Coverage Summary

| AC | Evidenced | Path(s) demonstrated |
|----|-----------|----------------------|
| AC-132-001 | Yes | single item, two items, empty (item_count=0), truncated (bounds violation → partial list) |
| AC-132-002 | Yes | type_id 0x00B2, 0x00B1, 0x0000; 2-field struct confirmed via test assertions |
| AC-132-003 | Yes | request (service+path), response (0x80-bit), too-short (None), truncated path (None) |
| AC-132-004 | Yes | 0x00B1 stored but skipped; 0x00B2 processed — F-P9-001 call-site gate confirmed |
| AC-132-005 | Yes | empty path, Class only, Class+Instance+Attribute, unrecognized skip, odd-length safe |
| AC-132-006 | Partial (structural) | F-P9-002 doc comment present in source; no automated test — see note above |
| AC-132-007 | Yes (unit tests) | 13 named codes, response-bit priority, Unknown; VP-032 Sub-D Kani harnesses deferred to `cargo kani` |
