use super::base::ParseError;
use super::lexer::{Cursor, TextParseErrorKind, lex, lex_error_to_parse_error};

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

pub type HeaderParseError = ParseError<TextParseErrorKind>;

// ── Productions on &mut Cursor ────────────────────────────────────────────────

/// Parse a single header item (`enum`/`#define`/`namespace`/`#ifdef`/
/// `#ifndef`) at the current cursor position. Public so the def parser can
/// consume file-local declarations embedded in `.def` files on the shared
/// cursor (replaces the clone-bridge).
pub fn parse_item_on_cursor(
    cursor: &mut Cursor<'_>,
) -> Result<HeaderItem, ParseError<TextParseErrorKind>> {
    match cursor.peek().kind {
        super::lexer::TokenKind::Enum => {
            cursor.bump();
            Ok(HeaderItem::Enum(parse_enum_body(cursor)?))
        }
        super::lexer::TokenKind::Define => {
            cursor.bump();
            Ok(HeaderItem::Define(parse_define_body(cursor)?))
        }
        super::lexer::TokenKind::Namespace => {
            cursor.bump();
            Ok(HeaderItem::Namespace(parse_namespace_body(cursor)?))
        }
        super::lexer::TokenKind::Ifdef | super::lexer::TokenKind::Ifndef => {
            let inverted = cursor.peek().kind == super::lexer::TokenKind::Ifndef;
            cursor.bump();
            Ok(HeaderItem::IfDef(parse_if_def_body(cursor, inverted)?))
        }
        _ => Err(cursor.err(TextParseErrorKind::UnknownItem)),
    }
}

fn parse_enum_body(cursor: &mut Cursor<'_>) -> Result<EnumDecl, ParseError<TextParseErrorKind>> {
    let name = if cursor.at(super::lexer::TokenKind::Ident) {
        Some(cursor.expect_ident("enum name")?)
    } else {
        None
    };
    cursor.expect(super::lexer::TokenKind::LBrace)?;
    let variants = parse_enum_variants(cursor)?;
    cursor.expect(super::lexer::TokenKind::RBrace)?;
    if cursor.at(super::lexer::TokenKind::Semi) {
        cursor.bump();
    }
    Ok(EnumDecl { name, variants })
}

