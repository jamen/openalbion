use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CDragonActionSwoopDef` — C++ `CDragonActionSwoopDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct DragonActionSwoopDef {
        "IntoDelay" => pub into_delay: f32,
        "OutOfDelay" => pub out_of_delay: f32,
        "Distance" => pub distance: f32,
        "MidPointsControlOffset" => pub mid_points_control_offset: f32,
        "StartHeight" => pub start_height: f32,
        "MidHeight" => pub mid_height: f32,
        "EndHeight" => pub end_height: f32,
        "FlightSpeed" => pub flight_speed: f32,
        "MaxSplineProgressForSplineAdjustment" => pub max_spline_progress_for_spline_adjustment: f32,
        "MaxSplineProgressForBurninationEffect" => pub max_spline_progress_for_burnination_effect: f32,
        "MinSplineProgressForWingFlap" => pub min_spline_progress_for_wing_flap: f32,
        "MaxExplosionRange" => pub max_explosion_range: f32,
        "ExplosionLineProjectionAngle" => pub explosion_line_projection_angle: f32,
        "FramesBetweenExplosions" => pub frames_between_explosions: i32,
        "WindSpeed" => pub wind_speed: f32,
    }
}
