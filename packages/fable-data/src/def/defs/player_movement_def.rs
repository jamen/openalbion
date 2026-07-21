use crate::def_struct;

def_struct! {
    /// `PLAYER_MOVEMENT` — C++ `CPlayerMovementDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct PlayerMovementDef {
        "JoystickMovementStationaryBand" => pub joystick_movement_stationary_band: f32,
        "JoystickMovementSlowBand" => pub joystick_movement_slow_band: f32,
        "JoystickMovementWalkingBand" => pub joystick_movement_walking_band: f32,
        "JoystickMovementJoggingBand" => pub joystick_movement_jogging_band: f32,
        "MinRequiredAngleFor180" => pub min_required_angle_for180: f32,
        "SecondsToStoreLastMovementFor" => pub seconds_to_store_last_movement_for: f32,
        "NoMovementFramesToCheckFor180" => pub no_movement_frames_to_check_for180: i32,
        "NoMovementFramesToCheckFor180PriorStraightLine" => pub no_movement_frames_to_check_for180_prior_straight_line: i32,
        "MinNoRunSecondsForSkidHaltMove" => pub min_no_run_seconds_for_skid_halt_move: f32,
        "NoRunningFramesToPreventSkidHalt" => pub no_running_frames_to_prevent_skid_halt: i32,
        "MaxAllowedAngleChangeForSkidHalt" => pub max_allowed_angle_change_for_skid_halt: f32,
        "MaxTurnSpeed" => pub max_turn_speed: f32,
        "MaxTurnSpeedHeavyWeapon" => pub max_turn_speed_heavy_weapon: f32,
        "LeanAngleTurnFactor" => pub lean_angle_turn_factor: f32,
        "LeanAngleLimitAngle" => pub lean_angle_limit_angle: f32,
        "LeanAngleStepPerFrame" => pub lean_angle_step_per_frame: f32,
    }
}
