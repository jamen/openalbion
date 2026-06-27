use super::base::{ConsumeCharError, ParseError, ParserBase, SkipTriviaError};
use derive_more::Display;

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
}

pub struct HeaderParser<'a> {
    parser: ParserBase<'a>,
}

pub type HeaderParseError = ParseError<HeaderParseErrorKind>;

#[derive(Debug, Display)]
pub enum HeaderParseErrorKind {
    #[display("unexpected end of input")]
    UnexpectedEnd,
    #[display("unterminated namespace")]
    UnterminatedNamespace,
    #[display("unterminated #ifdef")]
    UnterminatedIfDef,
    #[display("unterminated enum")]
    UnterminatedEnum,
    #[display("left over bytes")]
    LeftOverBytes,
    #[display("consume char error: {_0}")]
    ConsumeChar(ConsumeCharError),
    #[display("skip trivia: {_0}")]
    SkipTrivia(SkipTriviaError),
    #[display("unknown item")]
    UnknownItem,
    #[display("invalid number")]
    InvalidNumber,
    #[display("unexpected character: {found}")]
    UnexpectedCharacter { found: char },
}

impl<'a> HeaderParser<'a> {
    pub fn parse_file(&mut self) -> Result<Header, HeaderParseError> {
        let mut header = Header::default();
        self.skip_prologue()?;
        loop {
            self.skip_trivia()?;
            if self.is_eof() || self.at_pp_directive("#endif") {
                break;
            }
            header.items.push(self.parse_item()?);
        }
        self.skip_epilogue()?;
        Ok(header)
    }

    fn parse_item(&mut self) -> Result<HeaderItem, HeaderParseError> {
        if self.try_consume_keyword("enum") {
            Ok(HeaderItem::Enum(self.parse_enum_body()?))
        } else if self.try_consume_pp_directive("#define") {
            Ok(HeaderItem::Define(self.parse_define_body()?))
        } else if self.try_consume_keyword("namespace") {
            Ok(HeaderItem::Namespace(self.parse_namespace_body()?))
        } else if self.try_consume_pp_directive("#ifdef") {
            Ok(HeaderItem::IfDef(self.parse_if_def_body()?))
        } else {
            Err(self.err(HeaderParseErrorKind::UnknownItem))
        }
    }

    fn at_keyword(&self, name: &str) -> bool {
        let Some(after) = self.rest().strip_prefix(name) else {
            return false;
        };
        after.chars().next().is_none_or(|c| c.is_whitespace() || c == '{')
    }

    fn at_pp_directive(&self, name: &str) -> bool {
        let Some(after) = self.rest().strip_prefix(name) else {
            return false;
        };
        after.chars().next().is_none_or(|c| c.is_whitespace())
    }

    fn try_consume_keyword(&mut self, name: &str) -> bool {
        if !self.at_keyword(name) { return false; }
        self.advance(name.len());
        true
    }

    fn try_consume_pp_directive(&mut self, name: &str) -> bool {
        if !self.at_pp_directive(name) { return false; }
        self.advance(name.len());
        true
    }

    fn skip_prologue(&mut self) -> Result<(), HeaderParseError> {
        self.skip_trivia()?;
        if self.try_consume_pp_directive("#pragma") {
            self.skip_to_end_of_line();
            self.skip_trivia()?;
        }
        if self.try_consume_pp_directive("#ifndef") {
            self.skip_to_end_of_line();
            self.skip_trivia()?;
            if self.try_consume_pp_directive("#define") {
                self.skip_to_end_of_line();
            }
        }
        Ok(())
    }

    fn skip_epilogue(&mut self) -> Result<(), HeaderParseError> {
        self.skip_trivia()?;
        if self.try_consume_pp_directive("#endif") {
            self.skip_to_end_of_line();
            self.skip_trivia()?;
        }
        if !self.is_eof() {
            return Err(self.err(HeaderParseErrorKind::LeftOverBytes));
        }
        Ok(())
    }

