use crate::def_struct;

def_struct! {
    /// `SPECIAL_ABILITIES_ASSASSIN_RUSH_DEF` — C++ `CSpecialAbilitiesAssassinRushDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesAssassinRushDef {
        "DistanceAwayFromtargetToAssassinRushTo" => pub distance_away_fromtarget_to_assassin_rush_to: f32,
        "SafeCollideRadiusPercentage" => pub safe_collide_radius_percentage: f32,
        "DistFromFloor" => pub dist_from_floor: f32,
        "AssassinStartSeconds" => pub assassin_start_seconds: f32,
        "AssassinEndSeconds" => pub assassin_end_seconds: f32,
        "AssassinThreshold" => pub assassin_threshold: f32,
        "AssassinLength" => pub assassin_length: f32,
        "AssassinIntensity" => pub assassin_intensity: f32,
        "AssassinFade" => pub assassin_fade: f32,
        "AssassinRange" => pub assassin_range: f32,
        "AssassinTwist" => pub assassin_twist: f32,
        "RunSpeed" => pub run_speed: f32,
        "TrailDelaySecs" => pub trail_delay_secs: f32,
        "SlowTimeLength" => pub slow_time_length: f32,
        "AssassinAlpha" => pub assassin_alpha: i32,
        "SlowTimeMultiplier" => pub slow_time_multiplier: i32,
        "RushDelay" => pub rush_delay: f32,
        "DashDistance" => pub dash_distance: Vec<f32>,
    }
}
