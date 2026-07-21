use crate::def_struct;

def_struct! {
    /// `CDecapitationDef` — C++ `CDecapitationDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct DecapitationDef {
        "BodyParticleEffect" => pub body_particle_effect: i32,
        "Head" => pub head: i32,
    }
}
