use super::base::{ConsumeCharError, ParseError, ParserBase, SkipTriviaError};
use derive_more::Display;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct DefFile {
    pub definitions: Vec<Definition>,
    pub by_name: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub is_template: bool,
    pub def_type: String,
    pub name: String,
    pub specializes: Option<String>,
    pub body: Vec<Statement>,
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
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct MethodCall {
    pub object: PropertyPath,
    pub call: Call,
}

#[derive(Debug, Clone)]
pub struct TaggedBlock {
    pub tag: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct PropertyPath {
    pub segments: Vec<PathSegment>,
}

impl PropertyPath {
    pub fn simple(name: impl Into<String>) -> Self {
        Self { segments: vec![PathSegment::Field(name.into())] }
    }
}

impl std::fmt::Display for PropertyPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, seg) in self.segments.iter().enumerate() {
            match seg {
                PathSegment::Field(name) => {
                    if i > 0 { f.write_str(".")?; }
                    f.write_str(name)?;
                }
                PathSegment::Index(expr) => write!(f, "[{expr}]")?,
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Integer(n) => write!(f, "{n}"),
            Expr::Float(x) => write!(f, "{x}"),
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

fn fmt_separated(f: &mut std::fmt::Formatter<'_>, terms: &[Expr], sep: &str) -> std::fmt::Result {
    for (i, term) in terms.iter().enumerate() {
        if i > 0 { f.write_str(sep)?; }
        write!(f, "{term}")?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    Field(String),
    Index(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Float(f32),
    Bool(bool),
    String(String),
    Symbol(String),
    Constructor(Call),
    BitOr(Vec<Expr>),
    Add(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub name: String,
    pub arguments: Vec<Expr>,
}

pub struct DefParser<'a> {
    parser: ParserBase<'a>,
}

pub type DefParseError = ParseError<DefParseErrorKind>;

#[derive(Debug, Display)]
pub enum DefParseErrorKind {
    #[display("unexpected end of input")]
    UnexpectedEnd,
    #[display("unexpected character {_0}")]
    UnexpectedChar(char),
    #[display("consume char error: {_0}")]
    ConsumeChar(ConsumeCharError),
    #[display("skip trivia: {_0}")]
    SkipTrivia(SkipTriviaError),
    #[display("unexpected token. expected {expected}")]
    UnexpectedToken { expected: String },
    #[display("invalid number: {_0}")]
    InvalidNumber(String),
    #[display("unterminated string")]
    UnterminatedString,
    #[display("unterminated block comment")]
    UnterminatedBlockComment,
    #[display("mismatched tag: opened <{opened}>, closed <\\{closed}>")]
    MismatchedTag { opened: String, closed: String },
}

impl<'a> DefParser<'a> {
    pub fn new(input: &'a str) -> Self { Self { parser: ParserBase::new(input) } }

    fn input(&self) -> &str { self.parser.input() }
    fn pos(&self) -> usize { self.parser.pos() }
    fn advance(&mut self, add: usize) { self.parser.advance(add); }
    fn seek_to(&mut self, pos: usize) { self.parser.seek_to(pos); }
    fn rest(&self) -> &str { self.parser.rest() }
    fn is_eof(&self) -> bool { self.parser.is_eof() }
    fn peek_char(&self) -> Option<char> { self.parser.peek_char() }
    fn peek_char_at(&self, offset: usize) -> Option<char> { self.parser.peek_char_at(offset) }

    fn consume_char(&mut self, expected: char) -> Result<(), DefParseError> {
        self.parser.consume_char(expected)
            .map_err(|e| DefParseError::new(e.pos, DefParseErrorKind::ConsumeChar(e.inner)))
    }

    fn skip_trivia(&mut self) -> Result<(), DefParseError> {
        self.parser.skip_trivia()
            .map_err(|e| DefParseError::new(e.pos, DefParseErrorKind::SkipTrivia(e.inner)))
    }

    fn try_consume(&mut self, s: &str) -> bool { self.parser.try_consume(s) }
    fn at_line_start(&mut self) -> bool { self.parser.at_line_start() }
    fn err(&self, inner: DefParseErrorKind) -> ParseError<DefParseErrorKind> { ParseError::new(self.pos(), inner) }
}

impl<'a> DefParser<'a> {
    pub fn parse_file(&mut self) -> Result<DefFile, DefParseError> {
        let mut file = DefFile::default();
        while self.skip_to_next_definition() {
            let def = self.parse_definition()?;
            let name_index = file.definitions.len();
            let def_name = def.name.clone();
            file.definitions.push(def);
            file.by_name.insert(def_name, name_index);
        }
        Ok(file)
    }

    fn skip_to_next_definition(&mut self) -> bool {
        loop {
            if self.at_definition_keyword() && self.at_line_start() { return true; }
            match self.peek_char() {
                Some(c) => self.advance(c.len_utf8()),
                None => return false,
            }
        }
    }

    fn at_definition_keyword(&self) -> bool {
        let rest = self.rest();
        let after = rest.strip_prefix("#definition_template")
            .or_else(|| rest.strip_prefix("#definition"));
        after.is_some_and(|s| s.chars().next().is_some_and(|c| c.is_whitespace()))
    }

    fn parse_definition(&mut self) -> Result<Definition, DefParseError> {
        let is_template = if self.try_consume("#definition_template") { true }
        else if self.try_consume("#definition") { false }
        else { return Err(self.err(DefParseErrorKind::UnexpectedToken { expected: "#definition or #definition_template".into() })); };

        self.skip_trivia()?;
        let def_type = self.parse_identifier()?;
        self.skip_trivia()?;
        let name = self.parse_identifier()?;
        self.skip_trivia()?;

        let specializes = if self.try_consume("specialises") {
            self.skip_trivia()?;
            Some(self.parse_identifier()?)
        } else { None };

        let mut body = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.try_consume("#end_definition") { let _ = self.try_consume(";"); break; }
            if self.is_eof() { return Err(self.err(DefParseErrorKind::UnexpectedToken { expected: "#end_definition".into() })); }
            body.push(self.parse_statement()?);
        }

        Ok(Definition { is_template, def_type, name, specializes, body })
    }

    fn parse_statement(&mut self) -> Result<Statement, DefParseError> {
        self.skip_trivia()?;
        if self.peek_char() == Some('<') && self.peek_char_at(1) != Some('\\') {
            return self.parse_tagged_block().map(Statement::TaggedBlock);
        }
        let path = self.parse_property_path()?;
        self.skip_trivia()?;
        if self.peek_char() == Some('(') {
            let (object, method) = self.split_method_path(path)?;
            let call = self.parse_call_with_name(method)?;
            self.skip_trivia()?;
            self.consume_char(';')?;
            return Ok(Statement::MethodCall(MethodCall { object, call }));
        }
        let value = self.parse_expr()?;
        self.skip_trivia()?;
        self.consume_char(';')?;
        Ok(Statement::Field(Field { path, expr: value }))
    }

    fn parse_tagged_block(&mut self) -> Result<TaggedBlock, DefParseError> {
        self.consume_char('<')?;
        let tag = self.parse_identifier()?;
        self.consume_char('>')?;
        let mut body = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.peek_char() == Some('<') && self.peek_char_at(1) == Some('\\') {
                self.consume_char('<')?;
                self.consume_char('\\')?;
                let close_tag = self.parse_identifier()?;
                self.consume_char('>')?;
                if close_tag != tag { return Err(self.err(DefParseErrorKind::MismatchedTag { opened: tag, closed: close_tag })); }
                break;
            }
            if self.is_eof() { return Err(self.err(DefParseErrorKind::UnexpectedToken { expected: format!("<\\{}>", tag) })); }
            body.push(self.parse_statement()?);
        }
        Ok(TaggedBlock { tag, body })
    }

    fn parse_property_path(&mut self) -> Result<PropertyPath, DefParseError> {
        let mut segments = Vec::new();
        let first = self.parse_identifier()?;
        segments.push(PathSegment::Field(first));
        loop {
            if self.peek_char() == Some('.') {
                self.consume_char('.')?;
                let field = self.parse_identifier()?;
                segments.push(PathSegment::Field(field));
            } else if self.peek_char() == Some('[') {
                self.consume_char('[')?;
                self.skip_trivia()?;
                let idx = self.parse_expr()?;
                self.skip_trivia()?;
                self.consume_char(']')?;
                segments.push(PathSegment::Index(idx));
            } else { break; }
        }
        Ok(PropertyPath { segments })
    }

    fn split_method_path(&self, path: PropertyPath) -> Result<(PropertyPath, String), DefParseError> {
        let mut segments = path.segments;
        if let Some(PathSegment::Field(method)) = segments.pop() {
            Ok((PropertyPath { segments }, method))
        } else {
            Err(self.err(DefParseErrorKind::UnexpectedToken { expected: "method name".into() }))
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr, DefParseError> {
        self.parse_bitor_expr()
    }

    fn parse_bitor_expr(&mut self) -> Result<Expr, DefParseError> {
        let first = self.parse_add_expr()?;
        let mut terms = vec![first];
        self.skip_trivia()?;
        while self.try_consume("|") {
            self.skip_trivia()?;
            terms.push(self.parse_add_expr()?);
            self.skip_trivia()?;
        }
        Ok(if terms.len() == 1 { terms.pop().unwrap() } else { Expr::BitOr(terms) })
    }

    fn parse_add_expr(&mut self) -> Result<Expr, DefParseError> {
        let first = self.parse_leaf_expr()?;
        let mut terms = vec![first];
        self.skip_trivia()?;
        while self.try_consume("+") {
            self.skip_trivia()?;
            terms.push(self.parse_leaf_expr()?);
            self.skip_trivia()?;
        }
        Ok(if terms.len() == 1 { terms.pop().unwrap() } else { Expr::Add(terms) })
    }

    fn parse_leaf_expr(&mut self) -> Result<Expr, DefParseError> {
        self.skip_trivia()?;
        if self.peek_char() == Some('"') { return self.parse_string().map(Expr::String); }
        let p0 = self.peek_char();
        let next_is_digit = p0.is_some_and(|c| c.is_ascii_digit());
        let neg_then_digit = p0 == Some('-') && self.peek_char_at(1).is_some_and(|c| c.is_ascii_digit());
        if next_is_digit || neg_then_digit { return self.parse_number(); }
        let ident = self.parse_identifier()?;
        if ident == "TRUE" || ident == "BTRUE" { return Ok(Expr::Bool(true)); }
        if ident == "FALSE" || ident == "BFALSE" { return Ok(Expr::Bool(false)); }
        self.skip_trivia()?;
        if self.peek_char() == Some('(') {
            Ok(Expr::Constructor(self.parse_call_with_name(ident)?))
        } else { Ok(Expr::Symbol(ident)) }
    }

    fn parse_call_with_name(&mut self, name: String) -> Result<Call, DefParseError> {
        self.consume_char('(')?;
        let arguments = self.parse_arguments()?;
        self.consume_char(')')?;
        Ok(Call { name, arguments })
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expr>, DefParseError> {
        let mut args = Vec::new();
        self.skip_trivia()?;
        if self.peek_char() == Some(')') { return Ok(args); }
        loop {
            self.skip_trivia()?;
            args.push(self.parse_expr()?);
            self.skip_trivia()?;
            if self.try_consume(",") { continue; } else { break; }
        }
        Ok(args)
    }

    fn parse_number(&mut self) -> Result<Expr, DefParseError> {
        let start = self.pos();
        match self.parse_number_inner() {
            Ok(x) => Ok(x),
            Err(e) => { self.seek_to(start); Err(e.with_pos(start)) }
        }
    }

    fn parse_number_inner(&mut self) -> Result<Expr, DefParseError> {
        let start = self.pos();
        let mut has_dot = false;
        let mut has_f_suffix = false;
        if self.peek_char() == Some('-') { self.advance(1); }
        while self.peek_char().map(|c| c.is_ascii_digit()) == Some(true) { self.advance(1); }
        if self.peek_char() == Some('.') {
            has_dot = true;
            self.advance(1);
            while self.peek_char().map(|c| c.is_ascii_digit()) == Some(true) { self.advance(1); }
        }
        if self.peek_char() == Some('f') { has_f_suffix = true; self.advance(1); }
        let text = self.input();
        let text = &text[start..self.pos()];
        if has_dot || has_f_suffix {
            let text = text.trim_end_matches('f');
            text.parse::<f32>().map(Expr::Float).map_err(|_| self.err(DefParseErrorKind::InvalidNumber(text.into())))
        } else {
            text.parse::<i64>().map(Expr::Integer).map_err(|_| self.err(DefParseErrorKind::InvalidNumber(text.into())))
        }
    }

    fn parse_string(&mut self) -> Result<String, DefParseError> {
        self.consume_char('"')?;
        let start = self.pos();
        while let Some(c) = self.peek_char() {
            if c == '"' {
                let s = self.input();
                let s = s[start..self.pos()].to_string();
                self.advance(1);
                return Ok(s);
            }
            self.advance(c.len_utf8());
        }
        Err(self.err(DefParseErrorKind::UnterminatedString))
    }

    fn parse_identifier(&mut self) -> Result<String, DefParseError> {
        let start = self.pos();
        match self.peek_char() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => { self.advance(1); }
            Some(found) => return Err(self.err(DefParseErrorKind::ConsumeChar(ConsumeCharError::UnexpectedCharacter { found }))),
            None => return Err(self.err(DefParseErrorKind::UnexpectedEnd)),
        }
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' { self.advance(1); } else { break; }
        }
        let s = self.input();
        let s = s[start..self.pos()].to_string();
        Ok(s)
    }
}

pub fn parse_def_file(input: &str) -> Result<DefFile, DefParseError> {
    let mut parser = DefParser::new(input);
    parser.parse_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_def(body: &str) -> Definition {
        let input = format!("#definition OBJECT T\n{body}\n#end_definition");
        parse_def_file(&input).unwrap().definitions.pop().unwrap()
    }

    fn parse_first_def(input: &str) -> Definition {
        parse_def_file(input).unwrap().definitions.pop().unwrap()
    }

    fn parse_err(input: &str) -> DefParseErrorKind {
        parse_def_file(input).unwrap_err().inner
    }

    fn parse_stmt(stmt: &str) -> Statement {
        parse_def(stmt).body.pop().unwrap()
    }

    fn parse_expr(value: &str) -> Expr {
        match parse_stmt(&format!("X {value};")) {
            Statement::Field(f) => f.expr,
            other => panic!("expected Field, got {other:?}"),
        }
    }

    fn parse_path(path: &str) -> PropertyPath {
        let Statement::Field(f) = parse_stmt(&format!("{path} 0;")) else { panic!() };
        f.path
    }

    #[test]
    fn integer() {
        assert_eq!(parse_expr("42"), Expr::Integer(42));
        assert_eq!(parse_expr("42282949"), Expr::Integer(42282949));
    }

    #[test]
    fn negative_integer() {
        assert_eq!(parse_expr("-42"), Expr::Integer(-42));
        assert_eq!(parse_expr("-42282949"), Expr::Integer(-42282949));
    }

    #[test]
    fn float() {
        let Expr::Float(f) = parse_expr("4.2") else { panic!() };
        assert!((f - 4.2).abs() < f32::EPSILON);
        let Expr::Float(f) = parse_expr("4.2f") else { panic!() };
        assert!((f - 4.2).abs() < f32::EPSILON);
        let Expr::Float(f) = parse_expr("4.") else { panic!() };
        assert_eq!(f, 4.0);
    }

    #[test]
    fn negative_float() {
        let Expr::Float(f) = parse_expr("-4.2") else { panic!() };
        assert!((f - -4.2).abs() < f32::EPSILON);
        let Expr::Float(f) = parse_expr("-4.2f") else { panic!() };
        assert!((f - -4.2).abs() < f32::EPSILON);
        let Expr::Float(f) = parse_expr("-4.") else { panic!() };
        assert_eq!(f, -4.0);
    }

    #[test]
    fn string() {
        let Expr::String(s) = parse_expr(r#""Hello, World!""#) else { panic!() };
        assert_eq!(s, "Hello, World!");
    }

    #[test]
    fn bool_test() {
        assert!(matches!(parse_expr("TRUE"), Expr::Bool(true)));
        assert!(matches!(parse_expr("FALSE"), Expr::Bool(false)));
    }

    #[test]
    fn bool_b_prefix() {
        assert!(matches!(parse_expr("BTRUE"), Expr::Bool(true)));
        assert!(matches!(parse_expr("BFALSE"), Expr::Bool(false)));
    }

    #[test]
    fn add_n_ary() {
        let Expr::Add(terms) = parse_expr("1 + 2 + 3") else { panic!() };
        assert_eq!(terms.len(), 3);
    }

    #[test]
    fn bitor_n_ary() {
        let Expr::BitOr(terms) = parse_expr("A | B | C") else { panic!() };
        assert_eq!(terms.len(), 3);
    }

    #[test]
    fn bitor_precedence_lower_than_add() {
        let Expr::BitOr(terms) = parse_expr("A | B + C") else { panic!() };
        assert_eq!(terms.len(), 2);
        assert!(matches!(&terms[0], Expr::Symbol(s) if s == "A"));
        let Expr::Add(add_terms) = &terms[1] else { panic!() };
        assert_eq!(add_terms.len(), 2);
    }

    #[test]
    fn constructor_with_args() {
        let Expr::Constructor(c) = parse_expr("CRGBColour(255, 128, 64, 255)") else { panic!() };
        assert_eq!(c.name, "CRGBColour");
        assert_eq!(c.arguments.len(), 4);
    }

    #[test]
    fn empty_constructor() {
        let Expr::Constructor(c) = parse_expr("CRGBColour()") else { panic!() };
        assert!(c.arguments.is_empty());
    }

    #[test]
    fn identifier() {
        let Expr::Symbol(s) = parse_expr("GRAPHIC_NULL") else { panic!() };
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
        assert!(matches!(&p.segments[1], PathSegment::Index(Expr::Integer(0))));
    }

    #[test]
    fn negative_index() {
        let p = parse_path("Time[-1]");
        assert!(matches!(&p.segments[1], PathSegment::Index(Expr::Integer(-1))));
    }

    #[test]
    fn ident_index() {
        let p = parse_path("Foo[BAR_CONST]");
        let PathSegment::Index(Expr::Symbol(s)) = &p.segments[1] else { panic!() };
        assert_eq!(s, "BAR_CONST");
    }

    #[test]
    fn string_index() {
        let p = parse_path("Map[\"DAY\"]");
        let PathSegment::Index(Expr::String(s)) = &p.segments[1] else { panic!() };
        assert_eq!(s, "DAY");
    }

    #[test]
    fn expression_index() {
        let p = parse_path("States[STATE + 1]");
        assert!(matches!(&p.segments[1], PathSegment::Index(Expr::Add(_))));
    }

    #[test]
    fn nested_field_and_index() {
        let p = parse_path("Time[0].SkyTexture0");
        assert_eq!(p.segments.len(), 3);
        assert!(matches!(&p.segments[0], PathSegment::Field(s) if s == "Time"));
        assert!(matches!(&p.segments[1], PathSegment::Index(Expr::Integer(0))));
        assert!(matches!(&p.segments[2], PathSegment::Field(s) if s == "SkyTexture0"));
    }

    #[test]
    fn field_assignment() {
        let Statement::Field(f) = parse_stmt("Health 100;") else { panic!() };
        assert_eq!(f.path.segments.len(), 1);
        assert!(matches!(f.expr, Expr::Integer(100)));
    }

    #[test]
    fn method_call() {
        let Statement::MethodCall(mc) = parse_stmt("Components.Add(\"CTCPhysicsStandard\");") else { panic!() };
        assert_eq!(mc.call.name, "Add");
        assert_eq!(mc.call.arguments.len(), 1);
    }

    #[test]
    fn tagged_block() {
        let Statement::TaggedBlock(tb) = parse_stmt("<CCreatureDef>\n  Health 100;\n<\\CCreatureDef>") else { panic!() };
        assert_eq!(tb.tag, "CCreatureDef");
        assert_eq!(tb.body.len(), 1);
    }

    #[test]
    fn template_flag() {
        let def = parse_first_def("#definition_template OBJECT T\n#end_definition");
        assert!(def.is_template);
    }

    #[test]
    fn specialises() {
        let def = parse_first_def("#definition OBJECT CHILD specialises PARENT\n  Health 50;\n#end_definition");
        assert_eq!(def.specializes.as_deref(), Some("PARENT"));
    }

    #[test]
    fn end_definition_trailing_semicolon() {
        let file = parse_def_file("#definition OBJECT T\n  Health 100;\n#end_definition;").unwrap();
        assert_eq!(file.definitions.len(), 1);
    }

    #[test]
    fn multiple_definitions_preserve_order() {
        let file = parse_def_file(r#"
    #definition OBJECT FIRST
    #end_definition

    #definition OBJECT SECOND
    #end_definition
    "#).unwrap();
        assert_eq!(file.definitions.len(), 2);
        assert_eq!(file.definitions[0].name, "FIRST");
        assert_eq!(file.definitions[1].name, "SECOND");
        assert_eq!(file.by_name["FIRST"], 0);
        assert_eq!(file.by_name["SECOND"], 1);
    }

    #[test]
    fn line_comment_in_body() {
        let def = parse_def("// just a comment\nHealth 100;");
        assert_eq!(def.body.len(), 1);
    }

    #[test]
    fn block_comment_in_body() {
        let def = parse_def("/* block comment */\nHealth 100;");
        assert_eq!(def.body.len(), 1);
    }

    #[test]
    fn block_comment_inline() {
        let def = parse_def("Name /* inline */ \"Test\";");
        assert_eq!(def.body.len(), 1);
        let Statement::Field(f) = &def.body[0] else { panic!() };
        let Expr::String(s) = &f.expr else { panic!() };
        assert_eq!(s, "Test");
    }

    #[test]
    fn block_comment_multiline() {
        let def = parse_def("/* multi\n   line\n   comment */\nHealth 100;");
        assert_eq!(def.body.len(), 1);
    }

    #[test]
    fn err_unterminated_block_comment() {
        let kind = parse_err("#definition OBJECT T\n  Health /* never closes\n#end_definition");
        assert!(matches!(kind, DefParseErrorKind::SkipTrivia(SkipTriviaError::UnterminatedBlockComment)));
    }

    #[test]
    fn err_unterminated_string() {
        let kind = parse_err("#definition OBJECT T\n  Name \"no close\n#end_definition");
        assert!(matches!(kind, DefParseErrorKind::UnterminatedString));
    }

    #[test]
    fn err_mismatched_tag() {
        let kind = parse_err("#definition OBJECT T\n  <A>\n  <\\B>\n#end_definition");
        assert!(matches!(kind, DefParseErrorKind::MismatchedTag { .. }));
    }

    #[test]
    fn err_missing_semicolon() {
        let kind = parse_err("#definition OBJECT T\n  Health 100\n#end_definition");
        assert!(matches!(kind, DefParseErrorKind::ConsumeChar(ConsumeCharError::MismatchedCharacter { .. })));
    }

    #[test]
    fn err_missing_end_definition() {
        let kind = parse_err("#definition OBJECT T\n  Health 100;\n");
        assert!(matches!(kind, DefParseErrorKind::UnexpectedToken { .. }));
    }

    #[test]
    fn empty_file() {
        assert!(parse_def_file("").unwrap().definitions.is_empty());
    }

    #[test]
    fn whitespace_only() {
        assert!(parse_def_file("   \n\t  \n  ").unwrap().definitions.is_empty());
    }

    #[test]
    fn comments_only() {
        let input = "// line comment\n/* block\n   comment */\n";
        assert!(parse_def_file(input).unwrap().definitions.is_empty());
    }

    #[test]
    fn skips_between_def_junk() {
        let input = r#"
    //#definition OBJECT COMMENTED_OUT_NEVER_PARSED
    //   Health 999;
    //#end_definition

    enum EFoo { A = 1, B = 2 };

    ******** banner without slashes ********

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
        assert_eq!(file.definitions[0].name, "FIRST");
        assert_eq!(file.definitions[1].name, "SECOND");
    }
}
