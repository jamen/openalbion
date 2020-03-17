mod decode;
mod encode;

use crate::script::Expression;

#[derive(Debug)]
pub struct Def {
    pub body: Vec<DefItem>,
}

#[derive(Debug)]
pub enum DefItem {
    Between(String),
    Definition(Definition),
}

#[derive(Debug)]
pub struct Definition {
    pub is_template: bool,
    pub group: String,
    pub name: String,
    pub specializes: Option<String>,
    pub body: Vec<Expression>,
}