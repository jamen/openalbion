use crate::DefStruct;

/// `CCameraCollisionDef` — C++ `CCameraCollisionDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct CameraCollisionDef {
    #[def("CameraCollisionMesh")]
    pub camera_collision_mesh: i32,
}
