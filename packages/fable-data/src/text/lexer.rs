//! The tokenizer for `.tng` / `.wld`, and the cursor the parser runs on.
//!
//! Modelled on `fable-defs`' `text::lexer`, trimmed to the tokens these files
//! actually contain. Lexing all 397 retail `.tng` files plus `FinalAlbion.wld`
//! produces exactly this set and nothing else:
//!
//! ```text
//! Ident 853,461 · Semi 668,784 · Number 382,277 · Str 54,200 · Dot 39,567
//! LBracket/RBracket 24,962 · Comma 24,821 · LParen/RParen 12,313
//! ```
//!
//! So the def grammar's `#` directives, `<tagged blocks>`, `|`, `+`, `=`, `{}`
//! and `<<` are all absent here, and each one left out is a class of error
//! message that cannot be produced. Anything else is [`TextErrorKind::UnexpectedChar`].
//!
//! The lexer *classifies and delimits*; it never interprets. Numbers stay raw
//! (the slice, interpreted per-field later) and strings keep their quotes (the
//! parser strips them). `TRUE`/`FALSE` stay [`TokenKind::Ident`] and are
//! recognised by the parser.

use super::base::{Span, TextError, TextErrorKind};

/// A lexical token kind. Flat, `Copy`, payload-free — the raw text lives in
/// [`Token::source`] and the range in [`Token::span`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Ident,
    Number,
    Str,
    Dot,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Comma,
    Semi,
    Eof,
}

impl TokenKind {
    /// A short human name, for "expected …, found …" messages.
    pub fn describe(self) -> &'static str {
        match self {
            TokenKind::Ident => "identifier",
            TokenKind::Number => "number",
            TokenKind::Str => "string",
            TokenKind::Dot => "`.`",
            TokenKind::LBracket => "`[`",
            TokenKind::RBracket => "`]`",
            TokenKind::LParen => "`(`",
            TokenKind::RParen => "`)`",
            TokenKind::Comma => "`,`",
            TokenKind::Semi => "`;`",
            TokenKind::Eof => "end of input",
        }
    }
}

/// A token: its kind, its byte span, and the exact source slice it covers.
///
/// `source` is `&input[span.start..span.end]`, so `Number`/`Str` slices
/// reproduce their source byte-for-byte. `Str`'s `source` includes the quotes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: Span,
    pub source: &'a str,
}

/// The tokenizer. Borrows the source for the token slices' lifetime.
pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Tokenize the whole input, ending with a single [`TokenKind::Eof`].
    /// The first error aborts and is returned.
    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>, TextError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.pos >= self.input.len() {
                tokens.push(self.token(TokenKind::Eof, self.pos, self.pos));
                return Ok(tokens);
            }
            tokens.push(self.next_token()?);
        }
    }

    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.rest().chars().nth(offset)
    }

    fn token(&self, kind: TokenKind, start: usize, end: usize) -> Token<'a> {
        Token {
            kind,
            span: Span::new(start, end),
            source: &self.input[start..end],
        }
    }

    fn error(&self, kind: TextErrorKind, start: usize, end: usize) -> TextError {
        TextError::new(Span::new(start, end.min(self.input.len())), kind)
    }

    /// Consume whitespace (`\r` included — the corpus is CRLF), `//` line
    /// comments and `/* */` block comments.
    ///
    /// The shipped corpus contains **no comments at all**; they are tolerated
    /// because these files are round-tripped through Fable's own editor and
    /// hand-edited by modders, and the sibling def grammar accepts them.
    /// Deliberately *not* carried over from the def lexer: its decorative-banner
    /// rule, which exists only to rescue `/*****` section dividers in `Defs/`.
    fn skip_trivia(&mut self) -> Result<(), TextError> {
        loop {
            while let Some(c) = self.peek() {
                if c.is_whitespace() {
                    self.pos += c.len_utf8();
                } else {
                    break;
                }
            }
            if self.rest().starts_with("//") {
                self.skip_to_end_of_line();
                continue;
            }
            if let Some(after_open) = self.rest().strip_prefix("/*") {
                let Some(off) = after_open.find("*/") else {
                    // Anchor on the `/*` that was never closed, not on the rest
                    // of the file: the opener is what has to be fixed.
                    return Err(self.error(
                        TextErrorKind::UnterminatedBlockComment,
                        self.pos,
                        self.pos + 2,
                    ));
                };
                self.pos += 2 + off + 2;
                continue;
            }
            return Ok(());
        }
    }

    /// Advance past the next `\n` (or to EOF if none remains).
    fn skip_to_end_of_line(&mut self) {
        while let Some(c) = self.peek() {
            self.pos += c.len_utf8();
            if c == '\n' {
                break;
            }
        }
    }

    /// Dispatch on the next non-trivia character. Precondition: not at EOF.
    fn next_token(&mut self) -> Result<Token<'a>, TextError> {
        let c = self.peek().expect("next_token called at EOF");
        match c {
            '"' => self.lex_string(),
            '0'..='9' => Ok(self.lex_number()),
            '-' if self.peek_at(1).is_some_and(|d| d.is_ascii_digit()) => Ok(self.lex_number()),
            c if c == '_' || c.is_ascii_alphabetic() => Ok(self.lex_ident()),
            _ => self.lex_punct(),
        }
    }

    /// Raw number slice, kept uninterpreted: optional leading `-`, digits,
    /// optional `.frac`. The corpus contains nothing else — no exponents, no
    /// leading `.`, no `f` suffix — but the shape is the def grammar's minus the
    /// `f`, so the two agree on every literal either can see.
    ///
    /// Precondition: a digit is present (guaranteed by [`Self::next_token`]).
    fn lex_number(&mut self) -> Token<'a> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.peek() == Some('.') {
            self.pos += 1;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        self.token(TokenKind::Number, start, self.pos)
    }

    /// Raw string slice including the quotes.
    ///
    /// **No escape processing**, matching the game's serialiser: level paths are
    /// written `"FinalAlbion\LookoutPoint.lev"` and the backslash is a literal
    /// byte, not the start of an escape.
    ///
    /// Precondition: positioned on the opening `"`.
    fn lex_string(&mut self) -> Result<Token<'a>, TextError> {
        let start = self.pos;
        self.pos += 1; // opening quote
        while let Some(c) = self.peek() {
            if c == '"' {
                self.pos += 1; // closing quote
                return Ok(self.token(TokenKind::Str, start, self.pos));
            }
            self.pos += c.len_utf8();
        }
        // The scan ran to EOF, but the byte to go fix is where the string began.
        Err(self.error(TextErrorKind::UnterminatedString, start, start + 1))
    }

    /// `[A-Za-z_][A-Za-z0-9_]*`. Precondition: positioned on `_` or a letter.
    fn lex_ident(&mut self) -> Token<'a> {
        let start = self.pos;
        self.pos += 1; // first char (letter or `_`)
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.pos += 1;
        }
        self.token(TokenKind::Ident, start, self.pos)
    }

    fn lex_punct(&mut self) -> Result<Token<'a>, TextError> {
        let start = self.pos;
        let c = self.peek().expect("lex_punct called at EOF");
        let kind = match c {
            '.' => TokenKind::Dot,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semi,
            other => {
                return Err(self.error(
                    TextErrorKind::UnexpectedChar(other),
                    start,
                    start + other.len_utf8(),
                ));
            }
        };
        self.pos += c.len_utf8(); // all matched punctuation is single-byte ASCII
        Ok(self.token(kind, start, self.pos))
    }
}

