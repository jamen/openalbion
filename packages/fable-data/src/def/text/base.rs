use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
#[display("parse error at {pos}: {inner}")]
pub struct ParseError<InnerError> {
    pub pos: usize,
    pub inner: InnerError,
}

impl<T> ParseError<T> {
    pub(crate) fn new(pos: usize, inner: T) -> Self {
        Self { pos, inner }
    }
    pub(crate) fn with_pos(mut self, pos: usize) -> Self {
        self.pos = pos;
        self
    }
}

pub(crate) struct ParserBase<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> ParserBase<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }
    pub(crate) fn input(&self) -> &str {
        self.input
    }
    pub(crate) fn pos(&self) -> usize {
        self.pos
    }
    pub(crate) fn advance(&mut self, add: usize) {
        self.pos += add;
    }
    pub(crate) fn seek_to(&mut self, pos: usize) {
        self.pos = pos;
    }
    pub(crate) fn rest(&self) -> &str {
        &self.input[self.pos..]
    }
    pub(crate) fn err<T>(&self, inner: T) -> ParseError<T> {
        ParseError::new(self.pos, inner)
    }
    pub(crate) fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
    pub(crate) fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }
    pub(crate) fn peek_char_at(&self, offset: usize) -> Option<char> {
        self.input[self.pos..].chars().nth(offset)
    }
    pub(crate) fn try_consume(&mut self, s: &str) -> bool {
        if self.input[self.pos..].starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }
    pub(crate) fn at_line_start(&self) -> bool {
        let bytes = self.input().as_bytes();
        let mut i = self.pos();
        while i > 0 {
            i -= 1;
            match bytes[i] {
                b' ' | b'\t' => continue,
                b'\n' => return true,
                _ => return false,
            }
        }
        true
    }
}

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum ConsumeCharError {
    #[display("unexpected end of input")]
    UnexpectedEnd,
    #[display("expected {expected}, found {found}")]
    MismatchedCharacter { expected: char, found: char },
    #[display("found {found}")]
    UnexpectedCharacter { found: char },
}

impl<'a> ParserBase<'a> {
    pub(crate) fn consume_char(
        &mut self,
        expected: char,
    ) -> Result<(), ParseError<ConsumeCharError>> {
        use ConsumeCharError as E;
        match self.peek_char() {
            Some(c) if c == expected => {
                self.pos += c.len_utf8();
                Ok(())
            }
            Some(found) => Err(self.err(E::MismatchedCharacter { expected, found })),
            None => Err(self.err(E::UnexpectedEnd)),
        }
    }
}

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum SkipTriviaError {
    #[display("unterminated block comment")]
    UnterminatedBlockComment,
}

impl<'a> ParserBase<'a> {
    pub(crate) fn skip_trivia(&mut self) -> Result<(), ParseError<SkipTriviaError>> {
        use SkipTriviaError as E;
        loop {
            while let Some(c) = self.peek_char() {
                if c.is_whitespace() {
                    self.pos += c.len_utf8();
                } else {
                    break;
                }
            }
            let rest = &self.input[self.pos..];
            if rest.starts_with("//") {
                while let Some(c) = self.peek_char() {
                    self.pos += c.len_utf8();
                    if c == '\n' {
                        break;
                    }
                }
                continue;
            }
            if rest.starts_with("/*") {
                let comment_start = self.pos;
                self.pos += 2;
                loop {
                    if self.input[self.pos..].starts_with("*/") {
                        self.pos += 2;
                        break;
                    }
                    match self.peek_char() {
                        Some(c) => self.pos += c.len_utf8(),
                        None => {
                            self.pos = comment_start;
                            return Err(self.err(E::UnterminatedBlockComment));
                        }
                    }
                }
                continue;
            }
            break;
        }
        Ok(())
    }
}
