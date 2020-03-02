mod decode;
mod encode;

use crate::script::Expression;

/// For user.ini, userst.ini, etc.
///
/// A list of calls, fields, and comments.
pub struct Ini {
    pub body: Vec<Expression>,
}