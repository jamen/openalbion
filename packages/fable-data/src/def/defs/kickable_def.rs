use crate::DefStruct;
use crate::def::prelude::*;

/// `CKickableDef` — C++ `CKickableDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct KickableDef {
    #[def("BaseDistanceToTravel")]
    pub base_distance_to_travel: f32,
    #[def("StrengthScaleRange")]
    pub strength_scale_range: f32,
    #[def("SpeedScaleRange")]
    pub speed_scale_range: f32,
    #[def("BaseSpeedScaleRange")]
    pub base_speed_scale_range: f32,
    #[def("RandomBase")]
    pub random_base: f32,
    #[def("RandomRange")]
    pub random_range: f32,
    #[def("ShotToAttach")]
    pub shot_to_attach: i32,
    #[def("OpinionDeedType")]
    pub opinion_deed_type: OpinionDeedType,
    #[def("OpinionDeedFaction")]
    pub opinion_deed_faction: i32,
    #[def("OpinionDeedTypeToEnemies")]
    pub opinion_deed_type_to_enemies: OpinionDeedType,
    #[def("IgnoreNavigationTest")]
    pub ignore_navigation_test: bool,
}
