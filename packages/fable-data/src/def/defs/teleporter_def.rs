use crate::DefStruct;
use crate::def::prelude::*;

/// `CTeleporterDef` — C++ `CTeleporterDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct TeleporterDef {
    #[def("Radius")]
    pub radius: f32,
    #[def("ActiveEffect")]
    pub active_effect: DefIndex,
    #[def("ActiveByProximity")]
    pub active_by_proximity: bool,
}
