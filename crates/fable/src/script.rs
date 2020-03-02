//! This is module for a scripting syntax shared in several formats.

mod decode;
mod encode;

pub use decode::*;
pub use encode::*;

#[derive(Debug,PartialEq)]
pub enum Expression {
    /// A plaintext string such as `MESH_OBJECT_POINTER`.
    Name(String),
    BoolLiteral(bool),
    IntegerLiteral(i32),
    /// A big integer for 64-bit ids.
    BigIntegerLiteral(u64),
    FloatLiteral(f32),
    StringLiteral(String),
    Field(Field),
    Call(Call),
    Markup(Markup),
    Comment(Comment),
    BinaryOp(BinaryOp),
}

/// The named part of calls and fields.
#[derive(Debug,PartialEq)]
pub enum Reference {
    Name(String),
    Accessor(Accessor),
}

/// A complex reference containing another reference or expression.
#[derive(Debug,PartialEq)]
pub struct Accessor {
    pub name: String,
    pub path: Vec<AccessorPath>,
}

#[derive(Debug,PartialEq)]
pub enum AccessorPath {
    /// For the syntax `A.B`.
    Name(String),
    /// For the syntax `A[B]` and more complex expression.
    Expression(Expression),
}

/// Reference and expression.
///
/// Random example:
///
/// ```txt
/// DialogueLayers[DIALOGUE_LAYER_BACKGROUND].ResponseTimeSecsAttack		4.0;
/// ```
#[derive(Debug,PartialEq)]
pub struct Field {
    pub reference: Reference,
    pub value: Box<Expression>,
}

/// A call.
///
/// Random example:
///
/// ```txt
/// PermittedAINarrators.Add("AF3_1_5");
/// ```
#[derive(Debug,PartialEq)]
pub struct Call {
    pub reference: Reference,
    pub arguments: Vec<Expression>
}

/// XML-like markup (but not real XML).
///
/// Random example:
///
/// ```txt
/// <CHeroMarriageDef>
///	    //marriage event
///     WeddingTimeForScreenToFadeOut       4.0;
///     WeddingTimeForScreenToFadeIn        3.0;
///     ...
/// <\CHeroMarriageDef>
/// ```
#[derive(Debug,PartialEq)]
pub struct Markup {
    pub name: String,
    pub body: Vec<Expression>
}

/// A line or block comment.
#[derive(Debug,PartialEq)]
pub enum Comment {
    Block(String),
    Line(String),
}

/// A binary operation.
#[derive(Debug,PartialEq)]
pub struct BinaryOp {
    pub kind: BinaryOpKind,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

#[derive(Debug,PartialEq)]
pub enum BinaryOpKind {
    Add,
    Subtract,
    Multiply,
    Divide,
    BitOr,
    // TODO more?
}