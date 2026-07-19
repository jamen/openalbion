//! `CReplaceableMeshDef` — C++ `CReplaceableMeshDef`.

use crate::{def_struct, wire_struct};
use crate::def::prelude::*;

wire_struct! {
    /// One entry of `NGraphicAppearance::CReplaceableMeshes` — original PC release layout.
    /// Built by `Meshes.Add(<EEngineGraphicType>, <mesh>)`: `bank_index` = the mesh
    /// (arg1), `graphic_type` (the trailing byte) = the graphic type (arg0), and
    /// `anim_step`/`render_size_x` take the ctor default 1.0. Verified against retail.
    pub struct ReplaceableMeshesEntry {
        pub bank_index: i32,
        pub anim_step: f32,
        pub render_size_x: f32,
        pub additive_alpha: u8,
        pub graphic_type: u8,
    }
}

wire_struct! {
    /// C++ `NGraphicAppearance::CReplaceableMeshes` — original PC release layout.
    pub struct ReplaceableMeshes {
        pub vector: Vec<ReplaceableMeshesEntry>,
    }
}

def_struct! {
    /// `CReplaceableMeshDef` — original PC release.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ReplaceableMeshDef {
        "Meshes" => pub meshes: ReplaceableMeshes,
    }
}