fn parse_enum_variants(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<EnumVariant>, ParseError<TextParseErrorKind>> {
    let mut variants = Vec::new();
    loop {
        if cursor.at(super::lexer::TokenKind::Eof) {
            return Err(cursor.err(TextParseErrorKind::UnterminatedEnum));
        }
        if cursor.at(super::lexer::TokenKind::RBrace) {
            break;
        }
        variants.push(parse_enum_variant(cursor)?);
        if cursor.at(super::lexer::TokenKind::Comma) {
            cursor.bump();
        } else {
            break;
        }
    }
    Ok(variants)
}

fn parse_enum_variant(
    cursor: &mut Cursor<'_>,
) -> Result<EnumVariant, ParseError<TextParseErrorKind>> {
    let name = cursor.expect_ident("identifier")?;
    let value = if cursor.at(super::lexer::TokenKind::Eq) {
        cursor.bump();
        Some(parse_enum_expr(cursor)?)
    } else {
        None
    };
    Ok(EnumVariant { name, value })
}

fn parse_enum_expr(cursor: &mut Cursor<'_>) -> Result<EnumExpr, ParseError<TextParseErrorKind>> {
    parse_enum_bitor(cursor)
}

fn parse_enum_bitor(
    cursor: &mut Cursor<'_>,
) -> Result<EnumExpr, ParseError<TextParseErrorKind>> {
    let first = parse_enum_shift(cursor)?;
    let mut terms = vec![first];
    while cursor.at(super::lexer::TokenKind::Pipe) {
        cursor.bump();
        terms.push(parse_enum_shift(cursor)?);
    }
    Ok(if terms.len() == 1 {
        terms.pop().unwrap()
    } else {
        EnumExpr::BitOr(terms)
    })
}

fn parse_enum_shift(
    cursor: &mut Cursor<'_>,
) -> Result<EnumExpr, ParseError<TextParseErrorKind>> {
    let first = parse_enum_leaf(cursor)?;
    let mut terms = vec![first];
    while cursor.at(super::lexer::TokenKind::Shl) {
        cursor.bump();
        terms.push(parse_enum_leaf(cursor)?);
    }
    Ok(if terms.len() == 1 {
        terms.pop().unwrap()
    } else {
        EnumExpr::Shift(terms)
    })
}

fn parse_enum_leaf(cursor: &mut Cursor<'_>) -> Result<EnumExpr, ParseError<TextParseErrorKind>> {
    use super::lexer::TokenKind;
    match cursor.peek().kind {
        TokenKind::Number => {
            let t = cursor.bump();
            let n = t
                .source
                .parse::<i64>()
                .map_err(|_| ParseError::new(t.span.start, TextParseErrorKind::InvalidNumber))?;
            Ok(EnumExpr::Int(n))
        }
        TokenKind::Ident => {
            let t = cursor.bump();
            Ok(EnumExpr::Ident(t.source.to_string()))
        }
        _ => Err(cursor.err(TextParseErrorKind::UnexpectedToken {
            expected: format!(
                "number or identifier, found {}",
                super::lexer::describe(cursor.peek().kind)
            ),
        })),
    }
}

fn parse_define_body(
    cursor: &mut Cursor<'_>,
) -> Result<Define, ParseError<TextParseErrorKind>> {
    let name = cursor.expect_ident("identifier")?;
    let t = cursor.peek();
    if t.kind != super::lexer::TokenKind::Number {
        return Err(cursor.err(TextParseErrorKind::UnexpectedToken {
            expected: format!("number, found {}", super::lexer::describe(t.kind)),
        }));
    }
    cursor.bump();
    let value = t
        .source
        .parse::<i64>()
        .map_err(|_| ParseError::new(t.span.start, TextParseErrorKind::InvalidNumber))?;
    Ok(Define { name, value })
}

fn parse_namespace_body(
    cursor: &mut Cursor<'_>,
) -> Result<Namespace, ParseError<TextParseErrorKind>> {
    let name = cursor.expect_ident("identifier")?;
    cursor.expect(super::lexer::TokenKind::LBrace)?;
    let mut items = Vec::new();
    loop {
        if cursor.at(super::lexer::TokenKind::Eof) {
            return Err(cursor.err(TextParseErrorKind::UnterminatedNamespace));
        }
        if cursor.at(super::lexer::TokenKind::RBrace) {
            break;
        }
        items.push(parse_item_on_cursor(cursor)?);
    }
    cursor.expect(super::lexer::TokenKind::RBrace)?;
    if cursor.at(super::lexer::TokenKind::Semi) {
        cursor.bump();
    }
    Ok(Namespace { name, items })
}

fn parse_if_def_body(
    cursor: &mut Cursor<'_>,
    inverted: bool,
) -> Result<IfDef, ParseError<TextParseErrorKind>> {
    let condition = cursor.expect_ident("identifier")?;
    let mut if_branch = Vec::new();
    loop {
        if cursor.at(super::lexer::TokenKind::Eof) {
            return Err(cursor.err(TextParseErrorKind::UnterminatedIfDef));
        }
        if cursor.at(super::lexer::TokenKind::Else) || cursor.at(super::lexer::TokenKind::Endif) {
            break;
        }
        if_branch.push(parse_item_on_cursor(cursor)?);
    }
    let else_branch = if cursor.at(super::lexer::TokenKind::Else) {
        cursor.bump();
        let mut else_branch = Vec::new();
        loop {
            if cursor.at(super::lexer::TokenKind::Eof) {
                return Err(cursor.err(TextParseErrorKind::UnterminatedIfDef));
            }
            if cursor.at(super::lexer::TokenKind::Endif) {
                break;
            }
            else_branch.push(parse_item_on_cursor(cursor)?);
        }
        Some(else_branch)
    } else {
        None
    };
    cursor.expect(super::lexer::TokenKind::Endif)?;
    Ok(IfDef {
        condition,
        if_branch,
        else_branch,
        inverted,
    })
}

// ── Standalone file parser (thin wrapper over Cursor) ─────────────────────────

pub struct HeaderParser<'a> {
    cursor: Cursor<'a>,
}