/// Tokenize `input` into a `Vec` ending with a single `Eof`.
pub fn lex(input: &str) -> Result<Vec<Token<'_>>, TextError> {
    Lexer::new(input).tokenize()
}

/// A token cursor over one source, with the source kept so a production can
/// take a slice spanning several tokens.
///
/// The slice is what makes `NewThing Holy Site;` representable: a thing's kind
/// is a *type name* written by `CThing::GetTypeName()`
/// (`fablelib/thing_manager.cpp:5638` writes `"NewThing " + GetTypeName()`), not
/// an identifier, so one of the nine kinds has a space in it.
pub struct Cursor<'a> {
    input: &'a str,
    tokens: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str, tokens: Vec<Token<'a>>) -> Self {
        Self {
            input,
            tokens,
            pos: 0,
        }
    }

    /// The current token. Always valid: the stream ends in exactly one `Eof` and
    /// [`bump`](Self::bump) never advances past it.
    pub fn peek(&self) -> Token<'a> {
        self.tokens[self.pos]
    }

    /// The token `n` ahead, saturating at the trailing `Eof`.
    pub fn peek_at(&self, n: usize) -> Token<'a> {
        *self
            .tokens
            .get(self.pos + n)
            .unwrap_or_else(|| self.tokens.last().expect("stream ends in Eof"))
    }

    /// Return the current token and advance (never past `Eof`).
    pub fn bump(&mut self) -> Token<'a> {
        let token = self.tokens[self.pos];
        if token.kind != TokenKind::Eof {
            self.pos += 1;
        }
        token
    }

    pub fn at(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    /// End offset of the most recently consumed token — used for node spans.
    pub fn prev_end(&self) -> usize {
        self.tokens[self.pos.saturating_sub(1)].span.end
    }

    /// The source text between two byte offsets, verbatim.
    pub fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.input[start..end]
    }

    /// An error about the token the cursor is sitting on, so the span covers the
    /// whole token rather than a single byte.
    pub fn err(&self, kind: TextErrorKind) -> TextError {
        TextError::new(self.peek().span, kind)
    }

    /// An `UnexpectedToken` about the current token, describing what the grammar
    /// wanted. The "found …" half is filled in from the token.
    pub fn unexpected(&self, expected: impl Into<std::borrow::Cow<'static, str>>) -> TextError {
        self.err(TextErrorKind::UnexpectedToken {
            expected: expected.into(),
            found: self.peek().kind.describe(),
        })
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<Token<'a>, TextError> {
        if self.at(kind) {
            Ok(self.bump())
        } else {
            Err(self.unexpected(kind.describe()))
        }
    }

    pub fn expect_ident(&mut self, what: &'static str) -> Result<&'a str, TextError> {
        if self.at(TokenKind::Ident) {
            Ok(self.bump().source)
        } else {
            Err(self.unexpected(what))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TokenKind::*;
    use super::*;

    /// Lex, dropping the trailing `Eof`, and collect `(kind, source)` pairs.
    fn toks(input: &str) -> Vec<(TokenKind, &str)> {
        let mut tokens = lex(input).expect("lex ok");
        assert_eq!(tokens.last().map(|t| t.kind), Some(Eof));
        tokens.pop();
        tokens.into_iter().map(|t| (t.kind, t.source)).collect()
    }

    fn kinds(input: &str) -> Vec<TokenKind> {
        toks(input).into_iter().map(|(k, _)| k).collect()
    }

    fn lex_err(input: &str) -> TextErrorKind {
        lex(input).expect_err("expected lex error").kind
    }

    #[test]
    fn numbers_keep_their_source() {
        assert_eq!(toks("42"), vec![(Number, "42")]);
        assert_eq!(toks("-0.651341"), vec![(Number, "-0.651341")]);
        // UIDs are u64 and overflow i64 — the lexer never interprets them.
        assert_eq!(
            toks("18446741874686301490"),
            vec![(Number, "18446741874686301490")]
        );
    }

    #[test]
    fn string_keeps_quotes_and_backslashes() {
        // `.wld` level paths: the backslash is a literal byte, not an escape.
        assert_eq!(
            toks(r#""FinalAlbion\LookoutPoint.lev""#),
            vec![(Str, r#""FinalAlbion\LookoutPoint.lev""#)]
        );
    }

    #[test]
    fn bools_stay_idents() {
        // Recognised by the parser, not here.
        assert_eq!(kinds("TRUE"), vec![Ident]);
        assert_eq!(kinds("FALSE"), vec![Ident]);
    }

    #[test]
    fn indexed_path() {
        assert_eq!(
            kinds("KeyCameras[0].Position"),
            vec![Ident, LBracket, Number, RBracket, Dot, Ident]
        );
    }

    #[test]
    fn ident_index() {
        // `.wld`: MiniMapRegionExitTextOffsetX[HeroGuildComplexInside] 0.0;
        assert_eq!(
            kinds("Offset[HeroGuildComplexInside]"),
            vec![Ident, LBracket, Ident, RBracket]
        );
    }

    #[test]
    fn constructor_call() {
        assert_eq!(
            kinds("C3DCoordF(-3359.448242,-3569.195801,-29.327202)"),
            vec![Ident, LParen, Number, Comma, Number, Comma, Number, RParen]
        );
    }

    #[test]
    fn crlf_is_whitespace() {
        assert_eq!(
            toks("PositionX\r\n10.0;\r\n"),
            vec![(Ident, "PositionX"), (Number, "10.0"), (Semi, ";")]
        );
    }

    #[test]
    fn comments_are_trivia() {
        assert_eq!(kinds("// note\nHealth 1.0;"), vec![Ident, Number, Semi]);
        assert_eq!(kinds("/* note */ Health 1.0;"), vec![Ident, Number, Semi]);
    }

    #[test]
    fn errors_point_at_the_offender() {
        assert_eq!(lex_err("Health @ 1"), TextErrorKind::UnexpectedChar('@'));
        // A bare `-` not followed by a digit is not a number start.
        assert_eq!(lex_err("- 1"), TextErrorKind::UnexpectedChar('-'));
        assert_eq!(
            lex_err("Name \"no close\n"),
            TextErrorKind::UnterminatedString
        );
        assert_eq!(
            lex_err("Health /* never closes"),
            TextErrorKind::UnterminatedBlockComment
        );
        let e = lex("Health @").expect_err("lex error");
        assert_eq!(e.span, Span::new(7, 8));
    }

    #[test]
    fn the_def_grammars_punctuation_is_rejected() {
        // Each of these is a token in `Defs/` and cannot occur here; leaving them
        // out is what makes the error specific instead of a confusing parse.
        for input in ["<CPhysicsDef>", "A | B", "1 + 2", "x = 1", "#definition"] {
            assert!(matches!(
                lex_err(input),
                TextErrorKind::UnexpectedChar(_)
            ));
        }
    }

    #[test]
    fn always_ends_with_eof() {
        let tokens = lex("Health 1.0;").unwrap();
        let last = tokens.last().unwrap();
        assert_eq!(last.kind, Eof);
        assert_eq!(last.source, "");
        assert_eq!(last.span, Span::new(11, 11));
    }

    #[test]
    fn empty_input_is_just_eof() {
        for input in ["", "   \r\n\t ", "// only a comment\n"] {
            assert_eq!(
                lex(input).unwrap().into_iter().map(|t| t.kind).collect::<Vec<_>>(),
                vec![Eof]
            );
        }
    }
}
