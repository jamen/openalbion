use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CPerceivedThingDef` — C++ `CPerceivedThingDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct PerceivedThingDef {
        "FieldOfView" => pub field_of_view: f32,
        "ExtendedSightRadius" => pub extended_sight_radius: f32,
        "SightRadius" => pub sight_radius: f32,
        "SoundRadius" => pub sound_radius: f32,
        "SmellRadius" => pub smell_radius: f32,
        "Type" => pub type_: PerceivedThingType,
        "AwarenessTimer" => pub awareness_timer: f32,
        "GiveUpChaseRadius" => pub give_up_chase_radius: f32,
        "DayTimeVisibilityMultiplier" => pub day_time_visibility_multiplier: f32,
        "NightTimeVisibilityMultiplier" => pub night_time_visibility_multiplier: f32,
    }
}
