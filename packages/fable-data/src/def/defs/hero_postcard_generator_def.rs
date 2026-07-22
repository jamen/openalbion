use crate::DefStruct;
use crate::def::prelude::*;

/// `CHeroPostcardGeneratorDef` — C++ `CHeroPostcardGeneratorDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct HeroPostcardGeneratorDef {
    #[def("MeshAreaTLPos")]
    pub mesh_area_tl_pos: Vector2D,
    #[def("MeshAreaBRPos")]
    pub mesh_area_br_pos: Vector2D,
    #[def("DollCentreOffset")]
    pub doll_centre_offset: Vector3D,
    #[def("DollBoundingSphereRadius")]
    pub doll_bounding_sphere_radius: f32,
}
