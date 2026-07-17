use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `LOCAL_DETAIL_GENERATOR` — C++ `CEngineLocalDetailGeneratorDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct EngineLocalDetailGeneratorDef {
        "Layers" => pub layers: Vec<EngineLocalDetailLayerDef>,
    }
}
