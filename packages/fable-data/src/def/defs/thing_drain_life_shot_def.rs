use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CThingDrainLifeShotDef` — C++ `CThingDrainLifeShotDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ThingDrainLifeShotDef {
        "MaxAngle" => pub max_angle: f32,
        "MaxDistance" => pub max_distance: f32,
        "MaxAngleToTurnDegs" => pub max_angle_to_turn_degs: f32,
        "InitialSpeed" => pub initial_speed: f32,
        "SpeedRange" => pub speed_range: f32,
    }
}
