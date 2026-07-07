//! AC-149-001 / PERF-001 source-inspection tests.
//!
//! Verifies that `try_parse_records` in `src/analyzer/tls.rs` acquires at
//! most one HashMap borrow per invocation (`flows.get` / `flows.get_mut`
//! combined) and carries the "SINGLE-BORROW INVARIANT" comment marker at
//! the single borrow acquisition site.
//!
//! GREEN (STORY-149): `try_parse_records` now acquires exactly one
//! `flows.get_mut(` borrow per loop iteration via `prepare_record_step()`.
//! The "SINGLE-BORROW INVARIANT" comment marker is present at the borrow
//! acquisition site. Both tests pass after the single-borrow restructure
//! in STORY-149.
//!
//! `#![allow(non_snake_case)]` required per factory BC-naming mandate.
#![allow(non_snake_case)]

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

/// AC-149-001 (PERF-001): `try_parse_records` must acquire at most one
/// HashMap borrow (`flows.get` or `flows.get_mut`) per invocation to
/// eliminate repeated FlowKey re-hashing.
///
/// After the STORY-149 single-borrow restructure the function acquires
/// exactly one `flows.get_mut(` and zero `flows.get(` calls, allowing the
/// compiler to keep the mutable reference live throughout without re-hashing
/// the `FlowKey` on every sub-step.
///
/// GREEN (STORY-149): after the single-borrow restructure, `try_parse_records`
/// has exactly 1 `flows.get_mut(` borrow call site (total <= 1). This test passes.
#[test]
fn test_BC_149_001_at_most_one_flows_borrow_in_try_parse_records() {
    let source = std::fs::read_to_string("src/analyzer/tls.rs").unwrap_or_else(|e| {
        panic!(
            "failed to read src/analyzer/tls.rs \
             (cargo test must be run from the crate root): {e}"
        )
    });

    let body = extract_fn_body(&source, "fn try_parse_records(");

    let get_mut_count = body.matches("flows.get_mut(").count();
    let get_count = body.matches("flows.get(").count();
    let total = get_mut_count + get_count;

    assert!(
        total <= 1,
        "AC-149-001 (PERF-001): try_parse_records must acquire at most 1 \
         HashMap borrow (flows.get / flows.get_mut) per invocation to \
         eliminate repeated FlowKey re-hashing (STORY-149). \
         Found {get_mut_count} flows.get_mut( + {get_count} flows.get( \
         = {total} total borrow call site(s). \
         Refactor to single-borrow pattern before implementing."
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
    let source = std::fs::read_to_string("src/analyzer/tls.rs").unwrap_or_else(|e| {
        panic!(
            "failed to read src/analyzer/tls.rs \
             (cargo test must be run from the crate root): {e}"
        )
    });

    let body = extract_fn_body(&source, "fn try_parse_records(");

    assert!(
        body.contains("SINGLE-BORROW INVARIANT"),
        "AC-149-001: try_parse_records must contain the comment marker \
         'SINGLE-BORROW INVARIANT' at the single borrow acquisition site \
         to enforce the pattern inline and make it visible in future diffs \
         (STORY-149)."
    );
}
