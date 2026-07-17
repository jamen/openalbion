use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CSmokeGeneratorDef` — C++ `CSmokeGeneratorDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SmokeGeneratorDef {
        "GenerationDistance" => pub generation_distance: f32,
        "ParticleEmitter" => pub particle_emitter: DefIndex,
    }
}
