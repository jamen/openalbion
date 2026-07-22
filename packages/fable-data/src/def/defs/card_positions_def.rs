use crate::DefStruct;
use crate::def::prelude::*;

/// C++ `CCardPositionsDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct CardPositionsDef {
    #[def("Offset")]
    pub offset: Vec<Vector2D>,
}
