//! The parser for Fable's level-text files: `.tng` (things) and `.wld` (world).
//!
//! These are a simplified member of the same family as `Defs/*.def`, so this is
//! modelled directly on `fable-defs`' `text` module — the same shape (a
//! payload-free token enum, a `Cursor`, recursive-descent productions returning
//! `Spanned` nodes), with the def-only machinery left out. What it keeps from
//! there is the important half: **the parser produces a faithful tree and never
//! interprets**. Numbers stay raw slices; deciding that `PositionX` is an `f32`
//! and `UID` a `u64` belongs to [`crate::tng`], not here.
//!
//! # The grammar
//!
//! ```text
//! body       := statement*
//! statement  := block | field | flag
//! block      := open kind? ';' body close ';'
//! field      := path value ';'
//! flag       := ident ';'
//! path       := ident ( '.' ident | '[' value ']' | '(' ')' )*
//! value      := number | string | TRUE | FALSE | ident | ident '(' args? ')'
//! args       := value ( ',' value )*
//! ```
//!
//! # Blocks
//!
//! There are two families, both read off the corpus and both traceable to how
//! the game writes these files:
//!
//! | Open | Kind | Close | Where |
//! |---|---|---|---|
//! | `NewThing <TypeName>;` | free-form type name | `EndThing;` | `.tng` |
//! | `XXXSectionStart <Name>;` | identifier | `XXXSectionEnd;` | `.tng` |
//! | `NewMap <n>;` / `NewRegion <n>;` | number | `EndMap;` / `EndRegion;` | `.wld` |
//! | `Start<Name>;` | — | `End<Name>;` | `.tng` components |
//! | `START_<NAME>;` | — | `END_<NAME>;` | `.wld` |
//!
//! The first family needs an explicit keyword table ([`KEYED_BLOCKS`]) because
//! `NewMap 1;` and `NewDisplayName "TXT_…";` are structurally identical — only
//! the keyword distinguishes an opener from an ordinary field. The second is a
//! prefix rule, which is what makes the 61 distinct `StartCTC*` component blocks
//! work without listing any of them.
//!
//! `NewThing`'s kind is taken as the **raw source slice** up to the `;`, not an
//! identifier, because the game writes `"NewThing " + CThing::GetTypeName()`
//! (`fablelib/thing_manager.cpp:5638`) and one of the nine type names in the
//! corpus is `Holy Site`.
//!
//! **The prefix rule only fires on a *bare* statement.** That is not a detail:
//! every `TrackNode` in the game carries top-level fields literally named
//! `Start` and `End` (`Start FALSE;` / `End FALSE;`, 319 of each), which would
//! otherwise open a block that never closes.

pub mod base;
pub mod lexer;

pub use self::base::{ParseContext, Span, Spanned, TextError, TextErrorKind};
pub use self::lexer::{Cursor, Lexer, Token, TokenKind, lex};

use std::fmt;

/// A sequence of statements: a whole file, or one block's contents.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Body<'a> {
    pub statements: Vec<Spanned<Statement<'a>>>,
}

