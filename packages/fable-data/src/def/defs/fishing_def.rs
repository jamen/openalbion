use crate::def_struct;

def_struct! {
    /// `CFishingDef` — C++ `CFishingDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FishingDef {
        "FishTypes" => pub fish_types: Vec<i32>,
        "FishingSpotProximity" => pub fishing_spot_proximity: f32,
        "FishingSpotMinDepth" => pub fishing_spot_min_depth: f32,
        "FishingSpotMinHeightDiff" => pub fishing_spot_min_height_diff: f32,
        "FishingSpotMaxHeightDiff" => pub fishing_spot_max_height_diff: f32,
        "PointsPerFishingLevel" => pub points_per_fishing_level: i32,
        "SlowFishingSpeed" => pub slow_fishing_speed: f32,
        "NormalFishingSpeed" => pub normal_fishing_speed: f32,
        "FastFishingSpeed" => pub fast_fishing_speed: f32,
        "FastestFishingSpeed" => pub fastest_fishing_speed: f32,
        "FishWeightRainMult" => pub fish_weight_rain_mult: f32,
        "FishWeightSnowMult" => pub fish_weight_snow_mult: f32,
        "FishWeightMistMult" => pub fish_weight_mist_mult: f32,
        "BaseWaitPeriodSeconds" => pub base_wait_period_seconds: f32,
        "MaxRandomWaitPeriodSeconds" => pub max_random_wait_period_seconds: f32,
        "HookWindowSeconds" => pub hook_window_seconds: f32,
        "BaseHookPresses" => pub base_hook_presses: i32,
        "MaxRandomHookPresses" => pub max_random_hook_presses: i32,
        "StrainIncreasePerPress" => pub strain_increase_per_press: f32,
    }
}
