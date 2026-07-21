use crate::def_struct;

def_struct! {
    /// `CBriarRoseDef` — C++ `CBriarRoseDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BriarRoseDef {
        "SecondsToTeleport" => pub seconds_to_teleport: f32,
        "SecondsToBeIntangible" => pub seconds_to_be_intangible: f32,
        "SecondsToChargeDrainLife" => pub seconds_to_charge_drain_life: f32,
        "GuranteedImpostersHitBeforeAllowedToHitBriar" => pub guranteed_imposters_hit_before_allowed_to_hit_briar: i32,
        "NumberOfHitsToKillImposters" => pub number_of_hits_to_kill_imposters: i32,
        "EnvironmentThemeForIntangibleMode" => pub environment_theme_for_intangible_mode: i32,
        "SecondsForIntangibleEnvironmentThemeTransitionIn" => pub seconds_for_intangible_environment_theme_transition_in: f32,
        "SecondsForIntangibleEnvironmentThemeTransitionOut" => pub seconds_for_intangible_environment_theme_transition_out: f32,
        "SecondsBetweenWillUse" => pub seconds_between_will_use: f32,
        "SecondsBetweenTeleportAway" => pub seconds_between_teleport_away: f32,
        "DistanceForcePushThreshold" => pub distance_force_push_threshold: f32,
    }
}
