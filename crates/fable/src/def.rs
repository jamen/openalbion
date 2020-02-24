pub mod decode;
pub mod encode;

use crate::script::ScriptExpression;

pub struct Def {
    pub body: Vec<Definition>,
}

pub enum DefItem {
    ScriptExpression(ScriptExpression),
    Definition(Definition),
}

pub struct Definition {
    pub is_template: bool,
    pub category: String,
    pub name: String,
    pub specializes: Option<String>,
    pub body: Vec<ScriptExpression>,
}