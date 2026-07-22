use crate::DefStruct;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ExpressionSubDef {
    #[def("ExpressionDef")]
    pub expression_def: i32,
}
