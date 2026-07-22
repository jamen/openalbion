use crate::DefStruct;
use crate::def::values::Vector2D;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct CardPositionsDef {
    #[def("Offset")]
    pub offset: Vec<Vector2D>,
}
