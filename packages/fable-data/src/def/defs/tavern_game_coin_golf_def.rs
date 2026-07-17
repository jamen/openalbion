use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTavernGameCoinGolfDef` — C++ `CTavernGameCoinGolfDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TavernGameCoinGolfDef {
        "TargetDistanceScale" => pub target_distance_scale: f32,
        "JoystickScaleXboxX" => pub joystick_scale_xbox_x: f32,
        "JoystickScaleXboxY" => pub joystick_scale_xbox_y: f32,
        "JoystickScaleX" => pub joystick_scale_x: f32,
        "JoystickScaleY" => pub joystick_scale_y: f32,
        "JoystickMagBelow" => pub joystick_mag_below: f32,
        "AverageJoystickMagAbove" => pub average_joystick_mag_above: f32,
        "AverageDrift" => pub average_drift: f32,
        "MaxPower" => pub max_power: f32,
        "RestSpeed" => pub rest_speed: f32,
        "Courses" => pub courses: DefIndex,
        "GoalRadius" => pub goal_radius: f32,
        "StartingCourse" => pub starting_course: DefIndex,
    }
}
