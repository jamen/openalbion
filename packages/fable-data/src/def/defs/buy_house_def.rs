use crate::DefStruct;

/// `CBuyHouseDef` — C++ `CBuyHouseDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct BuyHouseDef {
    #[def("Main", default = true)]
    pub main: bool,
}
