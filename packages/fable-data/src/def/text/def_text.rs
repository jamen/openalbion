use super::base::{ParseError, Span, Spanned};
use super::header::{self, HeaderItem};
use super::lexer::{
    Cursor, TextParseErrorKind, TokenKind, describe, lex,
    lex_error_to_parse_error,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct DefFile {
    pub definitions: Vec<Spanned<Definition>>,
    pub by_name: HashMap<String, usize>,
    /// File-local `enum`/`#define` declarations embedded in the `.def` (some
    /// files, e.g. `engine_local_detail.def`, declare symbol constants at the
    /// top). Evaluate these into the [`SymbolTable`](super::SymbolTable) so the
    /// definitions in this file can reference them.
    pub headers: Vec<HeaderItem>,
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub is_template: bool,
    pub def_type: String,
    pub name: String,
    pub specializes: Option<String>,
    pub specializes_span: Option<Span>,
    pub body: Vec<Spanned<Statement>>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Field(Field),
    MethodCall(MethodCall),
    TaggedBlock(TaggedBlock),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub path: PropertyPath,
    pub expr: Spanned<Expr>,
}

#[derive(Debug, Clone)]
pub struct MethodCall {
    pub object: PropertyPath,
    pub call: Call,
}

#[derive(Debug, Clone)]
pub struct TaggedBlock {
    pub tag: String,
    pub body: Vec<Spanned<Statement>>,
}

#[derive(Debug, Clone)]
pub struct PropertyPath {
    pub segments: Vec<PathSegment>,
}

impl PropertyPath {
    pub fn simple(name: impl Into<String>) -> Self {
        Self {
            segments: vec![PathSegment::Field(name.into())],
        }
    }
}

impl std::fmt::Display for PropertyPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, seg) in self.segments.iter().enumerate() {
            match seg {
                PathSegment::Field(name) => {
                    if i > 0 {
                        f.write_str(".")?;
                    }
                    f.write_str(name)?;
                }
                PathSegment::Index(expr) => write!(f, "[{}]", expr.value)?,
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Number(s) => f.write_str(s),
            Expr::Bool(b) => f.write_str(if *b { "TRUE" } else { "FALSE" }),
            Expr::String(s) => write!(f, "\"{s}\""),
            Expr::Symbol(s) => f.write_str(s),
            Expr::Constructor(c) => {
                write!(f, "{}(", c.name)?;
                fmt_separated(f, &c.arguments, ", ")?;
                f.write_str(")")
            }
            Expr::BitOr(terms) => fmt_separated(f, terms, " | "),
            Expr::Add(terms) => fmt_separated(f, terms, " + "),
        }
    }
}

