use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTeleporterDef` — C++ `CTeleporterDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TeleporterDef {
        "Radius" => pub radius: f32,
        "ActiveEffect" => pub active_effect: DefIndex,
        "ActiveByProximity" => pub active_by_proximity: bool,
    }
}
