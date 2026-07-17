use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CResurrectionItemDef` — C++ `CResurrectionItemDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ResurrectionItemDef {
        "OnUseParticleEffect" => pub on_use_particle_effect: DefIndex,
    }
}
