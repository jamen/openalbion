use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `BRAIN` — C++ `CBrainDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BrainDef {
        "UpdateZones" => pub update_zones: Vec<BrainUpdateZone>,
        "Behaviours" => pub behaviours: Vec<BrainBehaviour>,
        "DisabledBehaviours" => pub disabled_behaviours: Vec<DefString>,
    }
}
