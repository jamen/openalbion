use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CHeroPostcardGeneratorDef` — C++ `CHeroPostcardGeneratorDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroPostcardGeneratorDef {
        "MeshAreaTLPos" => pub mesh_area_tl_pos: Vector2D,
        "MeshAreaBRPos" => pub mesh_area_br_pos: Vector2D,
        "DollCentreOffset" => pub doll_centre_offset: Vector3D,
        "DollBoundingSphereRadius" => pub doll_bounding_sphere_radius: f32,
    }
}
