use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CKickableDef` — C++ `CKickableDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct KickableDef {
        "BaseDistanceToTravel" => pub base_distance_to_travel: f32,
        "StrengthScaleRange" => pub strength_scale_range: f32,
        "SpeedScaleRange" => pub speed_scale_range: f32,
        "BaseSpeedScaleRange" => pub base_speed_scale_range: f32,
        "RandomBase" => pub random_base: f32,
        "RandomRange" => pub random_range: f32,
        "ShotToAttach" => pub shot_to_attach: i32,
        "OpinionDeedType" => pub opinion_deed_type: OpinionDeedType,
        "OpinionDeedFaction" => pub opinion_deed_faction: i32,
        "OpinionDeedTypeToEnemies" => pub opinion_deed_type_to_enemies: OpinionDeedType,
        "IgnoreNavigationTest" => pub ignore_navigation_test: bool,
    }
}
