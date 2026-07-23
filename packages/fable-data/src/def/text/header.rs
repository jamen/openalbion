use super::base::ParseError;
use super::lexer::{LexError, LexErrorKind, Token, TokenKind, lex};
use derive_more::Display;

// ── Data types (unchanged from the char-parser era) ───────────────────────────

#[derive(Debug, Clone, Default)]
pub struct Header {
    pub items: Vec<HeaderItem>,
}

#[derive(Debug, Clone)]
pub enum HeaderItem {
    Enum(EnumDecl),
    Define(Define),
    Namespace(Namespace),
    IfDef(IfDef),
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: Option<String>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<EnumExpr>,
}

#[derive(Debug, Clone)]
pub enum EnumExpr {
    Int(i64),
    Ident(String),
    Shift(Vec<EnumExpr>),
    BitOr(Vec<EnumExpr>),
}

#[derive(Debug, Clone)]
pub struct Define {
    pub name: String,
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct Namespace {
    pub name: String,
    pub items: Vec<HeaderItem>,
}

#[derive(Debug, Clone)]
pub struct IfDef {
    pub condition: String,
    pub if_branch: Vec<HeaderItem>,
    pub else_branch: Option<Vec<HeaderItem>>,
    pub inverted: bool,
}

// ── Error types ───────────────────────────────────────────────────────────────

pub type HeaderParseError = ParseError<HeaderParseErrorKind>;

#[derive(Debug, Display)]
pub enum HeaderParseErrorKind {
    #[display("expected {expected}")]
    UnexpectedToken { expected: String },
    #[display("unterminated namespace")]
    UnterminatedNamespace,
    #[display("unterminated #ifdef")]
    UnterminatedIfDef,
    #[display("unterminated enum")]
    UnterminatedEnum,
    #[display("unknown item")]
    UnknownItem,
    #[display("invalid number")]
    InvalidNumber,
    #[display("unterminated string")]
    UnterminatedString,
    #[display("unterminated block comment")]
    UnterminatedBlockComment,
    #[display("unexpected character {_0:?}")]
    UnexpectedChar(char),
}

fn lex_error_to_header_error(e: LexError) -> HeaderParseError {
    let kind = match e.kind {
        LexErrorKind::UnterminatedString => HeaderParseErrorKind::UnterminatedString,
        LexErrorKind::UnterminatedBlockComment => HeaderParseErrorKind::UnterminatedBlockComment,
        LexErrorKind::UnexpectedChar(c) => HeaderParseErrorKind::UnexpectedChar(c),
    };
    ParseError::new(e.span.start, kind)
}

// ── Token-based header parser ─────────────────────────────────────────────────

fn describe(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::Ident => "identifier",
        TokenKind::Number => "number",
        TokenKind::Str => "string",
        TokenKind::Definition => "#definition",
        TokenKind::DefinitionTemplate => "#definition_template",
        TokenKind::EndDefinition => "#end_definition",
        TokenKind::Define => "#define",
        TokenKind::Ifdef => "#ifdef",
        TokenKind::Ifndef => "#ifndef",
        TokenKind::Else => "#else",
        TokenKind::Endif => "#endif",
        TokenKind::Pragma => "#pragma",
        TokenKind::Namespace => "namespace",
        TokenKind::Enum => "enum",
        TokenKind::Dot => ".",
        TokenKind::LBracket => "[",
        TokenKind::RBracket => "]",
        TokenKind::LParen => "(",
        TokenKind::RParen => ")",
        TokenKind::LBrace => "{",
        TokenKind::RBrace => "}",
        TokenKind::Lt => "<",
        TokenKind::Gt => ">",
        TokenKind::Backslash => "\\",
        TokenKind::Pipe => "|",
        TokenKind::Plus => "+",
        TokenKind::Shl => "<<",
        TokenKind::Eq => "=",
        TokenKind::Comma => ",",
        TokenKind::Semi => ";",
        TokenKind::Eof => "end of input",
    }
}

