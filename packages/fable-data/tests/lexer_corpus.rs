//! Corpus smoke test for the unified lexer (§11.4): lex every non-deprecated
//! text-def source under `$OA_TEXT_DIR/Defs` and assert zero lex errors.
//!
//! The corpus is the `.def`/`.tpl` files the compiler actually processes
//! (`fable-def-compiler`'s `walk_def_files`, mirrored here since `fable-data`
//! sits below that crate): 174 non-deprecated `.def` files plus the `.tpl`
//! templates. Skips when `OA_TEXT_DIR` is unset, matching the golden test.
//!
//!   OA_TEXT_DIR=~/doc/Fable_Anniversary-2013-02-25/Fable/Data \
//!     cargo test -p fable-data --test lexer_corpus -- --nocapture

use fable_data::def::text::lexer::lex;
use std::path::{Path, PathBuf};

fn collect_def_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_def_sources(&path, out);
            continue;
        }
        let is_source = matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("def") | Some("tpl")
        );
        let deprecated = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.contains("_deprecated."));
        if is_source && !deprecated {
            out.push(path);
        }
    }
}

#[test]
fn corpus_lexes_without_errors() {
    let Ok(text_dir) = std::env::var("OA_TEXT_DIR") else {
        eprintln!("SKIP corpus_lexes_without_errors: set OA_TEXT_DIR to run");
        return;
    };

    let mut files = Vec::new();
    collect_def_sources(&Path::new(&text_dir).join("Defs"), &mut files);
    files.sort();
    assert!(
        files.len() >= 174,
        "expected >= 174 non-deprecated corpus files, found {} under {text_dir}/Defs",
        files.len()
    );

    let mut errors = Vec::new();
    for path in &files {
        let raw = std::fs::read(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
        // Match the compiler's decode (`build.rs` uses `from_utf8_lossy`).
        let text = String::from_utf8_lossy(&raw).into_owned();
        if let Err(e) = lex(&text) {
            errors.push(format!("{}: {} at byte {}", path.display(), e.kind, e.span.start));
        }
    }

    assert!(
        errors.is_empty(),
        "lexed {} files; {} had lex errors:\n{}",
        files.len(),
        errors.len(),
        errors.join("\n")
    );
    eprintln!("lexed {} corpus files with zero lex errors", files.len());
}
