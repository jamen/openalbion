use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CMazeBattleDef` — C++ `CMazeBattleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MazeBattleDef {
        "ShieldUpTime" => pub shield_up_time: f32,
        "LightningDuration" => pub lightning_duration: f32,
        "MinThrowShotRange" => pub min_throw_shot_range: f32,
        "EnflameRange" => pub enflame_range: f32,
        "LightningRange" => pub lightning_range: f32,
        "TutorialNumGetHits" => pub tutorial_num_get_hits: i32,
        "MinTimeBetweenHeals" => pub min_time_between_heals: f32,
        "HealSpellLevel" => pub heal_spell_level: i32,
        "TutorialTeleportTime" => pub tutorial_teleport_time: f32,
    }
}
