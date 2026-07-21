use crate::def_struct;

def_struct! {
    /// `CBuyHouseDef` — C++ `CBuyHouseDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BuyHouseDef {
        "Main" => pub main: bool = true,
    }
}
