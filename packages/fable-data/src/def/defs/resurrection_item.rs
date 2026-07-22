use crate::DefStruct;
use crate::def::{
    wire::DefIndex,
};


#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ResurrectionItemDef {
    #[def("OnUseParticleEffect")]
    pub on_use_particle_effect: DefIndex,
}