    fn parse_namespace_body(&mut self) -> Result<Namespace, HeaderParseError> {
        self.skip_trivia()?;
        let name = self.parse_identifier()?;
        self.skip_trivia()?;
        self.consume_char('{')?;
        let mut items = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.is_eof() { return Err(self.err(HeaderParseErrorKind::UnterminatedNamespace)); }
            if self.peek_char() == Some('}') { break; }
            items.push(self.parse_item()?);
        }
        self.consume_char('}')?;
        self.skip_trivia()?;
        let _ = self.try_consume(";");
        Ok(Namespace { name, items })
    }

    fn parse_if_def_body(&mut self) -> Result<IfDef, HeaderParseError> {
        self.skip_horizontal_ws();
        let condition = self.parse_identifier()?;
        self.skip_to_end_of_line();
        let mut if_branch = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.is_eof() { return Err(self.err(HeaderParseErrorKind::UnterminatedIfDef)); }
            if self.at_pp_directive("#else") || self.at_pp_directive("#endif") { break; }
            if_branch.push(self.parse_item()?);
        }
        let else_branch = if self.at_pp_directive("#else") {
            self.skip_to_end_of_line();
            let mut else_branch = Vec::new();
            loop {
                self.skip_trivia()?;
                if self.is_eof() { return Err(self.err(HeaderParseErrorKind::UnterminatedIfDef)); }
                if self.at_pp_directive("#endif") { break; }
                else_branch.push(self.parse_item()?);
            }
            Some(else_branch)
        } else { None };
        if !self.try_consume_pp_directive("#endif") {
            return Err(self.err(HeaderParseErrorKind::UnterminatedIfDef));
        }
        self.skip_to_end_of_line();
        Ok(IfDef { condition, if_branch, else_branch })
    }

    fn parse_define_body(&mut self) -> Result<Define, HeaderParseError> {
        self.skip_horizontal_ws();
        let name = self.parse_identifier()?;
        self.skip_horizontal_ws();
        let value = self.parse_integer()?;
        Ok(Define { name, value })
    }

    fn skip_horizontal_ws(&mut self) {
        while let Some(c) = self.peek_char() {
            if c == ' ' || c == '\t' { self.advance(c.len_utf8()); } else { break; }
        }
    }

    fn skip_to_end_of_line(&mut self) {
        while let Some(c) = self.peek_char() {
            self.advance(c.len_utf8());
            if c == '\n' { break; }
        }
    }

    fn parse_enum_body(&mut self) -> Result<EnumDecl, HeaderParseError> {
        self.skip_trivia()?;
        let name = if matches!(self.peek_char(), Some(c) if c.is_ascii_alphabetic() || c == '_') {
            Some(self.parse_identifier()?)
        } else { None };
        self.skip_trivia()?;
        self.consume_char('{')?;
        let variants = self.parse_enum_variants()?;
        self.consume_char('}')?;
        self.skip_trivia()?;
        let _ = self.try_consume(";");
        Ok(EnumDecl { name, variants })
    }

    fn parse_enum_variants(&mut self) -> Result<Vec<EnumVariant>, HeaderParseError> {
        let mut variants = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.is_eof() { return Err(self.err(HeaderParseErrorKind::UnterminatedEnum)); }
            if self.peek_char() == Some('}') { break; }
            variants.push(self.parse_enum_variant()?);
            self.skip_trivia()?;
            if !self.try_consume(",") { break; }
        }
        Ok(variants)
    }

    fn parse_enum_variant(&mut self) -> Result<EnumVariant, HeaderParseError> {
        let name = self.parse_identifier()?;
        self.skip_trivia()?;
        let value = if self.try_consume("=") { Some(self.parse_enum_expr()?) } else { None };
        Ok(EnumVariant { name, value })
    }

    fn parse_enum_expr(&mut self) -> Result<EnumExpr, HeaderParseError> {
        self.parse_bitor_expr()
    }

    fn parse_bitor_expr(&mut self) -> Result<EnumExpr, HeaderParseError> {
        let first = self.parse_shift_expr()?;
        let mut terms = vec![first];
        while self.try_consume("|") { terms.push(self.parse_shift_expr()?); }
        Ok(if terms.len() == 1 { terms.pop().unwrap() } else { EnumExpr::BitOr(terms) })
    }

    fn parse_shift_expr(&mut self) -> Result<EnumExpr, HeaderParseError> {
        let first = self.parse_leaf_expr()?;
        let mut terms = vec![first];
        self.skip_trivia()?;
        while self.try_consume("<<") {
            terms.push(self.parse_leaf_expr()?);
            self.skip_trivia()?;
        }
        Ok(if terms.len() == 1 { terms.pop().unwrap() } else { EnumExpr::Shift(terms) })
    }

    fn parse_leaf_expr(&mut self) -> Result<EnumExpr, HeaderParseError> {
        self.skip_trivia()?;
        let p = self.peek_char();
        let is_number = p.is_some_and(|c| c.is_ascii_digit())
            || (p == Some('-') && self.peek_char_at(1).is_some_and(|c| c.is_ascii_digit()));
        if is_number { Ok(EnumExpr::Int(self.parse_integer()?)) }
        else { Ok(EnumExpr::Ident(self.parse_identifier()?)) }
    }

    fn parse_integer(&mut self) -> Result<i64, HeaderParseError> {
        let start = self.pos();
        if self.peek_char() == Some('-') { self.advance(1); }
        let digits_start = self.pos();
        while self.peek_char().is_some_and(|c| c.is_ascii_digit()) { self.advance(1); }
        if self.pos() == digits_start {
            self.seek_to(start);
            return Err(self.err(HeaderParseErrorKind::InvalidNumber));
        }
        let text = &self.input()[start..self.pos()];
        text.parse::<i64>().map_err(|_| self.err(HeaderParseErrorKind::InvalidNumber))
    }

    fn parse_identifier(&mut self) -> Result<String, HeaderParseError> {
        let start = self.pos();
        match self.peek_char() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => self.advance(c.len_utf8()),
            Some(found) => return Err(self.err(HeaderParseErrorKind::UnexpectedCharacter { found })),
            None => return Err(self.err(HeaderParseErrorKind::UnexpectedEnd)),
        }
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' { self.advance(c.len_utf8()); } else { break; }
        }
        Ok(self.input()[start..self.pos()].to_string())
    }
}

