use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCreatureGeneratorDef` — C++ `CCreatureGeneratorDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CreatureGeneratorDef {
        "RandomEffects" => pub random_effects: Vec<i32>,
        "RandomSoundCriteria" => pub random_sound_criteria: DefString,
        "SecondsBetweenEffects" => pub seconds_between_effects: i32,
        "SecondsBetweenSoundEffects" => pub seconds_between_sound_effects: i32,
    }
}
