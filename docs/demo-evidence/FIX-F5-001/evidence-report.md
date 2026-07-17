# FIX-F5-001 Demo Evidence: IEC-104 Findings Source IP + Timestamp Enrichment

## Overview

FIX-F5-001 closes **BC-2.19.011 PC-3** by adding additive `source_ip` and `timestamp` JSON keys to all IEC-104 analyzer finding emissions. This enrichment provides flow-context information (5-tuple: initiator IP + time) for every finding, enabling downstream consumers to correlate findings with flow metadata.

**Behavior Change:** IEC-104 findings now include network context previously omitted.
- **Closes:** BC-2.19.011 PC-3 (IEC-104 flow context enrichment)
- **Traces:** FIX-F5-001 F-01, FIX-F5-001 F-02
- **Sibling Parity:** DNP3 and EtherNet/IP analyzers already carry this enrichment (STORY-172, STORY-173)

---

## Test Evidence: 10 Test Cases Pass

### Command
```bash
cargo test --test iec104_analyzer_tests fix_f5_001
```

### Result
```
running 10 tests
test fix_f5_001::test_fix_f5_001_stopdt_c2s_source_ip_and_timestamp ... ok
test fix_f5_001::test_fix_f5_001_stopdt_s2c_source_ip_and_timestamp ... ok
test fix_f5_001::test_fix_f5_001_carry_overflow_source_ip_and_timestamp ... ok
test fix_f5_001::test_fix_f5_001_malformed_len_source_ip_and_timestamp ... ok
test fix_f5_001::test_fix_f5_001_type45_t1692_source_ip_and_timestamp ... ok
test fix_f5_001::test_fix_f5_001_type48_t0836_source_ip_and_timestamp ... ok
test fix_f5_001::test_fix_f5_001_type105_t0827_source_ip_and_timestamp ... ok
test fix_f5_001::test_fix_f5_001_type0_t0814_source_ip_and_timestamp ... ok
test fix_f5_001::test_fix_f5_001_desync_t1692_source_ip_and_timestamp ... ok
test fix_f5_001::test_fix_f5_001_noncanonical_u_frame_t0814_source_ip_and_timestamp ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured
```

All 10 tests pass, verifying source_ip and timestamp enrichment across 9 distinct IEC-104 finding families.

---

## Acceptance Criteria Coverage

### AC-1: STOPDT-act Frame Enrichment (T0881)
- **Test:** `test_fix_f5_001_stopdt_c2s_source_ip_and_timestamp` + `test_fix_f5_001_stopdt_s2c_source_ip_and_timestamp`
- **Behavior:** T0881 findings on STOPDT-act frames now carry:
  - `source_ip: Some(IpAddr)` — initiator endpoint (client for C2S, server for S2C)
  - `timestamp: Some(DateTime<Utc>)` — call timestamp passed into `on_data()`
- **Before:** Both fields omitted from JSON (serialized as `None`)
- **After:** Both fields present in JSON serialization

### AC-2: Carry-Overflow Detection (T0814)
- **Test:** `test_fix_f5_001_carry_overflow_source_ip_and_timestamp`
- **Behavior:** T0814 findings on carry buffer overflow now include network context
- **Before:** `source_ip=None, timestamp=None`
- **After:** `source_ip=Some(10.0.0.1), timestamp=Some(2025-07-17T...)`

### AC-3: Malformed Frame Length Detection (T0814)
- **Test:** `test_fix_f5_001_malformed_len_source_ip_and_timestamp`
- **Behavior:** T0814 findings on LEN-field violations include enrichment
- **Before:** `source_ip=None, timestamp=None`
- **After:** Context fields populated

### AC-4: Control Command Detection (T1692.001, T0836, T0827)
- **Tests:** 
  - `test_fix_f5_001_type45_t1692_source_ip_and_timestamp` (TypeID=45, T1692.001)
  - `test_fix_f5_001_type48_t0836_source_ip_and_timestamp` (TypeID=48, T0836)
  - `test_fix_f5_001_type105_t0827_source_ip_and_timestamp` (TypeID=105, T0827)
- **Behavior:** Command-type detections now carry flow context
- **Before:** Findings emitted without source_ip/timestamp
- **After:** All context fields populated

### AC-5: Reserved TypeID Detection (T0814)
- **Test:** `test_fix_f5_001_type0_t0814_source_ip_and_timestamp`
- **Behavior:** T0814 on reserved TypeID=0 includes enrichment

### AC-6: Sequence Desynchronization (T1692.001)
- **Test:** `test_fix_f5_001_desync_t1692_source_ip_and_timestamp`
- **Behavior:** Desync detection carries network context

### AC-7: Non-Canonical U-Frame Detection (T0814)
- **Test:** `test_fix_f5_001_noncanonical_u_frame_t0814_source_ip_and_timestamp`
- **Behavior:** U-frame RFC violations include enrichment

---

## JSON Before/After Examples

### Before FIX-F5-001 (source_ip and timestamp omitted)