impl<'a> HeaderParser<'a> {
    pub fn new(input: &'a str) -> Self { Self { parser: ParserBase::new(input) } }
    fn input(&self) -> &str { self.parser.input() }
    fn pos(&self) -> usize { self.parser.pos() }
    fn advance(&mut self, add: usize) { self.parser.advance(add) }
    fn seek_to(&mut self, pos: usize) { self.parser.seek_to(pos) }
    fn rest(&self) -> &str { self.parser.rest() }
    fn is_eof(&self) -> bool { self.parser.is_eof() }
    fn peek_char(&self) -> Option<char> { self.parser.peek_char() }
    fn peek_char_at(&self, offset: usize) -> Option<char> { self.parser.peek_char_at(offset) }
    fn consume_char(&mut self, expected: char) -> Result<(), HeaderParseError> {
        self.parser.consume_char(expected)
            .map_err(|e| HeaderParseError::new(e.pos, HeaderParseErrorKind::ConsumeChar(e.inner)))
    }
    fn skip_trivia(&mut self) -> Result<(), HeaderParseError> {
        self.parser.skip_trivia()
            .map_err(|e| HeaderParseError::new(e.pos, HeaderParseErrorKind::SkipTrivia(e.inner)))
    }
    fn try_consume(&mut self, s: &str) -> bool { self.parser.try_consume(s) }
    fn err(&self, inner: HeaderParseErrorKind) -> ParseError<HeaderParseErrorKind> {
        ParseError::new(self.pos(), inner)
    }
}

pub fn parse_header_file(input: &str) -> Result<Header, HeaderParseError> {
    let mut parser = HeaderParser::new(input);
    parser.parse_file()
}