impl<'a> HeaderParser<'a> {
    pub fn new(input: &'a str) -> Result<Self, HeaderParseError> {
        let tokens = lex(input).map_err(|e| {
            let (pos, kind) = lex_error_to_parse_error(e);
            ParseError::new(pos, kind)
        })?;
        Ok(Self {
            cursor: Cursor::new(tokens),
        })
    }

    pub fn parse_file(&mut self) -> Result<Header, HeaderParseError> {
        let mut header = Header::default();
        skip_prologue(&mut self.cursor);
        loop {
            let tk = self.cursor.peek().kind;
            if tk == super::lexer::TokenKind::Eof
                || tk == super::lexer::TokenKind::Endif
            {
                break;
            }
            header.items.push(parse_item_on_cursor(&mut self.cursor)?);
        }
        skip_epilogue(&mut self.cursor);
        Ok(header)
    }
}

fn skip_prologue(cursor: &mut Cursor<'_>) {
    use super::lexer::TokenKind;
    if cursor.at(TokenKind::Pragma) {
        cursor.bump();
        if cursor.at(TokenKind::Ident) {
            cursor.bump(); // `once`
        }
    }
    if cursor.at(TokenKind::Ifndef) {
        cursor.bump();
        let _ = cursor.expect_ident("identifier"); // guard name
        if cursor.at(TokenKind::Define) {
            cursor.bump();
            let _ = cursor.expect_ident("identifier"); // guard name
            if cursor.at(TokenKind::Number) {
                cursor.bump(); // optional value
            }
        }
    }
}

/// The old parser's `skip_to_end_of_line` consumed any guard-name token
/// that follows `#endif` without a `//` prefix (e.g.
/// `#endif __IDLE_STATE_GROUP_DEF_H__`).
fn skip_epilogue(cursor: &mut Cursor<'_>) {
    if cursor.at(super::lexer::TokenKind::Endif) {
        cursor.bump();
    }
    while !cursor.at(super::lexer::TokenKind::Eof) {
        cursor.bump();
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
    use super::super::lexer::TextParseErrorKind;

    fn parse_h(input: &str) -> Header {
        parse_header_file(input).expect("header parse ok")
    }

    fn parse_h_err(input: &str) -> TextParseErrorKind {
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
        assert!(matches!(kind, TextParseErrorKind::UnexpectedToken { .. }));
    }

    #[test]
    fn err_unterminated_enum_eof_after_comma() {
        // EOF after a comma is a genuine UnterminatedEnum.
        let kind = parse_h_err("enum EFoo { A = 1,");
        assert!(matches!(kind, TextParseErrorKind::UnterminatedEnum));
    }

    #[test]
    fn err_unterminated_namespace() {
        let kind = parse_h_err("namespace NFoo { enum EA { X = 1 };");
        assert!(matches!(kind, TextParseErrorKind::UnterminatedNamespace));
    }

    #[test]
    fn err_unterminated_ifdef() {
        let kind = parse_h_err("#ifdef _WINDOWS\n#define FOO 1\n");
        assert!(matches!(kind, TextParseErrorKind::UnterminatedIfDef));
    }

    #[test]
    fn err_unknown_item() {
        let kind = parse_h_err("foo bar");
        assert!(matches!(kind, TextParseErrorKind::UnknownItem));
    }

    #[test]
    fn err_expected_number_in_define() {
        let kind = parse_h_err("#define FOO abc");
        assert!(matches!(kind, TextParseErrorKind::UnexpectedToken { .. }));
    }

    #[test]
    fn stray_guard_name_after_endif_consumed() {
        // `#endif __GUARD__` (no `//`) is consumed by the epilogue, matching
        // the old parser's `skip_to_end_of_line`.
        let h = parse_h("#define FOO 1\n#endif __GUARD_NAME__");
        assert_eq!(h.items.len(), 1); // FOO
    }
}
