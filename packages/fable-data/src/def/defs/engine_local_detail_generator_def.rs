use crate::DefStruct;
use crate::def::prelude::*;

/// `LOCAL_DETAIL_GENERATOR` — C++ `CEngineLocalDetailGeneratorDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct EngineLocalDetailGeneratorDef {
    #[def("Layers")]
    pub layers: Vec<EngineLocalDetailLayerDef>,
}
