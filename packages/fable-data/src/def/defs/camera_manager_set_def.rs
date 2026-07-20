use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CAMERA_MANAGER_SET` — C++ `CCameraManagerSetDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CameraManagerSetDef {
        "CameraManagerMain" => pub camera_manager_main: DefIndex,
        "CameraManagerCombat" => pub camera_manager_combat: DefIndex,
        "CameraManagerPCMain" => pub camera_manager_pc_main: DefIndex,
        "CameraManagerPCCombat" => pub camera_manager_pc_combat: DefIndex,
    }
}
