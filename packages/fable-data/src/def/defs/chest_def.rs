use crate::def_struct;

def_struct! {
    /// `CChestDef` — C++ `CChestDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ChestDef {
        "OpenParticleEffect" => pub open_particle_effect: i32,
        "PersistOnOpening" => pub persist_on_opening: bool,
        "DisplayMessageOnEmpty" => pub display_message_on_empty: bool = true,
        "OpenAnimationForCreature" => pub open_animation_for_creature: String,
        "OpenerObject" => pub opener_object: i32,
        "OpenersRequired" => pub openers_required: i32,
    }
}
