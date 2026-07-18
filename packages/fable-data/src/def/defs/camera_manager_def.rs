use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CAMERA_MANAGER` — C++ `CCameraManagerDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CameraManagerDef {
        "CameraList" => pub camera_list: Vec<i32>,
        "SpecialityCameraList" => pub speciality_camera_list: Vec<i32>,
        "InitialMode" => pub initial_mode: i32,
        "ZTargetMode" => pub z_target_mode: i32,
        "ProjectileWeaponMode" => pub projectile_weapon_mode: i32,
        "BaseHeightToKeepAboveFlatLandscape" => pub base_height_to_keep_above_flat_landscape: f32 = 0.25,
        "BaseHeightToKeepAboveWater" => pub base_height_to_keep_above_water: f32 = 1.25,
        "MaxHeightRelativeToGround" => pub max_height_relative_to_ground: f32 = 10.0,
        "MaxHeightStepForPointAcceptance" => pub max_height_step_for_point_acceptance: f32 = 3.0,
        "AngleBetweenGroundNormalAndVerticalForLandscapeToBeConsideredNotFlat" => pub angle_between_ground_normal_and_vertical_for_landscape_to_be_considered_not_flat: f32 = 30.0,
    }
}
