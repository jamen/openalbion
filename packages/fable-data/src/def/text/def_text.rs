use super::base::{ParseError, Span, Spanned};
use super::header::{HeaderItem, HeaderParser};
use super::lexer::{LexError, LexErrorKind, Token, TokenKind, lex};
use derive_more::Display;
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
            Expr::BitOr(terms) => fmt_separated_spanned(f, terms, " | "),
            Expr::Add(terms) => fmt_separated_spanned(f, terms, " + "),
        }
    }
}

fn fmt_separated(f: &mut std::fmt::Formatter<'_>, terms: &[Spanned<Expr>], sep: &str) -> std::fmt::Result {
    for (i, term) in terms.iter().enumerate() {
        if i > 0 {
            f.write_str(sep)?;
        }
        write!(f, "{}", term.value)?;
    }
    Ok(())
}

fn fmt_separated_spanned(f: &mut std::fmt::Formatter<'_>, terms: &[Spanned<Expr>], sep: &str) -> std::fmt::Result {
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

pub type DefParseError = ParseError<DefParseErrorKind>;

#[derive(Debug, Display)]
pub enum DefParseErrorKind {
    #[display("expected {expected}")]
    UnexpectedToken { expected: String },
    #[display("mismatched tag: opened <{opened}>, closed <\\{closed}>")]
    MismatchedTag { opened: String, closed: String },
    #[display("unterminated string")]
    UnterminatedString,
    #[display("unterminated block comment")]
    UnterminatedBlockComment,
    #[display("unexpected character {_0:?}")]
    UnexpectedChar(char),
}

/// A strict, token-based parser for the text-def grammar (§11.5). It walks the
/// flat [`Token`] stream produced by [`lex`] — trivia and banner lines are
/// already gone — and produces one AST or the file's single [`DefParseError`]
/// (§11.2: strict, one error per file, no recovery).
pub struct DefParser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}

/// Map a [`LexError`] onto the file's single [`DefParseError`]: a lex failure is
/// this file's parse error, rendered like any other (§11.5).
fn lex_error_to_parse_error(e: LexError) -> DefParseError {
    let kind = match e.kind {
        LexErrorKind::UnterminatedString => DefParseErrorKind::UnterminatedString,
        LexErrorKind::UnterminatedBlockComment => DefParseErrorKind::UnterminatedBlockComment,
        LexErrorKind::UnexpectedChar(c) => DefParseErrorKind::UnexpectedChar(c),
    };
    DefParseError::new(e.span.start, kind)
}

/// A short human name for a token kind, for "expected …, found …" messages.
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

impl<'a> DefParser<'a> {
    /// Lex `input` and build a parser over its tokens. On a lex error the stream
    /// is a lone `Eof`, so [`parse_expr`](Self::parse_expr) fails cleanly; use
    /// [`parse_def_file`] when the lex error must be surfaced.
    pub fn new(input: &'a str) -> Self {
        let tokens = lex(input).unwrap_or_else(|_| {
            vec![Token {
                kind: TokenKind::Eof,
                span: Span {
                    start: input.len(),
                    end: input.len(),
                },
                source: &input[input.len()..],
            }]
        });
        Self { tokens, pos: 0 }
    }

    // ── Token cursor ──────────────────────────────────────────────────────────

