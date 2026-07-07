//! AC-149-001 / PERF-001 source-inspection tests.
//!
//! Verifies that the bounded-borrow budget (AC-149-001, amended v1.2) is
//! enforced in `src/analyzer/tls.rs`:
//!
//! - `try_parse_records` body: EXACTLY 1 `flows.get_mut(`/`flows.get(`
//!   acquisition site (SINGLE-BORROW INVARIANT marker required at that site).
//! - `process_handshake_carry` body: at most 3 acquisition sites.
//! - Grand total across both functions: at most 4.
//! - Anti-gameability guard: neither body may alias `self.flows` or use
//!   `entry()`/`iter_mut()` — patterns that would hide re-hashing from the count.
//! - `process_handshake_carry` BORROW BUDGET annotation coverage: body contains
//!   at least one `BORROW BUDGET` inline marker, and the marker count equals the
//!   acquisition-site count — so any new unannotated borrow site fails CI
//!   (AC-149-001 Architecture Compliance Rule; F-S149P5-001).
//!
//! All tests pass after the STORY-149 single-borrow restructure.
//! Tests are wrapped in `mod bc_149_single_borrow` per DF-TEST-NAMESPACE-001.
//!
//! `#![allow(non_snake_case)]` required per factory BC-naming mandate.
#![allow(non_snake_case)]

mod bc_149_single_borrow {
    #[allow(unused_imports)]
    use super::*;