fn fmt_separated(
    f: &mut std::fmt::Formatter<'_>,
    terms: &[Spanned<Expr>],
    sep: &str,
) -> std::fmt::Result {
    for (i, term) in terms.iter().enumerate() {
        if i > 0 {
            f.write_str(sep)?;
        }
        write!(f, "{}", term.value)?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    Field(String),
    Index(Spanned<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric literal, kept as its raw source slice (optional leading `-`,
    /// digits, optional `.frac`, optional trailing `f`). Interpretation — int
    /// vs float, truncation, range — is deferred to evaluation (§11.2), where it
    /// is type-specific per field. Use [`number_is_float`] to classify the shape.
    Number(String),
    Bool(bool),
    String(String),
    Symbol(String),
    Constructor(Call),
    BitOr(Vec<Spanned<Expr>>),
    Add(Vec<Spanned<Expr>>),
}

/// Whether a [`Expr::Number`] literal is float-shaped — it contains a `.` or a
/// trailing `f` — mirroring the old parser's int-vs-float split (`has_dot ||
/// has_f_suffix`). Integer-shaped literals parse as integers; float-shaped ones
/// parse as `f32` (after stripping the `f`).
pub fn number_is_float(s: &str) -> bool {
    s.contains('.') || s.ends_with('f')
}

impl Expr {
    /// Interpret a numeric literal as `i32` the way the pre-token parser's
    /// `Expr::Integer(i64)` arm did — integer-shaped only, truncating `i64` →
    /// `i32`. Float-shaped literals and non-numbers yield `None`. (Used by the
    /// sky-keyframe reader for `Time[idx]` indices, which only accepted integer
    /// literals.)
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Expr::Number(s) if !number_is_float(s) => s.parse::<i64>().ok().map(|n| n as i32),
            _ => None,
        }
    }

    /// Interpret a numeric literal as `f32`, matching the pre-token parser's
    /// keyframe-property arms (`Float(f) => f`, `Integer(i) => i as f32`).
    /// Non-numbers yield `None`.
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Expr::Number(s) if number_is_float(s) => s.trim_end_matches('f').parse::<f32>().ok(),
            Expr::Number(s) => s.parse::<i64>().ok().map(|n| n as f32),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub name: String,
    pub arguments: Vec<Spanned<Expr>>,
}

pub type DefParseError = ParseError<TextParseErrorKind>;

/// Whether `kind` can never appear inside a def body — a directive or header
/// keyword, or EOF. Hitting one before `#end_definition` is the precise "missing
/// `#end_definition`" error (§11.5). `EndDefinition` is *not* here: the body loop
/// matches it first as the valid closer.
fn is_body_terminator(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Definition
            | TokenKind::DefinitionTemplate
            | TokenKind::Define
            | TokenKind::Ifdef
            | TokenKind::Ifndef
            | TokenKind::Else
            | TokenKind::Endif
            | TokenKind::Pragma
            | TokenKind::Namespace
            | TokenKind::Enum
            | TokenKind::Eof
    )
}

// ── Public entry points ───────────────────────────────────────────────────────

pub fn parse_def_file(input: &str) -> Result<DefFile, DefParseError> {
    let tokens = lex(input).map_err(|e| {
        let (pos, kind) = lex_error_to_parse_error(e);
        ParseError::new(pos, kind)
    })?;
    let mut cursor = Cursor::new(tokens);
    parse_file(&mut cursor)
}

/// Parse a single expression from `input`. Used by tests that need to evaluate
/// expressions without going through a full definition.
pub fn parse_expr_str(input: &str) -> Result<Spanned<Expr>, DefParseError> {
    let tokens = lex(input).map_err(|e| {
        let (pos, kind) = lex_error_to_parse_error(e);
        ParseError::new(pos, kind)
    })?;
    let mut cursor = Cursor::new(tokens);
    parse_expr(&mut cursor)
}

// ── Productions on &mut Cursor ────────────────────────────────────────────────

fn parse_file(cursor: &mut Cursor<'_>) -> Result<DefFile, ParseError<TextParseErrorKind>> {
    let mut file = DefFile::default();
    loop {
        match cursor.peek().kind {
            TokenKind::Eof => break,
            TokenKind::Definition | TokenKind::DefinitionTemplate => {
                let def = parse_definition(cursor)?;
                let name_index = file.definitions.len();
                let def_name = def.value.name.clone();
                file.definitions.push(def);
                file.by_name.insert(def_name, name_index);
            }
            // File-local `enum`/`#define` declarations at `.def` top level.
            // Parsed directly on the shared cursor — no clone-bridge.
            TokenKind::Enum | TokenKind::Define => {
                file.headers
                    .push(header::parse_item_on_cursor(cursor).map_err(|e| {
                        let pos = e.pos;
                        ParseError::new(pos, TextParseErrorKind::UnexpectedToken {
                            expected: format!("enum or #define declaration: {}", e.inner),
                        })
                    })?);
            }
            // Stray tokens between top-level items are skipped, as the
            // pre-token parser did (its `skip_to_next_top_level_item` walked
            // over anything up to the next `#definition`/`enum`/`#define`).
            // This is not body-recovery — the strict body loop still errors
            // on a missing `#end_definition`.
            _ => {
                cursor.bump();
            }
        }
    }
    Ok(file)
}