/// One statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'a> {
    /// `Path value;`
    Field(Field<'a>),
    /// `Name;` — a name with no value and no block, e.g. `.wld`'s
    /// `AppearOnWorldMap;`.
    Flag(&'a str),
    /// `Open kind?; … Close;`
    Block(Block<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field<'a> {
    pub path: Path<'a>,
    pub value: Spanned<Value<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block<'a> {
    /// The opening keyword: `NewThing`, `XXXSectionStart`, `StartCTCPhysicsStandard`.
    pub keyword: &'a str,
    /// The text between the keyword and the `;`, verbatim — `Object`,
    /// `Holy Site`, `NULL`, `1`. `None` for the `Start…`/`START_…` family, whose
    /// name is the keyword itself.
    pub kind: Option<Spanned<&'a str>>,
    pub body: Body<'a>,
}

/// A field name, possibly qualified and indexed: `PositionX`,
/// `KeyCameras[0].Position`, `MiniMapRegionExitTextOffsetX[HeroGuildComplexInside]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Path<'a> {
    pub segments: Vec<PathSegment<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment<'a> {
    Field(&'a str),
    Index(Spanned<Value<'a>>),
    /// A zero-argument call inside the path: `Shape[0].size() 10;`.
    ///
    /// The persist context names a field by the C++ expression that reaches it,
    /// and a shape's point count is reached through `std::vector::size()`. The
    /// only one in the corpus (47 occurrences, all `Shape[N].size()`), and the
    /// only place parentheses appear anywhere other than a constructor value —
    /// so an argument list here is rejected rather than guessed at.
    Call(&'a str),
}

/// A leaf value, uninterpreted.
#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    /// The raw literal slice. Kept as text because the corpus mixes `f32`
    /// coordinates with `u64` UIDs that overflow every signed type, and only the
    /// receiving field knows which it is.
    Number(&'a str),
    Bool(bool),
    /// Unquoted contents. No escape processing: `"FinalAlbion\LookoutPoint.lev"`
    /// contains a literal backslash.
    String(&'a str),
    /// A bare identifier used as a value: `ScriptName GuardTrack;`.
    Symbol(&'a str),
    /// `C3DCoordF(x, y, z)`, `C2DCoordF(x, y)`, `CRGBColour(r, g, b, a)`.
    Call(Call<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call<'a> {
    pub name: &'a str,
    pub args: Vec<Spanned<Value<'a>>>,
}

// ── Accessors ─────────────────────────────────────────────────────────────────

impl<'a> Path<'a> {
    /// The path when it is a single unindexed name — the overwhelmingly common
    /// case, and the one field lookups match on.
    pub fn as_name(&self) -> Option<&'a str> {
        match self.segments.as_slice() {
            [PathSegment::Field(name)] => Some(name),
            _ => None,
        }
    }
}

impl fmt::Display for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, segment) in self.segments.iter().enumerate() {
            match segment {
                PathSegment::Field(name) => {
                    if i > 0 {
                        f.write_str(".")?;
                    }
                    f.write_str(name)?;
                }
                PathSegment::Index(index) => write!(f, "[{}]", index.value)?,
                PathSegment::Call(name) => {
                    if i > 0 {
                        f.write_str(".")?;
                    }
                    write!(f, "{name}()")?;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(s) | Value::Symbol(s) => f.write_str(s),
            Value::Bool(b) => f.write_str(if *b { "TRUE" } else { "FALSE" }),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Call(call) => {
                write!(f, "{}(", call.name)?;
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, "{}", arg.value)?;
                }
                f.write_str(")")
            }
        }
    }
}

impl<'a> Value<'a> {
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Value::Number(n) => n.parse().ok(),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::Number(n) => n.parse().ok(),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Number(n) => n.parse().ok(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// A quoted string or a bare identifier, which the format uses
    /// interchangeably — `ScriptName GuardTrack;` next to `ScriptData "NULL";`.
    pub fn as_str(&self) -> Option<&'a str> {
        match self {
            Value::String(s) | Value::Symbol(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_call(&self) -> Option<&Call<'a>> {
        match self {
            Value::Call(call) => Some(call),
            _ => None,
        }
    }
}

impl<'a> Body<'a> {
    pub fn fields(&self) -> impl Iterator<Item = &Field<'a>> {
        self.statements.iter().filter_map(|s| match &s.value {
            Statement::Field(field) => Some(field),
            _ => None,
        })
    }

    pub fn blocks(&self) -> impl Iterator<Item = &Block<'a>> {
        self.statements.iter().filter_map(|s| match &s.value {
            Statement::Block(block) => Some(block),
            _ => None,
        })
    }

    pub fn flags(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.statements.iter().filter_map(|s| match &s.value {
            Statement::Flag(name) => Some(*name),
            _ => None,
        })
    }

    /// The **last** value assigned to a single-segment field called `name`.
    ///
    /// Last, not first: the game's reader assigns as it goes
    /// (`CTCPhysicsStandard::OnSerialise`, `tc_physics_standard.cpp:471`), so a
    /// repeated key overwrites. That is not hypothetical — all 319 `TrackNode`
    /// things in the game write `ScriptName` twice.
    pub fn field(&self, name: &str) -> Option<&Spanned<Value<'a>>> {
        self.fields()
            .filter(|f| f.path.as_name() == Some(name))
            .map(|f| &f.value)
            .last()
    }

    /// Every block opened with `keyword`, in file order.
    pub fn blocks_named(&self, keyword: &str) -> impl Iterator<Item = &Block<'a>> {
        self.blocks().filter(move |b| b.keyword == keyword)
    }

    /// The first block opened with `keyword`.
    pub fn block(&self, keyword: &str) -> Option<&Block<'a>> {
        self.blocks_named(keyword).next()
    }
}

