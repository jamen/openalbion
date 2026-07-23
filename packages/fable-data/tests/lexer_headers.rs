//! Corpus smoke test for the unified lexer on header files (Phase 3):
//! lex every `.h` file under `$OA_TEXT_DIR/Defs` and assert zero lex errors.
//!
//!   OA_TEXT_DIR=~/doc/Fable_Anniversary-2013-02-25/Fable/Data \
//!     cargo test -p fable-data --test lexer_headers -- --nocapture

use fable_data::def::text::lexer::lex;
use fable_data::def::text::manifest;
use std::path::Path;

#[test]
fn header_files_lex_without_errors() {
    let Ok(text_dir) = std::env::var("OA_TEXT_DIR") else {
        eprintln!("SKIP header_files_lex_without_errors: set OA_TEXT_DIR to run");
        return;
    };

    let defs_dir = Path::new(&text_dir).join("Defs");
    let header_names: Vec<&&str> = manifest::SHARED_HEADERS
        .iter()
        .chain(manifest::PC_HEADERS.iter())
        .collect();
    let expected = manifest::SHARED_HEADERS.len() + manifest::PC_HEADERS.len();

    let mut errors = Vec::new();
    for name in &header_names {
        let path = defs_dir.join(name);
        let Ok(raw) = std::fs::read(&path) else {
            errors.push(format!("could not read {}", path.display()));
            continue;
        };
        let text = String::from_utf8_lossy(&raw).into_owned();
        match lex(&text) {
            Ok(_) => {}
            Err(e) => errors.push(format!("{}: {} at byte {}", path.display(), e.kind, e.span.start)),
        }
    }

    if errors.is_empty() {
        eprintln!("lexed {expected} header files with zero lex errors");
    } else {
        panic!(
            "lexed {expected} header files; {} had lex errors:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
}
