//! This is module for a scripting syntax shared in several formats.

mod decode;
mod encode;

pub use decode::*;
pub use encode::*;

#[derive(Debug,PartialEq)]
pub enum ScriptExpression {
    ScriptValue(ScriptValue),
    ScriptField(ScriptField),
    ScriptCall(ScriptCall),
    ScriptMarkup(ScriptMarkup),
    ScriptComment(ScriptComment),
    ScriptBinaryOp(ScriptBinaryOp),
}

/// The named part of calls and fields.
#[derive(Debug,PartialEq)]
pub enum ScriptReference {
    Name(String),
    ScriptAccessor(ScriptAccessor),
}

/// A complex reference containing another reference or expression.
#[derive(Debug,PartialEq)]
pub struct ScriptAccessor {
    pub name: String,
    pub path: Vec<ScriptAccessorPath>,
}

#[derive(Debug,PartialEq)]
pub enum ScriptAccessorPath {
    /// For the syntax `A.B`.
    Name(String),
    /// For the syntax `A[B]` and more complex expression.
    Expression(ScriptExpression),
}

/// Reference and expression.
///
/// Random example:
///
/// ```txt
/// DialogueLayers[DIALOGUE_LAYER_BACKGROUND].ResponseTimeSecsAttack		4.0;
/// ```
#[derive(Debug,PartialEq)]
pub struct ScriptField {
    pub reference: ScriptReference,
    pub value: Box<ScriptExpression>,
}

/// A value.
#[derive(Debug,PartialEq)]
pub enum ScriptValue {
    /// A plaintext string such as `MESH_OBJECT_POINTER`.
    Name(String),
    BoolLiteral(bool),
    IntegerLiteral(i32),
    /// A big integer for 64-bit ids.
    BigIntegerLiteral(u64),
    FloatLiteral(f32),
    StringLiteral(String),
}

/// A call.
///
/// Random example:
///
/// ```txt
/// PermittedAINarrators.Add("AF3_1_5");
/// ```
#[derive(Debug,PartialEq)]
pub struct ScriptCall {
    pub reference: ScriptReference,
    pub arguments: Vec<ScriptExpression>
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
pub struct ScriptMarkup {
    pub name: String,
    pub body: Vec<ScriptExpression>
}

/// A line or block comment.
#[derive(Debug,PartialEq)]
pub enum ScriptComment {
    Block(String),
    Line(String),
}

/// A binary operation.
#[derive(Debug,PartialEq)]
pub struct ScriptBinaryOp {
    pub kind: ScriptBinaryOpKind,
    pub lhs: Box<ScriptExpression>,
    pub rhs: Box<ScriptExpression>,
}

#[derive(Debug,PartialEq)]
pub enum ScriptBinaryOpKind {
    Add,
    Subtract,
    Multiply,
    Divide,
    BitOr,
    // TODO more?
}