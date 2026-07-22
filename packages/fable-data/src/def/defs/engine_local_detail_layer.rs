use crate::DefStruct;
use crate::def::defs::EngineLocalDetailObjectDef;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct EngineLocalDetailLayerDef {
    #[def("SpacingFromLayer")]
    pub spacing_from_layer: Vec<f32>,
    #[def("Objects")]
    pub objects: Vec<EngineLocalDetailObjectDef>,
}
