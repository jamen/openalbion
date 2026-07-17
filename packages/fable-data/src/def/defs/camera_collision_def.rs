use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCameraCollisionDef` — C++ `CCameraCollisionDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CameraCollisionDef {
        "CameraCollisionMesh" => pub camera_collision_mesh: i32,
    }
}
