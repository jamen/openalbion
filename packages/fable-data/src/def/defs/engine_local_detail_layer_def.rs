use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CEngineLocalDetailLayerDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct EngineLocalDetailLayerDef {
        "SpacingFromLayer" => pub spacing_from_layer: Vec<f32>,
        "Objects" => pub objects: Vec<EngineLocalDetailObjectDef>,
    }
}
