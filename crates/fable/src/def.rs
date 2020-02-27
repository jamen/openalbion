pub mod decode;
pub mod encode;

use crate::script::{ScriptComment,ScriptExpression};

#[derive(Debug)]
pub struct Def {
    pub body: Vec<DefItem>,
}

#[derive(Debug)]
pub enum DefItem {
    Comment(ScriptComment),
    Definition(Definition),
}

#[derive(Debug)]
pub struct Definition {
    pub is_template: bool,
    pub group: String,
    pub name: String,
    pub specializes: Option<String>,
    pub body: Vec<ScriptExpression>,
}