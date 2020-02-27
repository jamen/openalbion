//! For user.ini, userst.ini, etc.

mod decode;
mod encode;

use crate::script::ScriptExpression;

// A list of calls, fields, and comments.
pub struct Ini {
    pub body: Vec<ScriptExpression>,
}