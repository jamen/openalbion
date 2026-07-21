use crate::def_struct;

def_struct! {
    /// `CStealthDef` — C++ `CStealthDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct StealthDef {
        "DistanceMovementCanBeHeardFrom" => pub distance_movement_can_be_heard_from: f32,
        "RunningNoiseMultiplier" => pub running_noise_multiplier: f32 = 1.0,
        "JoggingNoiseMultiplier" => pub jogging_noise_multiplier: f32 = 1.0,
        "WalkingNoiseMultiplier" => pub walking_noise_multiplier: f32 = 1.0,
        "DayTimeLampVisibilityMultiplier" => pub day_time_lamp_visibility_multiplier: f32 = 1.0,
        "NightTimeLampVisibilityMultiplier" => pub night_time_lamp_visibility_multiplier: f32 = 1.0,
        "DangerFeedbackHangtimeSeconds" => pub danger_feedback_hangtime_seconds: f32 = 1.0,
        "RunningAwayFromCombatHangtimeSeconds" => pub running_away_from_combat_hangtime_seconds: f32 = 1.0,
    }
}