fn parse_definition(
    cursor: &mut Cursor<'_>,
) -> Result<Spanned<Definition>, ParseError<TextParseErrorKind>> {
    let header_tok = cursor.peek();
    let def_start = header_tok.span.start;
    let is_template = match header_tok.kind {
        TokenKind::DefinitionTemplate => {
            cursor.bump();
            true
        }
        TokenKind::Definition => {
            cursor.bump();
            false
        }
        _ => {
            return Err(cursor.err_at(
                header_tok.span.start,
                TextParseErrorKind::UnexpectedToken {
                    expected: "#definition or #definition_template".into(),
                },
            ));
        }
    };

    let def_type = cursor.expect_ident("definition type")?;
    let name = cursor.expect_ident("definition name")?;

    let specializes = if cursor.at_ident("specialises") {
        let spec_kw = cursor.bump();
        let parent_span_start = cursor.peek().span.start;
        let parent = cursor.expect_ident("specialised parent")?;
        let spec_span = Span {
            start: spec_kw.span.start,
            end: parent_span_start + parent.len(),
        };
        Some((parent, spec_span))
    } else {
        None
    };

    let mut body = Vec::new();
    let def_end = loop {
        let tk = cursor.peek().kind;
        if tk == TokenKind::EndDefinition {
            let mut end = cursor.bump().span.end;
            if cursor.at(TokenKind::Semi) {
                end = cursor.bump().span.end;
            }
            break end;
        }
        if is_body_terminator(tk) {
            let err = cursor
                .err(TextParseErrorKind::UnexpectedToken {
                    expected: "#end_definition".into(),
                })
                .with_def_header(def_start);
            return Err(err);
        }
        body.push(
            parse_statement(cursor).map_err(|e| e.with_def_header(def_start))?,
        );
    };

    let (specializes, specializes_span) = match specializes {
        Some((name, span)) => (Some(name), Some(span)),
        None => (None, None),
    };

    Ok(Spanned {
        span: Span {
            start: def_start,
            end: def_end,
        },
        value: Definition {
            is_template,
            def_type,
            name,
            specializes,
            specializes_span,
            body,
        },
    })
}

fn parse_statement(
    cursor: &mut Cursor<'_>,
) -> Result<Spanned<Statement>, ParseError<TextParseErrorKind>> {
    let stmt_start = cursor.peek().span.start;

    // Tagged block: `<` not followed by `\` (a `<\` opens a *close* tag).
    if cursor.at(TokenKind::Lt) && cursor.peek_at(1).kind != TokenKind::Backslash {
        let tb = parse_tagged_block(cursor)?;
        return Ok(Spanned {
            span: Span {
                start: stmt_start,
                end: cursor.prev_end(),
            },
            value: Statement::TaggedBlock(tb),
        });
    }

    let path = parse_property_path(cursor)?;

    // Method call: the path is followed by an argument list.
    if cursor.at(TokenKind::LParen) {
        let (object, method) = split_method_path(cursor, path)?;
        let call = parse_call_with_name(cursor, method)?;
        if cursor.at(TokenKind::Semi) {
            cursor.bump();
        }
        return Ok(Spanned {
            span: Span {
                start: stmt_start,
                end: cursor.prev_end(),
            },
            value: Statement::MethodCall(MethodCall { object, call }),
        });
    }

    // Field assignment: `path expr`.
    let expr = parse_expr(cursor)?;
    if cursor.at(TokenKind::Semi) {
        cursor.bump();
    }
    Ok(Spanned {
        span: Span {
            start: stmt_start,
            end: cursor.prev_end(),
        },
        value: Statement::Field(Field { path, expr }),
    })
}

