//! Parser for Fable's `.def` definition files.
//!
//! These are human-readable text files that define game objects, creatures,
//! items, environments, UI layouts, and more.
//!
//! # Grammar Overview
//!
//! ```text
//! file         = (comment | definition)*
//! definition   = "#definition" type name ["specialises" parent] statement* "#end_definition"
//!              | "#definition_template" type name ["specialises" parent] statement* "#end_definition"
//!
//! statement    = assignment | method_call | tagged_block | comment
//! assignment   = path value ";"
//! method_call  = path "." ident "(" [args] ")" ";"
//! tagged_block = "<" ident ">" statement* "<\" ident ">"
//!
//! path         = ident ("." ident | "[" expr "]")*
//! value        = expr
//! expr         = term ("|" term)*
//! term         = number | string | bool | ident | constructor
//! constructor  = ident "(" [args] ")"
//! args         = expr ("," expr)*
//! ```

use derive_more::{Display, Error};
use std::collections::HashMap;

/// A complete def file containing multiple definitions.
#[derive(Debug, Clone, Default)]
pub struct DefFile {
    /// All definitions in the file, keyed by name.
    pub definitions: HashMap<String, Definition>,
}

/// A single definition (e.g., `#definition OBJECT OBJECT_PATROL_MARKER`).
#[derive(Debug, Clone)]
pub struct Definition {
    /// Whether this is a template (`#definition_template`) or regular definition.
    pub is_template: bool,
    /// The type of definition (e.g., "OBJECT", "CREATURE", "BUILDING").
    pub def_type: String,
    /// The name/identifier of this definition.
    pub name: String,
    /// Optional parent definition this specializes.
    pub specializes: Option<String>,
    /// The body of the definition - a list of statements.
    pub body: Vec<Statement>,
}

/// A statement inside a definition.
#[derive(Debug, Clone)]
pub enum Statement {
    /// A property assignment: `path = value;`
    Assignment(Assignment),
    /// A method call: `Components.Add("CTCPhysicsStandard");`
    MethodCall(MethodCall),
    /// A tagged block: `<CCreatureDef> ... <\CCreatureDef>`
    TaggedBlock(TaggedBlock),
}

/// A property assignment like `Time[0].SkyTexture0 GRAPHIC_ATMOSPHERIC_SKY_MIDNIGHT;`
#[derive(Debug, Clone)]
pub struct Assignment {
    /// The property path (e.g., `Time[0].SkyTexture0`).
    pub path: PropertyPath,
    /// The assigned value.
    pub value: Value,
}

/// A method call like `Components.Add("CTCPhysicsStandard");`
#[derive(Debug, Clone)]
pub struct MethodCall {
    /// The object path (e.g., `Components`).
    pub object: PropertyPath,
    /// The method name (e.g., `Add`).
    pub method: String,
    /// The arguments.
    pub arguments: Vec<Value>,
}

/// A tagged block like `<CCreatureDef> ... <\CCreatureDef>`
#[derive(Debug, Clone)]
pub struct TaggedBlock {
    /// The tag name (e.g., `CCreatureDef`).
    pub tag: String,
    /// The statements inside the block.
    pub body: Vec<Statement>,
}

/// A property path like `Time[0].SkyTexture0` or `Graphic.BankIndex`.
#[derive(Debug, Clone)]
pub struct PropertyPath {
    /// The segments of the path.
    pub segments: Vec<PathSegment>,
}

impl PropertyPath {
    /// Create a simple path from a single identifier.
    pub fn simple(name: impl Into<String>) -> Self {
        Self {
            segments: vec![PathSegment::Field(name.into())],
        }
    }

    /// Get the full path as a string for display.
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (i, seg) in self.segments.iter().enumerate() {
            match seg {
                PathSegment::Field(name) => {
                    if i > 0 {
                        result.push('.');
                    }
                    result.push_str(name);
                }
                PathSegment::Index(idx) => {
                    result.push('[');
                    result.push_str(&idx.to_string());
                    result.push(']');
                }
            }
        }
        result
    }
}

/// A segment of a property path.
#[derive(Debug, Clone)]
pub enum PathSegment {
    /// A field access (e.g., `SkyTexture0`).
    Field(String),
    /// An array index (e.g., `[0]`).
    Index(i32),
}

