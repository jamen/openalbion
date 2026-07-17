use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CAMERA_MANAGER_SET` — C++ `CCameraManagerSetDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CameraManagerSetDef {
        "CameraManagerMain" => pub camera_manager_main: i32,
        "CameraManagerCombat" => pub camera_manager_combat: i32,
        "CameraManagerPCMain" => pub camera_manager_pc_main: i32,
        "CameraManagerPCCombat" => pub camera_manager_pc_combat: i32,
    }
}