fn parse_tagged_block(
    cursor: &mut Cursor<'_>,
) -> Result<TaggedBlock, ParseError<TextParseErrorKind>> {
    cursor.expect(TokenKind::Lt)?;
    let tag = cursor.expect_ident("tag name")?;
    cursor.expect(TokenKind::Gt)?;
    let mut body = Vec::new();
    loop {
        let tk = cursor.peek().kind;
        if tk == TokenKind::Lt && cursor.peek_at(1).kind == TokenKind::Backslash {
            cursor.bump(); // `<`
            cursor.bump(); // `\`
            let close_tag = cursor.expect_ident("closing tag name")?;
            cursor.expect(TokenKind::Gt)?;
            if close_tag != tag {
                return Err(ParseError::new(
                    cursor.prev_end(),
                    TextParseErrorKind::MismatchedTag {
                        opened: tag,
                        closed: close_tag,
                    },
                ));
            }
            break;
        }
        // A directive/keyword, EOF, or `#end_definition` inside the block
        // means it was never closed (§11.5, strict).
        if is_body_terminator(tk) || tk == TokenKind::EndDefinition {
            return Err(cursor.err(TextParseErrorKind::UnexpectedToken {
                expected: format!("<\\{tag}>"),
            }));
        }
        body.push(parse_statement(cursor)?);
    }
    Ok(TaggedBlock { tag, body })
}

fn parse_property_path(
    cursor: &mut Cursor<'_>,
) -> Result<PropertyPath, ParseError<TextParseErrorKind>> {
    let mut segments = vec![PathSegment::Field(cursor.expect_ident("field name")?)];
    loop {
        if cursor.at(TokenKind::Dot) {
            cursor.bump();
            segments.push(PathSegment::Field(cursor.expect_ident("field name")?));
        } else if cursor.at(TokenKind::LBracket) {
            cursor.bump();
            let idx = parse_expr(cursor)?;
            cursor.expect(TokenKind::RBracket)?;
            segments.push(PathSegment::Index(idx));
        } else {
            break;
        }
    }
    Ok(PropertyPath { segments })
}

fn split_method_path(
    cursor: &Cursor<'_>,
    path: PropertyPath,
) -> Result<(PropertyPath, String), ParseError<TextParseErrorKind>> {
    let mut segments = path.segments;
    if let Some(PathSegment::Field(method)) = segments.pop() {
        Ok((PropertyPath { segments }, method))
    } else {
        Err(cursor.err(TextParseErrorKind::UnexpectedToken {
            expected: "method name".into(),
        }))
    }
}

pub fn parse_expr(
    cursor: &mut Cursor<'_>,
) -> Result<Spanned<Expr>, ParseError<TextParseErrorKind>> {
    parse_bitor_expr(cursor)
}

fn parse_bitor_expr(
    cursor: &mut Cursor<'_>,
) -> Result<Spanned<Expr>, ParseError<TextParseErrorKind>> {
    let start = cursor.peek().span.start;
    let first = parse_add_expr(cursor)?;
    let mut terms = vec![first];
    while cursor.at(TokenKind::Pipe) {
        cursor.bump();
        terms.push(parse_add_expr(cursor)?);
    }
    if terms.len() == 1 {
        Ok(terms.pop().unwrap())
    } else {
        Ok(Spanned {
            span: Span {
                start,
                end: cursor.prev_end(),
            },
            value: Expr::BitOr(terms),
        })
    }
}

fn parse_add_expr(
    cursor: &mut Cursor<'_>,
) -> Result<Spanned<Expr>, ParseError<TextParseErrorKind>> {
    let start = cursor.peek().span.start;
    let first = parse_leaf_expr(cursor)?;
    let mut terms = vec![first];
    while cursor.at(TokenKind::Plus) {
        cursor.bump();
        terms.push(parse_leaf_expr(cursor)?);
    }
    if terms.len() == 1 {
        Ok(terms.pop().unwrap())
    } else {
        Ok(Spanned {
            span: Span {
                start,
                end: cursor.prev_end(),
            },
            value: Expr::Add(terms),
        })
    }
}