/// A value in an assignment or argument.
#[derive(Debug, Clone)]
pub enum Value {
    /// An integer literal.
    Integer(i64),
    /// A floating-point literal.
    Float(f64),
    /// A boolean literal (TRUE/FALSE).
    Bool(bool),
    /// A string literal ("...").
    String(String),
    /// An identifier or enum value (e.g., `GRAPHIC_ATMOSPHERIC_SKY_MIDNIGHT`, `NULL`).
    Identifier(String),
    /// A constructor call (e.g., `CRGBColour(40,40,40,255)`).
    Constructor(Constructor),
    /// A bitwise OR expression (e.g., `PHYSICS_COLLIDE_LANDSCAPE | PHYSICS_COLLIDE_THINGS`).
    BitwiseOr(Vec<Value>),
}

impl Value {
    /// Try to get this value as an integer.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get this value as a float.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Try to get this value as a bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get this value as a string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get this value as an identifier.
    pub fn as_identifier(&self) -> Option<&str> {
        match self {
            Value::Identifier(s) => Some(s),
            _ => None,
        }
    }
}

/// A constructor call like `CRGBColour(40,40,40,255)`.
#[derive(Debug, Clone)]
pub struct Constructor {
    /// The constructor name (e.g., `CRGBColour`).
    pub name: String,
    /// The arguments.
    pub arguments: Vec<Value>,
}

// =============================================================================
// Parser
// =============================================================================

#[derive(Debug, Display, Error)]
pub enum ParseError {
    #[display("unexpected end of input")]
    UnexpectedEnd,
    #[display("unexpected character: {_0}")]
    UnexpectedChar(#[error(not(source))] char),
    #[display("expected {expected}, found {found}")]
    Expected {
        #[error(not(source))]
        expected: String,
        #[error(not(source))]
        found: String,
    },
    #[display("invalid number: {_0}")]
    InvalidNumber(#[error(not(source))] String),
    #[display("unterminated string")]
    UnterminatedString,
    #[display("mismatched tag: opened <{opened}>, closed <\\{closed}>")]
    MismatchedTag {
        #[error(not(source))]
        opened: String,
        #[error(not(source))]
        closed: String,
    },
}

/// Parser for def files.
pub struct DefParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> DefParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Parse a complete def file.
    pub fn parse_file(&mut self) -> Result<DefFile, ParseError> {
        let mut file = DefFile::default();

        loop {
            self.skip_whitespace_and_comments();
            if self.is_eof() {
                break;
            }

            let def = self.parse_definition()?;
            file.definitions.insert(def.name.clone(), def);
        }

        Ok(file)
    }

    /// Parse a single definition.
    fn parse_definition(&mut self) -> Result<Definition, ParseError> {
        // Parse #definition or #definition_template
        let is_template = if self.try_consume("#definition_template") {
            true
        } else if self.try_consume("#definition") {
            false
        } else {
            return Err(ParseError::Expected {
                expected: "#definition or #definition_template".into(),
                found: self.peek_word().unwrap_or("EOF".into()),
            });
        };

        self.skip_whitespace();

        // Parse type
        let def_type = self.parse_identifier()?;
        self.skip_whitespace();

        // Parse name
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Optional: specialises PARENT
        let specializes = if self.try_consume("specialises") {
            self.skip_whitespace();
            Some(self.parse_identifier()?)
        } else {
            None
        };

        // Parse body
        let mut body = Vec::new();
        loop {
            self.skip_whitespace_and_comments();

            if self.try_consume("#end_definition") {
                break;
            }

            if self.is_eof() {
                return Err(ParseError::Expected {
                    expected: "#end_definition".into(),
                    found: "EOF".into(),
                });
            }

            let stmt = self.parse_statement()?;
            body.push(stmt);
        }

        Ok(Definition {
            is_template,
            def_type,
            name,
            specializes,
            body,
        })
    }

    /// Parse a statement (assignment, method call, or tagged block).
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        self.skip_whitespace_and_comments();

        // Check for tagged block: <TagName>
        if self.peek_char() == Some('<') && self.peek_char_at(1) != Some('\\') {
            return self.parse_tagged_block().map(Statement::TaggedBlock);
        }

        // Parse path (could be assignment or method call)
        let path = self.parse_property_path()?;
        self.skip_whitespace();

        // Check if this is a method call: path.method(args);
        if self.peek_char() == Some('(') {
            // Extract method name from path
            let (object, method) = self.split_method_path(path)?;
            self.consume_char('(')?;
            let arguments = self.parse_arguments()?;
            self.consume_char(')')?;
            self.skip_whitespace();
            self.consume_char(';')?;

            return Ok(Statement::MethodCall(MethodCall {
                object,
                method,
                arguments,
            }));
        }

        // Otherwise it's an assignment: path value;
        let value = self.parse_value()?;
        self.skip_whitespace();
        self.consume_char(';')?;

        Ok(Statement::Assignment(Assignment { path, value }))
    }

