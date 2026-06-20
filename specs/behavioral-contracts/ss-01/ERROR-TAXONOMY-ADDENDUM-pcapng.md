---
document_type: error-taxonomy-addendum
status: CONSUMED
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: F2
feature: FE-001-pcapng-reader-support
integration_target: .factory/specs/prd-supplements/error-taxonomy.md
integration_section: "### INP: Input / File Errors"
integration_burst: INTEGRATE (sub-burst 2)
integration_completed: 2026-06-19
note: >
  CONSUMED 2026-06-19 by INTEGRATE sub-burst. E-INP-008..011 have been spliced into
  error-taxonomy.md v2.3. E-INP-002 "or pcapng format" note removed. This staging
  file is retained for audit trail only. Do not use as a source of truth.
next_free_error_code: E-INP-012
---

# Error Taxonomy Addendum: pcapng Reader Support (FE-001)

## Proposed New INP Error Entries

These four entries extend the INP category in `error-taxonomy.md`. They are to be inserted
after the current last INP entry (E-INP-007) when the INTEGRATE burst runs.

### Table (splice into the INP table after E-INP-007)

| Error Code | Category | Severity | Exit Code | Source Location | Message Format | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|----------------|--------|-------|
| E-INP-008 | Input | `broken` | 1 | `src/reader.rs` (pcapng SHB/IDB parse path) | `Failed to parse pcapng <block-type>: <underlying>` | BC-2.01.010, BC-2.01.011, BC-2.01.017 | Covers structural parse failures at the SHB or IDB level: truncated file, missing BOM, malformed block-total-length, unsupported major version. `<block-type>` is one of "Section Header Block", "Interface Description Block". `<underlying>` is the anyhow root cause. Surfaced via anyhow chain; ultimate wrapper is E-INP-005 ("Failed to read \<path\>: \<underlying\>"). |
| E-INP-009 | Input | `broken` | 1 | `src/reader.rs` (pcapng EPB parse path, pre-IDB guard) | `pcapng Enhanced Packet Block encountered before any Interface Description Block` | BC-2.01.012, BC-2.01.017 | Emitted when an EPB is encountered and the interface table is empty (no IDB has been seen in the current section). This is a pcapng structural violation. The file is broken — no safe way to interpret the packet's linktype or timestamp resolution. |
| E-INP-010 | Input | `broken` | 1 | `src/reader.rs` (pcapng EPB/SPB/unknown-block parse path) | `Failed to parse pcapng <block-type> (block \#<seq>): <underlying>` | BC-2.01.012, BC-2.01.013, BC-2.01.015, BC-2.01.017 | Covers packet-data-level truncation in EPB and SPB (captured_length inconsistent with block_total_length), and block_total_length < 8 in unknown-block skip. `<block-type>` is "Enhanced Packet Block", "Simple Packet Block", or "unknown block (type=0x{N:08X})". `<seq>` is the 1-based block sequence number within the file for debuggability. |
| E-INP-011 | Input | `broken` | 1 | `src/reader.rs` (pcapng multi-IDB agreement check) | `pcapng multi-interface link-type conflict: interface 0 has <first:?>, interface <n> has <other:?>` | BC-2.01.018, BC-2.01.017 | Emitted when two or more IDBs in a section carry different `linktype` values. `<first:?>` and `<other:?>` are the `DataLink` Debug repr values. This reflects the fail-closed multi-IDB policy (ADR-009 Decision 3). Known limitation: rejects legitimate multi-NIC captures mixing Ethernet and Linux Cooked interfaces. |

## Integration Notes for INTEGRATE Burst

1. Insert the four rows above into the INP table in `error-taxonomy.md`, after E-INP-007.
2. Update the error-taxonomy.md `version:` field (current v2.2 → v2.3 or next appropriate increment).
3. Update the `modified:` field with the integration timestamp and FE-001 reference.
4. Do NOT change E-INP-001 through E-INP-007.
5. The `note` in E-INP-002 currently mentions "or pcapng format" as a trigger condition.
   Per F1 delta analysis §4.1, that note MUST be revised after pcapng support lands:
   remove "or pcapng format" from the E-INP-002 Notes column, since pcapng files now
   route to E-INP-008..011, not E-INP-002.
6. After integration, `next_free_error_code` = E-INP-012.

## Ordering Rationale

- E-INP-008: SHB/IDB structural failures — earliest in the block-walk sequence.
- E-INP-009: EPB-before-IDB semantic violation — occurs only if SHB parsed but IDB absent.
- E-INP-010: EPB/SPB/unknown-block data-level failures — per-packet errors.
- E-INP-011: Multi-IDB linktype conflict — policy error discovered after all IDBs seen.
