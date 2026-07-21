use crate::def_struct;

def_struct! {
    /// `CNymphDef` — C++ `CNymphDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct NymphDef {
        "MinAttackRange" => pub min_attack_range: f32,
        "MinHeight" => pub min_height: f32,
        "MaxHeight" => pub max_height: f32,
        "CircleRange" => pub circle_range: f32,
        "FlyAroundTime" => pub fly_around_time: i32,
        "ShotTrapType" => pub shot_trap_type: i32,
        "ShotsBeforeIdle" => pub shots_before_idle: i32,
        "IdleTimeBetweenShots" => pub idle_time_between_shots: f32 = 1.0,
        "FadeInTime" => pub fade_in_time: f32 = 1.0,
        "FadeOutTime" => pub fade_out_time: f32 = 1.0,
    }
}
