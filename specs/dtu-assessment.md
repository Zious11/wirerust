---
artifact: dtu-assessment
DTU_REQUIRED: false
traces_to: .factory/specs/architecture/ARCH-INDEX.md
version: "1.0"
status: complete
producer: architect
timestamp: 2026-05-20T00:00:00Z
---

# Digital Twin Universe (DTU) Assessment

## Summary

**DTU_REQUIRED: false**

wirerust has zero external service dependencies. It is an offline, single-binary,
single-pass CLI that reads only local files. No behavioral clone (DTU) is needed
for any testing or verification purpose.

---

## Category-by-Category Assessment

### 1. Inbound Data Sources (External -> Product)

**None identified.**

Rationale: wirerust receives no data from external services. Its only inputs are
local classic-pcap files provided as CLI arguments. There are no polled APIs,
consumed feeds, received webhooks, sensor streams, or event queues.

The `pcap-file` and `etherparse` crates parse file bytes locally. They are Rust
library dependencies, not external services. Library dependencies do not require DTUs.

### 2. Outbound Operations (Product -> External)

**None identified.**

Rationale: wirerust emits findings to stdout, stderr, or a local output file.
It makes no HTTP requests, sends no notifications, creates no tickets, processes
no payments, and executes no remote commands. Zero outbound network calls exist
in the codebase (confirmed by the "no network I/O" invariant and the absence of
any socket-related dependency in Cargo.toml).

### 3. Identity and Access (Bidirectional)

**None identified.**

Rationale: wirerust has no user authentication, no API key management, no OAuth
flows, and no credential stores. It is invoked directly by the local user and
processes local files under that user's filesystem permissions. No identity
provider of any kind is involved.

### 4. Persistence and State (Product <-> Storage)

**None identified.**

Rationale: wirerust writes no persistent state. Output files (`--json <FILE>`,
`--csv <FILE>`) are written to the local filesystem and are output artifacts, not
external storage services. There is no database, no cache server, no object store,
and no message queue. The `chrono` and `serde_json` crates are local libraries.

### 5. Observability and Export (Product -> Monitoring)

**None identified.**

Rationale: wirerust has no telemetry pipeline. It writes to stdout/stderr only.
`indicatif` renders a progress bar to stderr -- this is a local TTY write, not a
metrics platform or log aggregator. There is no OpenTelemetry, Prometheus, Datadog,
Splunk, or any other observability export.

### 6. Enrichment and Lookup (External -> Product)

**None identified.**

Rationale: All data classification is performed against compiled-in static tables
(the MITRE technique catalog in `mitre.rs`, the cipher-name tables in `tls.rs`,
the HTTP method lists in `dispatcher.rs` and `http.rs`). There is no external
threat-intel feed, no NVD lookup, no GeoIP service, and no cloud-hosted enrichment
API. The MITRE catalog is a static match block in Rust source -- it cannot be
updated without a recompile.

---

## Conclusion

wirerust satisfies every "no external dependency" guarantee claimed by its
architecture:

- No network I/O of any kind (zero socket syscalls)
- No async runtime (no network scheduler)
- No process-to-process state (no IPC)
- Single-binary deployment (no sidecar services)

These guarantees are structural -- they are enforced by the absence of any
network-capable dependency in Cargo.toml, not merely by convention. A future
feature that introduces a network call (e.g., automatic MITRE catalog updates,
threat-intel enrichment) would require a new DTU assessment for the specific
service being integrated.

**Disposition:** No DTU work streams are spawned. DTU sections in all downstream
VSDD phase plans are marked N/A for wirerust.
