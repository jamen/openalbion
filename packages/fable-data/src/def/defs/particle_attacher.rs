use crate::DefStruct;
use crate::def::values::ParticleAttachmentInfo;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ParticleAttacherDef {
    #[def("ParticlesToAttach")]
    pub particles_to_attach: Vec<ParticleAttachmentInfo>,
}
