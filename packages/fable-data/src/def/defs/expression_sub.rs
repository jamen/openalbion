use crate::DefStruct;
use crate::def::wire::DefIndex;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ExpressionSubDef {
    #[def("ExpressionDef")]
    pub expression_def: DefIndex,
}