pub struct HeaderParser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> HeaderParser<'a> {
    /// Build a parser over the tokenized input (for standalone `.h` files).
    pub fn new(input: &'a str) -> Result<Self, HeaderParseError> {
        let tokens = lex(input).map_err(lex_error_to_header_error)?;
        Ok(Self { tokens, pos: 0 })
    }

    /// Build a parser from an existing token stream (for inline `enum`/
    /// `#define` in `.def` files). After parsing call [`consumed`](Self::consumed)
    /// to learn how many tokens were consumed.
    pub fn from_tokens(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Number of tokens consumed so far.
    pub fn consumed(&self) -> usize {
        self.pos
    }

    // ── Token cursor ──────────────────────────────────────────────────────────

    fn peek(&self) -> Token<'a> {
        self.tokens[self.pos]
    }

    fn bump(&mut self) -> Token<'a> {
        let tok = self.tokens[self.pos];
        if tok.kind != TokenKind::Eof {
            self.pos += 1;
        }
        tok
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn err(&self, kind: HeaderParseErrorKind) -> HeaderParseError {
        ParseError::new(self.peek().span.start, kind)
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token<'a>, HeaderParseError> {
        if self.at(kind) {
            Ok(self.bump())
        } else {
            let found = self.peek();
            Err(self.err(HeaderParseErrorKind::UnexpectedToken {
                expected: format!("{}, found {}", describe(kind), describe(found.kind)),
            }))
        }
    }

    fn expect_ident(&mut self) -> Result<String, HeaderParseError> {
        let t = self.peek();
        if t.kind == TokenKind::Ident {
            self.bump();
            Ok(t.source.to_string())
        } else {
            Err(self.err(HeaderParseErrorKind::UnexpectedToken {
                expected: format!("identifier, found {}", describe(t.kind)),
            }))
        }
    }

    // ── Productions ───────────────────────────────────────────────────────────

    pub fn parse_file(&mut self) -> Result<Header, HeaderParseError> {
        let mut header = Header::default();
        self.skip_prologue();
        loop {
            let tk = self.peek().kind;
            if tk == TokenKind::Eof || tk == TokenKind::Endif {
                break;
            }
            header.items.push(self.parse_item()?);
        }
        self.skip_epilogue();
        Ok(header)
    }

    /// Parse a single header item (`enum`/`#define`/`namespace`/`#ifdef`/
    /// `#ifndef`) at the current position. Used by the def parser to consume
    /// file-local declarations embedded in a `.def` file.
    pub fn parse_one_item(&mut self) -> Result<HeaderItem, HeaderParseError> {
        self.parse_item()
    }

    fn skip_prologue(&mut self) {
        if self.at(TokenKind::Pragma) {
            self.bump();
            if self.at(TokenKind::Ident) {
                self.bump(); // `once`
            }
        }
        if self.at(TokenKind::Ifndef) {
            self.bump();
            let _ = self.expect_ident(); // guard name
            if self.at(TokenKind::Define) {
                self.bump();
                let _ = self.expect_ident(); // guard name
                if self.at(TokenKind::Number) {
                    self.bump(); // optional value
                }
            }
        }
    }

    /// The old parser's `skip_to_end_of_line` consumed any guard-name token
    /// that follows `#endif` without a `//` prefix (e.g.
    /// `#endif __IDLE_STATE_GROUP_DEF_H__`).
    fn skip_epilogue(&mut self) {
        if self.at(TokenKind::Endif) {
            self.bump();
        }
        while !self.at(TokenKind::Eof) {
            self.bump();
        }
    }

    fn parse_item(&mut self) -> Result<HeaderItem, HeaderParseError> {
        match self.peek().kind {
            TokenKind::Enum => {
                self.bump();
                Ok(HeaderItem::Enum(self.parse_enum_body()?))
            }
            TokenKind::Define => {
                self.bump();
                Ok(HeaderItem::Define(self.parse_define_body()?))
            }
            TokenKind::Namespace => {
                self.bump();
                Ok(HeaderItem::Namespace(self.parse_namespace_body()?))
            }
            TokenKind::Ifdef | TokenKind::Ifndef => {
                let inverted = self.peek().kind == TokenKind::Ifndef;
                self.bump();
                Ok(HeaderItem::IfDef(self.parse_if_def_body(inverted)?))
            }
            _ => Err(self.err(HeaderParseErrorKind::UnknownItem)),
        }
    }

    fn parse_enum_body(&mut self) -> Result<EnumDecl, HeaderParseError> {
        let name = if self.at(TokenKind::Ident) {
            Some(self.expect_ident()?)
        } else {
            None
        };
        self.expect(TokenKind::LBrace)?;
        let variants = self.parse_enum_variants()?;
        self.expect(TokenKind::RBrace)?;
        if self.at(TokenKind::Semi) {
            self.bump();
        }
        Ok(EnumDecl { name, variants })
    }

    fn parse_enum_variants(&mut self) -> Result<Vec<EnumVariant>, HeaderParseError> {
        let mut variants = Vec::new();
        loop {
            if self.at(TokenKind::Eof) {
                return Err(self.err(HeaderParseErrorKind::UnterminatedEnum));
            }
            if self.at(TokenKind::RBrace) {
                break;
            }
            variants.push(self.parse_enum_variant()?);
            if self.at(TokenKind::Comma) {
                self.bump();
            } else {
                break;
            }
        }
        Ok(variants)
    }

    fn parse_enum_variant(&mut self) -> Result<EnumVariant, HeaderParseError> {
        let name = self.expect_ident()?;
        let value = if self.at(TokenKind::Eq) {
            self.bump();
            Some(self.parse_enum_expr()?)
        } else {
            None
        };
        Ok(EnumVariant { name, value })
    }

    fn parse_enum_expr(&mut self) -> Result<EnumExpr, HeaderParseError> {
        self.parse_enum_bitor()
    }

    fn parse_enum_bitor(&mut self) -> Result<EnumExpr, HeaderParseError> {
        let first = self.parse_enum_shift()?;
        let mut terms = vec![first];
        while self.at(TokenKind::Pipe) {
            self.bump();
            terms.push(self.parse_enum_shift()?);
        }
        Ok(if terms.len() == 1 {
            terms.pop().unwrap()
        } else {
            EnumExpr::BitOr(terms)
        })
    }

    fn parse_enum_shift(&mut self) -> Result<EnumExpr, HeaderParseError> {
        let first = self.parse_enum_leaf()?;
        let mut terms = vec![first];
        while self.at(TokenKind::Shl) {
            self.bump();
            terms.push(self.parse_enum_leaf()?);
        }
        Ok(if terms.len() == 1 {
            terms.pop().unwrap()
        } else {
            EnumExpr::Shift(terms)
        })
    }

    fn parse_enum_leaf(&mut self) -> Result<EnumExpr, HeaderParseError> {
        match self.peek().kind {
            TokenKind::Number => {
                let t = self.bump();
                let n = t
                    .source
                    .parse::<i64>()
                    .map_err(|_| ParseError::new(t.span.start, HeaderParseErrorKind::InvalidNumber))?;
                Ok(EnumExpr::Int(n))
            }
            TokenKind::Ident => {
                let t = self.bump();
                Ok(EnumExpr::Ident(t.source.to_string()))
            }
            _ => Err(self.err(HeaderParseErrorKind::UnexpectedToken {
                expected: format!(
                    "number or identifier, found {}",
                    describe(self.peek().kind)
                ),
            })),
        }
    }

    fn parse_define_body(&mut self) -> Result<Define, HeaderParseError> {
        let name = self.expect_ident()?;
        let t = self.peek();
        if t.kind != TokenKind::Number {
            return Err(self.err(HeaderParseErrorKind::UnexpectedToken {
                expected: format!("number, found {}", describe(t.kind)),
            }));
        }
        self.bump();
        let value = t
            .source
            .parse::<i64>()
            .map_err(|_| ParseError::new(t.span.start, HeaderParseErrorKind::InvalidNumber))?;
        Ok(Define { name, value })
    }

    fn parse_namespace_body(&mut self) -> Result<Namespace, HeaderParseError> {
        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace)?;
        let mut items = Vec::new();
        loop {
            if self.at(TokenKind::Eof) {
                return Err(self.err(HeaderParseErrorKind::UnterminatedNamespace));
            }
            if self.at(TokenKind::RBrace) {
                break;
            }
            items.push(self.parse_item()?);
        }
        self.expect(TokenKind::RBrace)?;
        if self.at(TokenKind::Semi) {
            self.bump();
        }
        Ok(Namespace { name, items })
    }

    fn parse_if_def_body(&mut self, inverted: bool) -> Result<IfDef, HeaderParseError> {
        let condition = self.expect_ident()?;
        let mut if_branch = Vec::new();
        loop {
            if self.at(TokenKind::Eof) {
                return Err(self.err(HeaderParseErrorKind::UnterminatedIfDef));
            }
            if self.at(TokenKind::Else) || self.at(TokenKind::Endif) {
                break;
            }
            if_branch.push(self.parse_item()?);
        }
        let else_branch = if self.at(TokenKind::Else) {
            self.bump();
            let mut else_branch = Vec::new();
            loop {
                if self.at(TokenKind::Eof) {
                    return Err(self.err(HeaderParseErrorKind::UnterminatedIfDef));
                }
                if self.at(TokenKind::Endif) {
                    break;
                }
                else_branch.push(self.parse_item()?);
            }
            Some(else_branch)
        } else {
            None
        };
        self.expect(TokenKind::Endif)?;
        Ok(IfDef {
            condition,
            if_branch,
            else_branch,
            inverted,
        })
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn parse_header_file(input: &str) -> Result<Header, HeaderParseError> {
    let mut parser = HeaderParser::new(input)?;
    parser.parse_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_h(input: &str) -> Header {
        parse_header_file(input).expect("header parse ok")
    }

    fn parse_h_err(input: &str) -> HeaderParseErrorKind {
        parse_header_file(input).unwrap_err().inner
    }

    #[test]
    fn empty_file() {
        let h = parse_h("");
        assert!(h.items.is_empty());
    }

    #[test]
    fn just_include_guard() {
        let h = parse_h("#pragma once\n#ifndef __FOO_H__\n#define __FOO_H__\n#endif");
        assert!(h.items.is_empty());
    }

    #[test]
    fn named_enum() {
        let h = parse_h("enum EFoo { A = 1, B = 2 };");
        assert_eq!(h.items.len(), 1);
        let HeaderItem::Enum(decl) = &h.items[0] else { panic!() };
        assert_eq!(decl.name.as_deref(), Some("EFoo"));
        assert_eq!(decl.variants.len(), 2);
        assert_eq!(decl.variants[0].name, "A");
        assert!(matches!(decl.variants[0].value, Some(EnumExpr::Int(1))));
        assert_eq!(decl.variants[1].name, "B");
        assert!(matches!(decl.variants[1].value, Some(EnumExpr::Int(2))));
    }

    #[test]
    fn anonymous_enum() {
        let h = parse_h("enum { A = 1, B = 2 };");
        let HeaderItem::Enum(decl) = &h.items[0] else { panic!() };
        assert!(decl.name.is_none());
        assert_eq!(decl.variants.len(), 2);
    }

    #[test]
    fn auto_increment() {
        let h = parse_h("enum EFoo { A = 1, B, C = 5, D };");
        let HeaderItem::Enum(decl) = &h.items[0] else { panic!() };
        assert!(matches!(decl.variants[0].value, Some(EnumExpr::Int(1))));
        assert!(decl.variants[1].value.is_none()); // B = 2 (auto)
        assert!(matches!(decl.variants[2].value, Some(EnumExpr::Int(5))));
        assert!(decl.variants[3].value.is_none()); // D = 6 (auto)
    }

    #[test]
    fn enum_with_ident_value() {
        let h = parse_h("enum EFoo { A = NO_SOUND_TYPES };");
        let HeaderItem::Enum(decl) = &h.items[0] else { panic!() };
        assert!(matches!(
            &decl.variants[0].value,
            Some(EnumExpr::Ident(s)) if s == "NO_SOUND_TYPES"
        ));
    }

    #[test]
    fn enum_with_bitor_expression() {
        let h = parse_h("enum EFoo { A = 1 | 2 | 4 };");
        let HeaderItem::Enum(decl) = &h.items[0] else { panic!() };
        assert!(matches!(&decl.variants[0].value, Some(EnumExpr::BitOr(terms)) if terms.len() == 3));
    }

    #[test]
    fn enum_with_shift_expression() {
        let h = parse_h("enum EFoo { A = 1 << 0, B = 1 << 1 };");
        let HeaderItem::Enum(decl) = &h.items[0] else { panic!() };
        assert!(matches!(&decl.variants[0].value, Some(EnumExpr::Shift(terms)) if terms.len() == 2));
    }

    #[test]
    fn enum_trailing_comma() {
        let h = parse_h("enum EFoo { A = 1, B = 2, };");
        let HeaderItem::Enum(decl) = &h.items[0] else { panic!() };
        assert_eq!(decl.variants.len(), 2);
    }

    #[test]
    fn enum_no_trailing_semicolon() {
        let h = parse_h("enum EFoo { A = 1 }");
        assert_eq!(h.items.len(), 1);
    }

    #[test]
    fn define_positive() {
        let h = parse_h("#define FOO 42");
        let HeaderItem::Define(d) = &h.items[0] else { panic!() };
        assert_eq!(d.name, "FOO");
        assert_eq!(d.value, 42);
    }

    #[test]
    fn define_negative() {
        let h = parse_h("#define FOO -42");
        let HeaderItem::Define(d) = &h.items[0] else { panic!() };
        assert_eq!(d.value, -42);
    }

    #[test]
    fn namespace_with_enums() {
        let h = parse_h("namespace NFoo { enum EA { X = 1 }; }");
        let HeaderItem::Namespace(ns) = &h.items[0] else { panic!() };
        assert_eq!(ns.name, "NFoo");
        assert_eq!(ns.items.len(), 1);
    }

    #[test]
    fn ifdef_with_else() {
        let h = parse_h("#ifdef _WINDOWS\n#define FOO 1\n#else\n#define FOO 2\n#endif");
        let HeaderItem::IfDef(ifdef) = &h.items[0] else { panic!() };
        assert_eq!(ifdef.condition, "_WINDOWS");
        assert_eq!(ifdef.if_branch.len(), 1);
        assert_eq!(ifdef.else_branch.as_ref().unwrap().len(), 1);
        assert!(!ifdef.inverted);
    }

    #[test]
    fn ifndef_as_item() {
        // Put `#ifndef` after a namespace so it's not consumed by the prologue.
        let h = parse_h("namespace N { #ifndef _WINDOWS\n#define FOO 1\n#endif }");
        let HeaderItem::Namespace(ns) = &h.items[0] else { panic!() };
        assert_eq!(ns.items.len(), 1);
        let HeaderItem::IfDef(ifdef) = &ns.items[0] else { panic!() };
        assert!(ifdef.inverted);
        assert_eq!(ifdef.if_branch.len(), 1);
    }

    #[test]
    fn full_header_file() {
        let h = parse_h(
            "#pragma once\n\
             #ifndef __FOO_H__\n\
             #define __FOO_H__\n\
             #define MAX_THINGS 100\n\
             enum EFoo { A = 1, B = 2 };\n\
             #endif",
        );
        // Prologue consumed: #pragma, #ifndef, #define. Items: MAX_THINGS, EFoo.
        assert_eq!(h.items.len(), 2);
    }

    #[test]
    fn file_with_no_guard() {
        let h = parse_h("#define FOO 1\nenum EBar { A };\n");
        assert_eq!(h.items.len(), 2);
    }

    // --- error cases ---

    #[test]
    fn err_unterminated_enum() {
        let kind = parse_h_err("enum EFoo { A = 1");
        assert!(matches!(kind, HeaderParseErrorKind::UnexpectedToken { .. }));
    }

    #[test]
    fn err_unterminated_enum_eof_after_comma() {
        // EOF after a comma is a genuine UnterminatedEnum.
        let kind = parse_h_err("enum EFoo { A = 1,");
        assert!(matches!(kind, HeaderParseErrorKind::UnterminatedEnum));
    }

    #[test]
    fn err_unterminated_namespace() {
        let kind = parse_h_err("namespace NFoo { enum EA { X = 1 };");
        assert!(matches!(kind, HeaderParseErrorKind::UnterminatedNamespace));
    }

    #[test]
    fn err_unterminated_ifdef() {
        let kind = parse_h_err("#ifdef _WINDOWS\n#define FOO 1\n");
        assert!(matches!(kind, HeaderParseErrorKind::UnterminatedIfDef));
    }

    #[test]
    fn err_unknown_item() {
        let kind = parse_h_err("foo bar");
        assert!(matches!(kind, HeaderParseErrorKind::UnknownItem));
    }

    #[test]
    fn err_expected_number_in_define() {
        let kind = parse_h_err("#define FOO abc");
        assert!(matches!(kind, HeaderParseErrorKind::UnexpectedToken { .. }));
    }

    #[test]
    fn stray_guard_name_after_endif_consumed() {
        // `#endif __GUARD__` (no `//`) is consumed by the epilogue, matching
        // the old parser's `skip_to_end_of_line`.
        let h = parse_h("#define FOO 1\n#endif __GUARD_NAME__");
        assert_eq!(h.items.len(), 1); // FOO
    }
}
