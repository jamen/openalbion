use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CCardPositionsDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct CardPositionsDef {
        "Offset" => pub offset: Vec<Vector2D>,
    }
}
