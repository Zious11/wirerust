---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/main.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-11
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.009: needs_reassembly Logic; --no-reassemble Forces Off with Warning

## Description

TCP reassembly is activated when `needs_reassembly = cli.reassemble || enable_http || enable_tls`
and `!skip_reassembly`. When `--no-reassemble` is set (`skip_reassembly = true`) AND
`enable_http || enable_tls` is also true, a stderr warning is emitted before skipping
reassembly creation. In this case, HTTP and TLS analyzers are NOT constructed (because they
require reassembled streams).

## Preconditions

1. `run_analyze` is entered.
2. Various combinations of `cli.reassemble`, `cli.no_reassemble`, `enable_http`,
   `enable_tls` may be true.

## Postconditions

1. `needs_reassembly = cli.reassemble || enable_http || enable_tls` (main.rs:87).
2. `skip_reassembly = cli.no_reassemble` (main.rs:88).
3. When `needs_reassembly && !skip_reassembly`: reassembler is constructed.
4. When `skip_reassembly`: reassembler is None; HTTP and TLS analyzers are None.
5. When `(enable_http || enable_tls) && skip_reassembly`: stderr warning printed
   (main.rs:90-94).
6. DNS analyzer is constructed independently of reassembly (it operates per-packet).

## Invariants

1. The warning message text is: "Warning: --http/--tls require TCP reassembly, but
   --no-reassemble is set. Stream analysis will be skipped."
2. The warning is printed ONCE per `run_analyze` invocation when the condition fires.
3. `http_analyzer` and `tls_analyzer` are gated on `enable_X && !skip_reassembly`
   (main.rs:124-133).
4. `dns_analyzer` is always constructed (DNS is per-packet, not stream-based).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | --no-reassemble, --http | Warning emitted; http_analyzer=None |
| EC-002 | --no-reassemble, no --http/--tls | No warning; skip_reassembly=true, no analyzers |
| EC-003 | --reassemble, no --http/--tls | Reassembler constructed; no stream analyzers |
| EC-004 | --http without --no-reassemble | Reassembler + HTTP analyzer constructed; no warning |
| EC-005 | No flags | needs_reassembly=false; no reassembler; dns only if --dns |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| enable_http=true, skip_reassembly=true | Warning to stderr; http_analyzer=None | happy-path |
| enable_http=true, skip_reassembly=false | No warning; reassembler + http_analyzer created | happy-path |
| cli.reassemble=true, skip_reassembly=false | Reassembler created; no warning | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | needs_reassembly computation and no-reassemble override | unit: code-level verification (MEDIUM -- not directly tested) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 |
| Capability Anchor Justification | CAP-11 ("Reporting and Output") per capabilities.md §CAP-11 -- the reassembly enable/disable logic governs whether stream-analysis outputs (HTTP/TLS findings) are included in the report |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (main.rs, C-1) |
| Stories | S-TBD |
| Origin BC | BC-CLI-009 (pass-3 ingestion corpus, MEDIUM confidence -- no direct test exercises the warning path) |

## Related BCs

- BC-2.12.005 -- depends on (--reassemble/--no-reassemble flags are the inputs)
- BC-2.12.007 -- composes with (clap enforces they are never both true at parse time)

## Architecture Anchors

- `src/main.rs:87-96` -- needs_reassembly computation and skip warning
- `src/main.rs:96-122` -- reassembler construction conditional
- `src/main.rs:124-133` -- http_analyzer and tls_analyzer conditional construction

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/main.rs:87-133` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **documentation**: code is explicit; warning message is hardcoded
- **inferred**: no test exercises the warning path or the no-reassemble + http combination

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | writes warning to stderr (effectful) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | N/A (single-threaded) |
| **Overall classification** | effectful shell |

#### Refactoring Notes

To upgrade to HIGH: add an integration test that passes `--no-reassemble --http` and asserts
the warning appears on stderr.