// ── Block keywords ────────────────────────────────────────────────────────────

/// Openers that take a kind, paired with their closer.
///
/// An explicit table, because `NewMap 1;` (a block) and `NewDisplayName "…";` (a
/// field) are the same shape. These four are structural keywords of the format,
/// not content: the open-ended part of the grammar is the `Start…`/`START_…`
/// family below, which needs no table.
const KEYED_BLOCKS: &[(&str, &str)] = &[
    ("NewThing", "EndThing"),
    ("XXXSectionStart", "XXXSectionEnd"),
    ("NewMap", "EndMap"),
    ("NewRegion", "EndRegion"),
];

fn keyed_block_close(keyword: &str) -> Option<&'static str> {
    KEYED_BLOCKS
        .iter()
        .find(|(open, _)| *open == keyword)
        .map(|(_, close)| *close)
}

/// The closer for a bare `Start…;` / `START_…;` opener.
///
/// Case matters and the two prefixes are checked separately:
/// `START_INITIAL_QUESTS` does not begin with `Start`.
fn prefix_block_close(keyword: &str) -> Option<String> {
    if let Some(rest) = keyword.strip_prefix("START_") {
        return Some(format!("END_{rest}"));
    }
    keyword
        .strip_prefix("Start")
        .map(|rest| format!("End{rest}"))
}

/// Whether a bare name reads as a block closer — used to turn an unbalanced file
/// into a precise error instead of a stray flag.
///
/// Only ever consulted for *bare* statements, so the `TrackNode` fields `Start`
/// and `End` (which carry values) never reach it.
fn looks_like_close(name: &str) -> bool {
    name == "XXXSectionEnd" || name.starts_with("End") || name.starts_with("END_")
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Parse a whole `.tng` or `.wld`.
pub fn parse(input: &str) -> Result<Body<'_>, TextError> {
    parse_inner(input).map_err(|e| e.locate(input))
}

fn parse_inner(input: &str) -> Result<Body<'_>, TextError> {
    let tokens = lex(input)?;
    let mut cursor = Cursor::new(input, tokens);
    let body = parse_body(&mut cursor, None)?;
    if !cursor.at(TokenKind::Eof) {
        return Err(cursor.unexpected("a statement"));
    }
    Ok(body)
}

// ── Productions ───────────────────────────────────────────────────────────────

/// The block a body is inside: the keyword that opened it and the one that has
/// to close it. `None` at the top level, where EOF is the terminator.
#[derive(Copy, Clone)]
struct Enclosing<'a> {
    opener: &'a str,
    closer: &'a str,
}

/// Statements up to the enclosing block's closer (or to EOF at the top level).
fn parse_body<'a>(
    cursor: &mut Cursor<'a>,
    enclosing: Option<Enclosing<'_>>,
) -> Result<Body<'a>, TextError> {
    let mut statements = Vec::new();
    loop {
        if cursor.at(TokenKind::Eof) {
            return match enclosing {
                Some(block) => Err(cursor.err(TextErrorKind::UnclosedBlock {
                    opened: block.opener.to_string(),
                    expected: block.closer.to_string(),
                })),
                None => Ok(Body { statements }),
            };
        }

        // A bare `Name;` is the only shape a closer can take.
        let token = cursor.peek();
        if token.kind == TokenKind::Ident && cursor.peek_at(1).kind == TokenKind::Semi {
            if enclosing.is_some_and(|block| block.closer == token.source) {
                cursor.bump(); // name
                cursor.bump(); // `;`
                return Ok(Body { statements });
            }
            if looks_like_close(token.source) {
                return Err(cursor.err(TextErrorKind::MismatchedClose {
                    found: token.source.to_string(),
                    expected: enclosing.map(|block| block.closer.to_string()),
                }));
            }
        }

        statements.push(parse_statement(cursor)?);
    }
}