    /// Parse a tagged block like `<CCreatureDef> ... <\CCreatureDef>`.
    fn parse_tagged_block(&mut self) -> Result<TaggedBlock, ParseError> {
        self.consume_char('<')?;
        let tag = self.parse_identifier()?;
        self.consume_char('>')?;

        let mut body = Vec::new();
        loop {
            self.skip_whitespace_and_comments();

            // Check for closing tag: <\TagName>
            if self.peek_char() == Some('<') && self.peek_char_at(1) == Some('\\') {
                self.consume_char('<')?;
                self.consume_char('\\')?;
                let close_tag = self.parse_identifier()?;
                self.consume_char('>')?;

                if close_tag != tag {
                    return Err(ParseError::MismatchedTag {
                        opened: tag,
                        closed: close_tag,
                    });
                }
                break;
            }

            if self.is_eof() {
                return Err(ParseError::Expected {
                    expected: format!("<\\{}>", tag),
                    found: "EOF".into(),
                });
            }

            let stmt = self.parse_statement()?;
            body.push(stmt);
        }

        Ok(TaggedBlock { tag, body })
    }

    /// Parse a property path like `Time[0].SkyTexture0`.
    fn parse_property_path(&mut self) -> Result<PropertyPath, ParseError> {
        let mut segments = Vec::new();

        // First segment is always a field
        let first = self.parse_identifier()?;
        segments.push(PathSegment::Field(first));

        loop {
            if self.peek_char() == Some('.') {
                self.consume_char('.')?;
                let field = self.parse_identifier()?;
                segments.push(PathSegment::Field(field));
            } else if self.peek_char() == Some('[') {
                self.consume_char('[')?;
                self.skip_whitespace();
                let idx = self.parse_integer()? as i32;
                self.skip_whitespace();
                self.consume_char(']')?;
                segments.push(PathSegment::Index(idx));
            } else {
                break;
            }
        }

        Ok(PropertyPath { segments })
    }

    /// Split a path into object and method name for method calls.
    fn split_method_path(&self, path: PropertyPath) -> Result<(PropertyPath, String), ParseError> {
        let mut segments = path.segments;
        if let Some(PathSegment::Field(method)) = segments.pop() {
            Ok((PropertyPath { segments }, method))
        } else {
            Err(ParseError::Expected {
                expected: "method name".into(),
                found: "index".into(),
            })
        }
    }

    /// Parse a value.
    fn parse_value(&mut self) -> Result<Value, ParseError> {
        let first = self.parse_value_term()?;

        self.skip_whitespace();

        // Check for bitwise OR
        if self.peek_char() == Some('|') {
            let mut terms = vec![first];
            while self.try_consume("|") {
                self.skip_whitespace();
                terms.push(self.parse_value_term()?);
                self.skip_whitespace();
            }
            return Ok(Value::BitwiseOr(terms));
        }

        Ok(first)
    }

    /// Parse a single value term (not including |).
    fn parse_value_term(&mut self) -> Result<Value, ParseError> {
        self.skip_whitespace();

        // String literal
        if self.peek_char() == Some('"') {
            return self.parse_string().map(Value::String);
        }

        // Number (possibly negative)
        if self.peek_char() == Some('-')
            || self.peek_char().map(|c| c.is_ascii_digit()) == Some(true)
        {
            return self.parse_number();
        }

        // Identifier, boolean, NULL, or constructor
        let ident = self.parse_identifier()?;

        // Check for TRUE/FALSE (also BTRUE/BFALSE variant)
        if ident == "TRUE" || ident == "BTRUE" {
            return Ok(Value::Bool(true));
        }
        if ident == "FALSE" || ident == "BFALSE" {
            return Ok(Value::Bool(false));
        }

        self.skip_whitespace();

        // Check for constructor call: Name(args)
        if self.peek_char() == Some('(') {
            self.consume_char('(')?;
            let arguments = self.parse_arguments()?;
            self.consume_char(')')?;
            return Ok(Value::Constructor(Constructor {
                name: ident,
                arguments,
            }));
        }

        Ok(Value::Identifier(ident))
    }

