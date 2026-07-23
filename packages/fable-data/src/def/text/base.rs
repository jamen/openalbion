use derive_more::{Display, Error};

/// A byte range in the source text (half-open: `start..end`).
/// Byte offsets, not char offsets — `&source[start..end]` reproduces the text.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// A value annotated with its source span.
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

/// Converts byte offsets to (line, column, line-text) for diagnostic rendering.
pub struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, c) in source.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    pub fn lookup(&self, pos: usize) -> (usize, usize, &str) {
        let line = self
            .line_starts
            .binary_search(&pos)
            .unwrap_or_else(|i| i.saturating_sub(1));
        let col = pos - self.line_starts[line] + 1;
        (line + 1, col, "")
    }

    pub fn line_text<'a>(&self, source: &'a str, pos: usize) -> &'a str {
        let line = self
            .line_starts
            .binary_search(&pos)
            .unwrap_or_else(|i| i.saturating_sub(1));
        let start = self.line_starts[line];
        let end = source[start..]
            .find('\n')
            .map_or(source.len(), |off| start + off);
        &source[start..end]
    }
}

#[derive(Debug, Display, Error)]
#[display("{inner}")]
pub struct ParseError<InnerError> {
    pub pos: usize,
    pub def_header_pos: Option<usize>,
    pub inner: InnerError,
}

impl<T> ParseError<T> {
    pub(crate) fn new(pos: usize, inner: T) -> Self {
        Self {
            pos,
            def_header_pos: None,
            inner,
        }
    }

    pub(crate) fn with_def_header(mut self, def_header_pos: usize) -> Self {
        self.def_header_pos = Some(def_header_pos);
        self
    }
}