fn parse_statement<'a>(cursor: &mut Cursor<'a>) -> Result<Spanned<Statement<'a>>, TextError> {
    let start = cursor.peek().span.start;

    // A keyed opener has to be recognised before the path production, which
    // would otherwise read `NewThing Object;` as the field `NewThing = Object`.
    let token = cursor.peek();
    if token.kind == TokenKind::Ident
        && let Some(close) = keyed_block_close(token.source)
    {
        cursor.bump();
        let block = parse_block(cursor, token.source, true, close.to_string())?;
        return Ok(Spanned {
            span: Span::new(start, cursor.prev_end()),
            value: Statement::Block(block),
        });
    }

    let path = parse_path(cursor)?;

    if cursor.at(TokenKind::Semi) {
        // Only a plain name can stand alone; a qualified or indexed path with no
        // value would be a method-call statement, which this grammar does not have.
        let Some(name) = path.as_name() else {
            return Err(cursor.unexpected("a value"));
        };
        cursor.bump();
        // A bare name: a component block if it is `Start…`/`START_…`, else a flag.
        if let Some(close) = prefix_block_close(name) {
            let block = parse_block(cursor, name, false, close)?;
            return Ok(Spanned {
                span: Span::new(start, cursor.prev_end()),
                value: Statement::Block(block),
            });
        }
        return Ok(Spanned {
            span: Span::new(start, cursor.prev_end()),
            value: Statement::Flag(name),
        });
    }

    let value = parse_value(cursor)?;
    cursor.expect(TokenKind::Semi)?;
    Ok(Spanned {
        span: Span::new(start, cursor.prev_end()),
        value: Statement::Field(Field { path, value }),
    })
}

/// A block's kind (if it takes one) and body. The opening keyword is already
/// consumed; for a keyed block the `;` has not been.
fn parse_block<'a>(
    cursor: &mut Cursor<'a>,
    keyword: &'a str,
    keyed: bool,
    close: String,
) -> Result<Block<'a>, TextError> {
    let kind = if keyed {
        // Everything up to the `;`, verbatim — `Object`, `Holy Site`, `1`.
        let start = cursor.peek().span.start;
        while !cursor.at(TokenKind::Semi) {
            if cursor.at(TokenKind::Eof) {
                return Err(cursor.unexpected("`;`"));
            }
            cursor.bump();
        }
        let end = cursor.prev_end();
        cursor.bump(); // `;`
        (end > start).then(|| Spanned {
            span: Span::new(start, end),
            value: cursor.slice(start, end),
        })
    } else {
        None
    };

    let context = ParseContext::new(
        if keyword == "NewThing" { "thing" } else { "block" },
        Some(kind.map_or_else(|| keyword.to_string(), |k| k.value.to_string())),
    );
    let enclosing = Enclosing {
        opener: keyword,
        closer: &close,
    };
    let body = parse_body(cursor, Some(enclosing)).map_err(|e| e.within(&context))?;

    Ok(Block {
        keyword,
        kind,
        body,
    })
}

