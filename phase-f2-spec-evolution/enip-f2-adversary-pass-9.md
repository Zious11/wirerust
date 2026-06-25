# Adversarial Spec Review — feature-enip-v0.11.0 (SS-17), Pass 9

VERDICT: FAIL — 1 HIGH, 1 MEDIUM, 2 LOW-obs (0 CRITICAL). Novelty: MODERATE — genuinely-new protocol-correctness bug (NOT propagation-lag).

- F-P9-001 (HIGH): CPF Connected Data Items (0x00B1) prepend a 2-byte sequence-count before the CIP PDU. BC-2.17.008 gated RESPONSE extraction to 0x00B2-only, but request-side parse BC-2.17.006 read service=item_data[0] for BOTH 0x00B1/0x00B2 → all Connected-item CIP request detections (Stop/Reset/write/Identity/ForwardOpen) misparse by 2 bytes (false neg + false pos). RESOLVED via Option A: CIP service detection scoped to 0x00B2 unconnected carriers for v0.11.0; 0x00B1 connected-item CIP request detection DEFERRED to v0.12.0 (mirrors BC-2.17.008 response gate). Fixed in ADR-010 Decision 8 + BC-2.17.006/011/012/013/014/015 (BC-2.17.011 EC-004 false "0x00B1 detected" claim corrected to "NO finding"); ForwardOpen/Close confirmed unaffected (0x00B2 carriers).

- F-P9-002 (MEDIUM): parse_cip_header/parse_cpf_items (attacker-facing length-driven parsers) not covered by VP-032 Kani. RESOLVED: recorded F6 cargo-fuzz no-panic/bounds-safety obligation in ADR-010 Decision 8 DEFERRED + architecture-delta §4.3 + vp-032 scope note (no new VP). Boundary ECs added to BC-2.17.006.

- obs-1/obs-2 (LOW): obs-1 folded into F-P9-001; obs-2 = all other axes clean.

Severity trajectory: P1 4C/7H → P2 4C/3H → P3 3C/4H → P4 0C/1H → P5 0C/1H → P6 0C/0H → P7 0C/1H → P8 0C/0H → P9 0C/1H(FAIL — new 0x00B1 bug).
