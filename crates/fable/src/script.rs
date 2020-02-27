//! This is module for a scripting syntax shared in several formats.

pub mod decode;
pub mod encode;

#[derive(Debug,PartialEq)]
pub enum ScriptExpression {
    Value(ScriptValue),
    Field(ScriptField),
    Call(ScriptCall),
    Markup(ScriptMarkup),
    Comment(ScriptComment),
}

/// A property and value.
///
/// Random example:
///
/// ```txt
/// DialogueLayers[DIALOGUE_LAYER_BACKGROUND].ResponseTimeSecsAttack		4.0;
/// ```
#[derive(Debug,PartialEq)]
pub struct ScriptField {
    pub reference: ScriptReference,
    pub value: ScriptValue,
}

#[derive(Debug,PartialEq)]
pub enum ScriptReference {
    /// A name such as `A`
    Name(String),
    /// An accessor such as `A.B` or `A[0]`. It can also be chained further like `A[0].B`.
    Property((String, Vec<ScriptAccessor>)),
}

#[derive(Debug,PartialEq)]
pub enum ScriptAccessor {
    /// This is the `B` in `A.B`.
    Name(String),
    /// This is the `0` in `A[0]`.
    Index(i32),
    /// This is the `B` in `A[B]`.
    IndexName(String)
}

#[derive(Debug,PartialEq)]
pub enum ScriptValue {
    /// An empty value.
    None,
    /// A plaintext string such as `MESH_OBJECT_POINTER`.
    Name(String),
    /// A series of flags such as `A | B | C`.
    Flags(Vec<ScriptFlag>),
    Bool(bool),
    Number(i32),
    /// A big number (for 64-bit ids).
    BigNumber(u64),
    Float(f32),
    String(String),
}

#[derive(Debug,PartialEq)]
pub enum ScriptFlag {
    Name(String),
    Number(i32),
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