    /// Parse comma-separated arguments.
    fn parse_arguments(&mut self) -> Result<Vec<Value>, ParseError> {
        let mut args = Vec::new();

        self.skip_whitespace();
        if self.peek_char() == Some(')') {
            return Ok(args);
        }

        loop {
            self.skip_whitespace();
            args.push(self.parse_value()?);
            self.skip_whitespace();

            if self.try_consume(",") {
                continue;
            } else {
                break;
            }
        }

        Ok(args)
    }

    /// Parse a number (integer or float).
    fn parse_number(&mut self) -> Result<Value, ParseError> {
        let start = self.pos;
        let mut has_dot = false;
        let mut has_f_suffix = false;

        // Optional negative sign
        if self.peek_char() == Some('-') {
            self.pos += 1;
        }

        // Digits before decimal
        while self.peek_char().map(|c| c.is_ascii_digit()) == Some(true) {
            self.pos += 1;
        }

        // Optional decimal part
        if self.peek_char() == Some('.') {
            // Check it's followed by a digit (not a field access)
            if self.peek_char_at(1).map(|c| c.is_ascii_digit()) == Some(true) {
                has_dot = true;
                self.pos += 1;
                while self.peek_char().map(|c| c.is_ascii_digit()) == Some(true) {
                    self.pos += 1;
                }
            }
        }

        // Optional 'f' suffix for floats
        if self.peek_char() == Some('f') {
            has_f_suffix = true;
            self.pos += 1;
        }

        let text = &self.input[start..self.pos];

        if has_dot || has_f_suffix {
            let text = text.trim_end_matches('f');
            text.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| ParseError::InvalidNumber(text.into()))
        } else {
            text.parse::<i64>()
                .map(Value::Integer)
                .map_err(|_| ParseError::InvalidNumber(text.into()))
        }
    }

    /// Parse an integer.
    fn parse_integer(&mut self) -> Result<i64, ParseError> {
        let start = self.pos;

        if self.peek_char() == Some('-') {
            self.pos += 1;
        }

        while self.peek_char().map(|c| c.is_ascii_digit()) == Some(true) {
            self.pos += 1;
        }

        let text = &self.input[start..self.pos];
        text.parse::<i64>()
            .map_err(|_| ParseError::InvalidNumber(text.into()))
    }

    /// Parse a string literal.
    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.consume_char('"')?;
        let start = self.pos;

        while let Some(c) = self.peek_char() {
            if c == '"' {
                let s = self.input[start..self.pos].to_string();
                self.pos += 1;
                return Ok(s);
            }
            self.pos += 1;
        }

        Err(ParseError::UnterminatedString)
    }

    /// Parse an identifier.
    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let start = self.pos;

        // First character must be alphabetic or underscore
        match self.peek_char() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                self.pos += 1;
            }
            Some(c) => return Err(ParseError::UnexpectedChar(c)),
            None => return Err(ParseError::UnexpectedEnd),
        }

        // Rest can be alphanumeric or underscore
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }

        Ok(self.input[start..self.pos].to_string())
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek_char_at(&self, offset: usize) -> Option<char> {
        self.input[self.pos..].chars().nth(offset)
    }

    fn peek_word(&self) -> Option<String> {
        let rest = &self.input[self.pos..];
        let end = rest
            .find(|c: char| c.is_whitespace())
            .unwrap_or(rest.len())
            .min(20);
        if end > 0 {
            Some(rest[..end].to_string())
        } else {
            None
        }
    }

    fn consume_char(&mut self, expected: char) -> Result<(), ParseError> {
        match self.peek_char() {
            Some(c) if c == expected => {
                self.pos += c.len_utf8();
                Ok(())
            }
            Some(c) => Err(ParseError::Expected {
                expected: expected.to_string(),
                found: c.to_string(),
            }),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn try_consume(&mut self, s: &str) -> bool {
        if self.input[self.pos..].starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();

            // Skip // comments
            if self.input[self.pos..].starts_with("//") {
                while let Some(c) = self.peek_char() {
                    self.pos += 1;
                    if c == '\n' {
                        break;
                    }
                }
                continue;
            }

            break;
        }
    }
}

