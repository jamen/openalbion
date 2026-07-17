use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CLensFlareElementDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct LensFlareElementDef {
        "Radius" => pub radius: f32,
        "Texture" => pub texture: i32,
        "Position" => pub position: f32,
        "Colour" => pub colour: RGBColour,
    }
}
