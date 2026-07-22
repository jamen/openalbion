use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SmashableDef {
    #[def("Smashable")]
    pub smashable: bool,
    #[def("ReplacementObject")]
    pub replacement_object: DefIndex,
    #[def("SmashParticleEmitter")]
    pub smash_particle_emitter: DefIndex,
}
