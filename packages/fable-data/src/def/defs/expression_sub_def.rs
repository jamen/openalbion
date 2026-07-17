use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CExpressionSubDef` — C++ `CExpressionSubDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ExpressionSubDef {
        "ExpressionDef" => pub expression_def: i32,
    }
}