fn parse_leaf_expr(
    cursor: &mut Cursor<'_>,
) -> Result<Spanned<Expr>, ParseError<TextParseErrorKind>> {
    let tok = cursor.peek();
    match tok.kind {
        TokenKind::Str => {
            cursor.bump();
            let unquoted = tok.source[1..tok.source.len() - 1].to_string();
            Ok(Spanned {
                span: tok.span,
                value: Expr::String(unquoted),
            })
        }
        TokenKind::Number => {
            cursor.bump();
            Ok(Spanned {
                span: tok.span,
                value: Expr::Number(tok.source.to_string()),
            })
        }
        TokenKind::Ident => {
            cursor.bump();
            match tok.source {
                "TRUE" | "BTRUE" => Ok(Spanned {
                    span: tok.span,
                    value: Expr::Bool(true),
                }),
                "FALSE" | "BFALSE" => Ok(Spanned {
                    span: tok.span,
                    value: Expr::Bool(false),
                }),
                ident => {
                    if cursor.at(TokenKind::LParen) {
                        let call = parse_call_with_name(cursor, ident.to_string())?;
                        Ok(Spanned {
                            span: Span {
                                start: tok.span.start,
                                end: cursor.prev_end(),
                            },
                            value: Expr::Constructor(call),
                        })
                    } else {
                        Ok(Spanned {
                            span: tok.span,
                            value: Expr::Symbol(ident.to_string()),
                        })
                    }
                }
            }
        }
        _ => Err(cursor.err(TextParseErrorKind::UnexpectedToken {
            expected: format!("expression, found {}", describe(tok.kind)),
        })),
    }
}

fn parse_call_with_name(
    cursor: &mut Cursor<'_>,
    name: String,
) -> Result<Call, ParseError<TextParseErrorKind>> {
    cursor.expect(TokenKind::LParen)?;
    let arguments = parse_arguments(cursor)?;
    cursor.expect(TokenKind::RParen)?;
    Ok(Call { name, arguments })
}

