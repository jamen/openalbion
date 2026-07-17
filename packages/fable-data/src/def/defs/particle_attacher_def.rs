use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CParticleAttacherDef` — C++ `CParticleAttacherDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ParticleAttacherDef {
        "ParticlesToAttach" => pub particles_to_attach: Vec<ParticleAttachmentInfo>,
    }
}
