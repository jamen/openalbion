use crate::DefStruct;
use crate::def::prelude::*;

/// `BRAIN` — C++ `CBrainDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct BrainDef {
    #[def("UpdateZones")]
    pub update_zones: Vec<BrainUpdateZone>,
    #[def("Behaviours")]
    pub behaviours: Vec<BrainBehaviour>,
    #[def("DisabledBehaviours")]
    pub disabled_behaviours: Vec<DefString>,
}
