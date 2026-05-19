//! Regression tests for the module-level documentation convention
//! (LESSON-P1.06 + LESSON-P1.07 from the brownfield-ingest synthesis).
//!
//! Walks `src/` and asserts every `.rs` file starts with a `//!` doc
//! header. This is the structural enforcement that complements
//! `#![warn(missing_docs)]` in `src/lib.rs` — the warn lint covers
//! public-item docs, this test covers module-level headers (which
//! `missing_docs` only flags for `pub mod` declarations, not for
//! the module file itself).
//!
//! Removing a module header to silence this test would also have to
//! delete or modify this test, which is a visible regression signal.

use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read_dir on src") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn every_src_module_has_a_module_header() {
    // Resolve `src/` relative to CARGO_MANIFEST_DIR so the test works
    // regardless of the working directory cargo chose.
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is always set by cargo");
    let src_dir = PathBuf::from(&manifest_dir).join("src");
    assert!(
        src_dir.is_dir(),
        "expected `src/` directory at {}",
        src_dir.display()
    );

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);
    assert!(
        !files.is_empty(),
        "expected at least one .rs file under {}",
        src_dir.display()
    );

    let mut missing = Vec::new();
    for path in &files {
        let contents = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let first_nonblank = contents
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("");
        // Allow a leading shebang or attribute (rare in this crate) but
        // require the first *meaningful* line to be either a `//!` doc
        // or an inner attribute that conventionally precedes one.
        if !first_nonblank.starts_with("//!") {
            missing.push(
                path.strip_prefix(&manifest_dir)
                    .unwrap_or(path)
                    .display()
                    .to_string(),
            );
        }
    }

    assert!(
        missing.is_empty(),
        "LESSON-P1.07 regressed — these `.rs` files under `src/` are missing a `//!` \
         module header:\n  {}\n\nAdd a 2–5 line `//!` header describing the module's \
         role in the pipeline. See `src/mitre.rs` and `src/findings.rs` as canonical examples.",
        missing.join("\n  ")
    );
}

#[test]
fn lib_rs_enables_missing_docs_warning() {
    // LESSON-P1.06: the crate root must carry the `#![warn(missing_docs)]`
    // attribute so any newly-added public item without a doc comment
    // surfaces as a warning (turned into an error by CI's `-D warnings`).
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is always set by cargo");
    let lib_rs = PathBuf::from(&manifest_dir).join("src").join("lib.rs");
    let contents = fs::read_to_string(&lib_rs).expect("read src/lib.rs");
    assert!(
        contents.contains("#![warn(missing_docs)]"),
        "LESSON-P1.06 regressed — `src/lib.rs` must carry \
         `#![warn(missing_docs)]` to enable the crate-wide phased rollout"
    );
}