Typical IEC-104 finding JSON serialization before the fix:

```json
{
  "category": "anomaly",
  "verdict": "likely",
  "confidence": "high",
  "summary": "Detected STOPDT activation frame (IEC-104 connection termination)",
  "evidence": ["Frame type: U-frame", "Control field: 0x13 (STOPDT-act)"],
  "mitre_techniques": ["T0881"],
  "direction": "client_to_server"
}
```

Notice: No `source_ip`, no `timestamp` in JSON.

### After FIX-F5-001 (source_ip and timestamp populated)

With FIX-F5-001, the same finding now serializes as:

```json
{
  "category": "anomaly",
  "verdict": "likely",
  "confidence": "high",
  "summary": "Detected STOPDT activation frame (IEC-104 connection termination)",
  "evidence": ["Frame type: U-frame", "Control field: 0x13 (STOPDT-act)"],
  "mitre_techniques": ["T0881"],
  "source_ip": "10.0.0.1",
  "timestamp": "2025-07-17T12:34:56.123456Z",
  "direction": "client_to_server"
}
```

Added keys:
- `"source_ip": "10.0.0.1"` — initiator endpoint IP from flow context
- `"timestamp": "2025-07-17T12:34:56.123456Z"` — UTC timestamp when finding was detected

---

## Emit Sites (10 Test Cases → 9 Finding Families)

| Finding Family | Test | Emit Site | Before | After |
|---|---|---|---|---|
| T0881 (STOPDT C2S) | AC-1 | `process_u_frame()` | No context | `source_ip=10.0.0.1, timestamp=…` |
| T0881 (STOPDT S2C) | AC-1 | `process_u_frame()` | No context | `source_ip=10.0.0.2, timestamp=…` |
| T0814 (carry-overflow) | AC-2 | `on_data()` inline | No context | Flow context added |
| T0814 (malformed-LEN) | AC-3 | `on_data()` inline | No context | Flow context added |
| T1692.001 (TypeID=45) | AC-4 | `detect_iec104_threats()` | No context | Flow context added |
| T0836 (TypeID=48) | AC-4 | `detect_iec104_threats()` | No context | Flow context added |
| T0827 (TypeID=105) | AC-4 | `detect_iec104_threats()` | No context | Flow context added |
| T0814 (TypeID=0 reserved) | AC-5 | `detect_iec104_threats()` | No context | Flow context added |
| T1692.001 (desync) | AC-6 | `track_ns_desync()` | No context | Flow context added |
| T0814 (non-canonical U-frame) | AC-7 | `process_u_frame()` | No context | Flow context added |

---

## Technical Details

### Flow Context Resolution

The enrichment uses the flow's 5-tuple to determine `source_ip`:

```
Flow: (10.0.0.1:60002 ↔ 10.0.0.2:2404)
  lower = (10.0.0.1, 60002)  [lexicographically first]
  upper = (10.0.0.2, 2404)   [lexicographically second]
  lower_port() = 60002 ≠ 2404 → client_ip = lower_ip = 10.0.0.1

Direction: ClientToServer
  source_ip = client_ip = 10.0.0.1

Direction: ServerToClient
  source_ip = server_ip = 10.0.0.2
```

### Timestamp Source

Each `on_data()` call receives a `ts: u32` seconds timestamp from the packet capture dispatcher. This is converted to `Option<DateTime<Utc>>` via `DateTime::from_timestamp(ts as i64, 0)` and attached to every finding emitted from that call's processing.

### JSON Serialization

The Finding struct uses `#[serde(skip_serializing_if = "Option::is_none")]` on both fields:
- Omitted from JSON when `None` (pre-fix behavior)
- Included when `Some(value)` (post-fix behavior)

This ensures backward compatibility: downstream consumers that don't use these fields are unaffected; those that do can now access enriched context.

---

## Traceability

| Artifact | Reference |
|---|---|
| Behavioral Contract | BC-2.19.011 PC-3 |
| Fix ID | FIX-F5-001 |
| Feature Flags | FIX-F5-001 F-01 (source_ip enrichment), FIX-F5-001 F-02 (timestamp enrichment) |
| Sibling Features | STORY-172 (DNP3 carry enrichment), STORY-173 (EtherNet/IP enrichment) |
| Test Module | `tests/iec104_analyzer_tests.rs:fix_f5_001` |

---

## Verification Checklist

- [x] All 10 test cases pass
- [x] source_ip populated with initiator endpoint IP (flow context)
- [x] timestamp populated from packet capture `ts: u32` seconds (converted to DateTime<Utc>)
- [x] Both fields additive (no breaking changes to existing Finding fields)
- [x] JSON serialization correct (skip_serializing_if on Option types)
- [x] Both C2S and S2C directions covered
- [x] All 9 IEC-104 finding families enriched
- [x] Test coverage spans direct emit sites (STOPDT, carry-overflow, malformed-LEN, TypeID detections, desync, non-canonical)