    /// The current token. Always valid: the stream ends in exactly one `Eof` and
    /// [`bump`](Self::bump) never advances past it.
    fn peek(&self) -> Token<'a> {
        self.tokens[self.pos]
    }

    /// The token `n` ahead, saturating at the trailing `Eof`.
    fn peek_at(&self, n: usize) -> Token<'a> {
        *self
            .tokens
            .get(self.pos + n)
            .unwrap_or_else(|| self.tokens.last().expect("stream ends in Eof"))
    }

    /// Return the current token and advance (never past `Eof`).
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

    /// Whether the current token is an `Ident` whose text is exactly `name`
    /// (contextual keywords like `specialises`).
    fn at_ident(&self, name: &str) -> bool {
        let t = self.peek();
        t.kind == TokenKind::Ident && t.source == name
    }

    /// End offset of the most recently consumed token — used for node spans.
    fn prev_end(&self) -> usize {
        self.tokens[self.pos.saturating_sub(1)].span.end
    }

    fn err(&self, at: usize, expected: impl Into<String>) -> DefParseError {
        DefParseError::new(
            at,
            DefParseErrorKind::UnexpectedToken {
                expected: expected.into(),
            },
        )
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token<'a>, DefParseError> {
        if self.at(kind) {
            Ok(self.bump())
        } else {
            let found = self.peek();
            Err(self.err(
                found.span.start,
                format!("{}, found {}", describe(kind), describe(found.kind)),
            ))
        }
    }

    fn expect_ident(&mut self, what: &str) -> Result<String, DefParseError> {
        let t = self.peek();
        if t.kind == TokenKind::Ident {
            self.bump();
            Ok(t.source.to_string())
        } else {
            Err(self.err(t.span.start, format!("{what}, found {}", describe(t.kind))))
        }
    }

    // ── Productions ───────────────────────────────────────────────────────────

    pub fn parse_file(&mut self) -> Result<DefFile, DefParseError> {
        let mut file = DefFile::default();
        loop {
            match self.peek().kind {
                TokenKind::Eof => break,
                TokenKind::Definition | TokenKind::DefinitionTemplate => {
                    let def = self.parse_definition()?;
                    let name_index = file.definitions.len();
                    let def_name = def.value.name.clone();
                    file.definitions.push(def);
                    file.by_name.insert(def_name, name_index);
                }
                // A file-local `enum`/`#define` declaration at `.def` top level.
                // Parsed on tokens by the shared header grammar (Phase 3).
                TokenKind::Enum | TokenKind::Define => {
                    file.headers.push(self.parse_header_item_on_tokens()?);
                }
                // Stray tokens between top-level items are skipped, as the
                // pre-token parser did (its `skip_to_next_top_level_item` walked
                // over anything up to the next `#definition`/`enum`/`#define`).
                // This is not body-recovery — the strict body loop still errors
                // on a missing `#end_definition`.
                _ => {
                    self.bump();
                }
            }
        }
        Ok(file)
    }

    /// Parse one file-local header item using the token-based header grammar
    /// (Phase 3). Constructs a temporary [`HeaderParser`] over the remaining
    /// tokens, parses one item, then advances the def parser's cursor by the
    /// number of tokens consumed.
    fn parse_header_item_on_tokens(&mut self) -> Result<HeaderItem, DefParseError> {
        let remaining: Vec<Token<'a>> = self.tokens[self.pos..].to_vec();
        let mut hp = HeaderParser::from_tokens(remaining);
        let item = hp.parse_one_item().map_err(|e| {
            self.err(e.pos, format!("enum or #define declaration: {}", e.inner))
        })?;
        self.pos += hp.consumed();
        Ok(item)
    }

    fn parse_definition(&mut self) -> Result<Spanned<Definition>, DefParseError> {
        let header_tok = self.peek();
        let def_start = header_tok.span.start;
        let is_template = match header_tok.kind {
            TokenKind::DefinitionTemplate => {
                self.bump();
                true
            }
            TokenKind::Definition => {
                self.bump();
                false
            }
            _ => {
                return Err(self.err(
                    header_tok.span.start,
                    "#definition or #definition_template",
                ));
            }
        };

        let def_type = self.expect_ident("definition type")?;
        let name = self.expect_ident("definition name")?;

        let specializes = if self.at_ident("specialises") {
            self.bump();
            Some(self.expect_ident("specialised parent")?)
        } else {
            None
        };

        let mut body = Vec::new();
        let def_end = loop {
            let tk = self.peek().kind;
            if tk == TokenKind::EndDefinition {
                let mut end = self.bump().span.end;
                // The pre-token parser tolerated a trailing `;` after
                // `#end_definition` (an optional terminator, a clean rule).
                if self.at(TokenKind::Semi) {
                    end = self.bump().span.end;
                }
                break end;
            }
            if is_body_terminator(tk) {
                // A directive/keyword or EOF before `#end_definition`: the def
                // body was never closed (§11.5 — the precise error, no
                // skip-recovery swallowing the next def).
                return Err(self.err(self.peek().span.start, "#end_definition"));
            }
            body.push(self.parse_statement()?);
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
                body,
            },
        })
    }

    fn parse_statement(&mut self) -> Result<Spanned<Statement>, DefParseError> {
        let stmt_start = self.peek().span.start;

        // Tagged block: `<` not followed by `\` (a `<\` opens a *close* tag).
        if self.at(TokenKind::Lt) && self.peek_at(1).kind != TokenKind::Backslash {
            let tb = self.parse_tagged_block()?;
            return Ok(Spanned {
                span: Span {
                    start: stmt_start,
                    end: self.prev_end(),
                },
                value: Statement::TaggedBlock(tb),
            });
        }

        let path = self.parse_property_path()?;

        // Method call: the path is followed by an argument list.
        if self.at(TokenKind::LParen) {
            let (object, method) = self.split_method_path(path)?;
            let call = self.parse_call_with_name(method)?;
            if self.at(TokenKind::Semi) {
                self.bump();
            }
            return Ok(Spanned {
                span: Span {
                    start: stmt_start,
                    end: self.prev_end(),
                },
                value: Statement::MethodCall(MethodCall { object, call }),
            });
        }

        // Field assignment: `path expr`.
        let expr = self.parse_expr()?;
        if self.at(TokenKind::Semi) {
            self.bump();
        }
        Ok(Spanned {
            span: Span {
                start: stmt_start,
                end: self.prev_end(),
            },
            value: Statement::Field(Field { path, expr }),
        })
    }

    fn parse_tagged_block(&mut self) -> Result<TaggedBlock, DefParseError> {
        self.expect(TokenKind::Lt)?;
        let tag = self.expect_ident("tag name")?;
        self.expect(TokenKind::Gt)?;
        let mut body = Vec::new();
        loop {
            let tk = self.peek().kind;
            if tk == TokenKind::Lt && self.peek_at(1).kind == TokenKind::Backslash {
                self.bump(); // `<`
                self.bump(); // `\`
                let close_tag = self.expect_ident("closing tag name")?;
                self.expect(TokenKind::Gt)?;
                if close_tag != tag {
                    return Err(DefParseError::new(
                        self.prev_end(),
                        DefParseErrorKind::MismatchedTag {
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
                return Err(self.err(self.peek().span.start, format!("<\\{tag}>")));
            }
            body.push(self.parse_statement()?);
        }
        Ok(TaggedBlock { tag, body })
    }

    fn parse_property_path(&mut self) -> Result<PropertyPath, DefParseError> {
        let mut segments = vec![PathSegment::Field(self.expect_ident("field name")?)];
        loop {
            if self.at(TokenKind::Dot) {
                self.bump();
                segments.push(PathSegment::Field(self.expect_ident("field name")?));
            } else if self.at(TokenKind::LBracket) {
                self.bump();
                let idx = self.parse_expr()?;
                self.expect(TokenKind::RBracket)?;
                segments.push(PathSegment::Index(idx));
            } else {
                break;
            }
        }
        Ok(PropertyPath { segments })
    }

    fn split_method_path(
        &self,
        path: PropertyPath,
    ) -> Result<(PropertyPath, String), DefParseError> {
        let mut segments = path.segments;
        if let Some(PathSegment::Field(method)) = segments.pop() {
            Ok((PropertyPath { segments }, method))
        } else {
            Err(self.err(self.peek().span.start, "method name"))
        }
    }

    pub fn parse_expr(&mut self) -> Result<Spanned<Expr>, DefParseError> {
        self.parse_bitor_expr()
    }

    fn parse_bitor_expr(&mut self) -> Result<Spanned<Expr>, DefParseError> {
        let start = self.peek().span.start;
        let first = self.parse_add_expr()?;
        let mut terms = vec![first];
        while self.at(TokenKind::Pipe) {
            self.bump();
            terms.push(self.parse_add_expr()?);
        }
        if terms.len() == 1 {
            Ok(terms.pop().unwrap())
        } else {
            Ok(Spanned {
                span: Span {
                    start,
                    end: self.prev_end(),
                },
                value: Expr::BitOr(terms),
            })
        }
    }

    fn parse_add_expr(&mut self) -> Result<Spanned<Expr>, DefParseError> {
        let start = self.peek().span.start;
        let first = self.parse_leaf_expr()?;
        let mut terms = vec![first];
        while self.at(TokenKind::Plus) {
            self.bump();
            terms.push(self.parse_leaf_expr()?);
        }
        if terms.len() == 1 {
            Ok(terms.pop().unwrap())
        } else {
            Ok(Spanned {
                span: Span {
                    start,
                    end: self.prev_end(),
                },
                value: Expr::Add(terms),
            })
        }
    }

    fn parse_leaf_expr(&mut self) -> Result<Spanned<Expr>, DefParseError> {
        let tok = self.peek();
        match tok.kind {
            TokenKind::Str => {
                self.bump();
                // The lexer guarantees a closing quote (unterminated → lex
                // error), so both quotes are present to strip.
                let unquoted = tok.source[1..tok.source.len() - 1].to_string();
                Ok(Spanned {
                    span: tok.span,
                    value: Expr::String(unquoted),
                })
            }
            TokenKind::Number => {
                self.bump();
                Ok(Spanned {
                    span: tok.span,
                    value: Expr::Number(tok.source.to_string()),
                })
            }
            TokenKind::Ident => {
                self.bump();
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
                        if self.at(TokenKind::LParen) {
                            let call = self.parse_call_with_name(ident.to_string())?;
                            Ok(Spanned {
                                span: Span {
                                    start: tok.span.start,
                                    end: self.prev_end(),
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
            _ => Err(self.err(tok.span.start, format!("expression, found {}", describe(tok.kind)))),
        }
    }

    fn parse_call_with_name(&mut self, name: String) -> Result<Call, DefParseError> {
        self.expect(TokenKind::LParen)?;
        let arguments = self.parse_arguments()?;
        self.expect(TokenKind::RParen)?;
        Ok(Call { name, arguments })
    }

    fn parse_arguments(&mut self) -> Result<Vec<Spanned<Expr>>, DefParseError> {
        let mut args = Vec::new();
        if self.at(TokenKind::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expr()?);
            if self.at(TokenKind::Comma) {
                self.bump();
            } else {
                break;
            }
        }
        Ok(args)
    }
}

pub fn parse_def_file(input: &str) -> Result<DefFile, DefParseError> {
    let tokens = lex(input).map_err(lex_error_to_parse_error)?;
    let mut parser = DefParser { tokens, pos: 0 };
    parser.parse_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_def(body: &str) -> Spanned<Definition> {
        let input = format!("#definition OBJECT T\n{body}\n#end_definition");
        parse_def_file(&input).unwrap().definitions.pop().unwrap()
    }

    fn parse_first_def(input: &str) -> Spanned<Definition> {
        parse_def_file(input).unwrap().definitions.pop().unwrap()
    }

    fn parse_err(input: &str) -> DefParseErrorKind {
        parse_def_file(input).unwrap_err().inner
    }

    fn parse_stmt(stmt: &str) -> Spanned<Statement> {
        parse_def(stmt).value.body.pop().unwrap()
    }

    fn parse_expr(value: &str) -> Spanned<Expr> {
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
        match parse_expr(value).value {
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
        let Expr::String(s) = parse_expr(r#""Hello, World!""#).value else {
            panic!()
        };
        assert_eq!(s, "Hello, World!");
    }

    #[test]
    fn bool_test() {
        assert!(matches!(parse_expr("TRUE").value, Expr::Bool(true)));
        assert!(matches!(parse_expr("FALSE").value, Expr::Bool(false)));
    }

    #[test]
    fn bool_b_prefix() {
        assert!(matches!(parse_expr("BTRUE").value, Expr::Bool(true)));
        assert!(matches!(parse_expr("BFALSE").value, Expr::Bool(false)));
    }

    #[test]
    fn add_n_ary() {
        let Expr::Add(terms) = &parse_expr("1 + 2 + 3").value else {
            panic!()
        };
        assert_eq!(terms.len(), 3);
    }

    #[test]
    fn bitor_n_ary() {
        let Expr::BitOr(terms) = &parse_expr("A | B | C").value else {
            panic!()
        };
        assert_eq!(terms.len(), 3);
    }

    #[test]
    fn bitor_precedence_lower_than_add() {
        let Expr::BitOr(terms) = &parse_expr("A | B + C").value else {
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
        let Expr::Constructor(c) = &parse_expr("CRGBColour(255, 128, 64, 255)").value else {
            panic!()
        };
        assert_eq!(c.name, "CRGBColour");
        assert_eq!(c.arguments.len(), 4);
    }

    #[test]
    fn empty_constructor() {
        let Expr::Constructor(c) = &parse_expr("CRGBColour()").value else {
            panic!()
        };
        assert!(c.arguments.is_empty());
    }

    #[test]
    fn identifier() {
        let Expr::Symbol(s) = parse_expr("GRAPHIC_NULL").value else {
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
        assert!(matches!(kind, DefParseErrorKind::UnterminatedBlockComment));
    }

    #[test]
    fn err_unterminated_string() {
        let kind = parse_err("#definition OBJECT T\n  Name \"no close\n#end_definition");
        assert!(matches!(kind, DefParseErrorKind::UnterminatedString));
    }

    #[test]
    fn err_mismatched_tag() {
        // A mismatched close tag is a hard error, not a dropped statement.
        let kind = parse_err("#definition OBJECT T\n  <A>\n  <\\B>\n#end_definition");
        assert!(matches!(kind, DefParseErrorKind::MismatchedTag { .. }));
    }

    #[test]
    fn err_missing_end_definition() {
        let kind = parse_err("#definition OBJECT T\n  Health 100;\n");
        assert!(matches!(
            kind,
            DefParseErrorKind::UnexpectedToken { expected } if expected == "#end_definition"
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
            DefParseErrorKind::UnexpectedToken { expected } if expected == "#end_definition"
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
