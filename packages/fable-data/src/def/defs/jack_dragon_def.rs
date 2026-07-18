use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CJackDragonDef` — C++ `CJackDragonDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct JackDragonDef {
        "MaxHitsBeforeFlyAway" => pub max_hits_before_fly_away: i32,
        "MinTimeBetweenSlams" => pub min_time_between_slams: f32,
        "MaxFirebeamDistance" => pub max_firebeam_distance: f32,
        "MinSnapDistance" => pub min_snap_distance: f32,
        "MaxSnapDistance" => pub max_snap_distance: f32,
        "MaxTimeAtLongRangeBeforeFlyAway" => pub max_time_at_long_range_before_fly_away: f32,
        "MaxSlamDistance" => pub max_slam_distance: f32,
        "MaxHoverLoops" => pub max_hover_loops: i32,
        "PercentChanceOfFakeLandingSwoop" => pub percent_chance_of_fake_landing_swoop: f32 = 50.0,
        "MaxTimeOnGround" => pub max_time_on_ground: f32,
        "DragonGroundHeightOffsetBodge" => pub dragon_ground_height_offset_bodge: f32,
        "HoverBackOffOffset" => pub hover_back_off_offset: f32,
        "FramesBetweenFireBeamUpdate" => pub frames_between_fire_beam_update: i32 = 4,
        "FramesBetweenBurninateUpdate" => pub frames_between_burninate_update: i32 = 2,
        "HoverObstructionRadius" => pub hover_obstruction_radius: f32,
        "StrafeTriggerDistance" => pub strafe_trigger_distance: f32 = 23.0,
        "GetHitSnapBuildupTime" => pub get_hit_snap_buildup_time: f32,
        "NotHitSnapBuildupTime" => pub not_hit_snap_buildup_time: f32,
        "MaxTimeBeforeSnap" => pub max_time_before_snap: f32,
        "MinRoarDistance" => pub min_roar_distance: f32,
    }
}
