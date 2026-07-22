use crate::DefStruct;

/// `CDecapitationDef` — C++ `CDecapitationDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct DecapitationDef {
    #[def("BodyParticleEffect")]
    pub body_particle_effect: i32,
    #[def("Head")]
    pub head: i32,
}
