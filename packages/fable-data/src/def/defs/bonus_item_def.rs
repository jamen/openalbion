use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CBonusItemDef` — C++ `CBonusItemDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BonusItemDef {
        "HealthModifier" => pub health_modifier: f32,
        "AddsFatness" => pub adds_fatness: bool,
        "StaminaModifier" => pub stamina_modifier: i32,
        "SecondsToApplyHealthIncreaseOver" => pub seconds_to_apply_health_increase_over: f32,
        "WillExperience" => pub will_experience: i32,
        "SkillExperience" => pub skill_experience: i32,
        "StrengthExperience" => pub strength_experience: i32,
        "MoralityChange" => pub morality_change: i32,
        "AgreeablenessChange" => pub agreeableness_change: f32,
        "AttractivenessChange" => pub attractiveness_change: f32,
        "ScarinessChange" => pub scariness_change: f32,
        "MaxHealthIncrease" => pub max_health_increase: f32,
        "MaxStaminaIncrease" => pub max_stamina_increase: i32,
        "ChangeTimeOfDayToDay" => pub change_time_of_day_to_day: bool,
        "ChangeTimeOfDayToNight" => pub change_time_of_day_to_night: bool,
        "SpeedMultiplier" => pub speed_multiplier: f32,
        "SecondsForSpeedMultiplierToLast" => pub seconds_for_speed_multiplier_to_last: f32,
        "ParticleEffect" => pub particle_effect: i32,
        "AddsDrunkenness" => pub adds_drunkenness: bool,
        "AlcoholicStrength" => pub alcoholic_strength: f32,
    }
}