fn parse_arguments(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<Spanned<Expr>>, ParseError<TextParseErrorKind>> {
    let mut args = Vec::new();
    if cursor.at(TokenKind::RParen) {
        return Ok(args);
    }
    loop {
        args.push(parse_expr(cursor)?);
        if cursor.at(TokenKind::Comma) {
            cursor.bump();
        } else {
            break;
        }
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::lexer::TextParseErrorKind;

    fn parse_def(body: &str) -> Spanned<Definition> {
        let input = format!("#definition OBJECT T\n{body}\n#end_definition");
        parse_def_file(&input).unwrap().definitions.pop().unwrap()
    }

    fn parse_first_def(input: &str) -> Spanned<Definition> {
        parse_def_file(input).unwrap().definitions.pop().unwrap()
    }

    fn parse_err(input: &str) -> TextParseErrorKind {
        parse_def_file(input).unwrap_err().inner
    }

    fn parse_stmt(stmt: &str) -> Spanned<Statement> {
        parse_def(stmt).value.body.pop().unwrap()
    }

    fn parse_expr_test(value: &str) -> Spanned<Expr> {
        match &parse_stmt(&format!("X {value};")).value {
            Statement::Field(f) => f.expr.clone(),
            other => panic!("expected Field, got {other:?}"),
        }
    }

    fn parse_path(path: &str) -> PropertyPath {
        let Spanned {
            value: Statement::Field(f),
            ..
        } = parse_stmt(&format!("{path} 0;"))
        else {
            panic!()
        };
        f.path
    }

    fn number(value: &str) -> String {
        match parse_expr_test(value).value {
            Expr::Number(s) => s,
            other => panic!("expected Number, got {other:?}"),
        }
    }

    // --- numbers stay raw (interpretation is deferred to evaluation) -----------

    #[test]
    fn integer() {
        assert_eq!(number("42"), "42");
        assert_eq!(number("42282949"), "42282949");
    }

    #[test]
    fn negative_integer() {
        assert_eq!(number("-42"), "-42");
        assert_eq!(number("-42282949"), "-42282949");
    }

    #[test]
    fn float_keeps_source() {
        // Every float form is preserved verbatim — no `Float` node any more.
        assert_eq!(number("4.2"), "4.2");
        assert_eq!(number("4.2f"), "4.2f");
        assert_eq!(number("4."), "4.");
        assert_eq!(number("-4.2"), "-4.2");
        assert_eq!(number("-4.2f"), "-4.2f");
        assert_eq!(number("-4."), "-4.");
    }

    #[test]
    fn number_shape_classification() {
        assert!(!number_is_float("42"));
        assert!(!number_is_float("-42"));
        assert!(number_is_float("4.2"));
        assert!(number_is_float("4."));
        assert!(number_is_float("4.2f"));
    }

    #[test]
    fn string() {
        let Expr::String(s) = parse_expr_test(r#""Hello, World!""#).value else {
            panic!()
        };
        assert_eq!(s, "Hello, World!");
    }

    #[test]
    fn bool_test() {
        assert!(matches!(parse_expr_test("TRUE").value, Expr::Bool(true)));
        assert!(matches!(parse_expr_test("FALSE").value, Expr::Bool(false)));
    }

    #[test]
    fn bool_b_prefix() {
        assert!(matches!(parse_expr_test("BTRUE").value, Expr::Bool(true)));
        assert!(matches!(parse_expr_test("BFALSE").value, Expr::Bool(false)));
    }

    #[test]
    fn add_n_ary() {
        let Expr::Add(terms) = &parse_expr_test("1 + 2 + 3").value else {
            panic!()
        };
        assert_eq!(terms.len(), 3);
    }

    #[test]
    fn bitor_n_ary() {
        let Expr::BitOr(terms) = &parse_expr_test("A | B | C").value else {
            panic!()
        };
        assert_eq!(terms.len(), 3);
    }

    #[test]
    fn bitor_precedence_lower_than_add() {
        let Expr::BitOr(terms) = &parse_expr_test("A | B + C").value else {
            panic!()
        };
        assert_eq!(terms.len(), 2);
        assert!(matches!(&terms[0].value, Expr::Symbol(s) if s == "A"));
        let Expr::Add(add_terms) = &terms[1].value else {
            panic!()
        };
        assert_eq!(add_terms.len(), 2);
    }

    #[test]
    fn constructor_with_args() {
        let Expr::Constructor(c) = &parse_expr_test("CRGBColour(255, 128, 64, 255)").value else {
            panic!()
        };
        assert_eq!(c.name, "CRGBColour");
        assert_eq!(c.arguments.len(), 4);
    }

    #[test]
    fn empty_constructor() {
        let Expr::Constructor(c) = &parse_expr_test("CRGBColour()").value else {
            panic!()
        };
        assert!(c.arguments.is_empty());
    }

    #[test]
    fn identifier() {
        let Expr::Symbol(s) = parse_expr_test("GRAPHIC_NULL").value else {
            panic!()
        };
        assert_eq!(s, "GRAPHIC_NULL");
    }

    #[test]
    fn simple_path() {
        let p = parse_path("Health");
        assert_eq!(p.segments.len(), 1);
        assert!(matches!(&p.segments[0], PathSegment::Field(s) if s == "Health"));
    }

    #[test]
    fn nested_path() {
        let p = parse_path("Stats.ExperienceWorth");
        assert_eq!(p.segments.len(), 2);
    }

    #[test]
    fn integer_index() {
        let p = parse_path("Time[0]");
        assert!(matches!(
            &p.segments[1],
            PathSegment::Index(spanned) if matches!(&spanned.value, Expr::Number(s) if s == "0")
        ));
    }

    #[test]
    fn negative_index() {
        let p = parse_path("Time[-1]");
        assert!(matches!(
            &p.segments[1],
            PathSegment::Index(spanned) if matches!(&spanned.value, Expr::Number(s) if s == "-1")
        ));
    }

    #[test]
    fn ident_index() {
        let p = parse_path("Foo[BAR_CONST]");
        let PathSegment::Index(spanned) = &p.segments[1] else {
            panic!()
        };
        let Expr::Symbol(s) = &spanned.value else {
            panic!()
        };
        assert_eq!(s, "BAR_CONST");
    }

    #[test]
    fn string_index() {
        let p = parse_path("Map[\"DAY\"]");
        let PathSegment::Index(spanned) = &p.segments[1] else {
            panic!()
        };
        let Expr::String(s) = &spanned.value else {
            panic!()
        };
        assert_eq!(s, "DAY");
    }

    #[test]
    fn expression_index() {
        let p = parse_path("States[STATE + 1]");
        assert!(matches!(
            &p.segments[1],
            PathSegment::Index(spanned) if matches!(spanned.value, Expr::Add(_))
        ));
    }

    #[test]
    fn nested_field_and_index() {
        let p = parse_path("Time[0].SkyTexture0");
        assert_eq!(p.segments.len(), 3);
        assert!(matches!(&p.segments[0], PathSegment::Field(s) if s == "Time"));
        assert!(matches!(
            &p.segments[1],
            PathSegment::Index(spanned) if matches!(&spanned.value, Expr::Number(s) if s == "0")
        ));
        assert!(matches!(&p.segments[2], PathSegment::Field(s) if s == "SkyTexture0"));
    }

    #[test]
    fn field_assignment() {
        let Spanned {
            value: Statement::Field(f),
            ..
        } = parse_stmt("Health 100;")
        else {
            panic!()
        };
        assert_eq!(f.path.segments.len(), 1);
        assert!(matches!(&f.expr.value, Expr::Number(s) if s == "100"));
    }

    #[test]
    fn method_call() {
        let Spanned {
            value: Statement::MethodCall(mc),
            ..
        } = parse_stmt("Components.Add(\"CTCPhysicsStandard\");")
        else {
            panic!()
        };
        assert_eq!(mc.call.name, "Add");
        assert_eq!(mc.call.arguments.len(), 1);
    }

    #[test]
    fn tagged_block() {
        let Spanned {
            value: Statement::TaggedBlock(tb),
            ..
        } = parse_stmt("<CCreatureDef>\n  Health 100;\n<\\CCreatureDef>")
        else {
            panic!()
        };
        assert_eq!(tb.tag, "CCreatureDef");
        assert_eq!(tb.body.len(), 1);
    }

    #[test]
    fn template_flag() {
        let def = parse_first_def("#definition_template OBJECT T\n#end_definition");
        assert!(def.value.is_template);
    }

    #[test]
    fn specialises() {
        let def = parse_first_def(
            "#definition OBJECT CHILD specialises PARENT\n  Health 50;\n#end_definition",
        );
        assert_eq!(def.value.specializes.as_deref(), Some("PARENT"));
    }

    #[test]
    fn end_definition_trailing_semicolon() {
        let file = parse_def_file("#definition OBJECT T\n  Health 100;\n#end_definition;").unwrap();
        assert_eq!(file.definitions.len(), 1);
    }

    #[test]
    fn multiple_definitions_preserve_order() {
        let file = parse_def_file(
            r#"
    #definition OBJECT FIRST
    #end_definition

    #definition OBJECT SECOND
    #end_definition
    "#,
        )
        .unwrap();
        assert_eq!(file.definitions.len(), 2);
        assert_eq!(file.definitions[0].value.name, "FIRST");
        assert_eq!(file.definitions[1].value.name, "SECOND");
        assert_eq!(file.by_name["FIRST"], 0);
        assert_eq!(file.by_name["SECOND"], 1);
    }

    #[test]
    fn line_comment_in_body() {
        let def = parse_def("// just a comment\nHealth 100;");
        assert_eq!(def.value.body.len(), 1);
    }

    #[test]
    fn block_comment_in_body() {
        let def = parse_def("/* block comment */\nHealth 100;");
        assert_eq!(def.value.body.len(), 1);
    }

    #[test]
    fn block_comment_inline() {
        let def = parse_def("Name /* inline */ \"Test\";");
        assert_eq!(def.value.body.len(), 1);
        let Spanned {
            value: Statement::Field(f),
            ..
        } = &def.value.body[0]
        else {
            panic!()
        };
        let Expr::String(s) = &f.expr.value else { panic!() };
        assert_eq!(s, "Test");
    }

    #[test]
    fn block_comment_multiline() {
        let def = parse_def("/* multi\n   line\n   comment */\nHealth 100;");
        assert_eq!(def.value.body.len(), 1);
    }

    #[test]
    fn missing_semicolon_tolerated() {
        // The `;` terminator is optional (a clean grammar rule, not recovery) —
        // the only genuine tolerance the corpus relies on (§11.1).
        let def = parse_def("  Health 100");
        assert_eq!(def.value.body.len(), 1);
    }

    // --- strict errors (no recovery) -------------------------------------------

    #[test]
    fn err_unterminated_block_comment() {
        // A `/*` with real content and no closer is a lex error, surfaced as the
        // file's single parse error (§11.2, strict — no line-skip recovery).
        let kind = parse_err("#definition OBJECT T\n  Health /* never closes\n#end_definition");
        assert!(matches!(kind, TextParseErrorKind::UnterminatedBlockComment));
    }

    #[test]
    fn err_unterminated_string() {
        let kind = parse_err("#definition OBJECT T\n  Name \"no close\n#end_definition");
        assert!(matches!(kind, TextParseErrorKind::UnterminatedString));
    }

    #[test]
    fn err_mismatched_tag() {
        // A mismatched close tag is a hard error, not a dropped statement.
        let kind = parse_err("#definition OBJECT T\n  <A>\n  <\\B>\n#end_definition");
        assert!(matches!(kind, TextParseErrorKind::MismatchedTag { .. }));
    }

    #[test]
    fn err_missing_end_definition() {
        let kind = parse_err("#definition OBJECT T\n  Health 100;\n");
        assert!(matches!(
            kind,
            TextParseErrorKind::UnexpectedToken { expected } if expected == "#end_definition"
        ));
    }

    #[test]
    fn missing_end_definition_does_not_swallow_next_def() {
        // The regression this rearchitecture targets: a def missing its
        // `#end_definition` must fail *precisely* on that def, not silently eat
        // the following one. The whole file errors (strict, one error per file);
        // the point is the error is `missing #end_definition`, anchored here.
        let input = concat!(
            "#definition OBJECT FIRST\n",
            "  Health 100;\n",
            "#definition OBJECT SECOND\n",
            "  Health 200;\n",
            "#end_definition\n",
        );
        let err = parse_def_file(input).unwrap_err();
        assert!(matches!(
            err.inner,
            TextParseErrorKind::UnexpectedToken { expected } if expected == "#end_definition"
        ));
        // The error is anchored at the second `#definition` (the token that
        // revealed FIRST was never closed), not swallowed away.
        assert_eq!(err.pos, input.find("#definition OBJECT SECOND").unwrap());
    }

    #[test]
    fn empty_file() {
        assert!(parse_def_file("").unwrap().definitions.is_empty());
    }

    #[test]
    fn whitespace_only() {
        assert!(
            parse_def_file("   \n\t  \n  ")
                .unwrap()
                .definitions
                .is_empty()
        );
    }

    #[test]
    fn comments_only() {
        let input = "// line comment\n/* block\n   comment */\n";
        assert!(parse_def_file(input).unwrap().definitions.is_empty());
    }

    #[test]
    fn skips_between_def_junk() {
        // The lexer strips commented-out defs and decorative banner lines
        // (§11.4); the parser skips stray tokens between top-level items and
        // parses the file-local `enum` via the header bridge. Two defs survive.
        let input = r#"
    //#definition OBJECT COMMENTED_OUT_NEVER_PARSED
    //   Health 999;
    //#end_definition

    enum EFoo { A = 1, B = 2 };

    ****************************************

    #definition OBJECT FIRST
        Health 100;
    #end_definition

    stray_identifier;

    #definition OBJECT SECOND
        Health 200;
    #end_definition;
    "#;
        let file = parse_def_file(input).unwrap();
        assert_eq!(file.definitions.len(), 2);
        assert_eq!(file.definitions[0].value.name, "FIRST");
        assert_eq!(file.definitions[1].value.name, "SECOND");
        assert_eq!(file.headers.len(), 1);
    }
}
