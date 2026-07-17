use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CHeroMorphDef` — C++ `CHeroMorphDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroMorphDef {
        "TextureMorphs" => pub texture_morphs: TextureMorphs,
        "SkeletalMorphs" => pub skeletal_morphs: SkeletalMorphs,
        "ParticleMorphs" => pub particle_morphs: ParticleMorphs,
        "IdleParticleMorphs" => pub idle_particle_morphs: ParticleMorphs,
    }
}