fn parse_path<'a>(cursor: &mut Cursor<'a>) -> Result<Path<'a>, TextError> {
    let mut segments = vec![PathSegment::Field(cursor.expect_ident("a field name")?)];
    loop {
        if cursor.at(TokenKind::Dot) {
            cursor.bump();
            segments.push(PathSegment::Field(cursor.expect_ident("a field name")?));
        } else if cursor.at(TokenKind::LBracket) {
            cursor.bump();
            let index = parse_value(cursor)?;
            cursor.expect(TokenKind::RBracket)?;
            segments.push(PathSegment::Index(index));
        } else if cursor.at(TokenKind::LParen) {
            cursor.bump();
            cursor.expect(TokenKind::RParen)?;
            // The `(` follows the segment just read, which turns it into a call.
            match segments.pop() {
                Some(PathSegment::Field(name)) => segments.push(PathSegment::Call(name)),
                _ => return Err(cursor.unexpected("a method name before `()`")),
            }
        } else {
            return Ok(Path { segments });
        }
    }
}

fn parse_value<'a>(cursor: &mut Cursor<'a>) -> Result<Spanned<Value<'a>>, TextError> {
    let token = cursor.peek();
    match token.kind {
        TokenKind::Number => {
            cursor.bump();
            Ok(Spanned {
                span: token.span,
                value: Value::Number(token.source),
            })
        }
        TokenKind::Str => {
            cursor.bump();
            Ok(Spanned {
                span: token.span,
                // The lexer guarantees both quotes are present.
                value: Value::String(&token.source[1..token.source.len() - 1]),
            })
        }
        TokenKind::Ident => {
            cursor.bump();
            let value = match token.source {
                "TRUE" => Value::Bool(true),
                "FALSE" => Value::Bool(false),
                name if cursor.at(TokenKind::LParen) => {
                    cursor.bump();
                    let args = parse_args(cursor)?;
                    cursor.expect(TokenKind::RParen)?;
                    Value::Call(Call { name, args })
                }
                name => Value::Symbol(name),
            };
            Ok(Spanned {
                span: Span::new(token.span.start, cursor.prev_end()),
                value,
            })
        }
        _ => Err(cursor.unexpected("a value")),
    }
}

