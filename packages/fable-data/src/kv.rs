//! Shared key-value text parser for Fable's `.tng`/`.wld`/`.def` family.
//!
//! The grammar is `Field value;` and `StartX ... EndX;` / `NewThing ... EndThing;` blocks.
//! This is a simple line/text tokenizer — not a full def compiler.

use derive_more::Display;

#[derive(Debug, Display)]
#[display("parse error at line {line}: {inner}")]
pub struct KvError {
    pub line: usize,
    pub inner: KvErrorKind,
}

impl std::error::Error for KvError {}

#[derive(Debug, Display)]
pub enum KvErrorKind {
    #[display("unexpected end of input")]
    UnexpectedEnd,
    #[display("unterminated string")]
    UnterminatedString,
    #[display("unexpected token: {_0}")]
    Unexpected(String),
    #[display("expected semicolon")]
    ExpectedSemicolon,
}

impl KvError {
    pub fn new(line: usize, inner: KvErrorKind) -> Self {
        Self { line, inner }
    }
}

/// A token in the key-value grammar.
#[derive(Debug, Clone, PartialEq)]
pub enum KvToken<'a> {
    /// An identifier or keyword, e.g. `NewThing`, `DefinitionType`.
    Ident(&'a str),
    /// A double-quoted string.
    String(&'a str),
    /// A numeric value (integer or float).
    Number(&'a str),
    /// An `=` sign (unused in .tng/.wld).
    Equals,
    /// A semicolon `;`.
    Semicolon,
    /// A `{` (unused in .tng/.wld).
    OpenBrace,
    /// A `}` (unused in .tng/.wld).
    CloseBrace,
}

/// A parsed key-value statement.
#[derive(Debug, Clone, PartialEq)]
pub enum KvStatement<'a> {
    /// `Field value;`
    Field(&'a str, KvValue<'a>),
    /// `StartX ... EndX;` or `NewThing ... EndThing;` block.
    Block {
        keyword: &'a str,
        kind: &'a str,
        body: Vec<KvStatement<'a>>,
    },
}

/// A value in a key-value assignment.
#[derive(Debug, Clone, PartialEq)]
pub enum KvValue<'a> {
    String(&'a str),
    Number(&'a str),
    Ident(&'a str),
    Bool(bool),
}

pub struct KvParser<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
}

impl<'a> KvParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
        }
    }

    pub fn rest(&self) -> &str {
        &self.input[self.pos..]
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn err(&self, inner: KvErrorKind) -> KvError {
        KvError::new(self.line, inner)
    }

    /// Parse a sequence of statements until EOF.
    pub fn parse_statements(&mut self) -> Result<Vec<KvStatement<'a>>, KvError> {
        let mut stmts = Vec::new();
        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }
            // Skip section-end markers at the top level (they're handled in block parsing).
            if self.try_consume_ident("XXXSectionEnd") {
                break;
            }
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<KvStatement<'a>, KvError> {
        self.skip_whitespace();
        if self.is_eof() {
            return Err(self.err(KvErrorKind::UnexpectedEnd));
        }

        let ident = self.read_ident()?;

        match ident {
            // Block keywords: NewThing, XXXSectionStart, NewMap, NewRegion.
            "NewThing" | "XXXSectionStart" | "NewMap" | "NewRegion" => {
                let kind = self.read_ident()?;
                let end_kw = match ident {
                    "NewThing" => "EndThing",
                    "XXXSectionStart" => "XXXSectionEnd",
                    "NewMap" => "EndMap",
                    "NewRegion" => "EndRegion",
                    _ => unreachable!(),
                };
                self.skip_whitespace();
                self.expect_semicolon()?;
                let body = self.parse_block(end_kw)?;
                Ok(KvStatement::Block {
                    keyword: ident,
                    kind,
                    body,
                })
            }
            _ => {
                self.skip_whitespace();
                // Standalone keyword (no value, just semicolon) — e.g. "StartCTCPhysicsStandard;"
                if self.input[self.pos..].starts_with(';') {
                    self.expect_semicolon()?;
                    return Ok(KvStatement::Field(ident, KvValue::Ident("")));
                }
                let value = self.parse_value()?;
                self.skip_whitespace();
                self.expect_semicolon()?;
                Ok(KvStatement::Field(ident, value))
            }
        }
    }

    fn parse_block(&mut self, end_kw: &str) -> Result<Vec<KvStatement<'a>>, KvError> {
        let mut body = Vec::new();
        loop {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }
            // Check for end marker
            let rest = self.rest();
            let first_word = rest
                .split(|c: char| c.is_whitespace() || c == ';')
                .next()
                .unwrap_or("");
            if first_word == end_kw {
                self.consume_ident(end_kw)?;
                self.skip_whitespace();
                self.expect_semicolon()?;
                break;
            }
            // StartX/EndX pairs are fields at this level, not sub-blocks
            body.push(self.parse_statement()?);
        }
        Ok(body)
    }

    fn parse_value(&mut self) -> Result<KvValue<'a>, KvError> {
        self.skip_whitespace();
        let c = self
            .input[self.pos..]
            .chars()
            .next()
            .ok_or(self.err(KvErrorKind::UnexpectedEnd))?;

        match c {
            '"' => {
                self.pos += 1;
                let end = self.input[self.pos..]
                    .find('"')
                    .ok_or(self.err(KvErrorKind::UnterminatedString))?;
                let s = &self.input[self.pos..self.pos + end];
                self.pos += end + 1;
                Ok(KvValue::String(s))
            }
            'T' | 't' => {
                if self.try_consume_ident("TRUE") {
                    Ok(KvValue::Bool(true))
                } else {
                    let ident = self.read_ident()?;
                    Ok(KvValue::Ident(ident))
                }
            }
            'F' | 'f' => {
                if self.try_consume_ident("FALSE") {
                    Ok(KvValue::Bool(false))
                } else {
                    let ident = self.read_ident()?;
                    Ok(KvValue::Ident(ident))
                }
            }
            '0'..='9' | '-' | '+' | '.' => {
                let start = self.pos;
                if c == '-' || c == '+' {
                    self.pos += 1;
                }
                while self.pos < self.input.len() {
                    let d = self.input.as_bytes()[self.pos];
                    if d.is_ascii_digit() || d == b'.' || d == b'e' || d == b'E' || d == b'-' || d == b'+' {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                Ok(KvValue::Number(&self.input[start..self.pos]))
            }
            _ => {
                let ident = self.read_ident()?;
                Ok(KvValue::Ident(ident))
            }
        }
    }

    fn read_ident(&mut self) -> Result<&'a str, KvError> {
        self.skip_whitespace();
        let start = self.pos;
        while self.pos < self.input.len() {
            let b = self.input.as_bytes()[self.pos];
            if b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'\\' || b == b'-' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(self.err(KvErrorKind::UnexpectedEnd));
        }
        Ok(&self.input[start..self.pos])
    }

    fn try_consume_ident(&mut self, s: &str) -> bool {
        if self.input[self.pos..].starts_with(s) {
            let rest = &self.input[self.pos + s.len()..];
            let next = rest.chars().next();
            // Only match if followed by non-ident char or EOF
            if next
                .map(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '.')
                .unwrap_or(true)
            {
                self.pos += s.len();
                return true;
            }
        }
        false
    }

    fn consume_ident(&mut self, expected: &str) -> Result<(), KvError> {
        let ident = self.read_ident()?;
        if ident != expected {
            return Err(self.err(KvErrorKind::Unexpected("expected identifier".to_string())));
        }
        Ok(())
    }

    fn expect_semicolon(&mut self) -> Result<(), KvError> {
        self.skip_whitespace();
        if self.input[self.pos..].starts_with(';') {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err(KvErrorKind::ExpectedSemicolon))
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let bytes = &self.input.as_bytes()[self.pos..];

            // Count and skip whitespace, tracking line numbers.
            let mut advance = 0usize;
            let mut i = 0usize;
            while i < bytes.len() {
                let b = bytes[i];
                if b == b' ' || b == b'\t' {
                    i += 1;
                } else if b == b'\r' {
                    i += 1;
                    self.line += 1;
                    if i < bytes.len() && bytes[i] == b'\n' {
                        i += 1; // consume \n part of \r\n as one line
                    }
                } else if b == b'\n' {
                    i += 1;
                    self.line += 1;
                } else {
                    break;
                }
                advance = i;
            }
            self.pos += advance;

            // Skip line comments (// ...)
            if self.input[self.pos..].starts_with("//") {
                let rest = &self.input[self.pos..];
                if let Some(end) = rest.find('\n') {
                    self.pos += end + 1;
                    self.line += 1;
                } else {
                    self.pos = self.input.len();
                }
                continue;
            }

            // Skip block comments (/* ... */)
            if self.input[self.pos..].starts_with("/*") {
                let rest = &self.input[self.pos + 2..];
                if let Some(end) = rest.find("*/") {
                    let comment = &rest[..end];
                    self.line += comment.chars().filter(|&c| c == '\n').count();
                    self.pos += end + 4;
                } else {
                    self.pos = self.input.len();
                }
                continue;
            }

            break;
        }
    }
}