    /// Extract the body of the first function whose definition line contains
    /// `fn_sig` from `source`, using brace-depth counting to locate the matching
    /// closing `}`.
    ///
    /// Returns the text from the opening `{` (inclusive) through the matching
    /// closing `}` (inclusive). Panics if the signature or its brace-pair are
    /// not found.
    ///
    /// LIMITATION: brace characters inside string literals or line-comments
    /// contribute to the depth count. This is acceptable here because the
    /// function bodies in `src/analyzer/tls.rs` do not contain free-standing
    /// unmatched braces inside string literals or comments.
    fn extract_fn_body(source: &str, fn_sig: &str) -> String {
        let sig_pos = source.find(fn_sig).unwrap_or_else(|| {
            panic!("function signature {fn_sig:?} not found in source");
        });

        // The function body opens at the first '{' on or after the signature line.
        let rel_open = source[sig_pos..].find('{').unwrap_or_else(|| {
            panic!("no opening '{{' found after function signature {fn_sig:?}");
        });
        let open_pos = sig_pos + rel_open;

        // Walk forward counting brace depth to find the matching closing '}'.
        let tail = &source[open_pos..];
        let mut depth: usize = 0;
        let mut end_byte: usize = 0;

        for (byte_idx, ch) in tail.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end_byte = byte_idx + 1; // '}' is always 1 byte in UTF-8
                        break;
                    }
                }
                _ => {}
            }
        }

        assert!(
            end_byte > 0,
            "no matching closing '}}' found for {fn_sig:?}"
        );

        source[open_pos..open_pos + end_byte].to_string()
    }

    /// Read `src/analyzer/tls.rs` from disk, panicking with a clear message if
    /// the file is missing (indicates `cargo test` was run from the wrong directory).
    fn read_tls_source() -> String {
        std::fs::read_to_string("src/analyzer/tls.rs").unwrap_or_else(|e| {
            panic!(
                "failed to read src/analyzer/tls.rs \
                 (cargo test must be run from the crate root): {e}"
            )
        })
    }

    /// AC-149-001 (PERF-001): `try_parse_records` must acquire EXACTLY one
    /// HashMap borrow (`flows.get` or `flows.get_mut`) per invocation.
    ///
    /// "Exactly one" (not merely "at most one") — the SINGLE-BORROW INVARIANT
    /// requires the acquisition site to remain present as a structural marker;
    /// zero would indicate the function body was accidentally hollowed out
    /// (AC-149-001 v1.2, F-S149P1-001 sharpening).
    ///
    /// GREEN (STORY-149): `try_parse_records` acquires exactly one
    /// `flows.get_mut(` borrow via `prepare_record_step()`. This test passes.
    #[test]
    fn test_BC_149_001_exactly_one_flows_borrow_in_try_parse_records() {
        let source = read_tls_source();
        let body = extract_fn_body(&source, "fn try_parse_records(");

        let get_mut_count = body.matches("flows.get_mut(").count();
        let get_count = body.matches("flows.get(").count();
        let total = get_mut_count + get_count;

        assert_eq!(
            total, 1,
            "AC-149-001 (PERF-001): try_parse_records must acquire EXACTLY 1 \
             HashMap borrow (flows.get / flows.get_mut) per invocation — the \
             SINGLE-BORROW INVARIANT eliminates repeated FlowKey re-hashing \
             (STORY-149). \
             Found {get_mut_count} flows.get_mut( + {get_count} flows.get( \
             = {total} total borrow call site(s). \
             Maintain exactly one acquisition site in try_parse_records."
        );
    }

    /// AC-149-001: `try_parse_records` must contain the "SINGLE-BORROW
    /// INVARIANT" comment marker at the borrow acquisition site so that the
    /// invariant is asserted inline and visible in future diffs.
    ///
    /// GREEN (STORY-149): the "SINGLE-BORROW INVARIANT" marker is present
    /// in the restructured `try_parse_records`. This test passes.
    #[test]
    fn test_BC_149_001_single_borrow_invariant_comment_marker_present() {
        let source = read_tls_source();
        let body = extract_fn_body(&source, "fn try_parse_records(");

        assert!(
            body.contains("SINGLE-BORROW INVARIANT"),
            "AC-149-001: try_parse_records must contain the comment marker \
             'SINGLE-BORROW INVARIANT' at the single borrow acquisition site \
             to enforce the pattern inline and make it visible in future diffs \
             (STORY-149)."
        );
    }

    /// AC-149-001: `process_handshake_carry` must have at most 3 acquisition
    /// sites (re-borrows after the primary borrow is released); the grand total
    /// across `try_parse_records` + `process_handshake_carry` must not exceed 4
    /// (bounded-borrow budget, PERF-001, AC-149-001 v1.2).
    ///
    /// GREEN (STORY-149): `process_handshake_carry` has exactly 3
    /// `flows.get_mut(` sites (ClientHello flag-set, ServerHello flag-set,
    /// carry-restore); grand total = 1 + 3 = 4. This test passes.
    #[test]
    fn test_BC_149_001_process_handshake_carry_borrow_budget() {
        let source = read_tls_source();
        let tpr_body = extract_fn_body(&source, "fn try_parse_records(");
        let phc_body = extract_fn_body(&source, "fn process_handshake_carry(");

        let tpr_get_mut = tpr_body.matches("flows.get_mut(").count();
        let tpr_get = tpr_body.matches("flows.get(").count();
        let tpr_total = tpr_get_mut + tpr_get;

        let phc_get_mut = phc_body.matches("flows.get_mut(").count();
        let phc_get = phc_body.matches("flows.get(").count();
        let phc_total = phc_get_mut + phc_get;

        let grand_total = tpr_total + phc_total;

        assert!(
            phc_total <= 3,
            "AC-149-001 (PERF-001): process_handshake_carry must have at most 3 \
             flows.get_mut(/flows.get( acquisition sites (re-borrows after primary \
             borrow released; budget ≤ 3 within that helper — STORY-149). \
             Found {phc_get_mut} flows.get_mut( + {phc_get} flows.get( \
             = {phc_total} total in process_handshake_carry."
        );
        assert!(
            grand_total <= 4,
            "AC-149-001 (PERF-001): total bounded-borrow budget ≤ 4 acquisition \
             sites across try_parse_records + process_handshake_carry (STORY-149). \
             try_parse_records: {tpr_total}, process_handshake_carry: {phc_total}, \
             grand total: {grand_total}. Reduce acquisition sites to stay within budget."
        );
    }

    /// AC-149-001: every `flows.get_mut(`/`flows.get(` acquisition site in
    /// `process_handshake_carry` must carry a `BORROW BUDGET` inline annotation,
    /// and the count of annotations must equal the count of acquisition sites.
    ///
    /// This enforces the Architecture Compliance Rule: "SINGLE-BORROW INVARIANT
    /// marker + budget annotation ... enforced by source-inspection test" — so
    /// adding a new un-annotated borrow site immediately fails CI (F-S149P5-001).
    ///
    /// Count equality guards both directions:
    /// - More borrows than annotations → new unannotated site snuck in.
    /// - More annotations than borrows → stale annotation left behind after refactor.
    ///
    /// GREEN (STORY-149 / F-S149P5-001): `process_handshake_carry` body contains
    /// exactly 3 `BORROW BUDGET` inline markers matching its 3 acquisition sites.
    #[test]
    fn test_BC_149_001_process_handshake_carry_budget_annotations_match_sites() {
        let source = read_tls_source();
        let phc_body = extract_fn_body(&source, "fn process_handshake_carry(");

        let acquisition_sites =
            phc_body.matches("flows.get_mut(").count() + phc_body.matches("flows.get(").count();
        let budget_markers = phc_body.matches("BORROW BUDGET").count();

        assert!(
            budget_markers > 0,
            "AC-149-001 (F-S149P5-001): process_handshake_carry body must contain \
             at least one 'BORROW BUDGET' inline annotation — the Architecture \
             Compliance Rule requires budget annotations at every acquisition site \
             so that unannotated borrows are caught by this inspection test \
             (STORY-149). Found 0 'BORROW BUDGET' markers in process_handshake_carry."
        );
        assert_eq!(
            budget_markers, acquisition_sites,
            "AC-149-001 (F-S149P5-001): the count of 'BORROW BUDGET' inline \
             annotations in process_handshake_carry must equal the count of \
             flows.get_mut(/flows.get( acquisition sites — every borrow site \
             must be annotated and no stale annotations may remain (STORY-149). \
             Found {budget_markers} 'BORROW BUDGET' marker(s) but \
             {acquisition_sites} acquisition site(s) in process_handshake_carry."
        );
    }

    /// AC-149-001 anti-gameability guard: neither `try_parse_records` nor
    /// `process_handshake_carry` may contain patterns that would hide HashMap
    /// re-hashing from the acquisition-site count (F-S149P1-001).
    ///
    /// Forbidden patterns in both function bodies:
    /// - `= &mut self.flows` / `= &self.flows` — reference alias bypasses the
    ///   explicit `get_mut`/`get` count while still causing a re-hash.
    /// - `self.flows.entry(` — entry API is not counted by the `get_mut`/`get`
    ///   grep but still performs a hash lookup.
    /// - `self.flows.iter_mut(` — iterator bypasses the per-key borrow count.
    ///
    /// GREEN (STORY-149): neither function body contains any of these patterns.
    #[test]
    fn test_BC_149_001_no_aliasing_patterns_hide_borrow_count() {
        let source = read_tls_source();
        let tpr_body = extract_fn_body(&source, "fn try_parse_records(");
        let phc_body = extract_fn_body(&source, "fn process_handshake_carry(");

        for (fn_name, body) in [
            ("try_parse_records", tpr_body.as_str()),
            ("process_handshake_carry", phc_body.as_str()),
        ] {
            assert!(
                !body.contains("= &mut self.flows"),
                "AC-149-001 anti-gameability: `{fn_name}` must not alias self.flows \
                 via `= &mut self.flows` — a reference alias hides HashMap re-hashing \
                 from the acquisition-site count (STORY-149 / F-S149P1-001). \
                 Found alias binding in `{fn_name}`."
            );
            assert!(
                !body.contains("= &self.flows"),
                "AC-149-001 anti-gameability: `{fn_name}` must not alias self.flows \
                 via `= &self.flows` — a reference alias hides HashMap re-hashing \
                 from the acquisition-site count (STORY-149 / F-S149P1-001). \
                 Found alias binding in `{fn_name}`."
            );
            assert!(
                !body.contains("self.flows.entry("),
                "AC-149-001 anti-gameability: `{fn_name}` must not call \
                 self.flows.entry( — the entry API performs a hash lookup that is \
                 not counted by the flows.get_mut/flows.get acquisition-site grep \
                 (STORY-149 / F-S149P1-001). Found entry() call in `{fn_name}`."
            );
            assert!(
                !body.contains("self.flows.iter_mut("),
                "AC-149-001 anti-gameability: `{fn_name}` must not call \
                 self.flows.iter_mut( — iteration bypasses the per-key borrow-site \
                 count (STORY-149 / F-S149P1-001). Found iter_mut() call in \
                 `{fn_name}`."
            );
        }
    }
}