fn parse_args<'a>(cursor: &mut Cursor<'a>) -> Result<Vec<Spanned<Value<'a>>>, TextError> {
    let mut args = Vec::new();
    if cursor.at(TokenKind::RParen) {
        return Ok(args);
    }
    loop {
        args.push(parse_value(cursor)?);
        if cursor.at(TokenKind::Comma) {
            cursor.bump();
        } else {
            return Ok(args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(input: &str) -> Body<'_> {
        parse(input).expect("parse ok")
    }

    fn err(input: &str) -> TextError {
        parse(input).expect_err("expected a parse error")
    }

    fn only_block<'a>(body: &'a Body<'a>) -> &'a Block<'a> {
        body.blocks().next().expect("a block")
    }

    // ── values ────────────────────────────────────────────────────────────────

    #[test]
    fn value_shapes() {
        let b = body(
            r#"N 42; F -0.5; U 18446741874686301490; S "text"; I Ident; T TRUE; X FALSE;"#,
        );
        assert_eq!(b.field("N").unwrap().value.as_i32(), Some(42));
        assert_eq!(b.field("F").unwrap().value.as_f32(), Some(-0.5));
        assert_eq!(
            b.field("U").unwrap().value.as_u64(),
            Some(18446741874686301490)
        );
        assert_eq!(b.field("S").unwrap().value.as_str(), Some("text"));
        assert_eq!(b.field("I").unwrap().value.as_str(), Some("Ident"));
        assert_eq!(b.field("T").unwrap().value.as_bool(), Some(true));
        assert_eq!(b.field("X").unwrap().value.as_bool(), Some(false));
    }

    #[test]
    fn constructor_value() {
        let b = body("CoordBase C3DCoordF(-3359.448242,-3569.195801,-29.327202);");
        let call = b.field("CoordBase").unwrap().value.as_call().unwrap();
        assert_eq!(call.name, "C3DCoordF");
        assert_eq!(call.args.len(), 3);
        assert_eq!(call.args[2].value.as_f32(), Some(-29.327202));
    }

    #[test]
    fn empty_constructor() {
        let b = body("C CRGBColour();");
        assert!(b.field("C").unwrap().value.as_call().unwrap().args.is_empty());
    }

    #[test]
    fn indexed_and_qualified_paths() {
        let b = body(
            "KeyCameras[0].Position C3DCoordF(1.0,2.0,3.0);\
             \nMiniMapRegionExitTextOffsetX[HeroGuildComplexInside] 0.0;",
        );
        let paths: Vec<String> = b.fields().map(|f| f.path.to_string()).collect();
        assert_eq!(
            paths,
            vec![
                "KeyCameras[0].Position",
                "MiniMapRegionExitTextOffsetX[HeroGuildComplexInside]",
            ]
        );
        // An indexed path is not a plain name, so it never collides with one.
        assert!(b.field("KeyCameras").is_none());
    }

    #[test]
    fn call_in_a_path() {
        // `StartCTCShapeManager` writes a shape's point count through
        // `std::vector::size()`, which lands in the field path.
        let b = body("Shape[0].size() 10;");
        let field = b.fields().next().unwrap();
        assert_eq!(field.path.to_string(), "Shape[0].size()");
        assert_eq!(field.value.value.as_i32(), Some(10));
        assert!(matches!(
            field.path.segments.as_slice(),
            [
                PathSegment::Field("Shape"),
                PathSegment::Index(_),
                PathSegment::Call("size"),
            ]
        ));
    }

    #[test]
    fn call_arguments_in_a_path_are_rejected() {
        // Nothing in the corpus passes arguments here, so the grammar says so
        // rather than inventing a meaning for them.
        assert!(matches!(
            err("Shape[0].resize(4) 10;").kind,
            TextErrorKind::UnexpectedToken { .. }
        ));
    }

    // ── blocks ────────────────────────────────────────────────────────────────

    #[test]
    fn thing_block_with_component() {
        let b = body(
            "NewThing Object;\nDefinitionType \"OBJECT_X\";\n\
             StartCTCPhysicsStandard;\nPositionX 1.5;\nEndCTCPhysicsStandard;\nEndThing;",
        );
        let thing = only_block(&b);
        assert_eq!(thing.keyword, "NewThing");
        assert_eq!(thing.kind.map(|k| k.value), Some("Object"));
        assert_eq!(
            thing.body.field("DefinitionType").unwrap().value.as_str(),
            Some("OBJECT_X")
        );
        let physics = thing.body.block("StartCTCPhysicsStandard").unwrap();
        assert_eq!(physics.kind, None);
        assert_eq!(physics.body.field("PositionX").unwrap().value.as_f32(), Some(1.5));
    }

    #[test]
    fn thing_kind_may_contain_a_space() {
        // `NewThing Holy Site;` — the kind is `CThing::GetTypeName()`, not an
        // identifier (thing_manager.cpp:5638). 128 things in the game are these.
        let b = body("NewThing Holy Site;\nHealth 1.0;\nEndThing;");
        assert_eq!(only_block(&b).kind.map(|k| k.value), Some("Holy Site"));
    }

    #[test]
    fn numeric_block_kind() {
        let b = body("NewMap 1;\nMapX 3232;\nEndMap;");
        let map = only_block(&b);
        assert_eq!(map.keyword, "NewMap");
        assert_eq!(map.kind.map(|k| k.value), Some("1"));
        assert_eq!(map.body.field("MapX").unwrap().value.as_i32(), Some(3232));
    }

    #[test]
    fn underscore_prefixed_block() {
        // `.wld`: START_INITIAL_QUESTS does not begin with `Start`, so the two
        // prefixes are matched separately.
        let b = body("START_INITIAL_QUESTS;\nQ_Sunnyvale;\nEND_INITIAL_QUESTS;");
        let quests = only_block(&b);
        assert_eq!(quests.keyword, "START_INITIAL_QUESTS");
        assert_eq!(quests.body.flags().collect::<Vec<_>>(), vec!["Q_Sunnyvale"]);
    }

    #[test]
    fn flags_are_not_blocks() {
        let b = body("NewRegion 1;\nAppearOnWorldMap;\nEndRegion;");
        assert_eq!(
            only_block(&b).body.flags().collect::<Vec<_>>(),
            vec!["AppearOnWorldMap"]
        );
    }

    #[test]
    fn start_and_end_as_ordinary_fields() {
        // Every TrackNode writes `Start FALSE;` / `End FALSE;` at thing level.
        // The block rule fires on bare names only, so these stay fields.
        let b = body("NewThing TrackNode;\nStart FALSE;\nEnd FALSE;\nEndThing;");
        let thing = only_block(&b);
        assert_eq!(thing.body.field("Start").unwrap().value.as_bool(), Some(false));
        assert_eq!(thing.body.field("End").unwrap().value.as_bool(), Some(false));
        assert_eq!(thing.body.blocks().count(), 0);
    }

    #[test]
    fn repeated_field_takes_the_last() {
        // TrackNodes write ScriptName twice; the game's reader assigns as it goes.
        let b = body("ScriptName First;\nScriptName Second;");
        assert_eq!(b.field("ScriptName").unwrap().value.as_str(), Some("Second"));
    }

    #[test]
    fn sections_nest_things() {
        let b = body(
            "Version 2;\nXXXSectionStart Gameflow;\n\
             NewThing Marker;\nEndThing;\nNewThing Object;\nEndThing;\n\
             XXXSectionEnd;",
        );
        assert_eq!(b.field("Version").unwrap().value.as_i32(), Some(2));
        let section = only_block(&b);
        assert_eq!(section.kind.map(|k| k.value), Some("Gameflow"));
        assert_eq!(section.body.blocks().count(), 2);
    }

    // ── errors ────────────────────────────────────────────────────────────────

    #[test]
    fn unclosed_block_reports_the_opener() {
        let e = err("NewThing Object;\nHealth 1.0;\n");
        assert_eq!(
            e.kind,
            TextErrorKind::UnclosedBlock {
                opened: "NewThing".into(),
                expected: "EndThing".into(),
            }
        );
        // The opener is the keyword actually written, not one derived from the
        // closer — `NewThing` closes with `EndThing`, and the two do not rhyme.
        assert_eq!(
            err("StartCTCEditor;\n").kind,
            TextErrorKind::UnclosedBlock {
                opened: "StartCTCEditor".into(),
                expected: "EndCTCEditor".into(),
            }
        );
    }

    #[test]
    fn mismatched_close_names_both_sides() {
        let e = err("NewThing Object;\nStartCTCEditor;\nEndCTCPhysicsStandard;\nEndThing;");
        assert_eq!(
            e.kind,
            TextErrorKind::MismatchedClose {
                found: "EndCTCPhysicsStandard".into(),
                expected: Some("EndCTCEditor".into()),
            }
        );
    }

    #[test]
    fn stray_close_at_top_level() {
        let e = err("EndThing;");
        assert_eq!(
            e.kind,
            TextErrorKind::MismatchedClose {
                found: "EndThing".into(),
                expected: None,
            }
        );
    }

    #[test]
    fn error_carries_line_column_and_context() {
        // A missing `;` — the error is reported on the token that should have
        // been one, and names the thing it happened inside.
        let e = err("NewThing Object;\nHealth 1.0\nEndThing;");
        assert_eq!((e.line, e.column), (3, 1));
        assert_eq!(
            e.context(),
            Some(&ParseContext::new("thing", Some("Object".into())))
        );
        assert_eq!(
            e.to_string(),
            "3:1: expected `;`, found identifier (in thing `Object`)"
        );
    }

    #[test]
    fn a_name_with_no_value_is_a_flag_not_an_error() {
        // `Name;` is a legal statement, so an accidentally empty field reads as
        // a flag rather than failing. `.wld`'s `AppearOnWorldMap;` is the real use.
        let b = body("Health ;");
        assert_eq!(b.flags().collect::<Vec<_>>(), vec!["Health"]);
    }

    #[test]
    fn empty_input_is_an_empty_body() {
        assert_eq!(body("").statements.len(), 0);
    }
}
