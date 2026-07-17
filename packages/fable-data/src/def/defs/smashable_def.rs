use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CSmashableDef` — C++ `CSmashableDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SmashableDef {
        "Smashable" => pub smashable: bool,
        "ReplacementObject" => pub replacement_object: DefIndex,
        "SmashParticleEmitter" => pub smash_particle_emitter: DefIndex,
    }
}