/// Parse a def file from a string.
pub fn parse_def_file(input: &str) -> Result<DefFile, ParseError> {
    let mut parser = DefParser::new(input);
    parser.parse_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_definition() {
        let input = r#"
#definition OBJECT TEST_OBJECT
    Health 100;
    Name "Test";
    IsEnabled TRUE;
#end_definition
"#;
        let file = parse_def_file(input).unwrap();
        assert_eq!(file.definitions.len(), 1);

        let def = &file.definitions["TEST_OBJECT"];
        assert_eq!(def.def_type, "OBJECT");
        assert_eq!(def.name, "TEST_OBJECT");
        assert!(!def.is_template);
        assert!(def.specializes.is_none());
        assert_eq!(def.body.len(), 3);
    }

    #[test]
    fn test_specializes() {
        let input = r#"
#definition OBJECT CHILD_OBJECT specialises PARENT_OBJECT
    Health 50;
#end_definition
"#;
        let file = parse_def_file(input).unwrap();
        let def = &file.definitions["CHILD_OBJECT"];
        assert_eq!(def.specializes, Some("PARENT_OBJECT".into()));
    }

    #[test]
    fn test_indexed_property() {
        let input = r#"
#definition ENVIRONMENT TEST_ENV
    Time[0].SkyTexture0 GRAPHIC_SKY;
    Time[1].FogEndZ 100;
#end_definition
"#;
        let file = parse_def_file(input).unwrap();
        let def = &file.definitions["TEST_ENV"];
        assert_eq!(def.body.len(), 2);
    }

    #[test]
    fn test_method_call() {
        let input = r#"
#definition OBJECT TEST_OBJECT
    Components.Add("CTCPhysicsStandard");
    NavigatorTypes.Add(NAV_INIT_GROUND);
#end_definition
"#;
        let file = parse_def_file(input).unwrap();
        let def = &file.definitions["TEST_OBJECT"];

        match &def.body[0] {
            Statement::MethodCall(mc) => {
                assert_eq!(mc.method, "Add");
                assert_eq!(mc.arguments.len(), 1);
            }
            _ => panic!("Expected method call"),
        }
    }

    #[test]
    fn test_tagged_block() {
        let input = r#"
#definition CREATURE TEST_CREATURE
    <CCreatureDef>
        Health 100;
    <\CCreatureDef>
#end_definition
"#;
        let file = parse_def_file(input).unwrap();
        let def = &file.definitions["TEST_CREATURE"];

        match &def.body[0] {
            Statement::TaggedBlock(tb) => {
                assert_eq!(tb.tag, "CCreatureDef");
                assert_eq!(tb.body.len(), 1);
            }
            _ => panic!("Expected tagged block"),
        }
    }

    #[test]
    fn test_constructor() {
        let input = r#"
#definition OBJECT TEST_OBJECT
    Color CRGBColour(255, 128, 64, 255);
#end_definition
"#;
        let file = parse_def_file(input).unwrap();
        let def = &file.definitions["TEST_OBJECT"];

        match &def.body[0] {
            Statement::Assignment(a) => match &a.value {
                Value::Constructor(c) => {
                    assert_eq!(c.name, "CRGBColour");
                    assert_eq!(c.arguments.len(), 4);
                }
                _ => panic!("Expected constructor"),
            },
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_bitwise_or() {
        let input = r#"
#definition OBJECT TEST_OBJECT
    Flags PHYSICS_COLLIDE_LANDSCAPE | PHYSICS_COLLIDE_THINGS;
#end_definition
"#;
        let file = parse_def_file(input).unwrap();
        let def = &file.definitions["TEST_OBJECT"];

        match &def.body[0] {
            Statement::Assignment(a) => match &a.value {
                Value::BitwiseOr(terms) => {
                    assert_eq!(terms.len(), 2);
                }
                _ => panic!("Expected bitwise or"),
            },
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_real_sky_theme() {
        // Real content from Fable's sky_theme.def
        let input = r#"
//Sky themes
//************************************************

#definition			SKY					SKY_DEF

		SunRadius						100;
		SunTexture 						GRAPHIC_ATMOSPHERIC_SUN;
		MoonRadius						300;
		MoonTexture 					GRAPHIC_ATMOSPHERIC_MOON;
		SunFlareRadius					32000;
		SunFlareTexture					GRAPHIC_ATMOSPHERIC_SUN_FLARE;
//		StarChartTexture				SKY_STAR_CHART;
		StarTexture						GRAPHIC_ATMOSPHERIC_STAR_01;
		StarChartTextureSize			400;
		StarSize						7;
		StarChartFilter					16;
		TwinkleInterval					100;
		TwinkleSpeed					4.0;
		TwinkleMin						0.0;
		TwinkleMax						4.0;

		FlareElements[0].Texture		GRAPHIC_ATMOSPHERIC_LENSFLARE_01;
		FlareElements[0].Radius			6000;
		FlareElements[0].Colour			CRGBColour(255, 128, 128, 128);
		FlareElements[0].Position		1.0;

		FlareElements[1].Texture		GRAPHIC_ATMOSPHERIC_LENSFLARE_02;
		FlareElements[1].Radius			2000;
		FlareElements[1].Colour			CRGBColour(255, 128, 128, 128);
		FlareElements[1].Position		0.8;

		FlareElements[9].Texture		GRAPHIC_ATMOSPHERIC_LENSFLARE_07;
		FlareElements[9].Radius			6000;
		FlareElements[9].Colour			CRGBColour(255, 128, 128, 128);
		FlareElements[9].Position		-1.4;
#end_definition
"#;
        let file = parse_def_file(input).unwrap();
        assert_eq!(file.definitions.len(), 1);

        let def = &file.definitions["SKY_DEF"];
        assert_eq!(def.def_type, "SKY");
        assert_eq!(def.name, "SKY_DEF");
        assert!(!def.is_template);

        // Check we parsed the indexed property assignments
        let mut found_flare = false;
        for stmt in &def.body {
            if let Statement::Assignment(a) = stmt {
                let path_str = a.path.to_string();
                if path_str == "FlareElements[0].Colour" {
                    found_flare = true;
                    match &a.value {
                        Value::Constructor(c) => {
                            assert_eq!(c.name, "CRGBColour");
                            assert_eq!(c.arguments.len(), 4);
                        }
                        _ => panic!("Expected constructor"),
                    }
                }
            }
        }
        assert!(found_flare, "Should have found FlareElements[0].Colour");
    }

    #[test]
    fn test_real_creature_with_tagged_blocks() {
        // Real content from Fable's creature_bandits.def
        let input = r#"
#definition		CREATURE	CREATURE_BANDIT_BASE
	<CCreatureStatsDef>
		Speed								0.0;
		Dexterity							1.0;
	<\CCreatureStatsDef>

	<CTurncoatDef>
		Turncoatable						BTRUE;
	<\CTurncoatDef>

	<CCreatureDef>
		Stats.ExperienceWorth		12;
		Stats.RenownWorth		6;
	<\CCreatureDef>
	GroupDef					G_CREATURES_BANDIT;
#end_definition
"#;
        let file = parse_def_file(input).unwrap();
        let def = &file.definitions["CREATURE_BANDIT_BASE"];
        assert_eq!(def.def_type, "CREATURE");

        // Should have 4 statements: 3 tagged blocks + 1 assignment
        assert_eq!(def.body.len(), 4);

        // Check the first tagged block
        match &def.body[0] {
            Statement::TaggedBlock(tb) => {
                assert_eq!(tb.tag, "CCreatureStatsDef");
                assert_eq!(tb.body.len(), 2); // Speed and Dexterity
            }
            _ => panic!("Expected tagged block"),
        }

        // Check BTRUE was parsed as bool
        match &def.body[1] {
            Statement::TaggedBlock(tb) => {
                assert_eq!(tb.tag, "CTurncoatDef");
                match &tb.body[0] {
                    Statement::Assignment(a) => {
                        assert_eq!(a.value.as_bool(), Some(true));
                    }
                    _ => panic!("Expected assignment"),
                }
            }
            _ => panic!("Expected tagged block"),
        }
    }
}
