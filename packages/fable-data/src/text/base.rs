//! Spans, spanned values and the error type for the level-text grammar.
//!
//! Modelled on `fable-defs`' `text::base`, minus the parts that only a *def*
//! compiler needs. There is no `FileId` here: a `.tng` or `.wld` is parsed on
//! its own and its spans never outlive it, where a def template's statements
//! are re-read by every definition that inherits them and so have to carry the
//! file they came from.

use derive_more::{Display, Error};
use std::borrow::Cow;

/// A byte range in the source (half-open: `start..end`).
///
/// Byte offsets, not char offsets — `&source[start..end]` reproduces the text.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// A value annotated with its source span.
///
/// Equality ignores the span, so a parsed tree can be compared against an
/// expected one written by hand without spelling out byte offsets.
#[derive(Copy, Clone, Debug)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

/// The construct a parse error happened *inside*, so a failure deep in a thing
/// names the thing rather than only the offending byte.
///
/// One context, not a stack: the innermost production to see the error wins,
/// because it is the first to run on the way out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseContext {
    /// What kind of construct: "thing", "block", "section".
    pub what: &'static str,
    /// Its name, when it has one (`Object`, `CTCPhysicsStandard`).
    pub name: Option<String>,
}

impl ParseContext {
    pub fn new(what: &'static str, name: Option<String>) -> Self {
        Self { what, name }
    }

    /// The trailing clause of the message: "in thing `Object`" / "in this block".
    pub fn label(&self) -> String {
        match &self.name {
            Some(name) => format!("in {} `{name}`", self.what),
            None => format!("in this {}", self.what),
        }
    }
}

/// What went wrong, without the location.
#[derive(Clone, Debug, Display, Error, PartialEq, Eq)]
pub enum TextErrorKind {
    #[display("unterminated string")]
    UnterminatedString,
    #[display("unterminated block comment")]
    UnterminatedBlockComment,
    #[display("unexpected character {_0:?}")]
    UnexpectedChar(#[error(not(source))] char),
    /// `expected` names what the grammar wanted here; the "found" half is
    /// filled in uniformly by `Display`, so no construction site formats its own.
    #[display("expected {expected}, found {found}")]
    UnexpectedToken {
        expected: Cow<'static, str>,
        found: &'static str,
    },
    /// Ran to end of input with a block still open.
    #[display("unclosed `{opened}` — expected `{expected};`")]
    UnclosedBlock { opened: String, expected: String },
    /// A closing keyword that does not match the block we are inside.
    #[display("unexpected `{found};` — {}", expected_clause(expected))]
    MismatchedClose {
        found: String,
        /// `None` at the top level, where nothing is open to close.
        expected: Option<String>,
    },
}

fn expected_clause(expected: &Option<String>) -> String {
    match expected {
        Some(e) => format!("expected `{e};`"),
        None => "no block is open".to_string(),
    }
}

/// A parse error: what went wrong, where, and what it was inside.
///
/// `line`/`column` are 1-based and resolved once at the parse entry point (see
/// [`TextError::locate`]); productions raise errors carrying only a span, since
/// computing a line number per candidate error would mean indexing the source
/// on a path that usually succeeds.
#[derive(Clone, Debug, Display, Error, PartialEq, Eq)]
#[display("{line}:{column}: {kind}{}", context_suffix(context))]
pub struct TextError {
    pub span: Span,
    pub line: usize,
    pub column: usize,
    /// Boxed because it is large (a name and a discriminant) and usually
    /// absent — unboxed it dominates the size of every `Result` in the parser.
    pub context: Option<Box<ParseContext>>,
    pub kind: TextErrorKind,
}

fn context_suffix(context: &Option<Box<ParseContext>>) -> String {
    match context {
        Some(c) => format!(" ({})", c.label()),
        None => String::new(),
    }
}

impl TextError {
    /// An error at `span`, with line/column not yet resolved.
    pub fn new(span: Span, kind: TextErrorKind) -> Self {
        Self {
            span,
            line: 0,
            column: 0,
            context: None,
            kind,
        }
    }

    /// Attach the enclosing construct. Innermost wins: an outer production
    /// never overwrites context an inner one already set.
    pub fn within(mut self, context: &ParseContext) -> Self {
        if self.context.is_none() {
            self.context = Some(Box::new(context.clone()));
        }
        self
    }

    /// The construct the error happened inside, if any.
    pub fn context(&self) -> Option<&ParseContext> {
        self.context.as_deref()
    }

    /// Resolve `line`/`column` against the source the span came from. Called
    /// once, by the parse entry point, on the error path only.
    pub fn locate(mut self, source: &str) -> Self {
        let (line, column) = line_column(source, self.span.start);
        self.line = line;
        self.column = column;
        self
    }
}

/// 1-based line and column of `offset` in `source`.
///
/// Column counts bytes, not characters or display cells: these files are ASCII
/// (the corpus is, and the game writes them with a byte-oriented serialiser), so
/// the distinction cannot arise, and byte columns are what a `&source[..]` slice
/// agrees with.
fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(source.len());
    let before = &source[..offset];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let column = match before.rfind('\n') {
        Some(nl) => offset - nl,
        None => offset + 1,
    };
    (line, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_column_is_one_based() {
        assert_eq!(line_column("abc", 0), (1, 1));
        assert_eq!(line_column("abc", 2), (1, 3));
    }

    #[test]
    fn line_column_counts_crlf_lines() {
        // The corpus is CRLF; `\r` belongs to the preceding line, so the column
        // of the first byte after `\r\n` is 1.
        let src = "one\r\ntwo\r\nthree";
        assert_eq!(line_column(src, 5), (2, 1));
        assert_eq!(line_column(src, 10), (3, 1));
    }

    #[test]
    fn spanned_equality_ignores_span() {
        let a = Spanned {
            span: Span::new(0, 1),
            value: 7,
        };
        let b = Spanned {
            span: Span::new(99, 100),
            value: 7,
        };
        assert_eq!(a, b);
    }
}
